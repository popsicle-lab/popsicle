#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo test -p artifact-system golden_006_upstream_port_requires_checker -- --nocapture
