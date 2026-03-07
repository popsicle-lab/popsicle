#!/usr/bin/env python3
"""
Parse PRODUCT_FEATURE_MAP.md and TECHNICAL_REQUIREMENTS.md,
classify features into P0/P1/P2.

Usage:
    python classify.py <feature_map.md> [tech_req.md] -o output.json
"""

import re
import json
import argparse
from pathlib import Path

P0_MODULE_PREFIXES = {"ENG", "META", "QRY", "DEV", "SDW", "AUTH"}

P0_PRODUCT_MODULE_COMBOS = {("ST", "TNT")}

ACTIVE_STATUSES = {"done", "partial"}
STUB_STATUSES = {"stub", "planned"}

UPGRADE_E2E_NO_UNIT = True


def parse_feature_table(md_content: str) -> list[dict]:
    """Extract features from markdown tables.

    Expected table format (7 columns):
    | ID | 功能 | 描述 | 状态 | 边界 | 测试 | 关联文档 |
    """
    features = []

    # Match lines like: | ST-ENG-001 | ... | `done` | ... |
    # Also handles technical IDs like NFR-PERF-001
    line_pattern = re.compile(
        r"^\|\s*"
        r"((?:ST|VR|AR|AN|CO|PL|NFR|INF|SEC|TDT)-[A-Z]+-\d+)"
        r"\s*\|"
    )

    for line in md_content.splitlines():
        m = line_pattern.match(line)
        if not m:
            continue

        fid = m.group(1).strip()
        cells = [c.strip() for c in line.split("|")]
        # Remove empty first/last from split
        cells = [c for c in cells if c]

        if len(cells) < 6:
            continue

        name = cells[1]

        # Extract status from backtick-wrapped value
        status_match = re.search(r"`(\w+)`", cells[3])
        status = status_match.group(1) if status_match else cells[3]

        boundary = cells[4] if len(cells) > 4 else ""
        test_cov = cells[5] if len(cells) > 5 else ""

        parts = fid.split("-")
        product_prefix = parts[0]
        module_prefix = parts[1]
        seq = parts[2]

        features.append({
            "id": fid,
            "name": name,
            "status": status,
            "boundary": boundary,
            "test_coverage": test_cov,
            "product_prefix": product_prefix,
            "module_prefix": module_prefix,
            "seq": seq,
        })

    return features


def is_p0(feature: dict) -> bool:
    """Check if a feature qualifies as P0."""
    prod = feature["product_prefix"]
    mod = feature["module_prefix"]
    test_cov = feature["test_coverage"]

    # Core module path check
    if mod in P0_MODULE_PREFIXES:
        return True

    # Special product+module combos
    if (prod, mod) in P0_PRODUCT_MODULE_COMBOS:
        return True

    # Upgrade: has e2e but no unit test
    if UPGRADE_E2E_NO_UNIT:
        has_e2e = "e2e" in test_cov.lower()
        has_unit = "unit" in test_cov.lower()
        if has_e2e and not has_unit:
            return True

    return False


def classify(features: list[dict]) -> dict:
    """Classify features into P0/P1/P2."""
    p0, p1, p2 = [], [], []

    for f in features:
        status = f["status"]

        if status in STUB_STATUSES or status not in ACTIVE_STATUSES:
            f["priority"] = "P2"
            p2.append(f)
            continue

        if is_p0(f):
            f["priority"] = "P0"
            p0.append(f)
        else:
            f["priority"] = "P1"
            p1.append(f)

    return {
        "P0": p0,
        "P1": p1,
        "P2": p2,
        "summary": {
            "total": len(features),
            "P0": len(p0),
            "P1": len(p1),
            "P2": len(p2),
        },
    }


def main():
    parser = argparse.ArgumentParser(
        description="Classify features into P0/P1/P2 based on feature map documents"
    )
    parser.add_argument(
        "files",
        nargs="+",
        help="Markdown feature map files (PRODUCT_FEATURE_MAP.md, TECHNICAL_REQUIREMENTS.md)",
    )
    parser.add_argument(
        "-o", "--output",
        default="/tmp/ptg-classification.json",
        help="Output JSON path (default: /tmp/ptg-classification.json)",
    )
    args = parser.parse_args()

    all_features = []
    for fpath in args.files:
        p = Path(fpath)
        if not p.exists():
            print(f"Warning: {fpath} not found, skipping")
            continue
        content = p.read_text(encoding="utf-8")
        parsed = parse_feature_table(content)
        all_features.extend(parsed)
        print(f"Parsed {len(parsed)} features from {fpath}")

    if not all_features:
        print("No features found. Check input file format.")
        return

    result = classify(all_features)

    Path(args.output).write_text(
        json.dumps(result, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    print(f"\nClassified {result['summary']['total']} features:")
    print(f"  P0 (core path):    {result['summary']['P0']}")
    print(f"  P1 (supporting):   {result['summary']['P1']}")
    print(f"  P2 (future/stub):  {result['summary']['P2']}")
    print(f"\nOutput: {args.output}")

    if result["P0"]:
        print(f"\nP0 features:")
        for f in result["P0"]:
            print(f"  {f['id']:16s} {f['name']}")


if __name__ == "__main__":
    main()
