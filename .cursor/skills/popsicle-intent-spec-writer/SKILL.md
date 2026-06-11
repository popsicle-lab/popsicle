---
name: popsicle-intent-spec-writer
description: 把 prd-writer 产出的 acceptance-intent 种子收紧成可合并、可被 Z3 验证的正式 intent-lang。负责分层归位（acceptance / invariants / contracts）、剥离时间/性能/运行时约束（D2）、应用四条 intent-lang 硬写法规则、与目标产品现有 .intent 去重查冲突，并在交付前用 intent check 自验。是「intent → 机器验证」闭环里「种子 → 正式 intent」这一棒（决策 D3，从外部契约位收回内置）。 (from module: intent-coder)
---

> This skill is provided by the **intent-coder** module.

Perform the "intent-spec-writer" step in the Popsicle pipeline.

## Workflow

- **Initial state**: `ingesting`
- **Final state(s)**: `completed`
- **Transitions**:
  - `ingesting` → `tightening` via `ingested` (guard: `checklist_complete:Ingest Checklist`)
  - `tightening` → `verifying` via `ready-to-verify`
  - `verifying` → `tightening` via `fix`
  - `verifying` → `review` via `verified` (guard: `has_sections:Summary,分层归位,剥离的约束,验证结果,合并计划;checklist_complete:检查清单`)
  - `review` → `completed` via `approve` **⚠ requires human approval**
  - `review` → `tightening` via `revise`

## Inputs (upstream dependencies)

- `acceptance-intent-seed` from skill `prd-writer` (required)

## Prerequisites

An active pipeline run MUST exist before executing this skill. If `popsicle pipeline status` shows no active run, you MUST first create an Issue (`popsicle issue create`) then start it (`popsicle issue start <key>`). NEVER execute this skill outside of a pipeline run.

## Commands

```bash
# Verify an active pipeline run exists and this skill is the current step
popsicle pipeline next --format json

# Get enriched prompt with historical references and project context
popsicle prompt intent-spec-writer --run <run-id> --related --format json

# Create the document
popsicle doc create intent-spec-writer --title "<title>" --run <run-id>

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

# intent-spec-writer 使用指南

把 prd-writer 的 **acceptance-intent 种子** 收紧成可合并、可被 Z3 验证的
**正式 intent-lang**。这是「intent → 机器验证」闭环里承上启下的一棒：

```
product-debate → prd-writer →（种子）→ intent-spec-writer →（正式 .intent）→ intent-consistency-check（Z3 闸）
```

## 为什么内置（决策 D3）

种子已经是合法 intent-lang 骨架（prd-writer v0.2+），但「能不能真喂给 Z3、合并进
现有 `.intent` 不打架」这一步直接决定闭环能否成立——是 dogfood 必经环节，不外包。
本 skill 做的不是「发明语义」，而是**规范化 + 分层 + 查冲突 + 跑通**，保持薄。

## 五件事

1. **分层归位**：把种子里每条内容按 PRD § Intent Mapping 归到正确的层——
   - 操作后态 → `acceptance.intent`（require/ensure）
   - 跨操作保持型不变量 → `invariants.intent`（safety + primed）
   - 模块接口契约 → `contracts.intent`（goal + `[Awaiting ADR]`）
2. **剥离 D2 约束**：时间 / 性能 / 运行时事实 / 概率**不进** `.intent`，
   登记到对应 task 的「可观察的成功标志」，由测试守护。
3. **四规则审查**：见下。
4. **去重查冲突**：与目标产品现有 `.intent` 比对，复用已有 type、避免重名/矛盾。
5. **自验**：交付前用 `intent check`（经 `intent-validate` tool）跑出 exit 0。

## intent-lang 四条硬写法规则（来自 dogfood，违反必出错）

1. **后态用 primed `x'`**：safety/invariant 要约束操作**之后**的状态必须写 primed；
   unprimed 只验旧态 = 假通过。
2. **一个文件 = 一个验证作用域**：每条 `safety` 被无条件合并进文件内所有 intent，
   且靠**参数名**绑定。→ `acceptance.intent` 只放操作 intent，保持型不变量进
   `invariants.intent`；不相关操作分文件。
3. **无 frame 假设**：不默认「未提及字段不变」。要声明不改某字段必须显式
   `ensure x' == x`。
4. **纯 require+ensure = trivial verified**：`ensure` 只是假设，只有 `invariant`/
   `safety` 产生验证目标。acceptance 操作规约属 trivial verified（合法、可跑，但不
   被证伪）；真正的不变量验证靠 invariants 的 safety + 完整 ensure。

## 能力边界提醒

- intent-lang 不支持聚合（`count`/`where`）；集合基数 / 双实体唯一性只能写成
  struct-forall `theorem`，当前会被 `skipped`（仅声明意图，等 intent-lang 支持）。
- 报告里必须区分「trivial verified（操作规约）」与「真正验证了不变量」，
  别让一片绿色 ✅ 误导成「全都被证明了」。

## 产物

- `{slug}.acceptance-formal.intent`：收紧后的 acceptance 增量，可直接 intent check。
- `{slug}.intent-spec-report.md`：分层归位 / 剥离清单 / 四规则审查 / 验证结果 /
  冲突检查 / 合并计划。

合并：按报告「合并计划」追加到 `products/{target_product}/intents/acceptance.intent`，
再跑 `intent-consistency-check` 做 Z3 闸。
