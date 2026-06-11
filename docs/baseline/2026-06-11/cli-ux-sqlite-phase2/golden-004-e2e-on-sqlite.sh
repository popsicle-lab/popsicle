#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

# The smoke workspace is freshly initialized, so the whole IDD loop
# (issue -> run -> doc check -> stages -> close) runs on the SQLite backend.
cargo test -p cli-ux self_host_workflow_smoke_passes -- --nocapture
