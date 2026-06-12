#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux guidance_for_issue_recommends_tasks -- --nocapture
echo "golden-003 ok"
