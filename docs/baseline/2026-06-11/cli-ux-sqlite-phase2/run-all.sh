#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"

# Earlier baselines must keep passing (no regression); the usability run-all
# already chains self-host + command-alignment.
bash "$DIR/../cli-ux-usability/run-all.sh"

for script in "$DIR"/golden-*.sh; do
  echo "==> $(basename "$script")"
  bash "$script"
done

bash "$DIR/../cli-ux-ui/run-all.sh"
bash "$DIR/../cli-ux-global/run-all.sh"
bash "$DIR/../cli-ux-project-ui/run-all.sh"
bash "$DIR/../cli-ux-roadmap-workflow/run-all.sh"

echo "All cli-ux sqlite-phase2 golden baselines passed."
