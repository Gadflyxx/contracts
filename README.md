# remitpulse-contracts

Rust/Soroban contracts for RemitPulse, a passkey-powered Stellar remittance app.
This repository contains a Cargo workspace; each contract has its own `Cargo.toml`
and `src/`. SDK 22.0.8, OpenZeppelin v0.3.0 and Rust 1.88.0 are pinned for
compatibility with the extracted contracts. Cargo.lock must be committed.

| Crate | Purpose |
| --- | --- |
| `remitpulse-account` | WebAuthn quorum account and bounded Ed25519 sessions |
| `remitpulse-webauthn` | ES256 WebAuthn verification, also linked into the account |
| `remitpulse-subsidy` | Administrator-authorized token subsidy vault |
| `remitpulse-router` | Extracted atomic invocation router |
| `remitpulse-airdrop` | Extracted Merkle token distribution |
| `remitpulse-nft` | Extracted NFT contract |

`crates/webauthn-verifier` contains the shared verification library, so the account
does not export the standalone verifier endpoint.

The source repository includes only router, airdrop and NFT Rust contracts.
The account, verifier and subsidy vault are new RemitPulse implementations.
See [UPSTREAM.md](UPSTREAM.md) for the exact source revision and attribution.

## Build and test

Install Rust using rustup, then a Stellar CLI with Protocol 22 support (the
successor to `soroban-cli`). The toolchain file installs the required Rust target.
Deployment-script tests require Python 3.11 or later.

Run the complete local CI suite with:

```sh
make ci
```

Or run individual checks:

```sh
cargo test --workspace --locked
python3 -m unittest discover -s tests -v
cargo fmt --all -- --check
cargo build --workspace --locked --release --target wasm32v1-none
make build
```

`make build` and `deploy.sh` use `scripts/build.sh`, which explicitly selects the
Rust version from `rust-toolchain.toml` for the CLI and its Cargo subprocesses.
This also handles shells or CI jobs with `RUSTUP_TOOLCHAIN=stable`, which would
otherwise override the repository's pin. The Stellar CLI itself may be installed
using a newer compiler; contract compilation still uses the pinned toolchain.

The initial compiler incompatibilities are resolved by Rust **1.88.0**:

- Rust 1.99.1 rejects `ethnum` 1.5.2's integer-error transmute because the standard
  library's `TryFromIntError` representation changed.
- Rust 1.85.0 cannot compile OpenZeppelin v0.3.0's `is_multiple_of` call, which was
  not yet stable in that toolchain.
- Rust 1.88.0 supports the locked dependencies and passes the native test suite
  and release WASM builds. Keep the toolchain pin and `Cargo.lock` together.

For a manual build outside the wrapper, clear an inherited override with
`env -u RUSTUP_TOOLCHAIN cargo build --workspace --locked --release --target wasm32v1-none`.
Run builds from this repository so rustup finds its toolchain file.

`stellar contract build` builds and optimizes WASM by default with current CLI
versions. Artifacts are `target/wasm32v1-none/release/remitpulse_*.wasm`.
Older `soroban` CLI releases use a separate optimization step:

```sh
soroban contract optimize \
  --wasm target/wasm32v1-none/release/remitpulse_account.wasm \
  --wasm-out target/wasm32v1-none/release/remitpulse_account.optimized.wasm
```

The CLI must support `wasm32v1-none` and constructor arguments after `--`.
The deployment script supports `STELLAR_CLI=soroban` for such legacy releases.
[Stellar build/deploy documentation](https://developers.stellar.org/docs/build/smart-contracts/getting-started/hello-world).

Tests use the built-in Soroban test SDK. The extracted contracts retain their
original unit/integration suites. New tests cover session limits, expiry,
revocation, token scope, real signed custom-account authorization, SAC transfers,
replay protection, and subsidy authorization. Deployment script tests stub the CLI to verify constructor arguments, network selection,
preflight validation and build failure handling. No RPC or funded account is needed.

Run `make verify` before release. It runs the native suite, deployment checks,
format checks, builds the release artifacts, and runs the account suite again
against `remitpulse_account.wasm` inside the Soroban VM. CI uploads the exact
WASM files with a `SHA256SUMS` manifest, so release candidates can be verified
against the reviewed build output.

## Passkeys and sessions

Construct an account with `Config`: unique uncompressed SEC1 P-256 public keys
(65 bytes), quorum `threshold`, SHA-256 of the WebAuthn RP ID (`rp_id_hash`), and
an exact HTTPS origin encoded as bytes. Registration, attestation policy and
credential ownership verification belong in the client enrollment flow.
The constructor fixes this quorum for the lifetime of the account; this version
has no recovery, key rotation or upgrade mechanism.

Privileged operations require a `Signature::Passkeys` vector. Each assertion
contains its unique `key_index`, raw authenticator data, original client JSON and
64-byte ECDSA `r || s`. Clients must convert DER signatures to raw and normalize
S to low-S. The challenge is the unpadded base64url encoding of the **Soroban
authorization signature payload**, not an independently generated challenge.
Verification enforces RP ID hash, exact origin, `webauthn.get`, no cross-origin
ceremony, user presence and user verification. Soroban enforces authorization
nonce/ledger expiry. Authenticator sign counters are not tracked (synced
passkeys commonly report zero). JSON strings must use literal characters rather
than escaped representations for the compared fields.

1. Obtain a full passkey quorum authorizing `account.add_session(public_key, session)`.
2. Scope the session to a verified token contract address, optionally one fixed
   recipient, a positive per-transfer amount, total remaining budget and an
   absolute ledger timestamp expiry (seconds). Bind the recipient for a typical
   remittance payout; leaving it empty permits any recipient within the limits.
3. Sign subsequent Soroban authorization payloads with the session's Ed25519 key.
   Every authorized context must be that token's `transfer(account, recipient, amount)`.
   Transfers must be positive, individually under the cap and cumulatively under budget.
4. Authorize `revoke_session` with the full passkey quorum to terminate access.
   The same quorum can call `set_sessions_paused(true)` for an immediate
   account-wide session-key freeze, without preventing a passkey transfer or
   revocation. Resume with `set_sessions_paused(false)` after investigation.

Sessions cannot authorize approvals, arbitrary calls, deployments, new sessions
or revocations. A session permits any recipient within its limits; protect the
session secret on the client. Multiple transfer contexts consume their aggregate
amount. Failed transactions roll back the budget. A successful transfer may
extend account storage TTL; expired sessions never gain validity from TTL renewal.
Operators must monitor/extend contract instance and code TTL and restore archival
entries as needed. Instance storage retains budgets and revocations together.

## Multi-currency transfers

USDC, EURC and regional anchor assets such as ARST/BRLA use the same SEP-41
`transfer` interface, including Stellar Asset Contracts. Identify assets by
**network-specific contract address and verified issuer**, never ticker alone.
Configure one session key per asset; amounts are signed `i128` token base units.
Read each asset's `decimals()`; do not assume all assets have seven decimals.
Budgets are denominated in the selected token, without FX conversion or a price
oracle. RemitPulse does not mint these currencies or perform swaps. Recipients
of anchor assets may need trustlines and issuer authorization; failed transfers
revert atomically. Airdrop instances distribute one configured asset each.

## Deploy and initialize

Create and fund a CLI identity on testnet:

```sh
stellar keys generate remitpulse-deployer --network testnet --fund
export SOURCE_ACCOUNT=remitpulse-deployer
export ADMIN="$(stellar keys address remitpulse-deployer)"
```

Set `ACCOUNT_CONFIG` to JSON with real registered passkey material. Hex strings
represent byte fields; `keys` is an array of 130-character public-key hex strings,
`rp_id_hash` is 64 hex characters, and `origin` is the hex encoding of your origin.
Do not deploy with placeholder keys; there is no recovery path in this version.

```sh
export ACCOUNT_CONFIG='{"keys":["<SEC1_PUBLIC_KEY_HEX>"],"threshold":1,"rp_id_hash":"<SHA256_RP_ID_HEX>","origin":"<ORIGIN_UTF8_HEX>"}'
./deploy.sh testnet core
# For a funded mainnet identity with the same environment inputs configured:
./deploy.sh mainnet core
```

`deploy.sh` validates `ACCOUNT_CONFIG` before compilation or submission: exact
fields, threshold, duplicate keys, valid uncompressed on-curve P-256 keys,
32-byte RP ID hash, and a canonical HTTPS origin. It also validates the airdrop
root and NFT constructor input. It cannot derive the RP ID from its hash, so the
enrollment service must independently ensure the RP ID is valid for the origin.
Use the `config`, `session`, `sessions_paused`, `admin`, and `is_reimbursed`
read methods for operational monitoring. Account events use `session` topics for
set, revoked, paused, and spent transitions; subsidy events use `subsidy` plus
the reimbursement ID.

`core` deploys router, verifier, account and subsidy vault. Account initialization
and admin assignment happen atomically through constructors. A separate verifier
deployment is optional: account calls the same verifier code internally.
The script validates required variables, builds/optimizes before submitting,
stops on failures and records each successful ID in `deployments/<network>-*.env`.
Deployments are separate transactions; earlier deployments remain if a later one
fails. No network deployments are performed by tests or CI.

Deploy optional extracted components individually:

```sh
# Set TOKEN, ROOT_HASH, FUNDER, ADMIN to the asset address, Merkle root and owners.
./deploy.sh testnet airdrop
# Set OWNER, MAX_SUPPLY, METADATA (JSON with base_uri, name and symbol).
./deploy.sh testnet nft
```

`./deploy.sh testnet account`, `subsidy`, `router` or `webauthn` deploys just that
component. `make test`, `make build`, `make optimize`, `make deploy-testnet` and
`make deploy-mainnet` provide shortcuts. `COMPONENT=account` selects a component.

## Subsidies and operational boundaries

Fund the subsidy vault using a regular token transfer. Its administrator calls
`reimburse(id, asset, recipient, amount)` once per unique 32-byte reimbursement ID.
Only the administrator can select the asset, recipient and amount. Failed token
transfers roll back the replay marker. Markers are retained in instance storage;
this simple vault is intended for bounded deployments because markers grow with
usage. Use a new vault for a new reimbursement epoch rather than deleting markers.

This contract cannot pay Stellar transaction fees. A backend fee-bump sponsor
must submit and fund transactions, enforce fee caps and reimbursement policy,
and protect its signing key. Token reimbursement and fee sponsorship are separate
operations; the vault does not prove that a network fee was actually paid.

These contracts have not received an independent security audit. The extracted
upstream is no longer maintained. Mainnet deployment support is provided, but
production use requires review of the account, enrollment flow and retained
upstream contracts.
