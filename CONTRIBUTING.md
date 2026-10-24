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
