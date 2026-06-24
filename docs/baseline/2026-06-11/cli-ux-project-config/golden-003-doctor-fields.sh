#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-project-config-doctor.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

mkdir -p "$ROOT/.popsicle"
cat >"$ROOT/.popsicle/project.yaml" <<'EOF'
version: 1
agent:
  language: zh-CN
paths:
  products_dir: products
workflow:
  sync_agents_md: false
  inject_on_run: true
EOF

OUT="$("$BIN" doctor --project "$ROOT" --format json)"
echo "$OUT" | grep -qE '"agent_language":"(zh-CN|zh-cn)"'
echo "$OUT" | grep -q '"products_dir":"products"'
echo "$OUT" | grep -q 'project_config_path'

echo "golden-003 ok (doctor reports project config)"
