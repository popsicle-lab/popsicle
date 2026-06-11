#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p skill-runtime golden_003_load_slice_delivery_pipeline -- --nocapture
