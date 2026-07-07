#!/usr/bin/env bash
# Start agent-runtime stack (Podman Compose).
#
# Usage:
#   ./up.sh                 # start stack
#   ./up.sh --configure-mirrors   # one-time CN mirror setup, then start
#   ./up.sh --no-build      # passed through to compose
#
# Env:
#   AGENT_RUNTIME_PORT      HTTP port (default 8787)
#   AGENT_RUNTIME_PG_PORT   PostgreSQL host port (default 5433)
#   AGENT_RUNTIME_CONFIGURE_MIRRORS=1  same as --configure-mirrors
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "${ROOT}/lib.sh"
cd "$ROOT"

configure_mirrors=0
compose_args=()
for arg in "$@"; do
  case "$arg" in
    --configure-mirrors) configure_mirrors=1 ;;
    *) compose_args+=("$arg") ;;
  esac
done

if [[ "${AGENT_RUNTIME_CONFIGURE_MIRRORS:-}" =~ ^(1|true|yes|on)$ ]]; then
  configure_mirrors=1
fi

if command -v podman >/dev/null 2>&1; then
  ar_require_podman
  ar_ensure_podman_machine
  if [[ "$configure_mirrors" == 1 ]]; then
    "${ROOT}/configure-podman-mirrors.sh"
  fi
  ar_export_docker_host
  ar_podman_check_storage
elif command -v docker-compose >/dev/null 2>&1; then
  :
else
  echo "error: need podman or docker-compose" >&2
  exit 1
fi

ar_preflight_ports

if ar_use_native_podman; then
  if ((${#compose_args[@]} > 0)); then
    ar_podman_up "${compose_args[@]}"
  else
    ar_podman_up
  fi
elif ((${#compose_args[@]} > 0)); then
  ar_compose -f compose.yaml up -d --build "${compose_args[@]}"
else
  ar_compose -f compose.yaml up -d --build
fi

echo
echo "agent-runtime is up."
echo "  API:  http://127.0.0.1:${AGENT_RUNTIME_PORT:-8787}"
echo "  PG:   postgres://agent:agent@127.0.0.1:${AGENT_RUNTIME_PG_PORT:-5433}/agent_runtime"
if [[ "$configure_mirrors" != 1 ]]; then
  echo "  tip: first pull slow? run ./configure-podman-mirrors.sh once (or ./up.sh --configure-mirrors)"
fi
