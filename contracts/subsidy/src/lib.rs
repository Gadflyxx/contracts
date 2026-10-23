//! RemitPulse token subsidy vault. Network fees are sponsored by an off-chain fee-bump payer.
#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, BytesN, Env};
#[contracttype]
#[derive(Clone)]
enum Key {
    Admin,
    Used(BytesN<32>),
}
#[contract]
pub struct Subsidy;
#[contractimpl]
impl Subsidy {
    pub fn __constructor(e: Env, admin: Address) {
        e.storage().instance().set(&Key::Admin, &admin);
    }
    /// Admin-authorized, exactly-once token reimbursement. Fund the vault by SEP-41 transfer.
    /// Each token (USDC/EURC/ARST/BRLA) has its own balance and native base-unit amounts.
    pub fn reimburse(e: Env, id: BytesN<32>, asset: Address, recipient: Address, amount: i128) {
        let admin: Address = e.storage().instance().get(&Key::Admin).unwrap();
        admin.require_auth();
        assert!(amount > 0);
        // Instance storage keeps replay markers with the vault, including after archival restore.
        assert!(
            !e.storage().instance().has(&Key::Used(id.clone())),
            "already reimbursed"
        );
        e.storage().instance().set(&Key::Used(id), &true);
        token::Client::new(&e, &asset).transfer(&e.current_contract_address(), &recipient, &amount);
        e.storage().instance().extend_ttl(120_960, 518_400);
    }
}
#[cfg(test)]
mod test;
