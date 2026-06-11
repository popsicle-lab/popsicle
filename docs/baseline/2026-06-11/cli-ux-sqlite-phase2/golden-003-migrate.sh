#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux migrate_to_sqlite_preserves_rows_and_is_idempotent -- --nocapture
