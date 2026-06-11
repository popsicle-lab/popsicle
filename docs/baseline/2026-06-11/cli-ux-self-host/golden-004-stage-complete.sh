#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_004_stage_complete_requires_confirm_then_advances -- --nocapture
