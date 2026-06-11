#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
for f in \
  ui/src/lib/navigation.ts \
  ui/src/hooks/useWideLayout.ts \
  ui/src/components/LoadingState.tsx; do
  test -f "$f" || { echo "missing $f"; exit 1; }
done
grep -q 'master-detail' ui/src/index.css
grep -q 'sidebarCollapsed' ui/src/App.tsx
grep -q 'pageCrumbs' ui/src/components/AppHeader.tsx
echo "golden-003 ok (layout shell modules)"
