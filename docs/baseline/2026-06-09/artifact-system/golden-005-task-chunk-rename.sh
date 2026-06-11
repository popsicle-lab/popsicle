#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p artifact-system golden_005_task_chunk_rename_preserves_fields -- --nocapture
