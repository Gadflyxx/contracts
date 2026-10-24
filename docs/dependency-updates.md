# Dependency updates

Dependabot opens weekly pull requests for Cargo packages and GitHub Actions.
Every update uses the standard pull-request workflow, which checks formatting,
native tests, release WASM builds, WASM tests, and deployment-script tests.

Review dependency pull requests as normal code changes. In particular, inspect
`Cargo.lock` changes, confirm the required workflow is green, and verify that a
dependency update does not alter contract interfaces or deployment artifacts
unexpectedly.
