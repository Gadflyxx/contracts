#!/usr/bin/env bash
# Keep contract compilation on the tested toolchain, including inside CLI subprocesses.
set -euo pipefail
cd "$(dirname "$0")/.."
toolchain=$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)
: "${toolchain:?Missing channel in rust-toolchain.toml}"
# An inherited RUSTUP_TOOLCHAIN otherwise overrides rust-toolchain.toml.
export RUSTUP_TOOLCHAIN="$toolchain"
exec "${STELLAR_CLI:-stellar}" contract build