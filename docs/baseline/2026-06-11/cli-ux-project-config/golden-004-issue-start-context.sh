#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-project-config-start.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

cd "$ROOT"
"$BIN" init --format json >/dev/null

KEY="$("$BIN" issue create --type bug --title "cfg smoke" --spec smoke-spec --pipeline bugfix --format json | sed -n 's/.*"key":"\([^"]*\)".*/\1/p')"
OUT="$("$BIN" issue start "$KEY" --format json)"
echo "$OUT" | grep -q '"agent_context"'
echo "$OUT" | grep -q 'Project preferences'

echo "golden-004 ok (issue start injects agent_context)"
