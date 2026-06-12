#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

test -f intent-coder/guides/retro-doc-checklist.md
grep -q 'retro' intent-coder/guides/retro-doc-checklist.md
test -f ui/src/components/RetroDocBanner.tsx

echo "golden-004 ok (retro-doc-checklist guide + RetroDocBanner)"
