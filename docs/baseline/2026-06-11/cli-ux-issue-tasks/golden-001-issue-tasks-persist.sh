#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux issue_tasks_multi_linked_and_proposed_persist -- --nocapture
echo "golden-001 ok"
