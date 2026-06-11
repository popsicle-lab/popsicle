---
task_id: T-0005
slug: audit-trail
title: "我作为人类维护者审查一个 pipeline run 的完整产物链以做合规复盘"
journey_stage: admin
audience: ["human-maintainer", "compliance-reviewer"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-06-08
intent_kind: audit
involved_features: ["pipeline-verify", "pipeline-archive", "artifact-trace"]
prerequisites:
  - "目标 pipeline run 已经走完 init → living-docs 全部 stage（status: completed）"
limits:
  - "本 task 不修改任何 artifact（read + verify only）"
  - "归档后的 run 不可再 advance stage"
related_intents:
  - "invariants.intent#audit-trail-immutable"         # ⚠ orphan：intent-spec D2 跳过（intent-lang 无法表达可重建聚合），.intent 中无此 block。待人工
  - "invariants.intent#audit-trail-reconstructible"   # ⚠ orphan：同上，audit-trail 留在本 task 级
related_next_tasks:
  - T-0006
fact_cite:
  - "PDR-001 §Phase 4 表 3 (UI-5 人类维护者审查 pipeline 完整产物链供合规 / 复盘)"
  - "fact-extraction-report § api-contracts.md §popsicle-cli §pipeline verify/archive"
---

# 我作为人类维护者审查一个 pipeline run 的完整产物链以做合规复盘

---

## 本 task 可解答

- "怎么看一个 pipeline run 的全部产出（含每个 stage 的 doc + commit）？"
- "`popsicle pipeline verify` 检查什么？通过意味着什么？"
- "完成的 run 归档之后还能查得到吗？"

---

## 前提与限制

**你需要先**：
- 知道目标 run-id（`popsicle pipeline status` 或 commit message 里可查）
- 仓库 git history 完整（不 shallow clone）

**本 task 受以下限制**：
- read-only + verify-only；不修改任何 artifact
- 归档后的 run 不可再推进 stage（INV：audit-trail-immutable）

---

## 完成路径

1. **看 run 的全 stage + doc 摘要**：

   ```bash
   popsicle pipeline status --run <run-id>
   popsicle doc list --run <run-id>
   ```

2. **逐 doc 抽样检查**（重点看 frontmatter + checklist 终态）：

   ```bash
   popsicle doc show <doc-id>
   popsicle doc check status --doc <doc-id>
   ```

3. **对照 git history 找 PDR 决策档案**：

   ```bash
   git log --all --grep "PDR-" --oneline
   ```

   把 commit 信息里 PDR ID 与 `products/<product>/decisions/pdr/` 文件交叉对账。

4. **跑全 run 一致性校验**：

   ```bash
   popsicle pipeline verify --run <run-id>
   ```

   返回每个 stage 的完整性 PASS/FAIL + guard 评估。

5. **若一切 PASS，归档**：

   ```bash
   popsicle pipeline archive --run <run-id>
   ```

   把 run 移入归档分区；artifact 内容不动；后续 `popsicle pipeline status` 默认隐藏归档 run（`--include-archived` 可看）。

---

## 可观察的成功标志

`popsicle pipeline verify --run <run-id>` 返回全 stage PASS，且 archive 后 git log 仍能找到所有 PDR commit。

形式化定义：见 [`invariants.intent#audit-trail-reconstructible`](../../intents/invariants.intent)

---

## Related Next Tasks

- **[T-0006 - 发布新 skill 版本并把现有 pipeline 升上去](../lifecycle/T-0006-skill-version-bump.md)** — 复盘后若需升级 skill

---

## Charter Compliance

- [x] frontmatter 8 个必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 第一人称
- [x] 完成路径无大量 if-else

---

`Decision-Ref: PDR-002`
