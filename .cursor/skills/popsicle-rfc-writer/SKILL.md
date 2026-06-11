---
name: popsicle-rfc-writer
description: 把 arch-debate 的 rfc-draft 打磨成可落地的技术设计三件套——正式 RFC（含 ARCHITECTURE.md 顶层增量清单）+ contracts.intent 种子（`[Awaiting ADR]` 状态）+ ADR 骨架（Status 为 Proposed）。是产品侧 `prd-writer` 的技术侧对称体。质量评分 ≥ 90 才放行。 (from module: intent-coder)
---

> This skill is provided by the **intent-coder** module.

Perform the "rfc-writer" step in the Popsicle pipeline.

## Workflow

- **Initial state**: `ingesting`
- **Final state(s)**: `completed`
- **Transitions**:
  - `review` → `completed` via `approve` (guard: `checklist_complete:Review Checklist`) **⚠ requires human approval**
  - `review` → `drafting` via `revise`
  - `ingesting` → `drafting` via `ingested` (guard: `checklist_complete:Ingest Checklist`)
  - `scoring` → `drafting` via `refine`
  - `scoring` → `review` via `pass` (guard: `has_sections:Context,Goals,Proposed Design,Intent & Decision Mapping,File Manifest;checklist_complete:Quality Checklist`)
  - `drafting` → `scoring` via `ready-to-score`

## Inputs (upstream dependencies)

- `rfc-draft` from skill `arch-debate` (required)
- `tech-decision-matrix` from skill `arch-debate` (optional)
- `prd-overview` from skill `prd-writer` (optional)
- `api-contracts` from skill `fact-extractor` (optional)

## Prerequisites

An active pipeline run MUST exist before executing this skill. If `popsicle pipeline status` shows no active run, you MUST first create an Issue (`popsicle issue create`) then start it (`popsicle issue start <key>`). NEVER execute this skill outside of a pipeline run.

## Commands

```bash
# Verify an active pipeline run exists and this skill is the current step
popsicle pipeline next --format json

# Get enriched prompt with historical references and project context
popsicle prompt rfc-writer --run <run-id> --related --format json

# Create the document
popsicle doc create rfc-writer --title "<title>" --run <run-id>

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

# rfc-writer 使用指南

把 `arch-debate` 的 `rfc-draft` 打磨成可落地的技术设计三件套。是产品侧 `prd-writer`
的技术侧对称体——`prd-writer` 产「PRD + acceptance 种子 + PDR 骨架」，`rfc-writer` 产
「RFC + contracts 种子 + ADR 骨架」。

## 三件套

| Artifact | 对称的产品侧 | 角色 |
|---|---|---|
| **RFC**（`{slug}.rfc.md`）| prd-overview | 技术设计全文 + ARCHITECTURE.md 增量清单 |
| **contracts 种子**（`{slug}.contracts.intent`）| acceptance 种子 | goal 块，`[Awaiting ADR]` 状态 |
| **ADR 骨架**（`ADR-{id}-{slug}.md`）| PDR 骨架 | 一个核心技术决策，Status: Proposed |

## 在 Phase 3 链条里的位置

```
arch-debate →（rfc-draft）→ rfc-writer →（RFC + ADR 骨架 + contracts 种子）→
adr-writer（固化 ADR + 解锁 contracts）→ intent-spec-writer（收紧）→ intent-consistency-check（Z3 闸）
```

rfc-writer **产骨架、不固化决策**。ADR 的 Proposed→Accepted 由 adr-writer 把关；
contracts 种子的 `[Awaiting ADR]` 解锁也由 adr-writer 触发。

## 三条要点

1. **质量门 ≥ 90**：四维度（完整性 / 清晰度 / 可验证性 / IDD 适配度，IDD 占 30）。
   不达标退回 drafting；用户可强制 pass 但 RFC 首部打 bypass 水印。
2. **contracts 种子必须能 `intent check`**：用 `goal "..." { rationale / stakeholder /
   measure }` 声明意图，0 VC。技术细节（协议/字段/版本）放注释。
3. **D2 红线**：性能/时延/容量/QPS **不进** contracts 种子——它们是 RFC § Quality
   Attributes 里的 NFR 目标，由压测/SLO 守护。intent-lang 不验时间。

## 与 charter 的关系

- ADR 骨架的 § Intent Impact 必须声明影响哪层 intent（CI 拒缺项）。
- 触及 charter「四条铁律」/「Layer Map」的决策不能写进普通 ADR——标为 **CADR 候选**，
  先走 charter 修订流程。
- RFC § File Manifest 与 ADR § Consequences 必须镜像一致（CI 可 grep 文件路径校验）。
