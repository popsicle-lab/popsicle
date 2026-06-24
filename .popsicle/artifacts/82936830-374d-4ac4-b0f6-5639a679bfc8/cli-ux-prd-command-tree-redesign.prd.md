---
id: 0c6ae006-16c1-4800-9731-14f1bef606ae
doc_type: prd-overview
title: cli-ux prd command tree redesign
status: final
skill_name: prd-writer
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:33:24.707351Z
updated_at: 2026-06-09T10:33:24.707351Z
---

# PRD Overview — cli-ux command tree redesign

> **Status**: Approved
> **Target Product**: `cli-ux`
> **Source Debate**: `cli-ux-product-debate-command-tree-redesign.product-debate.md`
> **PDR**: `PDR-001-cli-ux-command-tree-redesign.md`
> **Quality Score**: 92/100
> **Last-Updated**: 2026-06-09

## Core Intent

AI agent 用一个 `popsicle` binary 初始化 workspace、启动 pipeline、生产/校验 artifact，并从错误输出中知道下一步；CLI 不复刻 legacy core，而是调用 `skill-runtime`、`artifact-system`、`storage`。

## Problem Statement

legacy `popsicle-cli` 暴露 22 个子命令（`api-contracts.md § Bounded Context：popsicle-cli`）。其中一部分是 IDD 主路径，一部分是重复命令或非主流程能力。cli-ux 需要把新设计后的 runtime/artifact 能力暴露给 agent，而不是完整冻结 legacy 命令行为。

## Success Metrics

| Metric | Baseline | Target | Measurement | Cite |
|---|---|---|---|---|
| IDD 主路径命令可用 | legacy 22 命令混杂 | 7 task 覆盖 init/run/doc/stage/error/admin/disposition | task checklist + golden | fact report § Summary |
| Drop 命令清晰 | `checklist`/`item`/`sync` 顶层存在 | 新 CLI 顶层不暴露这三者 | `popsicle --help` semantic golden | PDR-001 |
| 错误可诊断 | legacy 文案不作为契约 | error category + object id + next step | T-CU-0005 | PDR-001 |

## File Manifest

| 文件 | 动作 |
|---|---|
| `products/cli-ux/PRODUCT.md` | 更新用途、入口、roadmap |
| `products/cli-ux/tasks/README.md` | 新增 7 task 索引 |
| `products/cli-ux/tasks/onboarding/T-CU-0001-first-init-next-step.md` | 新增 |
| `products/cli-ux/tasks/daily-ops/T-CU-0002-create-issue-start-run.md` | 新增 |
| `products/cli-ux/tasks/daily-ops/T-CU-0003-doc-artifact-command.md` | 新增 |
| `products/cli-ux/tasks/daily-ops/T-CU-0004-pipeline-stage-advance.md` | 新增 |
| `products/cli-ux/tasks/troubleshooting/T-CU-0005-actionable-errors.md` | 新增 |
| `products/cli-ux/tasks/admin/T-CU-0006-admin-maintenance.md` | 新增 |
| `products/cli-ux/tasks/lifecycle/T-CU-0007-command-disposition.md` | 新增 |
| `products/cli-ux/intents/acceptance.intent` | 待 intent-spec-writer 收紧合并 |
| `products/cli-ux/intents/invariants.intent` | 待 intent-spec-writer 收紧合并 |
| `products/cli-ux/decisions/pdr/PDR-001-cli-ux-command-tree-redesign.md` | 新增 |

## User Intents Catalog

| User Query | Task | Journey Stage | Audience |
|---|---|---|---|
| "第一次初始化后下一步是什么？" | T-CU-0001 | onboarding | agent |
| "issue start 后 run id 在哪里？" | T-CU-0002 | daily-ops | agent |
| "doc create 写到哪个 artifact 文件？" | T-CU-0003 | daily-ops | agent |
| "stage complete 后怎么确认下游解锁？" | T-CU-0004 | daily-ops | agent |
| "guard failed 我要改哪个文档？" | T-CU-0005 | troubleshooting | agent |
| "migrate 为什么在 admin 下面？" | T-CU-0006 | admin | maintainer |
| "checklist/item/sync 为什么不见了？" | T-CU-0007 | lifecycle | maintainer |

## Intent Mapping

| # | 核心声明 | 目标 intent 层 | Task | Block |
|---|---|---|---|---|
| 1 | init 后 workspace 可读且 CLI 给 next step | acceptance.intent | T-CU-0001 | InitShowsNextStep |
| 2 | issue start 创建 run 并锁 spec | acceptance.intent | T-CU-0002 | IssueStartCreatesRun |
| 3 | doc create 写 artifact 文件和 document row | acceptance.intent | T-CU-0003 | DocCommandWritesArtifact |
| 4 | stage complete 后状态反映到 pipeline status | acceptance.intent | T-CU-0004 | StageAdvanceReflectsState |
| 5 | 错误输出包含 category/object/next step | acceptance.intent | T-CU-0005 | ErrorsAreActionable |
| 6 | migrate/reinit 必须在 admin 子树 | acceptance.intent | T-CU-0006 | AdminCommandsAreExplicit |
| 7 | checklist/item/sync 不出现在 MVP 顶层 | invariants.intent | T-CU-0007 | RemovedCommandsStayRemoved |
| 8 | CLI shell 不持有业务逻辑 | contracts.intent | — | CliShellDelegatesToDomainCrates（ADR-007） |

## Out of Tasks

- Tauri UI bridge 不进入 MVP。
- legacy stdout byte parity 不作为本 spec 验收。
- cloud sync 不进入 IDD 主路径。

## Risk Assessment

| Risk | Probability | Impact | Mitigation | Affected Tasks | Fact Cite |
|---|---|---|---|---|---|
| 命令树改动影响 agent 习惯 | Medium | High | 保留主路径命令名；只 drop 重复/非主流程命令 | T-CU-0001..0007 | fact report § Risk Hotspots |
| CLI 重新长出业务逻辑 | Medium | High | ADR-007 固化 IO shell 边界 | T-CU-0004 | fact report § 新设计输入 |

## Dependencies & Blockers

- ADR-007：`crates/cli-ux` 依赖方向与 command handler boundary。
- `skill-runtime` / `artifact-system` / `storage` 已 cutover-done 或可被 CLI shell 调用。

## Ingest Checklist

- [x] product-debate record 已读取，command disposition 已确认
- [x] fact-extraction-report 已读取，legacy `popsicle-cli` 22 个子命令作为事实输入
- [x] target_product 已锁定为 `cli-ux`
- [x] PDR ID 已分配（PDR-001）
- [x] Task ID 范围已分配（T-CU-0001..T-CU-0007）

## Quality Checklist

- [x] 文件清单与 PDR Consequences 完全一致
- [x] 每个新增 task 文件单独存在且符合 task 模板
- [x] User Intents Catalog 覆盖每个新增 task
- [x] Intent Mapping 与 acceptance seed block 一一对应
- [x] 无历史/未来叙事短语
- [x] 所有数字 / 模块名 / 风险条目 cite fact basis
- [x] Quality Score ≥ 90

## Review Checklist

- [x] PRD overview 已合并到 PRODUCT.md 顶层
- [x] 7 个 task 文件已落地
- [x] tasks/README 已重建
- [x] PDR-001 已创建
- [x] acceptance/invariants/contracts 交 intent-spec / ADR chain 处理
