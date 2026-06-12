#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
test -f intent-coder/skills/issue-author/skill.yaml
test -f intent-coder/skills/issue-author/guide.md
test -f intent-coder/skills/issue-author/templates/issue-create-report.md
rg -q "Pipeline 决策树" intent-coder/skills/issue-author/guide.md
echo "golden-004 ok"
