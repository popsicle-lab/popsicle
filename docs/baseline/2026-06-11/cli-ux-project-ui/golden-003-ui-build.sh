#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
(cd ui && npm ci --silent && npm run build --silent)
cargo build --features ui -p cli-ux -q
echo "golden-003 ok (ui npm build + cargo ui feature)"
