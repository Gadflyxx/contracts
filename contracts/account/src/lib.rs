//! RemitPulse passkey multisig account with scoped, expiring session keys.
#![no_std]
use remitpulse_webauthn_verifier::{verify_assertion, Assertion};
use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
    Symbol, TryFromVal, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub keys: Vec<BytesN<65>>,
    pub threshold: u32,
    pub rp_id_hash: BytesN<32>,
    pub origin: Bytes,
}
#[contracttype]
#[derive(Clone)]
pub struct Session {
    /// Network-specific SEP-41/SAC token address: USDC, EURC, ARST, BRLA, etc.
    pub token: Address,
    /// Some(address) restricts this key to a single remittance beneficiary.
    pub recipient: Option<Address>,
    /// Amounts are token base units, never a cross-currency USD valuation.
    pub per_transfer: i128,
    pub remaining: i128,
    pub expires_at: u64,
}
#[contracttype]
#[derive(Clone)]
pub enum Signature {
    Passkeys(Vec<Assertion>),
    Session(BytesN<32>, BytesN<64>),
}
#[contracttype]
#[derive(Clone)]
enum Key {
    Config,
    SessionsPaused,
    Session(BytesN<32>),
}
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Invalid = 1,
    Expired = 2,
    Scope = 3,
    Limit = 4,
    Paused = 5,
}
#[contract]
pub struct Account;

fn touch(e: &Env) {
    e.storage().instance().extend_ttl(120_960, 518_400);
}

#[contractimpl]
impl Account {
    pub fn __constructor(e: Env, config: Config) {
        assert!(
            config.threshold > 0
                && config.threshold <= config.keys.len()
                && config.keys.len() <= 10
        );
        assert!(!config.origin.is_empty());
        for i in 0..config.keys.len() {
            assert!(config.keys.get(i).unwrap().get(0) == Some(4));
            for j in 0..i {
                assert!(config.keys.get(i) != config.keys.get(j), "duplicate key");
            }
        }
        e.storage().instance().set(&Key::Config, &config);
        touch(&e);
    }

    /// Full passkey quorum is required. Replacing a key resets its explicit budget.
    pub fn add_session(e: Env, public_key: BytesN<32>, session: Session) {
        e.current_contract_address().require_auth();
        assert!(session.per_transfer > 0 && session.remaining >= session.per_transfer);
        assert!(session.expires_at > e.ledger().timestamp());
        e.storage()
            .instance()
            .set(&Key::Session(public_key.clone()), &session);
        e.events().publish(
            (symbol_short!("session"), symbol_short!("set"), public_key),
            session,
        );
        touch(&e);
    }
    pub fn revoke_session(e: Env, public_key: BytesN<32>) {
        e.current_contract_address().require_auth();
        e.storage()
            .instance()
            .remove(&Key::Session(public_key.clone()));
        e.events().publish(
            (
                symbol_short!("session"),
                symbol_short!("revoked"),
                public_key,
            ),
            (),
        );
        touch(&e);
    }
    /// A passkey quorum may freeze all sessions without blocking passkey transfers or revocation.
    pub fn set_sessions_paused(e: Env, paused: bool) {
        e.current_contract_address().require_auth();
        e.storage().instance().set(&Key::SessionsPaused, &paused);
        e.events()
            .publish((symbol_short!("session"), symbol_short!("paused")), paused);
        touch(&e);
    }
    pub fn sessions_paused(e: Env) -> bool {
        e.storage()
            .instance()
            .get(&Key::SessionsPaused)
            .unwrap_or(false)
    }
    /// Lets clients discover the credential order, quorum, RP binding and origin.
    pub fn config(e: Env) -> Config {
        e.storage().instance().get(&Key::Config).unwrap()
    }
    pub fn session(e: Env, public_key: BytesN<32>) -> Option<Session> {
        e.storage().instance().get(&Key::Session(public_key))
    }
}

#[contractimpl]
impl CustomAccountInterface for Account {
    type Signature = Signature;
    type Error = Error;
    fn __check_auth(
        e: Env,
        signature_payload: soroban_sdk::crypto::Hash<32>,
        signature: Signature,
        auth_contexts: Vec<Context>,
    ) -> Result<(), Error> {
        let signature_payload: BytesN<32> = signature_payload.into();
        match signature {
            Signature::Passkeys(assertions) => {
                let config: Config = e
                    .storage()
                    .instance()
                    .get(&Key::Config)
                    .ok_or(Error::Invalid)?;
                if assertions.len() < config.threshold || assertions.len() > config.keys.len() {
                    return Err(Error::Invalid);
                }
                let mut seen = Vec::new(&e);
                for assertion in assertions {
                    if seen.contains(&assertion.key_index) {
                        return Err(Error::Invalid);
                    }
                    let key = config.keys.get(assertion.key_index).ok_or(Error::Invalid)?;
                    verify_assertion(
                        &e,
                        &key,
                        &config.rp_id_hash,
                        &config.origin,
                        &signature_payload,
                        &assertion,
                    );
                    seen.push_back(assertion.key_index);
                }
            }
            Signature::Session(public_key, sig) => {
                if Self::sessions_paused(e.clone()) {
                    return Err(Error::Paused);
                }
                let storage_key = Key::Session(public_key.clone());
                let mut session: Session = e
                    .storage()
                    .instance()
                    .get(&storage_key)
                    .ok_or(Error::Invalid)?;
                if e.ledger().timestamp() >= session.expires_at {
                    return Err(Error::Expired);
                }
                if auth_contexts.is_empty() {
                    return Err(Error::Scope);
                }
                let mut total = 0i128;
                for context in auth_contexts {
                    let call = match context {
                        Context::Contract(c) => c,
                        _ => return Err(Error::Scope),
                    };
                    if call.contract != session.token
                        || call.fn_name != Symbol::new(&e, "transfer")
                        || call.args.len() != 3
                    {
                        return Err(Error::Scope);
                    }
                    let from = Address::try_from_val(&e, &call.args.get(0).unwrap())
                        .map_err(|_| Error::Scope)?;
                    let to = Address::try_from_val(&e, &call.args.get(1).unwrap())
                        .map_err(|_| Error::Scope)?;
                    let amount = i128::try_from_val(&e, &call.args.get(2).unwrap())
                        .map_err(|_| Error::Scope)?;
                    if session
                        .recipient
                        .as_ref()
                        .is_some_and(|recipient| *recipient != to)
                    {
                        return Err(Error::Scope);
                    }
                    if from != e.current_contract_address() {
                        return Err(Error::Scope);
                    }
                    if amount <= 0 || amount > session.per_transfer {
                        return Err(Error::Limit);
                    }
                    total = total.checked_add(amount).ok_or(Error::Limit)?;
                }
                if total > session.remaining {
                    return Err(Error::Limit);
                }
                e.crypto()
                    .ed25519_verify(&public_key, &Bytes::from(signature_payload), &sig);
                session.remaining -= total;
                e.storage().instance().set(&storage_key, &session);
                e.events().publish(
                    (symbol_short!("session"), symbol_short!("spent"), public_key),
                    (total, session.remaining),
                );
            }
        }
        touch(&e);
        Ok(())
    }
}
#[cfg(test)]
mod test;
