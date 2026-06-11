#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p artifact-system golden_003_context_assembly_order -- --nocapture
