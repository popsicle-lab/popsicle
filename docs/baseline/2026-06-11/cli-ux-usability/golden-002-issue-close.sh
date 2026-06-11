#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux tsv_issue_close_requires_completed_run -- --nocapture
