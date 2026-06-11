#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_002_issue_start_returns_run_id_and_lock_signal -- --nocapture
