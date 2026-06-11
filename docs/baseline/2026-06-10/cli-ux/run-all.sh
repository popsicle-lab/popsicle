#!/usr/bin/env bash
# Run all cli-ux golden baselines. Exit non-zero on any failure.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$ROOT"
DIR="$(dirname "$0")"
for script in "$DIR"/golden-*.sh; do
  echo "==> $(basename "$script")"
  bash "$script"
done
echo "All cli-ux golden baselines passed."
