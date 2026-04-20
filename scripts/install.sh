#!/usr/bin/env bash
# Install the popsicle CLI to your macOS command line.
#
# Default behavior:
#   1. Verify prerequisites (cargo/rustc, plus npm/node for the UI feature).
#   2. Build the UI bundle (cd ui && npm run build).
#   3. Build and install the `popsicle` binary with `--features ui`.
#   4. Ensure the install directory is on your PATH.
#   5. Install shell completions for your current shell (zsh/bash/fish).
#
# Usage:
#   scripts/install.sh [options]
#
# Options:
#   --prefix <dir>   Install binary into <dir> (default: ~/.cargo/bin via cargo install)
#   --shell <name>   Force completion shell: zsh | bash | fish | none
#   --no-ui          Build without the `ui` feature (skips npm build)
#   --no-completions Skip installing shell completions
#   --uninstall      Remove the popsicle binary and installed completions
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

if [[ ! -f "${REPO_ROOT}/crates/popsicle-cli/Cargo.toml" ]]; then
  error "Could not find crates/popsicle-cli/Cargo.toml relative to this script."
  error "Please run this script from inside the popsicle repository."
  exit 1
fi

# ----- parse args -------------------------------------------------------------
PREFIX=""
FORCE_SHELL=""
INSTALL_COMPLETIONS=1
BUILD_UI=1
MODE="install"

print_help() {
  sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)           PREFIX="${2:-}"; shift 2 ;;
    --shell)            FORCE_SHELL="${2:-}"; shift 2 ;;
    --no-ui)            BUILD_UI=0; shift ;;
    --no-completions)   INSTALL_COMPLETIONS=0; shift ;;
    --uninstall)        MODE="uninstall"; shift ;;
    -h|--help)          print_help; exit 0 ;;
    *) error "Unknown option: $1"; print_help; exit 1 ;;
  esac
done

# ----- sanity check: macOS ----------------------------------------------------
if [[ "$(uname -s)" != "Darwin" ]]; then
  warn "This script is tuned for macOS but will likely work on your $(uname -s) system."
fi

# ----- detect shell -----------------------------------------------------------
detect_shell() {
  if [[ -n "${FORCE_SHELL}" ]]; then
    echo "${FORCE_SHELL}"; return
  fi
  local name
  name="$(basename "${SHELL:-}")"
  case "${name}" in
    zsh|bash|fish) echo "${name}" ;;
    *) echo "zsh" ;;  # sensible macOS default
  esac
}

USER_SHELL="$(detect_shell)"

# ----- determine install dir --------------------------------------------------
CARGO_BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
INSTALL_DIR="${PREFIX:-${CARGO_BIN_DIR}}"

# ----- completion target dirs -------------------------------------------------
ZSH_COMP_DIR="${HOME}/.zsh/completions"
BASH_COMP_DIR="${HOME}/.local/share/bash-completion/completions"
FISH_COMP_DIR="${HOME}/.config/fish/completions"

completion_path() {
  case "$1" in
    zsh)  echo "${ZSH_COMP_DIR}/_popsicle" ;;
    bash) echo "${BASH_COMP_DIR}/popsicle" ;;
    fish) echo "${FISH_COMP_DIR}/popsicle.fish" ;;
    *)    echo "" ;;
  esac
}

# ==============================================================================
# Uninstall
# ==============================================================================
if [[ "${MODE}" == "uninstall" ]]; then
  info "Uninstalling popsicle..."

  if command -v cargo >/dev/null 2>&1; then
    if cargo uninstall popsicle-cli 2>/dev/null; then
      success "Removed popsicle-cli via cargo uninstall."
    else
      warn "cargo uninstall popsicle-cli did not remove anything."
    fi
  fi

  if [[ -n "${PREFIX}" && -f "${PREFIX}/popsicle" ]]; then
    rm -f "${PREFIX}/popsicle"
    success "Removed ${PREFIX}/popsicle."
  fi

  for sh in zsh bash fish; do
    p="$(completion_path "${sh}")"
    if [[ -f "${p}" ]]; then
      rm -f "${p}"
      success "Removed ${sh} completion at ${p}."
    fi
  done

  success "popsicle uninstalled."
  exit 0
fi

# ==============================================================================
# Install
# ==============================================================================
info "Installing popsicle from ${DIM}${REPO_ROOT}${RESET}"

# ----- prerequisites ----------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
  error "cargo not found."
  error "Install Rust first: https://rustup.rs   (curl https://sh.rustup.rs -sSf | sh)"
  exit 1
fi

info "Using $(cargo --version)"
info "Using $(rustc --version)"

# ----- build UI bundle (required for --features ui) ---------------------------
CARGO_FEATURES_ARGS=()
if [[ "${BUILD_UI}" -eq 1 ]]; then
  if [[ ! -d "${REPO_ROOT}/ui" ]]; then
    error "UI build requested but ${REPO_ROOT}/ui does not exist."
    error "Re-run with --no-ui to skip the UI feature."
    exit 1
  fi

  if ! command -v npm >/dev/null 2>&1; then
    error "npm not found, required to build the UI bundle."
    error "Install Node.js (https://nodejs.org) or run with --no-ui."
    exit 1
  fi

  info "Using $(node --version 2>/dev/null || echo 'node ?') / npm $(npm --version)"

  if [[ ! -d "${REPO_ROOT}/ui/node_modules" ]]; then
    info "Installing UI dependencies (npm install)..."
    ( cd "${REPO_ROOT}/ui" && npm install )
  fi

  info "Building UI bundle (npm run build)..."
  ( cd "${REPO_ROOT}/ui" && npm run build )

  CARGO_FEATURES_ARGS=(--features ui)
  success "UI bundle built."
else
  warn "Skipping UI build (--no-ui)."
fi

# ----- build & install binary -------------------------------------------------
if [[ -n "${PREFIX}" ]]; then
  info "Running: cargo build --release --bin popsicle ${CARGO_FEATURES_ARGS[*]:-}"
  ( cd "${REPO_ROOT}" && cargo build --release --bin popsicle "${CARGO_FEATURES_ARGS[@]}" )

  mkdir -p "${INSTALL_DIR}"
  cp "${REPO_ROOT}/target/release/popsicle" "${INSTALL_DIR}/popsicle"
  chmod +x "${INSTALL_DIR}/popsicle"
  success "Installed popsicle to ${INSTALL_DIR}/popsicle"
else
  info "Running: cargo install --path crates/popsicle-cli --force ${CARGO_FEATURES_ARGS[*]:-}"
  ( cd "${REPO_ROOT}" && cargo install --path crates/popsicle-cli --force "${CARGO_FEATURES_ARGS[@]}" )
  success "Installed popsicle to ${INSTALL_DIR}/popsicle"
fi

# ----- PATH check -------------------------------------------------------------
if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
  warn "${INSTALL_DIR} is not on your PATH."
  case "${USER_SHELL}" in
    zsh)  rc="${HOME}/.zshrc" ;;
    bash) rc="${HOME}/.bash_profile" ;;
    fish) rc="${HOME}/.config/fish/config.fish" ;;
    *)    rc="${HOME}/.zshrc" ;;
  esac
  if [[ "${USER_SHELL}" == "fish" ]]; then
    echo "    fish_add_path ${INSTALL_DIR}"
  else
    echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ${rc}"
    echo "    source ${rc}"
  fi
  warn "Add the line above to put popsicle on your PATH, then restart your shell."
fi

# ----- install completions ----------------------------------------------------
if [[ "${INSTALL_COMPLETIONS}" -eq 1 && "${USER_SHELL}" != "none" ]]; then
  POPSICLE_BIN="${INSTALL_DIR}/popsicle"
  if [[ ! -x "${POPSICLE_BIN}" ]]; then
    # Fall back to whatever is on PATH (cargo install case without PATH yet).
    POPSICLE_BIN="$(command -v popsicle || echo "${INSTALL_DIR}/popsicle")"
  fi

  case "${USER_SHELL}" in
    zsh)
      mkdir -p "${ZSH_COMP_DIR}"
      "${POPSICLE_BIN}" completions zsh > "${ZSH_COMP_DIR}/_popsicle"
      success "Installed zsh completion to ${ZSH_COMP_DIR}/_popsicle"
      if ! grep -qs "${ZSH_COMP_DIR}" "${HOME}/.zshrc" 2>/dev/null; then
        warn "Add the following to your ~/.zshrc so zsh picks up completions:"
        echo "    fpath=(${ZSH_COMP_DIR} \$fpath)"
        echo "    autoload -Uz compinit && compinit"
      fi
      ;;
    bash)
      mkdir -p "${BASH_COMP_DIR}"
      "${POPSICLE_BIN}" completions bash > "${BASH_COMP_DIR}/popsicle"
      success "Installed bash completion to ${BASH_COMP_DIR}/popsicle"
      ;;
    fish)
      mkdir -p "${FISH_COMP_DIR}"
      "${POPSICLE_BIN}" completions fish > "${FISH_COMP_DIR}/popsicle.fish"
      success "Installed fish completion to ${FISH_COMP_DIR}/popsicle.fish"
      ;;
    *)
      warn "Unknown shell '${USER_SHELL}', skipping completion setup."
      ;;
  esac
fi

# ----- done -------------------------------------------------------------------
success "Done. Try running: ${BOLD}popsicle --help${RESET}"
