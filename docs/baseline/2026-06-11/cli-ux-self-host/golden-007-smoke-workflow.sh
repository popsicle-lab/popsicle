#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
cargo test -p cli-ux self_host_workflow_smoke_passes -- --nocapture
