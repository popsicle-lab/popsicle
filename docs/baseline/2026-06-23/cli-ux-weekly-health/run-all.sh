#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
"$DIR/golden-001-pipeline-install.sh"
"$DIR/golden-002-issue-start-project-context.sh"
echo "cli-ux-weekly-health: all golden passed"
