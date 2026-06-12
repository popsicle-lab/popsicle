#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux tsv_start_issue_rejects_duplicate_active_run_and_spec_mismatch -- --nocapture
echo "golden-002 ok (epic_task_id persists on issue create)"
