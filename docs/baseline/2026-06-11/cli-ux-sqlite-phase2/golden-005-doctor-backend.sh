#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

# The real workspace was migrated via `admin migrate` (PROJ-25 dogfood):
# doctor must report the SQLite backend, and the legacy popsicle.db (legacy
# binary's database) must remain untouched at .popsicle/popsicle.db.
OUT="$(./target/debug/popsicle doctor --format json)"
echo "$OUT" | grep -q '"storage_backend":"sqlite (.popsicle/state.db)"' \
  || { echo "FAIL: doctor does not report sqlite backend: $OUT" >&2; exit 1; }
test -f .popsicle/state.db || { echo "FAIL: state.db missing" >&2; exit 1; }
echo "golden-005 ok (doctor reports sqlite backend)"
