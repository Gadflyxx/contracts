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

## Dependency vulnerability scanning

Dependabot proposes dependency updates but is not a complete vulnerability
gate. CI additionally runs a lockfile-based advisory scan that consumes
`Cargo.lock` and fails the build when a known advisory affects a locked
dependency.

### Scan command

The scan uses [`cargo-audit`](https://github.com/rustsec/rustsec) against the
committed `Cargo.lock`:

```sh
cargo audit --file Cargo.lock
```

CI runs this command in the dependency-scan job. Because the scan reads
`Cargo.lock`, results are reproducible for a given commit: the same lockfile
yields the same advisory set (modulo updates to the RustSec advisory database).

### Running locally

Install the pinned tool version and run the scan from the repository root:

```sh
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
```

A non-zero exit status indicates one or more known advisories affect the
lockfile. Resolve them by updating the affected dependency (and committing the
updated `Cargo.lock`), or by recording a time-bounded exception as described
below.

### Documented, time-bounded exceptions

An advisory may be accepted only when it is not exploitable in this project's
usage and no fixed release is available. Every accepted advisory must be
recorded in this section with:

- **Advisory ID** — the RustSec advisory identifier (for example `RUSTSEC-0000-0000`).
- **Justification** — why the advisory does not affect RemitPulse, including
  the affected code path and why it is unreachable or mitigated.
- **Expiry date** — the date by which the exception must be re-reviewed. The
  expiry must be no more than 90 days from the date the exception is recorded.

Exceptions are enforced by passing the advisory ID to `cargo audit --ignore`
in the CI scan and by listing it in the table below. An exception past its
expiry date is invalid: CI must fail until the exception is renewed with a new
justification and expiry, or the dependency is updated.

| Advisory ID | Justification | Expiry date |
| ----------- | ------------- | ----------- |
| _(none currently accepted)_ | | |

When an exception expires, re-evaluate the advisory: update the dependency if a
fix exists, otherwise renew the exception with an updated justification and a
new expiry date no more than 90 days out.