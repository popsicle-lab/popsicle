---
task_id: T-0002
slug: advance-stage
title: "我作为 AI agent 把 pipeline 推到下一个 stage（含审批暂停点）"
journey_stage: daily-ops
audience: ["ai-coding-agent", "human-maintainer"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-06-08
intent_kind: orchestrate
involved_features: ["stage-state-machine", "requires-approval-gate"]
prerequisites:
  - "至少有一个 pipeline run 处于 in_progress（见 [T-0001](../onboarding/T-0001-first-pipeline-run.md)）"
limits:
  - "一次只推一个 stage（state-machine 只允许向前转移）"
  - "requires_approval 的 stage 必须人类 / agent 显式 --confirm"
related_intents:
  - "acceptance.intent#StageAdvanceWithApproval"
  - "invariants.intent#stage-transitions-forward-only"   # ⚠ orphan：此 invariant 从未形式化，待人工（见 living-doc-sync-report）
related_next_tasks:
  - T-0003
  - T-0004
fact_cite:
  - "PDR-001 §Phase 4 表 1 (T01-T06 各阶段推进任务)"
  - "fact-extraction-report § api-contracts.md §popsicle-cli §pipeline stage"
---

# 我作为 AI agent 把 pipeline 推到下一个 stage（含审批暂停点）

---

## 本 task 可解答

- "怎么把一个 stage 标记为 completed？"
- "`stage complete` 一定要带 `--confirm` 吗？"
- "审批点 `requires_approval` 没过会怎么样？"

---

## 前提与限制

**你需要先**：
- 有一个 in_progress 的 pipeline run，且当前 stage 已经完成所有 artifact + 全部 checklist `4/6 → 6/6`

**本 task 受以下限制**：
- state transition 只允许向前（`pending → in_progress → completed`），向后 / 跳过的需求走 [T-0004](../troubleshooting/T-0004-recover-blocked.md)
- 一个 stage 内若有多个 doc，全部 doc 的 checklist 都必须勾全

---

## 完成路径

1. **检查当前 stage 的 doc checklist 是否全勾**：

   ```bash
   popsicle doc check status --run <run-id>
   ```

   预期：当前 stage 名下所有 doc 显示 `N/N checked`。任意一个未勾 → 不可推进，回去补。

2. **判断当前 stage 是否需要审批**：

   ```bash
   popsicle pipeline status
   ```

   stage 名称右侧带 `[requires_approval]` 标记的就是需要 `--confirm`。`init` / `debate` / `review` / `living-docs` 默认带；`facts` / `arch-debate` 不带。

3. **执行 stage complete**：

   - 普通 stage：

     ```bash
     popsicle pipeline stage complete <stage-name>
     ```

   - 需要审批的 stage：

     ```bash
     popsicle pipeline stage complete <stage-name> --confirm
     ```

   CLI 返回 `Stage '<name>' → completed; Unblocked: <next-stage>`。

4. **下一 stage 自动起 skill**（若 pipeline 模板配置了 auto-start）：

   ```bash
   popsicle pipeline status
   ```

   预期下一 stage 已从 `blocked` → `ready` 或 `in_progress`。

---

## 可观察的成功标志

`popsicle pipeline status` 中当前 stage 标记 `completed`，下一 stage 标记 `ready` 或 `in_progress`，无 stage 处于 `error`。

形式化定义：见 [`acceptance.intent#T-0002-stage-advance-idempotent`](../../intents/acceptance.intent)

---

## Related Next Tasks

- **[T-0003 - 我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态](T-0003-inspect-state.md)** — 推完后看看推到哪了
- **[T-0004 - 我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复](../troubleshooting/T-0004-recover-blocked.md)** — 推不动了走这里

---

## Charter Compliance

- [x] frontmatter 8 个必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 第一人称
- [x] 完成路径无 if-else 分支（审批/非审批是参数差异，不是分支）

---

`Decision-Ref: PDR-002`
