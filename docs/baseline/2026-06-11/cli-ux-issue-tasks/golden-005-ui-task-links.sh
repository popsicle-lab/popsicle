#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
(cd ui && npm run build --silent)
echo "golden-005 ok"
