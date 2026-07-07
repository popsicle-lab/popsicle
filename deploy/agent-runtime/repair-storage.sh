#!/usr/bin/env bash
# Recovery when podman reports overlay / graph driver errors.
#
# Usage:
#   ./repair-storage.sh              # print recovery steps
#   ./repair-storage.sh --prune      # remove stale layers (try first)
#   ./repair-storage.sh --reset-machine  # recreate Podman VM (destructive)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "${ROOT}/lib.sh"

ar_print_manual_steps() {
  cat <<'EOF'
Podman storage overlay error (readlink .../storage/overlay: invalid argument)
Usually caused by stale layers from earlier docker-compose + podman experiments.

Try in order:

  1. ./repair-storage.sh --prune
     ./up.sh

  2. podman machine stop && podman machine start
     ./up.sh

  3. ./repair-storage.sh --reset-machine
     ./configure-podman-mirrors.sh
     ./up.sh
EOF
}

ar_prune_storage() {
  ar_require_podman
  ar_ensure_podman_machine
  ar_export_docker_host

  echo "Removing stopped containers and dangling images..."
  podman rm -af 2>/dev/null || true
  podman system prune -af
  echo "Prune done. Run ./up.sh next."
}

ar_reset_machine() {
  ar_require_podman
  ar_is_macos || {
    echo "error: --reset-machine is for macOS Podman machine only" >&2
    exit 1
  }

  echo "Resetting Podman machine (deletes all VM containers/images)..."
  podman machine stop 2>/dev/null || true
  podman machine rm -f
  podman machine init
  podman machine start
  rm -f "${HOME}/.config/containers/.agent-runtime-vm-proxy-cleared"
  echo "Machine reset done. Run ./configure-podman-mirrors.sh then ./up.sh"
}

case "${1:-}" in
  --prune) ar_prune_storage ;;
  --reset-machine) ar_reset_machine ;;
  "" | --help | -h) ar_print_manual_steps ;;
  *)
    echo "error: unknown option: $1" >&2
    echo "usage: $0 [--prune | --reset-machine]" >&2
    exit 1
    ;;
esac
