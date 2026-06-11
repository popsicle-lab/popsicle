#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

if [ -x "./target/debug/popsicle" ]; then
  POPSICLE=./target/debug/popsicle
else
  POPSICLE=./popsicle
fi

cargo build -p cli-ux >/dev/null
out=$("$POPSICLE" help)
echo "$out" | grep -q '^ui$' || {
  echo "help must list ui top-level command" >&2
  exit 1
}
echo "$out" | grep -q 'ui \[--project' || {
  echo "usage must document ui [--project <path>]" >&2
  exit 1
}
echo "golden-001 ok"
