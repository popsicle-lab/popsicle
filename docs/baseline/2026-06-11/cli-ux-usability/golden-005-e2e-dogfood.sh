#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

# Full dogfood loop in an isolated workspace: init -> bug issue (default
# bugfix pipeline, D-101) -> start -> doc create/check fail -> fill -> check
# pass -> complete both stages -> run completed -> issue close.
cargo test -p cli-ux self_host_workflow_smoke_passes -- --nocapture
