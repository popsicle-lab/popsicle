# ADR-006 · artifact-system cutover（in-shadow → cutover-done）

> **Status**: Accepted
> **Date**: 2026-06-09
> **Product**: artifact-system
> **Generated-by**: cutover-author
> **Source-Equivalence**: artifact-system-equivalence-report.equivalence-report.md
> **Source-Coverage**: artifact-system-implementation-coverage.implementation-coverage.md

## Context

artifact-system 已完成 in-shadow 实现与 lib-level golden 对账。该 slice 承载 popsicle 的文档制品引擎：Document 模型、纯文档 guard、ContextLayer 装配、structured extraction、`work_item` → `task_chunk_entity` 重命名，以及 storage-facing `DocumentRow`。

本 ADR 授权将 `migration/progress.md` 中 artifact-system 标为 **cutover-done**，并把 legacy → new traceability 登记到 `migration/traceability.md`。CLI 字节主路径仍属于 `cli-ux` slice，不在本 ADR 范围内。

## Decision

1. **切流范围**：
   - `legacy/popsicle/crates/popsicle-core/src/model/document.rs` → `crates/artifact-system/src/document.rs`
   - `legacy/popsicle/crates/popsicle-core/src/engine/guard.rs` → `crates/artifact-system/src/guard.rs`
   - `legacy/popsicle/crates/popsicle-core/src/engine/context.rs` / `engine/context_layer.rs` → `crates/artifact-system/src/context.rs`
   - `legacy/popsicle/crates/popsicle-core/src/engine/extractor.rs` → `crates/artifact-system/src/extractor.rs`
   - `legacy/popsicle/crates/popsicle-core/src/model/work_item.rs` → `crates/artifact-system/src/task_chunk.rs`
   - `legacy/popsicle/crates/popsicle-core/src/storage/index.rs` document row shape → `crates/storage/src/document_row.rs`
2. **主路径切换**：popsicle-new 内部 lib consumers 以 `artifact-system` crate 为主路径；legacy CLI `doc` / `prompt` / `extract` 命令仍归 `cli-ux` slice 对账与切流。
3. **已知 divergence**：
   - D-001：legacy `Document::from_file_content` 对 body `trim_start()`；new 保持 body byte-exact，以满足 `DocumentRoundTrips`。
   - D-002：CLI byte parity 未切；`cli-ux` slice 负责命令级 golden。

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 继续 in-shadow 不切 | 阻塞 cli-ux 复用 document/guard/context/extractor 新主路径 |
| 无 golden 硬切 | 违反 CONTRIBUTING §4 的 ≥5 golden 门禁 |
| 为了 legacy YAML byte parity 放弃 body-preserving round-trip | 违反 Z3 verified `DocumentRoundTrips` 验收契约 |

## Consequences

- `migration/progress.md`：artifact-system 状态 → `cutover-done`
- `migration/traceability.md`：追加 slice-2 legacy → new 映射行
- `products/artifact-system/ARCHITECTURE.md`：File Manifest 状态列更新
- `products/artifact-system/PRODUCT.md`：Status / Last-Decision-Ref 更新
- `products/artifact-system/tasks/README.md`：已实施列回填 6/6

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 gate_ready | artifact-system intent-consistency-report（2026-06-09，11 VC verified）| pass |
| Equivalence ≥5 golden | artifact-system equivalence-report（6/6）| pass |
| cargo test | `cargo test -p artifact-system` | exit 0 |

## Cutover Gate Checklist

- [x] Intent Z3 gate_ready：11 VC verified
- [x] Equivalence gate：6/6 golden pass
- [x] cargo test：`cargo test -p artifact-system` exit 0
- [x] Cutover ADR status：Accepted
- [x] Traceability row set prepared for `migration/traceability.md`

## Waiver Checklist

- [x] CLI byte parity waiver documented as `cli-ux` scope
- [x] SQLite full wiring waiver documented as storage/cli-ux follow-up
- [x] Document body parse divergence documented in ADR-006
- [x] No untracked fail item hidden behind waiver

## Migration

切流后 legacy 该范围进入 **Sunset 候选**。物理删除 legacy 模块、重连 CLI 命令、以及命令级 byte parity 另由 `cli-ux` ADR 处理。

## 检查清单

- [x] Context / Decision / Consequences / Compliance 已填写
- [x] 切流范围列出 legacy ↔ new 路径
- [x] 已知 divergence 已登记
- [x] Approval 为 Accepted

## Approval

- **Status**: Accepted
- **Approved by**: @curtiseng（经 `pipeline stage complete cutover --confirm`）
- **Approval date**: 2026-06-09
