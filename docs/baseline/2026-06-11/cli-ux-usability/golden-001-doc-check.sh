#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux tsv_doc_check_fails_stub_and_passes_filled_doc -- --nocapture
