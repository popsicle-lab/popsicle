#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"

# Pre-existing self-host goldens must keep passing (no regression).
bash "$DIR/../cli-ux-self-host/run-all.sh"

for script in "$DIR"/golden-*.sh; do
  echo "==> $(basename "$script")"
  bash "$script"
done
echo "All cli-ux command-alignment golden baselines passed."
