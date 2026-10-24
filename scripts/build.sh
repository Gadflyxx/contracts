#!/usr/bin/env bash
# Keep contract compilation on the tested toolchain, including inside CLI subprocesses.
set -euo pipefail
cd "$(dirname "$0")/.."
toolchain=$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)
: "${toolchain:?Missing channel in rust-toolchain.toml}"
# An inherited RUSTUP_TOOLCHAIN otherwise overrides rust-toolchain.toml.
export RUSTUP_TOOLCHAIN="$toolchain"

# Optional output directory: pass as the first argument or via BUILD_OUT_DIR.
# When unset, the CLI default behavior is preserved (make build, deploy.sh).
out_dir="${1:-${BUILD_OUT_DIR:-}}"

if [ -n "$out_dir" ]; then
  mkdir -p "$out_dir"
  exec "${STELLAR_CLI:-stellar}" contract build --out-dir "$out_dir"
fi

exec "${STELLAR_CLI:-stellar}" contract build