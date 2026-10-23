.PHONY: test build optimize deploy-testnet deploy-mainnet fmt check
STELLAR_CLI ?= stellar
COMPONENT ?= core
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
