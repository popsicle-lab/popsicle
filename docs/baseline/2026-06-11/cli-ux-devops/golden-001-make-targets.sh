#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

# The legacy check trio must pass in the new workspace.
make fmt
make clippy
echo "golden-001 ok (fmt + clippy clean)"
