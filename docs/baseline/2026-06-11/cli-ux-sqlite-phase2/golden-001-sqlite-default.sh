#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux fresh_workspace_defaults_to_sqlite_backend -- --nocapture
