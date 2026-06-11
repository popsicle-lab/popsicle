#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

RUSTFLAGS="-Dwarnings" cargo test -p cli-ux --test workspace_readers scan_cli_ux_tasks_non_empty -- --nocapture
echo "golden-004 ok"
