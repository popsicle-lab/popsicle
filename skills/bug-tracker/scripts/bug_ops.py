#!/usr/bin/env python3
"""Bug Tracker — CRUD + 去重检查 + 报告生成"""

import argparse
import json
import sys
from datetime import datetime, timezone
from difflib import SequenceMatcher
from pathlib import Path

STATE_FILE = Path(__file__).resolve().parent.parent / "state" / "bugs.json"

# ─── helpers ───


def load_state() -> dict:
    if STATE_FILE.exists():
        return json.loads(STATE_FILE.read_text(encoding="utf-8"))
    return {"version": 1, "bugs": []}


def save_state(state: dict):
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    state["updated_at"] = utcnow()
    STATE_FILE.write_text(
        json.dumps(state, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def utcnow() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def next_bug_id(state: dict) -> str:
    year = datetime.now(timezone.utc).year
    existing = [
        b["bug_id"]
        for b in state["bugs"]
        if b["bug_id"].startswith(f"BUG-{year}-")
    ]
    if not existing:
        return f"BUG-{year}-0001"
    max_seq = max(int(bid.split("-")[-1]) for bid in existing)
    return f"BUG-{year}-{max_seq + 1:04d}"


def similarity(a: str, b: str) -> float:
    return SequenceMatcher(None, a.lower(), b.lower()).ratio()


# ─── dedup ───


def find_duplicate(state: dict, module: str, title: str = "",
                   code_location: str = "", test_name: str = "") -> dict | None:
    """按优先级匹配已有 Bug，返回匹配到的 Bug 或 None"""
    for bug in state["bugs"]:
        # Rule 1: code_location exact match
        if (code_location
                and bug.get("evidence", {}).get("code_location") == code_location):
            return bug
        # Rule 2: module + test_name
        if (test_name
                and bug.get("module") == module
                and bug.get("source", {}).get("test_name") == test_name):
            return bug
        # Rule 3: module + title similarity > 80%
        if (title
                and bug.get("module") == module
                and similarity(bug.get("title", ""), title) > 0.8):
            return bug
    return None


def reopen_bug(bug: dict, new_evidence: dict | None = None):
    """将已有 Bug 状态回退为 pending，更新时间戳"""
    bug["status"] = "pending"
    bug["updated_at"] = utcnow()
    if new_evidence:
        ev = bug.setdefault("evidence", {})
        for k, v in new_evidence.items():
            if v and v != ev.get(k):
                ev[k] = v


# ─── commands ───


def cmd_add(args):
    state = load_state()

    dup = find_duplicate(
        state,
        module=args.module,
        title=args.title,
        code_location=args.code_location or "",
        test_name=args.test_name or "",
    )

    if dup:
        reopen_bug(dup, {
            "log_snippet": args.log_snippet,
            "code_location": args.code_location,
        })
        save_state(state)
        print(json.dumps({
            "action": "reopened",
            "bug_id": dup["bug_id"],
            "title": dup["title"],
            "status": "pending",
        }, ensure_ascii=False))
        return

    now = utcnow()
    bug = {
        "bug_id": next_bug_id(state),
        "title": args.title,
        "status": "pending",
        "severity": args.severity or "medium",
        "module": args.module,
        "created_at": now,
        "updated_at": now,
        "source": {
            "test_file": args.test_file or "",
            "test_name": args.test_name or "",
            "test_framework": args.test_framework or "",
        },
        "reproduction": {
            "preconditions": split_list(args.preconditions),
            "steps": split_list(args.steps),
            "expected": args.expected or "",
            "actual": args.actual or "",
        },
        "evidence": {
            "log_snippet": args.log_snippet or "",
            "code_location": args.code_location or "",
            "related_code": args.related_code or "",
        },
        "tags": split_list(args.tags),
    }
    state["bugs"].append(bug)
    save_state(state)
    print(json.dumps({
        "action": "created",
        "bug_id": bug["bug_id"],
        "title": bug["title"],
        "status": "pending",
    }, ensure_ascii=False))


def cmd_match(args):
    state = load_state()
    dup = find_duplicate(
        state,
        module=args.module or "",
        title=args.title or "",
        code_location=args.code_location or "",
        test_name=args.test_name or "",
    )
    if dup:
        print(json.dumps({
            "matched": True,
            "bug_id": dup["bug_id"],
            "title": dup["title"],
            "status": dup["status"],
        }, ensure_ascii=False))
    else:
        print(json.dumps({"matched": False}, ensure_ascii=False))


def cmd_update(args):
    state = load_state()
    for bug in state["bugs"]:
        if bug["bug_id"] == args.bug_id:
            if args.status:
                bug["status"] = args.status
            bug["updated_at"] = utcnow()
            save_state(state)
            print(json.dumps({
                "action": "updated",
                "bug_id": bug["bug_id"],
                "status": bug["status"],
            }, ensure_ascii=False))
            return
    print(json.dumps({"error": f"Bug {args.bug_id} not found"}), file=sys.stderr)
    sys.exit(1)


def cmd_list(args):
    state = load_state()
    bugs = state["bugs"]
    if args.status:
        bugs = [b for b in bugs if b["status"] == args.status]
    if args.module:
        bugs = [b for b in bugs if b["module"] == args.module]

    for b in bugs:
        print(f"[{b['bug_id']}] [{b['status']}] [{b['severity']}] "
              f"{b['module']} — {b['title']}")

    if not bugs:
        print("(no bugs matching filter)")


def cmd_stats(args):
    state = load_state()
    bugs = state["bugs"]

    by_status: dict[str, int] = {}
    by_module: dict[str, dict[str, int]] = {}
    for b in bugs:
        by_status[b["status"]] = by_status.get(b["status"], 0) + 1
        mod = b["module"]
        if mod not in by_module:
            by_module[mod] = {}
        by_module[mod][b["status"]] = by_module[mod].get(b["status"], 0) + 1

    print(f"Total: {len(bugs)}")
    print("\n=== By Status ===")
    for s, c in sorted(by_status.items()):
        print(f"  {s}: {c}")
    print("\n=== By Module ===")
    for m, statuses in sorted(by_module.items()):
        total = sum(statuses.values())
        detail = ", ".join(f"{s}={c}" for s, c in sorted(statuses.items()))
        print(f"  {m}: {total} ({detail})")


def cmd_report(args):
    state = load_state()
    bugs = state["bugs"]
    now = utcnow()

    by_status: dict[str, int] = {}
    by_module: dict[str, dict[str, int]] = {}
    for b in bugs:
        by_status[b["status"]] = by_status.get(b["status"], 0) + 1
        mod = b["module"]
        if mod not in by_module:
            by_module[mod] = {}
        by_module[mod][b["status"]] = by_module[mod].get(b["status"], 0) + 1

    all_statuses = sorted(by_status.keys())

    lines = [
        f"# Bug Registry",
        f"",
        f"> 自动生成于 {now}，共 {len(bugs)} 条 Bug",
        f"",
        f"## 按状态统计",
        f"",
        f"| 状态 | 数量 |",
        f"|------|------|",
    ]
    for s in all_statuses:
        lines.append(f"| {s} | {by_status[s]} |")

    lines += [
        f"",
        f"## 按模块分布",
        f"",
    ]
    header = "| 模块 | " + " | ".join(all_statuses) + " | 总计 |"
    sep = "|------" + "|-------" * len(all_statuses) + "|------|"
    lines.append(header)
    lines.append(sep)
    for mod in sorted(by_module.keys()):
        cells = [str(by_module[mod].get(s, 0)) for s in all_statuses]
        total = sum(by_module[mod].values())
        lines.append(f"| {mod} | " + " | ".join(cells) + f" | {total} |")

    lines += [f"", f"## Bug 明细", f""]
    for b in bugs:
        lines.append(f"### {b['bug_id']} [{b['status']}] [{b['severity']}]")
        lines.append(f"**{b['title']}**")
        lines.append(f"- 模块: {b['module']}")
        lines.append(f"- 创建: {b['created_at']}")
        if b.get("updated_at") != b.get("created_at"):
            lines.append(f"- 更新: {b['updated_at']}")

        repro = b.get("reproduction", {})
        if repro.get("steps"):
            steps_str = " → ".join(repro["steps"])
            lines.append(f"- 复现: {steps_str}")
        if repro.get("expected"):
            lines.append(f"- 期望: {repro['expected']}")
        if repro.get("actual"):
            lines.append(f"- 实际: {repro['actual']}")

        ev = b.get("evidence", {})
        if ev.get("code_location"):
            lines.append(f"- 代码位置: `{ev['code_location']}`")
        if ev.get("related_code"):
            lines.append(f"- 根因: {ev['related_code']}")

        if b.get("tags"):
            lines.append(f"- 标签: {', '.join(b['tags'])}")
        lines.append(f"")

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Report written to {output} ({len(bugs)} bugs)")


# ─── utils ───


def split_list(val: str | None) -> list[str]:
    if not val:
        return []
    return [s.strip() for s in val.split("|") if s.strip()]


# ─── CLI ───


def main():
    parser = argparse.ArgumentParser(description="Bug Tracker CLI")
    sub = parser.add_subparsers(dest="command", required=True)

    # add
    p_add = sub.add_parser("add", help="Add or reopen a bug")
    p_add.add_argument("--module", required=True)
    p_add.add_argument("--title", required=True)
    p_add.add_argument("--severity", default="medium",
                       choices=["critical", "high", "medium", "low"])
    p_add.add_argument("--test-file")
    p_add.add_argument("--test-name")
    p_add.add_argument("--test-framework")
    p_add.add_argument("--preconditions", help="pipe-separated list")
    p_add.add_argument("--steps", help="pipe-separated list")
    p_add.add_argument("--expected")
    p_add.add_argument("--actual")
    p_add.add_argument("--log-snippet")
    p_add.add_argument("--code-location")
    p_add.add_argument("--related-code")
    p_add.add_argument("--tags", help="comma-separated")

    # match
    p_match = sub.add_parser("match", help="Check for duplicate")
    p_match.add_argument("--module", default="")
    p_match.add_argument("--title", default="")
    p_match.add_argument("--code-location", default="")
    p_match.add_argument("--test-name", default="")

    # update
    p_upd = sub.add_parser("update", help="Update bug status")
    p_upd.add_argument("bug_id")
    p_upd.add_argument("--status",
                       choices=["pending", "confirmed", "fixing",
                                "resolved", "verified", "closed", "wontfix"])

    # list
    p_ls = sub.add_parser("list", help="List bugs")
    p_ls.add_argument("--status")
    p_ls.add_argument("--module")

    # stats
    sub.add_parser("stats", help="Bug statistics")

    # report
    p_rpt = sub.add_parser("report", help="Generate markdown report")
    p_rpt.add_argument("--output", required=True)

    args = parser.parse_args()
    dispatch = {
        "add": cmd_add,
        "match": cmd_match,
        "update": cmd_update,
        "list": cmd_list,
        "stats": cmd_stats,
        "report": cmd_report,
    }
    dispatch[args.command](args)


if __name__ == "__main__":
    main()
