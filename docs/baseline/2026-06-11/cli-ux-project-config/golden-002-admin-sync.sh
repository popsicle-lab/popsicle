#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-project-config-sync.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

mkdir -p "$ROOT/.popsicle"
cat >"$ROOT/.popsicle/project.yaml" <<'EOF'
version: 1
agent:
  language: en
paths:
  products_dir: products
  default_product: cli-ux
workflow:
  sync_agents_md: true
  inject_on_run: true
EOF

cd "$ROOT"
"$BIN" admin sync-project-config --format json | grep -q '"synced":"true"'
grep -q 'English' AGENTS.md
grep -q 'cli-ux' AGENTS.md

echo "golden-002 ok (admin sync-project-config)"
