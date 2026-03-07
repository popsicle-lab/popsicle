#!/usr/bin/env python3
"""
Compare current classification with persisted state,
produce an action plan for incremental test execution.

Usage:
    python diff_state.py <classification.json> <state.json> -o action-plan.json

Actions:
    - generate+execute: new P0 or source changed
    - execute:          source unchanged but last run had failures
    - skip:             source unchanged and all passed
"""

import json
import hashlib
import argparse
from pathlib import Path
from datetime import datetime, timezone


def compute_source_hash(source_files: list[str]) -> str:
    """SHA256 of concatenated source file contents (sorted by path)."""
    hasher = hashlib.sha256()
    for fpath in sorted(source_files):
        p = Path(fpath)
        if p.exists():
            hasher.update(p.read_bytes())
        else:
            hasher.update(f"MISSING:{fpath}".encode())
    return hasher.hexdigest()[:16]


def build_action_plan(classification: dict, state: dict) -> list[dict]:
    """Determine action for each P0 feature based on state comparison."""
    state_features = state.get("features", {})
    plan = []

    for feature in classification.get("P0", []):
        fid = feature["id"]
        prev = state_features.get(fid)

        source_files = feature.get("source_files", [])
        current_hash = (
            compute_source_hash(source_files) if source_files else "unknown"
        )

        entry = {
            "id": fid,
            "name": feature.get("name", ""),
            "crate": feature.get("crate", ""),
            "source_hash": current_hash,
        }

        if prev is None:
            entry["action"] = "generate+execute"
            entry["reason"] = "new P0 feature"

        elif current_hash != prev.get("source_hash", ""):
            entry["action"] = "generate+execute"
            entry["reason"] = "source hash changed"
            entry["prev_hash"] = prev.get("source_hash")

        elif prev.get("result", {}).get("fail", 0) > 0:
            entry["action"] = "execute"
            entry["reason"] = "previous run had failures"

        else:
            entry["action"] = "skip"
            entry["reason"] = "unchanged and all passed"

        # Carry forward unresolved conflicts
        prev_conflicts = prev.get("conflicts", []) if prev else []
        unresolved = [c for c in prev_conflicts if c.get("resolution") == "C"]
        if unresolved:
            entry["pending_conflicts"] = len(unresolved)

        plan.append(entry)

    # Detect features that were P0 but no longer
    current_p0_ids = {f["id"] for f in classification.get("P0", [])}
    for fid, prev in state_features.items():
        if fid not in current_p0_ids and prev.get("priority") == "P0":
            plan.append({
                "id": fid,
                "name": prev.get("name", ""),
                "action": "skip",
                "reason": "downgraded from P0",
                "source_hash": prev.get("source_hash", ""),
            })

    return plan


def summarize_plan(plan: list[dict]) -> dict:
    """Count actions in the plan."""
    summary: dict[str, int] = {}
    for item in plan:
        a = item["action"]
        summary[a] = summary.get(a, 0) + 1
    return summary


def main():
    parser = argparse.ArgumentParser(
        description="Generate incremental action plan by comparing classification with state"
    )
    parser.add_argument(
        "classification",
        help="Classification JSON from classify.py",
    )
    parser.add_argument(
        "state",
        help="State JSON path (may not exist yet)",
    )
    parser.add_argument(
        "-o", "--output",
        default="/tmp/ptg-action-plan.json",
        help="Output action plan JSON path",
    )
    args = parser.parse_args()

    classification = json.loads(
        Path(args.classification).read_text(encoding="utf-8")
    )

    state_path = Path(args.state)
    if state_path.exists():
        state = json.loads(state_path.read_text(encoding="utf-8"))
    else:
        state = {}
        print(f"State file not found ({args.state}), treating all P0 as new")

    plan = build_action_plan(classification, state)
    summary = summarize_plan(plan)

    output = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "plan": plan,
        "summary": summary,
    }

    Path(args.output).write_text(
        json.dumps(output, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    total = len(plan)
    print(f"\nAction plan for {total} features:")
    for action in ["generate+execute", "execute", "skip"]:
        count = summary.get(action, 0)
        if count > 0:
            print(f"  {action:20s} {count}")

    # Show features that need work
    work_items = [i for i in plan if i["action"] != "skip"]
    if work_items:
        print(f"\nFeatures requiring action:")
        for item in work_items:
            print(f"  {item['id']:16s} → {item['action']:20s} ({item['reason']})")
    else:
        print(f"\nAll P0 features up to date. Nothing to execute.")

    print(f"\nOutput: {args.output}")


if __name__ == "__main__":
    main()
