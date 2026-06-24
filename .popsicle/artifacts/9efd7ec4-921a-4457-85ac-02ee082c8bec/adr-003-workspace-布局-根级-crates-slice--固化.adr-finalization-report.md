---
id: 91485707-7247-4049-a1a1-35b08fbe7ae8
doc_type: adr-finalization-report
title: ADR-003 workspace 布局：根级 crates/<slice>/ 固化
status: final
skill_name: adr-writer
pipeline_run_id: 9efd7ec4-921a-4457-85ac-02ee082c8bec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T09:28:56.146212Z
updated_at: 2026-06-08T09:34:13.187251Z
---

---
artifact: adr-finalization-report
slug: workspace-layout
generated_by: adr-writer
adr_id: ADR-003
target_product: popsicle
finalized: true              # true 表示本次已把 ADR 固化为 Accepted
last_updated: 2026-06-08
goals_unlocked: 0
handoff_to_spec_writer: 0
query_anchors:
  - "这个 ADR 固化了吗？审批人是谁？"
  - "哪些契约被解锁、可以收紧成 intent 了？"
  - "这个决策有没有偷改 charter？"
---

# ADR 固化报告 — workspace-layout

> 由 `adr-writer` 生成。adr-writer 是 contracts.intent 闭环里「解锁」这一棒——
> **不发明 ADR 内容**（那是 rfc-writer），只做固化门 + 解锁 + 一致性核对。

## Summary

| 指标 | 值 |
|---|---|
| ADR | ADR-003 |
| 固化结果 | Accepted |
| 解锁 goal 数 | 0（纯布局 RFC，无 contracts 种子）|
| 移交 intent-spec-writer 的逻辑保证 | 0 |
| CADR 风险 | 无（不触 charter 四铁律 / Layer Map）|

一句话结论：ADR-003 已固化为 Accepted——popsicle 单产品的 member crate 落根级 `crates/<slice>/`、
`members = ["crates/*"]`，无 contracts 解锁，Consequences 三处文件 + `crates/` 占位已全部落地。

## 固化检查

> 固化门五项。任一不过 → 退回 rfc-writer，不强行固化。

- [x] **决策无歧义**：§ Decision 现在时、明确（`crates/<slice>/` + `members=["crates/*"]`），无「将会/计划/视情况」
- [x] **Consequences 落地**：3 个文件路径 + `crates/` 目录均本次真实落地（见 § 解锁动作）
- [x] **Intent Impact 一致**：RFC § Intent & Decision Mapping 三行均「不进 intent」，无 contracts 种子 goal，一致
- [x] **CADR 合规**：纯目录/构建布局，未触及 charter 四铁律 / Layer Map
- [x] **Decision Context 充分**：触发（members 矛盾 + 用户单产品澄清）+ 6 角色辩论摘要 + A/B 否决理由齐全

不过项说明：无（五项全过）。

## 解锁动作

> 本次实际改动，一行一处。

| 文件 | 改动 |
|---|---|
| `products/skill-runtime/decisions/adr/ADR-003-workspace-layout.md` | 新建，Status: Accepted，填 Approval |
| `products/skill-runtime/ARCHITECTURE.md` § Last-Decision-Ref / File Manifest / Open Decisions | 指向 ADR-003；File Manifest 列 `crates/<slice>/` 三行；Open Decisions 划除 ADR-Workspace-Layout |
| `Cargo.toml`（root）| `members = []` → `members = ["crates/*"]`，改写占位注释 |
| `crates/.gitkeep` | 新建空 `crates/` 目录，使 glob 可解析（`cargo metadata` 通过）|

### 移交 intent-spec-writer 的收紧工单

> 现在可以收紧的逻辑保证（contracts 契约的前后置 → acceptance/invariants）。

| 契约 goal | 可收紧为 | 目标文件 | 形态 |
|---|---|---|---|
| —— | —— | —— | 无：纯布局 ADR 不产 contracts 种子，无收紧工单 |

## Intent Impact 核对

> 与 ADR § Intent Impact 表逐行对照，确认无遗漏、无多余。

| Intent 层 | block | ADR 声明 | 实际解锁 | 一致？ |
|---|---|---|---|---|
| contracts.intent | —— | 无（纯布局，不暴露跨模块 API）| 无解锁 | ✅ |
| invariants.intent | —— | 无 | 无 | ✅ |
| （质量属性目标）| 迁移局部性/编译期边界 | RFC § NFR，不进 intent（D2）| migration+CI 守护 | ✅ |

---

## 检查清单

- [x] 固化门五项已逐项核对
- [x] ADR Status 已改 Accepted、审批信息已填（@curtiseng，2026-06-08）
- [x] contracts 种子 [Awaiting] 已解锁为 [Accepted]——N/A，本 ADR 无 contracts 种子
- [x] 已列出移交 intent-spec-writer 的收紧工单——N/A，无收紧工单
- [x] 未自行收紧 contracts（职责单一）
- [x] Intent Impact 核对无遗漏 / 无多余

## Finalization Gate

- [x] Decision 无歧义、现在时
- [x] Consequences 每个文件路径真实可落地（3 文件 + crates/ 目录已落地）
- [x] Intent Impact 与 RFC / contracts 种子一致（均无 contracts，一致）
- [x] 未触及 charter 锁定内容（纯布局，非 CADR）
- [x] Decision Context 充分（触发 + 6 角色辩论 + A/B 备选否决）
