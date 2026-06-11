#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

HOME_DIR="$(mktemp -d /tmp/popsicle-global-golden-home.XXXXXX)"
PROJ_A="$(mktemp -d /tmp/popsicle-global-golden-a.XXXXXX)"
PROJ_B="$(mktemp -d /tmp/popsicle-global-golden-b.XXXXXX)"
trap 'rm -rf "$HOME_DIR" "$PROJ_A" "$PROJ_B"' EXIT

mkdir -p "$PROJ_A/.popsicle/self-host" "$PROJ_B/.popsicle/self-host"
export POPSICLE_HOME="$HOME_DIR"
unset POPSICLE_PROJECT || true

"$BIN" project add "$PROJ_A" --name alpha --format json | grep -q '"status":"ok"'
"$BIN" project add "$PROJ_B" --name beta --format json | grep -q '"status":"ok"'
"$BIN" project use alpha --format json | grep -q '"status":"ok"'

OUT="$("$BIN" issue list --format json)"
echo "$OUT" | grep -q '"status":"ok"'

OVERRIDE="$("$BIN" issue list --project "$PROJ_B" --format json)"
echo "$OVERRIDE" | grep -q '"status":"ok"'

echo "golden-001 ok (project registry + --project override)"
