# Dependency updates

Dependabot opens weekly pull requests for Cargo packages and GitHub Actions.
Every update uses the standard pull-request workflow, which checks formatting,
native tests, release WASM builds, WASM tests, and deployment-script tests.

Review dependency pull requests as normal code changes. In particular, inspect
`Cargo.lock` changes, confirm the required workflow is green, and verify that a
dependency update does not alter contract interfaces or deployment artifacts
unexpectedly.

## Vulnerability scanning

Dependabot proposes updates but is not a complete vulnerability gate. CI also
runs a lockfile-based vulnerability scan that consumes `Cargo.lock` and fails
the build when a known advisory affects a locked dependency. This scan runs as
part of the required workflow, so a pull request cannot merge while it reports
an unaddressed advisory.

To reproduce the scan locally, install `cargo-audit` and run it against the
committed lockfile from the repository root:

```sh
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
```

The command exits non-zero when a known advisory is found, matching the CI
result.

### Ignored advisories

If an advisory cannot be fixed immediately (for example, no patched release is
available yet), it may be ignored only through a documented, time-bounded
exception. Each ignored advisory must record:

- the advisory identifier,
- a short justification for why it is not currently exploitable or fixable,
- an expiry date after which the exception no longer applies.

Exceptions are tracked alongside the scan configuration. When an exception
expires, the advisory fails the scan again until it is either fixed or renewed
with a fresh justification and expiry date. Do not add open-ended ignores.