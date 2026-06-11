#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

(cd ui && npm ci && npm run build)
cargo build --features ui -p cli-ux
echo "golden-002 ok"
