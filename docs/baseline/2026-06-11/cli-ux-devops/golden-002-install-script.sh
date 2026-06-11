#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

bash -n scripts/install.sh
scripts/install.sh --help | grep -q -- "--prefix"
scripts/install.sh --help | grep -q -- "--uninstall"
# Legacy-only flags must be gone (no UI, completions deferred).
if scripts/install.sh --help | grep -qE -- "--no-ui|--no-completions"; then
  echo "FAIL: install.sh still advertises legacy UI/completions flags" >&2
  exit 1
fi
echo "golden-002 ok (install.sh adapted)"
