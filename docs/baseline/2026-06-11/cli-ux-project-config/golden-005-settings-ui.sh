#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

test -f ui/src/pages/SettingsView.tsx
grep -q 'get_project_config' crates/cli-ux/src/ui/mod.rs
grep -q 'kind: "settings"' ui/src/App.tsx

cd ui && npm run build --silent

echo "golden-005 ok (Settings UI page + Tauri commands + build)"
