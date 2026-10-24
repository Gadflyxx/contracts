# Contributing

Keep each change focused and include any required updates to tests or
documentation. Before opening a pull request, run the same validation used by
continuous integration:

```sh
make ci
```

The pinned Rust toolchain and WASM target are defined in `rust-toolchain.toml`.
Dependency-update pull requests follow the same checks; review their lockfile
changes and wait for the required workflow to pass before merging.

## Linting and security checks

`make ci` runs the full validation suite, which now includes Clippy,
ShellCheck, and a lockfile-based vulnerability scan. You can also run each
check individually while iterating:

### Clippy

Clippy runs across the whole workspace with warnings treated as errors, so any
project warning fails the check. Dependency lint output is excluded so the
check focuses on repository code.

```sh
cargo clippy --workspace --all-targets -- -D warnings
```

### ShellCheck

ShellCheck covers the deployment and build scripts (`deploy.sh` and
`scripts/build.sh`) to catch unsafe quoting, portability, and control-flow
problems. Any exclusions are narrowly scoped and explained inline with a
`# shellcheck disable=...` comment next to the affected line.

```sh
shellcheck deploy.sh scripts/build.sh
```

### Dependency vulnerability scan

The vulnerability scan consumes `Cargo.lock` and fails CI on known advisories.
Any exceptions must be documented and time-bounded. Run it locally with:

```sh
cargo audit
```

If you add a temporary exception, document the advisory ID, the reason, and an
expiry date alongside the configuration so it can be reviewed and removed.