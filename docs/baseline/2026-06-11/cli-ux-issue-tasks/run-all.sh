#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
for g in "$DIR"/golden-*.sh; do
  echo "==> $(basename "$g")"
  bash "$g"
done
echo "cli-ux-issue-tasks: all goldens ok"
