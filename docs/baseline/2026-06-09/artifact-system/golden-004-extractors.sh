#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p artifact-system golden_004_extractors_preserve_kind -- --nocapture
