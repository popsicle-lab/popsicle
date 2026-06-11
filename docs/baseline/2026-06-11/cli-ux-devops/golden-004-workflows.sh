#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

python3 - <<'EOF'
import yaml

ci = yaml.safe_load(open(".github/workflows/ci.yml"))
steps = str(ci["jobs"]["check"]["steps"])
assert "fmt" in steps and "clippy" in steps and "test" in steps, "ci.yml missing check trio"
assert "npm" not in steps and "webkit" not in steps, "ci.yml still has legacy UI deps"

rel = yaml.safe_load(open(".github/workflows/release.yml"))
targets = [m["target"] for m in rel["jobs"]["build"]["strategy"]["matrix"]["include"]]
assert "aarch64-apple-darwin" in targets and "x86_64-unknown-linux-gnu" in targets, "release matrix incomplete"
assert "release" in rel["jobs"], "release job missing"

print("golden-004 ok (workflows valid and adapted)")
EOF
