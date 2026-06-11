#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

bash -n "packaging/macos/Install CLI.command"
bash -n packaging/macos/build-dmg.sh
bash -n packaging/macos/generate-icons.sh

grep -q '.local/bin' "packaging/macos/Install CLI.command"
grep -q 'POPSICLE_HOME\|\.popsicle' "packaging/macos/Install CLI.command" || grep -q '.popsicle' "packaging/macos/Install CLI.command"

grep -q 'ensure_silent_if_app_bundle' crates/cli-ux/src/main.rs
grep -q 'ensure_silent' crates/cli-ux/src/cli_install.rs
grep -q 'launched_from_app_bundle' crates/cli-ux/src/cli_install.rs

echo "golden-002 ok (macos install scripts + silent app-bundle CLI install)"
