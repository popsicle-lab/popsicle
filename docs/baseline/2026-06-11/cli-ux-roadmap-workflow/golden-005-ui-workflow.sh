#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

test -f ui/src/components/ProductHealthPanel.tsx
test -f ui/src/components/MarkdownWithMermaid.tsx
test -f ui/src/lib/issueGroup.ts
grep -q 'workflow_profile' ui/src/pages/SettingsView.tsx
grep -q 'epic_task_id' ui/src/pages/IssuesView.tsx

cd ui && npm run build --silent

echo "golden-005 ok (workflow UI components + production build)"
