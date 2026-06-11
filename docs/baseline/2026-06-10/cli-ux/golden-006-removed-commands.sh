#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_006_removed_commands_return_actionable_errors -- --nocapture
