#!/usr/bin/env bash
# Install the popsicle CLI to your command line.
# (Migrated from legacy/popsicle scripts/install.sh, ADR-014. Differences:
#  no UI feature — the self-host CLI has no UI bundle; no shell completions —
#  `completions` is a deferred command, see AGENTS.md.)
#
# Default behavior:
#   1. Verify prerequisites (cargo/rustc).
#   2. Build and install the `popsicle` binary from crates/cli-ux.
#   3. Ensure the install directory is on your PATH.
#
# Usage:
#   scripts/install.sh [options]
#
# Options:
#   --prefix <dir>   Install binary into <dir> (default: ~/.cargo/bin via cargo install)
#   --uninstall      Remove the popsicle binary
#   -h, --help       Show this help

set -euo pipefail

# ----- pretty printing --------------------------------------------------------
BOLD="$(tput bold 2>/dev/null || true)"
DIM="$(tput dim 2>/dev/null || true)"
RED="$(tput setaf 1 2>/dev/null || true)"
GREEN="$(tput setaf 2 2>/dev/null || true)"
YELLOW="$(tput setaf 3 2>/dev/null || true)"
BLUE="$(tput setaf 4 2>/dev/null || true)"
RESET="$(tput sgr0 2>/dev/null || true)"

info()    { printf "%s[popsicle]%s %s\n" "${BLUE}${BOLD}" "${RESET}" "$*"; }
success() { printf "%s[popsicle]%s %s\n" "${GREEN}${BOLD}" "${RESET}" "$*"; }
warn()    { printf "%s[popsicle]%s %s\n" "${YELLOW}${BOLD}" "${RESET}" "$*"; }
error()   { printf "%s[popsicle]%s %s\n" "${RED}${BOLD}" "${RESET}" "$*" >&2; }

# ----- locate repo ------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [[ ! -f "${REPO_ROOT}/crates/cli-ux/Cargo.toml" ]]; then
  error "Could not find crates/cli-ux/Cargo.toml relative to this script."
  error "Please run this script from inside the popsicle repository."
  exit 1
fi

# ----- parse args -------------------------------------------------------------
PREFIX=""
MODE="install"

print_help() {
  sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)    PREFIX="${2:-}"; shift 2 ;;
    --uninstall) MODE="uninstall"; shift ;;
    -h|--help)   print_help; exit 0 ;;
    *) error "Unknown option: $1"; print_help; exit 1 ;;
  esac
done

CARGO_BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
INSTALL_DIR="${PREFIX:-${CARGO_BIN_DIR}}"

# ==============================================================================
# Uninstall
# ==============================================================================
if [[ "${MODE}" == "uninstall" ]]; then
  info "Uninstalling popsicle..."

  if command -v cargo >/dev/null 2>&1; then
    if cargo uninstall cli-ux 2>/dev/null; then
      success "Removed cli-ux (popsicle) via cargo uninstall."
    else
      warn "cargo uninstall cli-ux did not remove anything."
    fi
  fi

  if [[ -n "${PREFIX}" && -f "${PREFIX}/popsicle" ]]; then
    rm -f "${PREFIX}/popsicle"
    success "Removed ${PREFIX}/popsicle."
  fi

  success "popsicle uninstalled."
  exit 0
fi

# ==============================================================================
# Install
# ==============================================================================
info "Installing popsicle from ${DIM}${REPO_ROOT}${RESET}"

if ! command -v cargo >/dev/null 2>&1; then
  error "cargo not found."
  error "Install Rust first: https://rustup.rs   (curl https://sh.rustup.rs -sSf | sh)"
  exit 1
fi

info "Using $(cargo --version)"
info "Using $(rustc --version)"

if [[ -n "${PREFIX}" ]]; then
  info "Running: cargo build --release --bin popsicle"
  ( cd "${REPO_ROOT}" && cargo build --release --bin popsicle )

  mkdir -p "${INSTALL_DIR}"
  cp "${REPO_ROOT}/target/release/popsicle" "${INSTALL_DIR}/popsicle"
  chmod +x "${INSTALL_DIR}/popsicle"
  success "Installed popsicle to ${INSTALL_DIR}/popsicle"
else
  info "Running: cargo install --path crates/cli-ux --force"
  ( cd "${REPO_ROOT}" && cargo install --path crates/cli-ux --force )
  success "Installed popsicle to ${INSTALL_DIR}/popsicle"
fi

# ----- PATH check -------------------------------------------------------------
if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
  warn "${INSTALL_DIR} is not on your PATH."
  echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc"
  echo "    source ~/.zshrc"
  warn "Add the line above to put popsicle on your PATH, then restart your shell."
fi

# ----- workspace provenance note ----------------------------------------------
warn "Inside the popsicle dev workspace itself, prefer ./target/debug/popsicle"
warn "(doctor provenance guards against stale/system binaries — see AGENTS.md)."

success "Done. Try running: ${BOLD}popsicle help${RESET}"
