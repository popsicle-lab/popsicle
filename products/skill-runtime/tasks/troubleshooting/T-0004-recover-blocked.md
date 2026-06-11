---
task_id: T-0004
slug: recover-blocked
title: "我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复"
journey_stage: troubleshooting
audience: ["ai-coding-agent", "human-maintainer"]
task_type: 故障排查
decision_ref: PDR-002
last_updated: 2026-06-08
last_verified: 2026-06-08   # living-doc-author 回填：related_intent 经 intent-check Z3 verified
intent_kind: recover
involved_features: ["pipeline-unlock", "stage-state-rewind", "issue-restart"]
prerequisites:
  - "有一个 pipeline run，且至少一个 stage 处于 blocked / error 状态"
limits:
  - "不允许直接编辑 .popsicle/ 数据库；所有恢复走 CLI"
  - "已 completed 的 stage 不可回滚（charter 铁律 #2：决策档案只追加）"
related_intents:
  - "acceptance.intent#RecoveredPipelineCanAdvance"
related_next_tasks:
  - T-0002
  - T-0005
fact_cite:
  - "fact-extraction-report § api-contracts.md §popsicle-cli §pipeline unlock"
---

# 我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复

---

## 本 task 可解答

- "stage 卡在 blocked 是什么意思？"
- "popsicle pipeline unlock 在干嘛？什么时候用？"
- "stage 失败后怎么重跑而不丢前面的产出？"

---

## 前提与限制

**你需要先**：
- 跑过 `popsicle pipeline status`，确认确实有 stage 处于 `blocked` 或 `error`
- 已经看过该 stage 的 doc check（见 [T-0003](../daily-ops/T-0003-inspect-state.md)），确认是不是 checklist 没勾全

**本 task 受以下限制**：
- 已 `completed` 的 stage 不可回滚（charter 铁律 #2）
- spec 锁定的 run 必须用 `pipeline unlock` 释放后才能重启

---

## 完成路径

1. **诊断 blocked 原因**：

   ```bash
   popsicle pipeline review --run <run-id>
   ```

   返回 guard condition 评估结果。常见原因：上游 stage 未 completed / doc checklist 未勾全 / requires_approval 未确认。

2. **如果是 spec 锁定阻塞**（多个 run 抢同一 spec）：

   ```bash
   popsicle pipeline unlock --run <run-id>
   ```

   强制释放当前 run 持有的 spec 锁。仅在你**确认这个 run 不应再持锁**时使用。

3. **如果 stage 因 doc 不完整 blocked**：回 [T-0003](../daily-ops/T-0003-inspect-state.md) 看 doc check status → 补勾选 / 补 doc 内容 → 重试 `stage complete`。

4. **如果 stage 内部 skill 跑挂了**（artifact 半成品）：

   ```bash
   popsicle pipeline stage start <stage-name> --resume
   ```

   skill 从当前状态机恢复（不丢已产 artifact）。

5. **最终验证**：

   ```bash
   popsicle pipeline verify --run <run-id>
   ```

   返回所有 stage 完整性检查结果。

---

## 可观察的成功标志

`popsicle pipeline review` 返回所有 guard PASS，且 `popsicle pipeline stage complete <stage>` 可正常推进到下一 stage。

形式化定义：见 [`acceptance.intent#T-0004-recovered-pipeline-can-advance`](../../intents/acceptance.intent)

---

## Related Next Tasks

- **[T-0002 - 把 pipeline 推到下一个 stage](../daily-ops/T-0002-advance-stage.md)** — 恢复后继续推进
- **[T-0005 - 审查产物链做合规复盘](../admin/T-0005-audit-trail.md)** — 恢复事件本身需进 audit log

---

## Charter Compliance

- [x] frontmatter 8 个必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 第一人称
- [x] 完成路径无大量 if-else（5 步是顺序诊断流程）

---

`Decision-Ref: PDR-002`
