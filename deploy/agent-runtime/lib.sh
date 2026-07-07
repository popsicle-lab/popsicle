#!/usr/bin/env bash
# Shared helpers for deploy/agent-runtime/*.sh — source only, do not execute.
set -euo pipefail

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  echo "error: source lib.sh, do not execute directly" >&2
  exit 1
fi

AR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ar_is_macos() {
  [[ "$(uname -s)" == "Darwin" ]]
}

ar_require_command() {
  local name=$1
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "error: ${name} not found on PATH" >&2
    exit 1
  fi
}

ar_require_podman() {
  ar_require_command podman
}

ar_podman_machine_running() {
  podman machine list --format '{{.Running}}' 2>/dev/null | grep -q true
}

ar_ensure_podman_machine() {
  ar_is_macos || return 0
  if ar_podman_machine_running; then
    return 0
  fi
  echo "Starting podman machine..."
  if podman machine start 2>/dev/null; then
    return 0
  fi
  echo "Initializing podman machine (first run)..."
  podman machine init
  podman machine start
}

ar_export_docker_host() {
  ar_is_macos || return 0
  local socket
  socket="$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
  export DOCKER_HOST="unix://${socket}"
}

ar_podman_storage_repair_hint() {
  echo "  run: ./repair-storage.sh --prune" >&2
  echo "  or:  ./repair-storage.sh --reset-machine" >&2
}

ar_podman_is_storage_error() {
  grep -qiE 'overlay|graph driver|readlink.*invalid argument' <<< "$1"
}

ar_podman_fail() {
  local err=$1
  echo "$err" >&2
  if ar_podman_is_storage_error "$err"; then
    echo "error: podman storage is corrupted (often from earlier docker-compose use)" >&2
    ar_podman_storage_repair_hint
  fi
  exit 1
}

ar_podman() {
  local err
  if err="$(podman "$@" 2>&1)"; then
    [[ -n "$err" ]] && printf '%s\n' "$err"
    return 0
  fi
  ar_podman_fail "$err"
}

ar_podman_check_storage() {
  command -v podman >/dev/null 2>&1 || return 0
  local probe err
  for probe in info "ps -a -q" "images -q"; do
    # shellcheck disable=SC2086
    if podman $probe >/dev/null 2>&1; then
      continue
    fi
    # shellcheck disable=SC2086
    err="$(podman $probe 2>&1)" || true
    echo "error: podman storage check failed (podman ${probe})" >&2
    echo "$err" >&2
    if ar_podman_is_storage_error "$err"; then
      echo "error: podman storage is corrupted (often from earlier docker-compose use)" >&2
    fi
    ar_podman_storage_repair_hint
    exit 1
  done
}

# Use plain podman when podman-compose is missing. Avoids podman compose → docker-compose
# delegation, which breaks Podman machine storage on macOS (overlay readlink errors).
ar_use_native_podman() {
  command -v podman >/dev/null 2>&1 && ! command -v podman-compose >/dev/null 2>&1
}

ar_stack_project() { echo "agent-runtime"; }
ar_stack_postgres_container() { echo "$(ar_stack_project)-postgres-1"; }
ar_stack_server_container() { echo "$(ar_stack_project)-agent-server-1"; }
ar_stack_network() { echo "$(ar_stack_project)_default"; }
ar_stack_pg_volume() { echo "$(ar_stack_project)_pgdata"; }
ar_stack_image() { echo "$(ar_stack_project)-agent-server"; }

ar_prepare_compose_env() {
  if command -v podman >/dev/null 2>&1; then
    ar_require_podman
    ar_ensure_podman_machine
    ar_export_docker_host
    ar_podman_check_storage
    return 0
  fi
  if command -v docker-compose >/dev/null 2>&1; then
    return 0
  fi
  echo "error: need podman or docker-compose" >&2
  exit 1
}

ar_compose() {
  if command -v podman-compose >/dev/null 2>&1; then
    podman-compose "$@"
  elif command -v docker-compose >/dev/null 2>&1; then
    docker-compose "$@"
  else
    echo "error: need podman-compose or docker-compose" >&2
    exit 1
  fi
}

ar_host_port_in_use() {
  local port=$1
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
    return $?
  fi
  if command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -qE "[.]${port}[[:space:]]"
    return $?
  fi
  return 1
}

ar_podman_container_owns_port() {
  local port=$1 ps_out
  ps_out="$(podman ps --format '{{.Ports}}' 2>/dev/null)" || return 1
  grep -qE "0\.0\.0\.0:${port}->|127\.0\.0\.1:${port}->|:${port}->" <<< "$ps_out"
}

ar_compose_owns_port() {
  local port=$1
  if ar_use_native_podman; then
    ar_podman_container_owns_port "$port"
    return $?
  fi
  local ps_out
  ps_out="$(ar_compose -f "${AR_ROOT}/compose.yaml" ps 2>/dev/null)" || return 1
  grep -qE "0\.0\.0\.0:${port}->|127\.0\.0\.1:${port}->|:${port}->" <<< "$ps_out"
}

ar_preflight_ports() {
  local pg_port="${AGENT_RUNTIME_PG_PORT:-5433}"
  local http_port="${AGENT_RUNTIME_PORT:-8787}"
  local blocked=0

  if ar_host_port_in_use "$pg_port" && ! ar_compose_owns_port "$pg_port"; then
    echo "error: host port ${pg_port} already in use (AGENT_RUNTIME_PG_PORT)" >&2
    blocked=1
  fi
  if ar_host_port_in_use "$http_port" && ! ar_compose_owns_port "$http_port"; then
    echo "error: host port ${http_port} already in use (AGENT_RUNTIME_PORT)" >&2
    blocked=1
  fi

  if [[ "$blocked" == 1 ]]; then
    echo "hint: pick free ports, e.g. AGENT_RUNTIME_PG_PORT=5434 AGENT_RUNTIME_PORT=8788 ./up.sh" >&2
    exit 1
  fi
}

ar_wait_postgres_healthy() {
  local container=$1
  local attempt
  for attempt in $(seq 1 30); do
    if podman exec "$container" pg_isready -U agent -d agent_runtime >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "error: postgres did not become healthy in time" >&2
  exit 1
}

ar_podman_up() {
  local do_build=1
  local arg
  for arg in "$@"; do
    if [[ "$arg" == "--no-build" ]]; then
      do_build=0
    fi
  done

  ar_podman_check_storage

  local pg_port="${AGENT_RUNTIME_PG_PORT:-5433}"
  local http_port="${AGENT_RUNTIME_PORT:-8787}"
  local network vol pg_c srv_c image repo_root
  network="$(ar_stack_network)"
  vol="$(ar_stack_pg_volume)"
  pg_c="$(ar_stack_postgres_container)"
  srv_c="$(ar_stack_server_container)"
  image="$(ar_stack_image)"
  repo_root="$(cd "${AR_ROOT}/../.." && pwd)"

  if ! podman network exists "$network" 2>/dev/null; then
    ar_podman network create "$network"
  fi
  if ! podman volume exists "$vol" 2>/dev/null; then
    ar_podman volume create "$vol"
  fi

  if podman container exists "$pg_c" 2>/dev/null; then
    if podman inspect "$pg_c" --format '{{range .Mounts}}{{.Destination}} {{end}}' 2>/dev/null \
      | grep -q 'agent-runtime-postgres-entrypoint.sh'; then
      ar_podman start "$pg_c" >/dev/null
    else
      echo "Recreating postgres container (pg_hba network auth fix)..."
      podman rm -f "$pg_c" >/dev/null 2>&1 || true
    fi
  fi

  if ! podman container exists "$pg_c" 2>/dev/null; then
    ar_podman run -d \
      --name "$pg_c" \
      --network "$network" \
      --network-alias postgres \
      --entrypoint /usr/local/bin/agent-runtime-postgres-entrypoint.sh \
      -e POSTGRES_USER=agent \
      -e POSTGRES_PASSWORD=agent \
      -e POSTGRES_DB=agent_runtime \
      -v "${vol}:/var/lib/postgresql/data" \
      -v "${AR_ROOT}/postgres-entrypoint.sh:/usr/local/bin/agent-runtime-postgres-entrypoint.sh:ro" \
      -p "${pg_port}:5432" \
      --health-cmd "pg_isready -U agent -d agent_runtime" \
      --health-interval 5s \
      --health-timeout 5s \
      --health-retries 5 \
      docker.io/library/postgres:16-alpine \
      postgres
  fi

  ar_wait_postgres_healthy "$pg_c"

  if [[ "$do_build" == 1 ]]; then
    ar_podman build -t "$image" -f "${AR_ROOT}/Dockerfile" "$repo_root"
  elif ! podman image exists "$image" 2>/dev/null; then
    echo "error: image ${image} not found; run ./up.sh without --no-build" >&2
    exit 1
  fi

  ar_podman run -d --replace \
    --name "$srv_c" \
    --network "$network" \
    -p "${http_port}:8787" \
    -e AGENT_RUNTIME_PORT=8787 \
    -e AGENT_RUNTIME_DATABASE_URL=postgres://agent:agent@postgres:5432/agent_runtime \
    "$image"
}

ar_podman_down() {
  local remove_volumes=0
  local arg
  for arg in "$@"; do
    case "$arg" in
      -v | --volumes) remove_volumes=1 ;;
    esac
  done

  podman rm -f "$(ar_stack_server_container)" 2>/dev/null || true
  podman rm -f "$(ar_stack_postgres_container)" 2>/dev/null || true
  if [[ "$remove_volumes" == 1 ]]; then
    podman volume rm "$(ar_stack_pg_volume)" 2>/dev/null || true
  fi
  podman network rm "$(ar_stack_network)" 2>/dev/null || true
}
