#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
grep -q 'pipeline-split' ui/src/pages/PipelineView.tsx
grep -q 'explorer-split' ui/src/pages/ProductExplorerView.tsx
echo "golden-005 ok (pipeline + products split layouts)"
