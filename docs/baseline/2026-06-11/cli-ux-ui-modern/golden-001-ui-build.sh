#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
(cd ui && npm run build)
echo "golden-001 ok (ui production build)"
