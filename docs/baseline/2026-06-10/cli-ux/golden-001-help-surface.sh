#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_001_help_exposes_idd_main_path_without_removed_commands -- --nocapture
