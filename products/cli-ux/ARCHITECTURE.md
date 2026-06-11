# Architecture: cli-ux

> **Layer**: L4（实现视角）
> **Audience**: 工程师、AI agent
> **Status**: cutover-done + self-host Phase 1（ADR-010 Accepted 2026-06-11）
> **Last-Updated**: 2026-06-11
> **Last-Decision-Ref**: ADR-010（self-hosting Phase 1）

## 责任边界

cli-ux owns the `popsicle` binary command shell: argv parsing, command dispatch, output formatting, and actionable errors.

It does not own pipeline state-machine logic, document parsing, guard evaluation, context assembly, extraction, or persistence schema. Those remain in `skill-runtime`, `artifact-system`, and `storage`.

## 模块图

```
cli-ux (bin: popsicle)
  ├─ calls skill-runtime for skill/pipeline/issue/session behavior
  ├─ calls artifact-system for doc/guard/context/extractor behavior
  └─ calls storage for persistence-facing rows/files
```

## File Manifest（受 RFC 控制的目录与 crate）

| 路径 | 责任 | 状态 |
|---|---|---|
| `crates/cli-ux/src/lib.rs` | command parser, dispatcher contract, semantic shell helpers | cutover-done（ADR-008）|
| `crates/cli-ux/src/main.rs` | thin `popsicle` binary entrypoint | cutover-done（ADR-008）|
| `crates/cli-ux/src/self_host.rs` | TSV workspace + doctor + tool run | cutover-done（ADR-010）|
| `crates/storage/src/workspace.rs` | `WorkspaceStore` trait | accepted（Phase 2 PROJ-11）|
| `.popsicle/self-host/state.tsv` | Phase 1 issue/run/doc index | runtime |
| `products/cli-ux/decisions/adr/ADR-010-self-hosting-phase1-cutover.md` | self-host Phase 1 cutover | Accepted |
| `products/cli-ux/intents/contracts.intent` | `CliShellDelegatesToDomainCrates` | accepted（ADR-007）|
| `products/cli-ux/decisions/adr/ADR-007-cli-ux-io-shell-boundary.md` | IO shell boundary | Accepted |
| `products/cli-ux/decisions/adr/ADR-008-cli-ux-cutover.md` | cli-ux cutover | Accepted |

> 由 rfc-writer 写到 RFC 文档的 "ARCHITECTURE.md 增量" 章节，再在 RFC 接受时合并到本表。

## Contracts

`intents/contracts.intent` 持有跨模块 API 契约的形式化描述。任何 `crates/<name>/` 下
的 trait/struct 改动若影响 contracts，必须先走 ADR → 解锁 contracts → intent-spec-writer
收紧 → `intent check` 通过。

## Open Decisions

- Tauri UI bridge remains outside MVP.
- Storage-backed SQLite workspace → **PROJ-11**（Phase 2）；TSV Phase 1 done（ADR-010）.

---

> 本文件骨架；任何实质内容必须由 RFC（rfc-writer）+ ADR（adr-writer）固化后才能进。
> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
