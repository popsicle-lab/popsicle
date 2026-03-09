.PHONY: check fmt clippy test build build-ui install-hooks

check: fmt clippy test

fmt:
	cargo fmt --all -- --check

clippy:
	RUSTFLAGS="-Dwarnings" cargo clippy --all-targets --all-features

test:
	RUSTFLAGS="-Dwarnings" cargo test --all-targets --all-features

build:
	cargo build --release

build-ui:
	cd ui && npm run build
	cargo build --release --features ui

install-hooks:
	@mkdir -p .git/hooks
	@cp hooks/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks installed."
