#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_003_doc_create_writes_artifact_and_document_row -- --nocapture
