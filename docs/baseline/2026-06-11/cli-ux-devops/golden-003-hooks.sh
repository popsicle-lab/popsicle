#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

bash -n hooks/pre-commit
make install-hooks
test -x .git/hooks/pre-commit
echo "golden-003 ok (pre-commit hook installable)"
