#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
grep -q 'useWideLayout' ui/src/pages/IssuesView.tsx
grep -q 'variant="panel"' ui/src/pages/IssuesView.tsx
grep -q 'master-detail' ui/src/pages/IssuesView.tsx
echo "golden-004 ok (issues master-detail)"
