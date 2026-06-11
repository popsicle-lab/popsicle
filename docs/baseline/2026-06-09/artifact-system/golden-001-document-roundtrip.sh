#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p artifact-system golden_001_document_roundtrip -- --nocapture
