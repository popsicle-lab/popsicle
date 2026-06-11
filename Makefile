# DevOps entry points (migrated from legacy/popsicle Makefile, ADR-014).
# Differences from legacy: no UI bundle (build-ui dropped), golden/intent
# targets added for the IDD verification chain.

.PHONY: check fmt fmt-fix clippy test build golden intent install install-hooks

check: fmt clippy test

fmt:
	cargo fmt --all -- --check

fmt-fix:
	cargo fmt --all

clippy:
	RUSTFLAGS="-Dwarnings" cargo clippy --all-targets --all-features

test:
	RUSTFLAGS="-Dwarnings" cargo test --all-targets --all-features

build:
	cargo build --release

# Full golden-baseline chain (latest run-all chains all earlier baselines).
golden:
	bash docs/baseline/2026-06-11/cli-ux-sqlite-phase2/run-all.sh

# Z3 intent validation over the product specs (requires intent-lang).
intent:
	cargo build -p cli-ux
	./target/debug/popsicle tool run intent-validate path=products

install:
	scripts/install.sh

install-hooks:
	@mkdir -p .git/hooks
	@cp hooks/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks installed."
