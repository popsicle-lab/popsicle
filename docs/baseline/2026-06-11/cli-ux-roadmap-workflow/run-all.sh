#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"

for script in "$DIR"/golden-*.sh; do
  echo "==> $(basename "$script")"
  bash "$script"
done
echo "All cli-ux-roadmap-workflow golden baselines passed."
