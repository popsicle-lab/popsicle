#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

HOME_DIR="$(mktemp -d /tmp/popsicle-ui-startup-home.XXXXXX)"
PROJ_A="$(mktemp -d /tmp/popsicle-ui-startup-a.XXXXXX)"
PROJ_B="$(mktemp -d /tmp/popsicle-ui-startup-b.XXXXXX)"
trap 'rm -rf "$HOME_DIR" "$PROJ_A" "$PROJ_B"' EXIT

mkdir -p "$PROJ_A/.popsicle" "$PROJ_B/.popsicle"
export POPSICLE_HOME="$HOME_DIR"

cargo run -p cli-ux --bin popsicle --quiet -- project add "$PROJ_A" --name alpha --format json >/dev/null
cargo run -p cli-ux --bin popsicle --quiet -- project add "$PROJ_B" --name beta --format json >/dev/null
cargo run -p cli-ux --bin popsicle --quiet -- project use beta --format json >/dev/null

# Touch beta as most recent via open_project path in lib (through cargo test helper)
cargo test -p cli-ux --test global_config project_add_use_and_default_resolution -q

OUT="$(cargo run -p cli-ux --bin popsicle --quiet -- project list --format json)"
echo "$OUT" | grep -q '"status":"ok"'
echo "$OUT" | grep -q 'alpha'
echo "$OUT" | grep -q 'beta'

echo "golden-002 ok (project registry + list for UI startup)"
