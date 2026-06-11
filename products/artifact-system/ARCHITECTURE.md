# Architecture: artifact-system

> **Layer**: L4（实现视角）
> **Audience**: 工程师、AI agent
> **Status**: cutover-done（ADR-006 Accepted 2026-06-09；CLI 仍 legacy）
> **Last-Updated**: 2026-06-09
> **Last-Decision-Ref**: ADR-006（artifact-system cutover）

## 责任边界

artifact-system owns the document-artifact engine: `Document` serialization/revision behavior, pure document guards, prompt-context assembly, structured extraction, and the `task_chunk` replacement for legacy `work_item`.

It does **not** own CLI command wiring (`doc` / `prompt` / `extract`) or full SQLite cutover; those remain in `cli-ux` / storage follow-up work. `upstream_approved` is represented here as an injected port; the runtime implementation remains in skill-runtime.

## 模块图

```
skill-runtime ──uses──> artifact-system ──uses──> storage
                         ├─ document
                         ├─ guard (+ UpstreamApprovalChecker port)
                         ├─ context (+ ContextLayer trait)
                         ├─ extractor
                         └─ task_chunk
```

## File Manifest（受 RFC 控制的目录与 crate）

| 路径 | 责任 | 状态 |
|---|---|---|
| `crates/artifact-system/src/document.rs` | Document artifact model + file-content round-trip | cutover-done（ADR-006）|
| `crates/artifact-system/src/guard.rs` | `has_sections` / `checklist_complete` + `UpstreamApprovalChecker` port | cutover-done（ADR-004/006）|
| `crates/artifact-system/src/context.rs` | ContextLayer trait + deterministic context assembly | cutover-done（ADR-004/006）|
| `crates/artifact-system/src/extractor.rs` | total story/testcase/bug extraction | cutover-done（ADR-004/006）|
| `crates/artifact-system/src/task_chunk.rs` | `work_item` -> `task_chunk_entity` rename | cutover-done（ADR-006）|
| `crates/storage/src/document_row.rs` | storage-facing DocumentRow mirror | cutover-done（ADR-004/006；SQLite wiring deferred）|

> 由 rfc-writer 写到 RFC 文档的 "ARCHITECTURE.md 增量" 章节，再在 RFC 接受时合并到本表。

## Contracts

`intents/contracts.intent` 持有跨模块 API 契约的形式化描述。任何 `crates/<name>/` 下
的 trait/struct 改动若影响 contracts，必须先走 ADR → 解锁 contracts → intent-spec-writer
收紧 → `intent check` 通过。

## Open Decisions

- ADR-003 Workspace Layout（Accepted）
- ADR-004 artifact-system seams（Accepted）
- ADR-006 artifact-system cutover（Accepted）

---

> 本文件骨架；任何实质内容必须由 RFC（rfc-writer）+ ADR（adr-writer）固化后才能进。
> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
