# DevOps entry points (migrated from legacy/popsicle Makefile, ADR-014).
# Differences from legacy: no UI bundle (build-ui dropped), golden/intent
# targets added for the IDD verification chain.

.PHONY: check fmt fmt-fix clippy test build build-ui ui-dev build-dmg install-intent golden intent install install-hooks

check: fmt clippy test

fmt:
	cargo fmt --all -- --check

fmt-fix:
	cargo fmt --all

clippy:
	RUSTFLAGS="-Dwarnings" cargo clippy --all-targets

test:
	RUSTFLAGS="-Dwarnings" cargo test --all-targets

build:
	cargo build --release

build-ui:
	cd ui && npm ci && npm run build
	cargo build --features ui -p cli-ux

ui-dev:
	cd ui && npm run dev

build-dmg:
	bash packaging/macos/build-dmg.sh

# Install pinned intent-lang to ~/.local/bin/intent (macOS; see packaging/intent-lang-pin.toml).
install-intent:
	bash packaging/macos/install-intent.sh

# Full golden-baseline chain (latest run-all chains all earlier baselines).
golden:
	bash docs/baseline/2026-06-11/cli-ux-sqlite-phase2/run-all.sh

# Intent validation over product specs (requires intent v0.1.1+ on PATH; Z3 bundled).
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
