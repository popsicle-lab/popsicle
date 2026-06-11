#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p skill-runtime golden_002_load_migration_bootstrap_pipeline -- --nocapture
