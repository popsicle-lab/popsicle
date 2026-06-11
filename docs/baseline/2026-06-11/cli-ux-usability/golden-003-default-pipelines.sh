#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p cli-ux golden_010_issue_type_default_pipelines_are_bundled -- --nocapture
cargo test -p cli-ux golden_011_doc_check_and_issue_close_parse -- --nocapture
