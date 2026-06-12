#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux scan_cli_ux_product_health_ok -- --nocapture
echo "golden-003 ok (scan_product_health for cli-ux)"
