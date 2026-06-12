#!/usr/bin/env python3
"""One-shot: link historical issues to cli-ux tasks; remove mock issues."""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

# issue_key -> ordered linked task ids (cli-ux product)
# Rationale: semantic review 2026-06-11 — title + description vs task frontmatter
ISSUE_TASK_MAP: dict[str, list[str]] = {
    # PDR-001 命令面 / 移除清单对齐
    "PROJ-17": ["T-CU-0007"],
    # doc check + issue close + default pipelines
    "PROJ-24": ["T-CU-0002", "T-CU-0003", "T-CU-0004"],
    # SQLite backend: admin migrate + doctor storage_backend
    "PROJ-25": ["T-CU-0006", "T-CU-0008"],
    # Makefile/golden/hooks + install.sh/DMG chain
    "PROJ-26": ["T-CU-0008", "T-CU-0012"],
    # Tauri MVP: UI shell + issue/doc/pipeline 四页
    "PROJ-27": ["T-CU-0010", "T-CU-0002", "T-CU-0003", "T-CU-0004"],
    # Product explorer 无独立 task；guidance 归 T-CU-0002
    "PROJ-28": ["T-CU-0002"],
    "PROJ-29": ["T-CU-0009"],
    # UI switcher 消费 global registry
    "PROJ-30": ["T-CU-0010", "T-CU-0009"],
    "PROJ-34": ["T-CU-0011"],
    # retro spec：为多项目/UI/DMG/intent-coder 补 task+intent
    "PROJ-35": ["T-CU-0008", "T-CU-0009", "T-CU-0010", "T-CU-0011", "T-CU-0012"],
    # ADR-018 布局无独立 task；最近 UI onboarding 最接近
    "PROJ-36": ["T-CU-0010"],
    "PROJ-37": ["T-CU-0013"],
    "PROJ-38": ["T-CU-0002"],
    "PROJ-39": ["T-CU-0013"],
    # 首启静默装 CLI → DMG/install 旅程
    "PROJ-40": ["T-CU-0012"],
    "PROJ-41": ["T-CU-0012"],
    "PROJ-42": ["T-CU-0002", "T-CU-0013"],
    "PROJ-43": ["T-CU-0002"],
}

MOCK_ISSUE_KEYS = frozenset({"PROJ-44"})


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    db_path = root / ".popsicle/self-host/state.db"
    if not db_path.is_file():
        print(f"error: state db not found: {db_path}", file=sys.stderr)
        return 1

    conn = sqlite3.connect(db_path)
    try:
        conn.execute(
            """
            CREATE TABLE IF NOT EXISTS issue_tasks (
                issue_key      TEXT NOT NULL,
                sort_order     INTEGER NOT NULL,
                role           TEXT NOT NULL,
                task_id        TEXT,
                proposed_title TEXT,
                journey_stage  TEXT,
                source         TEXT NOT NULL,
                PRIMARY KEY (issue_key, sort_order)
            )
            """
        )

        existing = {
            row[0]
            for row in conn.execute("SELECT key FROM issues").fetchall()
        }

        for mock in MOCK_ISSUE_KEYS:
            if mock not in existing:
                continue
            conn.execute("DELETE FROM issue_tasks WHERE issue_key = ?", (mock,))
            conn.execute("DELETE FROM runs WHERE issue_key = ?", (mock,))
            conn.execute("DELETE FROM issues WHERE key = ?", (mock,))
            print(f"deleted mock issue {mock}")

        for issue_key, task_ids in ISSUE_TASK_MAP.items():
            if issue_key not in existing:
                print(f"skip missing issue {issue_key}")
                continue
            conn.execute("DELETE FROM issue_tasks WHERE issue_key = ?", (issue_key,))
            for sort_order, task_id in enumerate(task_ids):
                conn.execute(
                    """
                    INSERT INTO issue_tasks
                    (issue_key, sort_order, role, task_id, proposed_title, journey_stage, source)
                    VALUES (?, ?, 'linked', ?, NULL, NULL, 'history-backfill')
                    """,
                    (issue_key, sort_order, task_id),
                )
            first = task_ids[0] if task_ids else None
            conn.execute(
                "UPDATE issues SET epic_task_id = ? WHERE key = ?",
                (first, issue_key),
            )
            print(f"linked {issue_key} -> {', '.join(task_ids)}")

        conn.commit()
    finally:
        conn.close()

    print("done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
