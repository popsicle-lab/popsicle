#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p skill-runtime golden_005_canonical_state_machine -- --nocapture
