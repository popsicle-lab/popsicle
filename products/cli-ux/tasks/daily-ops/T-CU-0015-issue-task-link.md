---
task_id: T-CU-0015
slug: issue-task-link
title: "我维护 Issue 与 Task 的关联（issue link）"
journey_stage: daily-ops
audience: ["ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: PDR-005
last_updated: 2026-06-11
last_verified: ~
intent_kind: validate
involved_features: ["issue", "issue_tasks"]
prerequisites:
  - "已有 Issue 与 products 下 task 文件"
limits:
  - "不修改 task 正文，只改 issue_tasks 关联"
related_intents:
  - "acceptance.intent#IssueTaskLinksMutable"
related_next_tasks:
  - T-CU-0002
fact_cite:
  - "products/cli-ux/decisions/adr/ADR-025-issue-task-link-command.md"
---

# 我维护 Issue 与 Task 的关联（issue link）

## 本 task 可解答

- "task 文件已落地，怎么把 Issue 从 proposed 改成 linked？"
- "Issue 错链了别的 task，怎么改？"
- "living-doc 晋升后怎么回写 Issue？"

## 完成路径

1. 确认 `products/<product>/tasks/.../T-XXXX.md` 存在。
2. 运行：
   ```bash
   popsicle issue link <KEY> --tasks T-CU-0014 --replace --drop-proposed
   ```
3. `popsicle issue show <KEY> --format json` 核对 `task_link_*_role=linked`。
4. 若准备 `slice-delivery`，确保 `--description` 含每个 linked `task_id`。

## 可观察的成功标志

- `links_updated: true`，`linked_count` ≥ 1。
- 无效 task id 返回 `NotFound`。

## Charter Compliance

- `acceptance.intent#IssueTaskLinksMutable`
