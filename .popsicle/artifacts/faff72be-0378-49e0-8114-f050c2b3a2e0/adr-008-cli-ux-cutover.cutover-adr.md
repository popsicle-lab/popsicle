---
id: 6681285a-7dfc-4f7b-8e15-562d1b4b6474
doc_type: cutover-adr
title: ADR-008 cli-ux cutover
status: final
skill_name: cutover-author
pipeline_run_id: faff72be-0378-49e0-8114-f050c2b3a2e0
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T08:40:39.393690Z
updated_at: 2026-06-10T08:46:42.640990Z
---

# ADR-008 · cli-ux cutover（in-shadow → cutover-done）

> **Status**: Accepted
> **Date**: 2026-06-10
> **Product**: cli-ux
> **Generated-by**: cutover-author
> **Source-Equivalence**: cli-ux-equivalence-report.equivalence-report.md
> **Source-Coverage**: cli-ux-implementation-coverage.implementation-coverage.md

## Context

cli-ux 已完成 in-shadow 实现、semantic golden 对账与 thin binary entrypoint。该 slice 承载 popsicle-new 的命令行壳：argv parsing、command dispatch、output formatting、actionable errors，以及把 skill-runtime / artifact-system / storage 暴露给 AI agent 的命令表面。

PDR-001 与 ADR-007 已锁定：cli-ux 不复刻 legacy 22 个顶层命令，也不追求 stdout/stderr 字节级 parity；切流门禁锁定 semantic command effects。

## Decision

1. **切流范围**：
   - `legacy/popsicle/crates/popsicle-cli/src/main.rs` command surface → `crates/cli-ux/src/lib.rs::TOP_LEVEL_COMMANDS` / `top_level_help`
   - `legacy/popsicle/crates/popsicle-cli/src/commands/issue.rs` + `pipeline.rs` 的 issue start/run signal → `crates/cli-ux/src/lib.rs::start_issue_run`
   - `legacy/popsicle/crates/popsicle-cli/src/commands/doc.rs` 的 artifact + document row signal → `crates/cli-ux/src/lib.rs::create_document_artifact`
   - `legacy/popsicle/crates/popsicle-cli/src/commands/pipeline.rs` 的 stage complete approval signal → `crates/cli-ux/src/lib.rs::complete_pipeline_stage`
   - `legacy/popsicle/crates/popsicle-cli/src/commands/{admin,migrate,reinit}.rs` → `crates/cli-ux/src/lib.rs::AdminCommand` / `parse_args`
   - legacy `checklist` / `item` / `sync` command families → `REMOVED_TOP_LEVEL_COMMANDS` + actionable errors
   - binary entrypoint → `crates/cli-ux/src/main.rs` (`popsicle`)
2. **主路径切换**：popsicle-new 的 cli-ux slice 升为 `cutover-done`。新 `popsicle` binary 以 `crates/cli-ux` 的 semantic shell 为主路径；legacy CLI 仅保留为 `legacy/popsicle` fact source / sunset candidate。
3. **已知 divergence**：
   - D-001：CLI byte parity / full 22-command compatibility 不锁。PDR-001 与 ADR-007 接受 semantic IDD command shell；`checklist` / `item` / `sync` 不作为 MVP 顶层命令保留。
   - D-002：storage-backed real workspace mutation 仍由后续 storage/packaging 接线补齐；当前 cutover 锁定 thin shell 与 command-effect contract。

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 继续 in-shadow 不切 | 已有 6/6 semantic golden、8/8 intent coverage 与 binary entrypoint；继续阻塞 sunset |
| 无 binary entrypoint 切流 | 文档会显示 cutover-done 但 `popsicle` binary 仍悬空 |
| 复刻 legacy 22 命令与字节输出 | 违反 PDR-001；会把已裁剪的平台遗产带回 popsicle-new |
| 无 golden 硬切 | 违反 CONTRIBUTING §4 |

## Consequences

- `migration/progress.md`：cli-ux 状态 → `cutover-done`
- `migration/traceability.md`：slice-3 行 ADR 引用填实为 ADR-008，状态 → `cutover-done`
- `products/cli-ux/ARCHITECTURE.md`：File Manifest 状态更新为 cutover-done
- `products/cli-ux/PRODUCT.md`：Status / Last-Decision-Ref 更新
- `products/cli-ux/tasks/README.md`：7/7 task 已实施列回填
- legacy `popsicle-cli` 进入 Sunset 候选；物理删除 legacy 另开 ADR

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 gate_ready | cli-ux intent-consistency-report（2026-06-10；3 clean runs；7 VC verified）| pass |
| Equivalence >=5 golden | cli-ux equivalence-report（2026-06-10；6/6 semantic golden pass）| pass |
| cargo test | `cargo test -p cli-ux`（2026-06-10）| exit 0 |
| binary smoke | `cargo run -p cli-ux -- --help`（2026-06-10）| exit 0 |

## Cutover Gate Checklist

- [x] intent gate 已核对：3 clean runs；7 VC verified；gate_ready=true
- [x] equivalence gate 已核对：6/6 semantic golden pass
- [x] cargo test 已核对：`cargo test -p cli-ux` exit 0
- [x] binary smoke 已核对：`cargo run -p cli-ux -- --help` exit 0
- [x] 未通过项已列明 blocker：无

## Waiver Checklist

- [x] 用户书面确认豁免哪一门禁：N/A，无豁免
- [x] 豁免理由写入 ADR § Compliance：N/A，无豁免
- [x] 补偿措施已列出：D-001/D-002 为 scope divergence，已由 PDR-001/ADR-007/本 ADR 接受

## Migration

切流后 legacy 该范围进入 **Sunset 候选**。物理删除 legacy CLI、真实安装包替换策略、Tauri bridge 与 cloud sync 另开 ADR / slice，不在本 ADR 范围内。

## 检查清单

- [x] Context / Decision / Consequences / Compliance 已填写
- [x] 切流范围列出 legacy ↔ new 路径
- [x] 已知 divergence 已登记
- [x] Approval 状态与 pipeline `--confirm` 一致

## Approval

- **Status**: Accepted
- **Approved by**: @curtiseng（经 `pipeline stage complete cutover --confirm`）
- **Approval date**: 2026-06-10
