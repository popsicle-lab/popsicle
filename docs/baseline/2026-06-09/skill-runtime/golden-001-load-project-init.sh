#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p skill-runtime golden_001_load_project_init_skill -- --nocapture
