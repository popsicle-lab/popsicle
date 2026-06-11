---
name: popsicle-cutover-author
description: slice 切流审批闸：核验 intent gate_ready、equivalence 门禁、cargo test； 起草并固化切流 ADR(Accepted)；更新 migration/progress.md 为 cutover-done。 slice-delivery 第三棒，需用户 --confirm。 (from module: intent-coder)
---

> This skill is provided by the **intent-coder** module.

Perform the "cutover-author" step in the Popsicle pipeline.

## Workflow

- **Initial state**: `gating`
- **Final state(s)**: `completed`
- **Transitions**:
  - `drafting` → `review` via `drafted` (guard: `has_sections:Context,Decision,Consequences,Compliance;checklist_complete:检查清单`)
  - `review` → `completed` via `approve` **⚠ requires human approval**
  - `review` → `drafting` via `revise`
  - `gating` → `drafting` via `gates-pass` (guard: `checklist_complete:Cutover Gate Checklist`)
  - `gating` → `drafting` via `gates-waived` (guard: `checklist_complete:Waiver Checklist`)

## Inputs (upstream dependencies)

- `equivalence-report` from skill `equivalence-baseline` (required)
- `intent-consistency-report` from skill `intent-consistency-check` (optional)
- `implementation-coverage` from skill `shadow-implementer` (optional)

## Prerequisites

An active pipeline run MUST exist before executing this skill. If `popsicle pipeline status` shows no active run, you MUST first create an Issue (`popsicle issue create`) then start it (`popsicle issue start <key>`). NEVER execute this skill outside of a pipeline run.

## Commands

```bash
# Verify an active pipeline run exists and this skill is the current step
popsicle pipeline next --format json

# Get enriched prompt with historical references and project context
popsicle prompt cutover-author --run <run-id> --related --format json

# Create the document
popsicle doc create cutover-author --title "<title>" --run <run-id>

# View the created document
popsicle doc show <doc-id>

# ⚠ STOP — Do NOT auto-complete stages
# After creating all documents for a stage, STOP and show the user:
#   1. What documents were created
#   2. The stage completion command below
# Let the user review and decide when to complete the stage.
popsicle pipeline stage complete <stage-name>

# ⚠ This stage requires --confirm (approval gate):
# 禁止代用户执行。必须由用户本人审阅后在终端执行：
popsicle pipeline stage complete <stage-name> --confirm
```

## Writing Guide

# cutover-author 使用指南

单个 slice 的 **Strangler Fig 切流审批闸**。核验三门禁后固化切流 ADR，
更新 `migration/progress.md` 与 `migration/traceability.md`。

## 三门禁

| # | 门禁 | 证据 |
|---|---|---|
| 1 | Intent Z3 | `intent-consistency-report`：`gate_ready=true` 或连续 3 次 pass |
| 2 | Golden 等价 | `equivalence-report`：`equivalence_gate_pass=true` |
| 3 | 构建 | `cargo test -p <slice>` exit 0 |

豁免须用户明确确认，并写入 ADR § Compliance。

## 产出

- `products/<slice>/decisions/adr/ADR-XXX-<slice>-cutover.md`（Accepted）
- `migration/progress.md` 状态 → `cutover-done`
- `migration/traceability.md` 正式行

## 与 living-doc-author 的分工

cutover 改**决策与看板**；living-doc 改**活文档实现态**
（tasks/README 已实施列、ARCHITECTURE File Manifest、PRODUCT 双行头）。

## 红线

- 三门禁未过且未豁免 → 不 Accepted
- 不修改 legacy submodule（sunset 另开 ADR）
- ADR Accepted 后依 charter 不可改——错了写 Supersedes ADR
