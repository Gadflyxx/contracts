# Source provenance

Extracted from https://github.com/stellar/smart-wallet-demo-app at commit
`e10b447e756b71948f2d436332ad704ecea0c90d` (develop), retrieved 2026-10-23.

All Rust contracts present in that snapshot are retained in `contracts/router`,
`contracts/airdrop`, and `contracts/nft`, including their SDK tests. Manifests use
RemitPulse package names. Original source links and attribution are preserved.
The source package declares ISC in its root package.json; no root license file
was present in the retrieved snapshot. The repository's existing MIT LICENSE
is retained for RemitPulse additions; upstream licensing is not replaced by it.

The router credits Creit-Tech/Stellar-Router-Contract commit
04975c434dec362584aa99458ae2c25d803ec570. Airdrop and NFT depend on
OpenZeppelin stellar-contracts v0.3.0 (pinned in Cargo.lock), whose own license
and copyright apply. Review the upstream licensing before redistribution.

The requested Rust passkey controller, verifier, multisig/session keys and
paymaster are absent from the source snapshot. `contracts/account`,
`contracts/webauthn`, and `contracts/subsidy` are new RemitPulse implementations,
not extracted components. Backend TypeScript WebAuthn and fee submission code
is intentionally outside this Rust-only workspace.
