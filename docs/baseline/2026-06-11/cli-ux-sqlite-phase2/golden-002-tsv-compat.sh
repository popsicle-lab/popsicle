#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux legacy_tsv_workspace_still_loads_and_saves -- --nocapture
