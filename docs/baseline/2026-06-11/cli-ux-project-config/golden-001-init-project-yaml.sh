#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-project-config-init.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

cd "$ROOT"
"$BIN" init --format json | grep -q '"status":"ok"'
test -f .popsicle/project.yaml
grep -q 'products_dir' .popsicle/project.yaml
test -f AGENTS.md
grep -q 'popsicle:project-config:start' AGENTS.md

echo "golden-001 ok (init writes project.yaml + AGENTS.md marker)"
