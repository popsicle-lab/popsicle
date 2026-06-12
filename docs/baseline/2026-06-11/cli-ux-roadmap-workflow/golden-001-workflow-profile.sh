#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux workflow_profile_default_pipelines -- --nocapture
echo "golden-001 ok (WorkflowProfile default pipeline map)"
