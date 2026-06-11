#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux --test global_config open_project_records_recent_and_default -q
echo "golden-001 ok (open_project recents + default)"
