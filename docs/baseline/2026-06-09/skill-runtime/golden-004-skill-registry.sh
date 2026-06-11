#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p skill-runtime golden_004_skill_registry_count -- --nocapture
