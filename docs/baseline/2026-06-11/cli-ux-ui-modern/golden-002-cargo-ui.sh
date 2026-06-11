#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build --features ui -p cli-ux -q
echo "golden-002 ok (cargo build --features ui)"
