#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-project-context-inject.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

cd "$ROOT"
"$BIN" init --format json >/dev/null
mkdir -p products/smoke-spec

mkdir -p docs
cat >docs/PROJECT_CONTEXT.md <<'EOF'
# Project Context

## 工程画像

Golden smoke workspace.

## 现在状态

Should not inject.
EOF

KEY="$("$BIN" issue create --type bug --title "ctx smoke" --product smoke-spec --pipeline fix-regression --format json | sed -n 's/.*"key":"\([^"]*\)".*/\1/p')"
OUT="$("$BIN" issue start "$KEY" --format json)"
echo "$OUT" | grep -q '"agent_context"'
echo "$OUT" | grep -q 'Golden smoke workspace'
echo "$OUT" | grep -q 'Project context'
! echo "$OUT" | grep -q 'Should not inject'

echo "golden-002 ok (issue start injects PROJECT_CONTEXT engineering section)"
