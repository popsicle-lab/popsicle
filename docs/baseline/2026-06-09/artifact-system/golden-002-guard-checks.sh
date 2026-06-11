#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p artifact-system golden_002_guard_sections_and_checklist -- --nocapture
