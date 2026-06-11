#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

bash -n "packaging/macos/Install CLI.command"
bash -n packaging/macos/build-dmg.sh
bash -n packaging/macos/generate-icons.sh

grep -q '.local/bin' "packaging/macos/Install CLI.command"
grep -q 'POPSICLE_HOME\|\.popsicle' "packaging/macos/Install CLI.command" || grep -q '.popsicle' "packaging/macos/Install CLI.command"

echo "golden-002 ok (macos install scripts syntax + PATH)"
