#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_005_admin_commands_are_nested_under_admin -- --nocapture
