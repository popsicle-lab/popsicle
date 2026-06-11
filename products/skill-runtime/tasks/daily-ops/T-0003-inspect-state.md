---
task_id: T-0003
slug: inspect-state
title: "我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态"
journey_stage: daily-ops
audience: ["ai-coding-agent", "human-maintainer"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-06-08
intent_kind: inspect
involved_features: ["pipeline-status", "skill-list", "stage-show", "doc-list"]
prerequisites:
  - "有至少一个 pipeline run（不要求 in_progress）"
limits:
  - "本 task 全部为 read-only 命令（不改变任何状态）"
related_intents:
  - "invariants.intent#read-only-commands-do-not-mutate-state"   # ⚠ orphan：intent-spec 把此约束降级为 task 级断言（全局 read-only safety 会证伪所有 mutating 操作），.intent 中无此 block。待人工决定 re-point/drop
related_next_tasks:
  - T-0002
  - T-0005
fact_cite:
  - "fact-extraction-report § api-contracts.md §popsicle-cli §status / list / show 命令族"
---

# 我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态

---

## 本 task 可解答

- "怎么知道一个 pipeline run 现在到哪个 stage 了？"
- "popsicle 里 skill 有哪些？每个 skill 输入输出是什么？"
- "当前 stage 下有几个 doc？checklist 勾了几个？"

---

## 前提与限制

**你需要先**：
- 知道 pipeline run-id（或接受默认 `--run` 取最新 run）

**本 task 受以下限制**：
- 全部命令为 **read-only**（INV-T0003：read-only 不改变状态）
- 不暴露任何写入语义；写入操作分散在其它 task

---

## 完成路径

1. **看 pipeline 全局状态**（最常用）：

   ```bash
   popsicle pipeline status
   ```

   返回：run id / pipeline 名 / 所有 stage 的状态 + 各 stage 下的 doc 概要。

2. **看 skill 清单**（哪些 skill 当前可用 + 输入输出）：

   ```bash
   popsicle skill list
   popsicle skill show <skill-name>
   ```

   `show` 返回 skill 的 inputs / artifacts / state machine。

3. **看 doc checklist**（当前 stage 下 checklist 勾选进度）：

   ```bash
   popsicle doc check status --run <run-id>
   ```

   每行显示 `📋 <title> (<doc-id>) — N/M checked` + 未勾项列表。

4. **看单个 doc 详细信息**（含 frontmatter + body 摘要）：

   ```bash
   popsicle doc show <doc-id>
   ```

5. **看下一步推荐**（pipeline 自身建议）：

   ```bash
   popsicle pipeline next
   ```

   返回当前 run 下一步动作建议（含可执行命令）。

---

## 可观察的成功标志

任一查询命令成功返回结构化输出，且 `popsicle pipeline status` 不报 error。

形式化定义：见 [`invariants.intent#read-only-commands-do-not-mutate-state`](../../intents/invariants.intent)

---

## Related Next Tasks

- **[T-0002 - 把 pipeline 推到下一个 stage](T-0002-advance-stage.md)** — 看清楚后开始推
- **[T-0005 - 审查产物链做合规复盘](../admin/T-0005-audit-trail.md)** — 深度审查

---

## Charter Compliance

- [x] frontmatter 8 个必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 第一人称
- [x] 完成路径无 if-else

---

`Decision-Ref: PDR-002`
