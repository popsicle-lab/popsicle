#!/usr/bin/env bash
# One-time (or on-demand) Podman setup: drop HTTP proxy, enable docker.io CN mirrors.
#
# Usage:
#   ./configure-podman-mirrors.sh            # idempotent; skips unchanged files
#   ./configure-podman-mirrors.sh --test-pull  # also verify postgres image pull
#
# Normally invoked via ./up.sh --configure-mirrors on first run.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "${ROOT}/lib.sh"
MIRROR_CONF='001-cn-mirrors.conf'

test_pull=0
for arg in "$@"; do
  case "$arg" in
    --test-pull) test_pull=1 ;;
    *)
      echo "error: unknown option: $arg" >&2
      echo "usage: $0 [--test-pull]" >&2
      exit 1
      ;;
  esac
done

if [[ "${AGENT_RUNTIME_MIRROR_TEST_PULL:-}" =~ ^(1|true|yes|on)$ ]]; then
  test_pull=1
fi

remove_host_proxy() {
  local conf="${HOME}/.config/containers/containers.conf"
  [[ -f "$conf" ]] || return 0
  if grep -qiE 'http_proxy|https_proxy|7897' "$conf" 2>/dev/null; then
    if ar_is_macos; then
      sed -i '' '/^env = .*proxy/d' "$conf"
    else
      sed -i '/^env = .*proxy/d' "$conf"
    fi
    echo "host: removed http_proxy from ~/.config/containers/containers.conf"
  fi
}

remove_vm_proxy() {
  ar_require_podman
  ar_ensure_podman_machine

  local stamp="${HOME}/.config/containers/.agent-runtime-vm-proxy-cleared"
  if [[ -f "$stamp" ]]; then
    return 0
  fi

  local removed=0
  local f
  for f in \
    /etc/profile.d/default-env.sh \
    /etc/systemd/system.conf.d/default-env.conf \
    /etc/systemd/user.conf.d/default-env.conf; do
    if podman machine ssh -- "test -f $f" 2>/dev/null; then
      podman machine ssh -- "sudo rm -f $f"
      echo "vm: removed $f"
      removed=1
    fi
  done

  if [[ "$removed" != 1 ]]; then
    return 0
  fi

  podman machine ssh -- 'sudo systemctl daemon-reload' 2>/dev/null || true
  mkdir -p "$(dirname "$stamp")"
  touch "$stamp"
  echo "vm: proxy env cleared (no machine restart; re-run podman machine stop/start manually if pulls still use proxy)"
}

install_host_mirrors() {
  local dest="${HOME}/.config/containers/registries.conf.d/000-cn-mirrors.conf"
  mkdir -p "$(dirname "$dest")"
  if [[ -f "$dest" ]] && cmp -s "${ROOT}/${MIRROR_CONF}" "$dest"; then
    echo "host: mirrors unchanged ($dest)"
    return 0
  fi
  cp "${ROOT}/${MIRROR_CONF}" "$dest"
  if [[ ! -f "${HOME}/.config/containers/registries.conf" ]]; then
    printf '%s\n' 'unqualified-search-registries = ["docker.io"]' \
      >"${HOME}/.config/containers/registries.conf"
  fi
  echo "host: installed $dest"
}

install_vm_mirrors() {
  ar_require_podman
  ar_ensure_podman_machine

  if podman machine ssh -- "sudo test -f /etc/containers/registries.conf.d/${MIRROR_CONF}" 2>/dev/null \
    && podman machine ssh -- "sudo cmp -s /etc/containers/registries.conf.d/${MIRROR_CONF} -" \
      <"${ROOT}/${MIRROR_CONF}" 2>/dev/null; then
    echo "vm: mirrors unchanged (/etc/containers/registries.conf.d/${MIRROR_CONF})"
    return 0
  fi

  podman machine ssh -- "sudo tee /etc/containers/registries.conf.d/${MIRROR_CONF} >/dev/null" \
    <"${ROOT}/${MIRROR_CONF}"
  echo "vm: installed /etc/containers/registries.conf.d/${MIRROR_CONF}"
}

test_mirror_pull() {
  ar_require_podman
  ar_ensure_podman_machine
  ar_export_docker_host

  echo "Testing pull (postgres:16-alpine via CN mirrors)..."
  if podman pull --quiet docker.io/library/postgres:16-alpine; then
    echo "ok: pull succeeded"
  else
    echo "warn: pull failed — check network or mirror availability" >&2
    return 1
  fi
}

remove_host_proxy
remove_vm_proxy
install_host_mirrors
install_vm_mirrors

if [[ "$test_pull" == 1 ]]; then
  test_mirror_pull || true
fi

echo
echo "Done. docker.io CN mirrors configured."
echo "Next: ./up.sh"
