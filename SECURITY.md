# Security policy

Report a suspected vulnerability privately to the RemitPulse security contact.
Do not open a public issue with exploit steps, keys, transaction payloads, or
an affected contract ID. Include the network, contract ID and WASM hash, a
minimal reproduction, impact, and suggested mitigations. Acknowledgment and
coordination occur through the private channel.

## Mainnet deployment checklist

Before a mainnet deployment, complete the preflight checklist in
[`docs/MAINNET_CHECKLIST.md`](docs/MAINNET_CHECKLIST.md). It is the
authoritative, reviewable record for a release and covers:

- **Commit pinning** — the reviewed Git commit, Stellar CLI version, Rust
  toolchain, and dependency lockfile are pinned and recorded.
- **WASM checksum verification** — release WASM hashes are recorded and the CI
  artifact's `SHA256SUMS` is verified locally against the pinned commit.
- **Network configuration** — the target network, RPC/Horizon endpoints,
  passphrase, WebAuthn RP ID, and origin are confirmed for mainnet.
- **Administrator identity** — a dedicated hardware-backed administrator is
  used, with a realistic passkey quorum verified before funding.
- **Rollback and incident readiness** — an incident runbook exists for freezing
  and revoking session keys, and a rollback plan is documented.

Testnet deployments require only the commit pinning and network configuration
sections; checksum verification, administrator identity, and rollback readiness
are mainnet requirements. Record checklist completion in the change review and
release execution, not in this repository's tracked files.

Do not commit private keys, seed phrases, passkey material, or other sensitive
values. Reference secrets by their external location (for example, a hardware
wallet or secret manager) and keep fee-bump sponsorship policy and keys outside
this repository.

The current account has no on-chain recovery, key rotation, or upgrade path.
Use a realistic passkey quorum and verify account configuration before funding.
The subsidy vault authorizes token reimbursements; it does not sponsor Stellar
fees.