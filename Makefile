export RUSTUP_TOOLCHAIN := $(shell sed -n 's/^channel = //p' rust-toolchain.toml | tr -d '"')
.PHONY: test build optimize deploy-testnet deploy-mainnet fmt check test-wasm checksums verify ci
STELLAR_CLI ?= stellar
COMPONENT ?= core
WASM_DIR ?= target/wasm32v1-none/release
test:
	cargo test --workspace --locked
	python3 -m unittest discover -s tests -v
build:
	STELLAR_CLI="$(STELLAR_CLI)" ./scripts/build.sh
optimize: build
# Modern Stellar CLI optimizes as part of build. deploy.sh also supports legacy soroban.
deploy-testnet:
	STELLAR_CLI=$(STELLAR_CLI) ./deploy.sh testnet $(COMPONENT)
deploy-mainnet:
	STELLAR_CLI=$(STELLAR_CLI) ./deploy.sh mainnet $(COMPONENT)
fmt:
	cargo fmt --all
check:
	cargo fmt --all -- --check
	cargo test --workspace --locked
	bash -n deploy.sh scripts/build.sh
	python3 -m unittest discover -s tests -v

# Build first: this suite registers the release artifact inside the Soroban VM.
test-wasm:
	cargo build --workspace --locked --release --target wasm32v1-none
	cargo test -p remitpulse-account --locked --features wasm-tests

# Generate the SHA256 manifest for the release WASM binaries. This is the same
# manifest CI publishes alongside the artifacts so operators can verify them.
checksums:
	@set -e; \
	cd "$(WASM_DIR)"; \
	files=$$(ls remitpulse_*.wasm 2>/dev/null || true); \
	if [ -z "$$files" ]; then \
		echo "error: no remitpulse_*.wasm found in $(WASM_DIR); run 'make build' first" >&2; \
		exit 1; \
	fi; \
	sha256sum $$files > SHA256SUMS; \
	echo "wrote $(WASM_DIR)/SHA256SUMS:"; \
	cat SHA256SUMS

verify: check test-wasm checksums
ci: verify