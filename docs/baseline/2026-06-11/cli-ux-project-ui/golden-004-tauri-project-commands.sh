#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build --features ui -p cli-ux -q
rg -q 'open_project_cmd|list_registered_projects|pick_project_directory|get_active_project' \
  crates/cli-ux/src/ui/mod.rs
rg -q 'last_opened_at|open_project|resolve_ui_startup_root' \
  crates/cli-ux/src/global_config.rs
echo "golden-004 ok (tauri project commands registered)"
