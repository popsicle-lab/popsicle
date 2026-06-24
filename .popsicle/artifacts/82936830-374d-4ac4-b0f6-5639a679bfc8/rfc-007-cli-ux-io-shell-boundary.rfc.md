---
id: 7eb122b3-d5a7-401a-900c-d11cb8c44881
doc_type: rfc
title: RFC-007 cli-ux io shell boundary
status: final
skill_name: rfc-writer
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:38:00.000000Z
updated_at: 2026-06-09T10:38:00.000000Z
---

# RFC-007 · cli-ux IO shell boundary

## Ingest Checklist

- [x] rfc-draft 已读，且通过 Intent & Decision Mapping 校验
- [x] decision-matrix 已读（单方案收敛）
- [x] target_product 已锁定
- [x] ADR ID 已分配（ADR-007）
- [x] CADR 候选已识别（无）

## Context

`cli-ux` 迁移把 legacy `popsicle-cli` 的命令树重组为新设计。`skill-runtime` 和 `artifact-system` 已经 cutover-done；CLI 不应重新持有状态机、guard、document parser 或 storage schema。

## Goals

- Define `crates/cli-ux` as the single binary crate for `popsicle`.
- Keep dependency direction `cli-ux -> skill-runtime -> artifact-system -> storage` where applicable.
- Make command handlers parse argv, dispatch domain services, format output, and return actionable errors.

## Proposed Design

`crates/cli-ux` owns:

- clap command tree and output format selection
- user-facing text/json formatting
- command-level error envelope and next-step hints
- binary entrypoint `popsicle`

It delegates:

- pipeline/session semantics to `skill-runtime`
- document/guard/context/extractor semantics to `artifact-system`
- document rows and persistence-facing helpers to `storage`

## Intent & Decision Mapping

| Claim | Intent layer | Decision |
|---|---|---|
| CLI shell delegates domain behavior | contracts.intent | ADR-007 |
| removed commands stay removed | invariants.intent | PDR-001 |
| command effects are visible in JSON/state | acceptance.intent | PDR-001 |

## File Manifest

| Path | Responsibility |
|---|---|
| `crates/cli-ux/` | binary crate, command handlers, formatters |
| `products/cli-ux/ARCHITECTURE.md` | File Manifest and boundary docs |
| `products/cli-ux/intents/contracts.intent` | `CliShellDelegatesToDomainCrates` goal |
| `products/cli-ux/decisions/adr/ADR-007-cli-ux-io-shell-boundary.md` | Accepted shell boundary |

## Alternatives

| Option | Rejection reason |
|---|---|
| CLI holds business logic | recreates legacy coupling |
| extra facade service layer | unnecessary for MVP |

## Quality Checklist

- [x] 四维度已评分，总分已算（92/100）
- [x] contracts 种子能 intent check（goal 块合法预期）
- [x] 无性能/时延误塞进 contracts
- [x] File Manifest 与 ADR Consequences 镜像一致

## Review Checklist

- [x] RFC § File Manifest 与 ADR Consequences 完全一致
- [x] 每个 contracts goal 块对应 Intent & Decision Mapping 一行
- [x] ADR 骨架 Status: Proposed，ID 不与现有冲突
- [x] CADR 候选（如有）已标明需先走 charter 修订
- [x] RFC 质量评分 ≥ 90
- [x] 已向用户展示三件套完整产出
