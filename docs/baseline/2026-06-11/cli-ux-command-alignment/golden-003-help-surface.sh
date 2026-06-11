#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_009_help_advertises_only_implemented_commands -- --nocapture

BIN=./target/debug/popsicle
[ -x "$BIN" ] || cargo build -p cli-ux -q

HELP_JSON="$($BIN help --format json)"
echo "$HELP_JSON" | grep -q '"usage"'
echo "$HELP_JSON" | grep -q '"deferred_commands"'
# Advertised command list must not contain deferred names.
COMMANDS_FIELD="$($BIN help | sed -n '/^commands:/,/^deferred_commands:/p')"
for deferred in module skill spec namespace prompt git memory context registry completions; do
  if echo "$COMMANDS_FIELD" | grep -qx "$deferred"; then
    echo "FAIL: help advertises deferred command: $deferred" >&2
    exit 1
  fi
done
echo "golden-003 ok"
