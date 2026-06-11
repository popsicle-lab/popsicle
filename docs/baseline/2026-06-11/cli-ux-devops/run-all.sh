#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"

# Earlier baselines must keep passing (sqlite-phase2 chains all the rest).
bash "$DIR/../cli-ux-sqlite-phase2/run-all.sh"

for script in "$DIR"/golden-*.sh; do
  echo "==> $(basename "$script")"
  bash "$script"
done
echo "All cli-ux devops golden baselines passed."
