---
name: popsicle-arch-debate
description: 多角色技术架构辩论模拟器——在写 RFC 之前用 4-6 个技术角色（ARCH / SEC / PERF / OPS / DATA / DEV）就一个技术问题或架构选型充分辩论方案空间，暴露盲点。消费 prd-overview 里标了 `contracts.intent` / `[ADR 候选]` 的条目 + fact-extractor 的事实基（dependency-graph / api-contracts / risk hotspots），产出 RFC 草稿 + 技术决策矩阵 + 辩论纪要，喂给下游 rfc-writer。是产品侧 `product-debate` 的技术侧对称体。 (from module: intent-coder)
---

> This skill is provided by the **intent-coder** module.

Perform the "arch-debate" step in the Popsicle pipeline.

## Workflow

- **Initial state**: `setup`
- **Final state(s)**: `completed`
- **Transitions**:
  - `debating` → `concluding` via `conclude` (guard: `checklist_complete:Phase Coverage`)
  - `debating` → `setup` via `restart`
  - `setup` → `debating` via `start` (guard: `checklist_complete:Setup Checklist`) **⚠ requires human approval**
  - `concluding` → `completed` via `approve` (guard: `has_sections:Topic,Participants,Phase 1,Phase 2,Phase 3,Phase 4,Decision;checklist_complete:Output Checklist`) **⚠ requires human approval**
  - `concluding` → `debating` via `reopen`

## Inputs (upstream dependencies)

- `prd-overview` from skill `prd-writer` (optional)
- `api-contracts` from skill `fact-extractor` (optional)
- `fact-extraction-report` from skill `fact-extractor` (optional)
- `product-debate-record` from skill `product-debate` (optional)

## Prerequisites

An active pipeline run MUST exist before executing this skill. If `popsicle pipeline status` shows no active run, you MUST first create an Issue (`popsicle issue create`) then start it (`popsicle issue start <key>`). NEVER execute this skill outside of a pipeline run.

## Commands

```bash
# Verify an active pipeline run exists and this skill is the current step
popsicle pipeline next --format json

# Get enriched prompt with historical references and project context
popsicle prompt arch-debate --run <run-id> --related --format json

# Create the document
popsicle doc create arch-debate --title "<title>" --run <run-id>

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

# arch-debate 使用指南

多角色**技术架构**辩论模拟器，是产品侧 `product-debate` 的技术侧对称体。在写 RFC
之前，用 4-6 个技术角色就一个架构问题充分辩论方案空间，暴露盲点，产出可审计的
辩论纪要 + RFC 草稿 + 技术决策矩阵。

## 它在 Phase 3 链条里的位置

```
prd-writer →（PRD § Intent Mapping 标 [ADR 候选] 的行）→
arch-debate →（rfc-draft）→ rfc-writer →（ADR 骨架 + contracts 种子）→
adr-writer →（Accepted ADR + 解锁 contracts 种子）→ intent-spec-writer →（收紧合并）→ intent-consistency-check
```

arch-debate 只解决「产品侧留下的技术待决问题」——即 PRD 里标了 `contracts.intent` /
`[ADR 候选]` 的条目。它**不写 ARCHITECTURE.md**（那是 rfc-writer 的活），只产辩论摘要。

## 与 product-debate 的异同

| 维度 | product-debate | arch-debate |
|---|---|---|
| 角色 | PM/UXR/GROWTH/ENGLD/BIZ | ARCH/SEC/PERF/OPS/DATA/DEV |
| 主持 | PM | ARCH |
| 代言人 | UXR（代言用户）| SEC（代言攻击者）|
| 产出 intent 层 | acceptance / invariants | **contracts** / invariants |
| 决策载体 | PDR | **ADR**（触及 charter → CADR）|
| 下游 | prd-writer | rfc-writer |

机制完全复用 product-debate：4 Phase + 每 Phase 强制暂停 + 置信度调风格 + setup 需
人类 `start` 确认。角色定义见 `references/tech-roles.md`。

## 三条 IDD 硬纪律

1. **数字/LoC/模块名/churn 必须 cite** fact-extraction-report 或 api-contracts，
   否则标 `[未经事实基验证]`。
2. **每个核心技术声明在 Phase 4 由 ARCH 归类到 intent 层**——本链主产物是
   `contracts.intent`（模块间契约）；跨切面不变量进 `invariants.intent`。
3. **性能/时延/容量不进 `.intent`**（intent-lang 不验时间，D2）——写进 RFC「质量
   属性目标」，由压测/SLO 守护。触及 charter 铁律/Layer Map 的方案标「需要 CADR」。

## 产物

- `{slug}.arch-debate.md` — 辩论纪要（审计轨迹）
- `{slug}.rfc-draft.md` — RFC 草稿，含 § Intent & Decision Mapping（contracts 候选 +
  ADR/CADR 候选清单），喂给 rfc-writer
- `{slug}.tech-decision-matrix.md` — 方案 × NFR 打分矩阵（候选 ≥ 2 时）
