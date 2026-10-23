# Security policy

Report a suspected vulnerability privately to the RemitPulse security contact.
Do not open a public issue with exploit steps, keys, transaction payloads, or
an affected contract ID. Include the network, contract ID and WASM hash, a
minimal reproduction, impact, and suggested mitigations. Acknowledgment and
coordination occur through the private channel.

Before a mainnet deployment, pin the reviewed Git commit, release WASM hashes,
Stellar CLI version, Rust toolchain, dependency lockfile, administrator identity,
WebAuthn RP ID and origin. Verify the CI artifact's `SHA256SUMS` locally. A
mainnet deployment should use a dedicated hardware-backed administrator with
an incident runbook for freezing and revoking session keys.

The current account has no on-chain recovery, key rotation, or upgrade path.
Use a realistic passkey quorum and verify account configuration before funding.
The subsidy vault authorizes token reimbursements; it does not sponsor Stellar
fees. Keep fee-bump sponsorship policy and keys outside this repository.
