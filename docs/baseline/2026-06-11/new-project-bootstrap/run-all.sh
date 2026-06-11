#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$ROOT"
for script in "$(dirname "$0")"/golden-*.sh; do
  echo "==> $(basename "$script")"
  bash "$script"
done
echo "All new-project bootstrap baselines passed."
