#!/usr/bin/env bash
# Stop agent-runtime stack.
#
# Usage:
#   ./down.sh          # stop containers, keep volumes
#   ./down.sh -v       # also remove pgdata volume
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "${ROOT}/lib.sh"
cd "$ROOT"

ar_prepare_compose_env

if ar_use_native_podman; then
  ar_podman_down "$@"
else
  ar_compose -f compose.yaml down "$@"
fi
