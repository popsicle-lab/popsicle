#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

RUSTFLAGS="-Dwarnings" cargo test -p cli-ux --test workspace_readers -- --nocapture
echo "golden-003 ok"
