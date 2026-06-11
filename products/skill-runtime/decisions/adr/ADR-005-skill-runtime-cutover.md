# ADR-005 · skill-runtime in-shadow cutover

> **Status**: Accepted
> **Date**: 2026-06-09
> **Product**: skill-runtime
> **Generated-by**: cutover-author（草案）
> **Source-Baseline**: `docs/baseline/2026-06-09/skill-runtime/`

## Context

`crates/skill-runtime/` 已实现 loader、pipeline session、ADR-004 端口（upstream checker、
MemoriesLayer）、issue MVP，以及 6 条 lib-level golden baseline。Legacy `popsicle-core`
在 skill/pipeline/issue 范围仍为 CLI 主路径。

## Decision

1. **切流范围（in-shadow）**：`skill load` / pipeline 编排 / issue 解析的 **库实现**
   以 `skill-runtime` crate 为准；CLI 仍调 legacy，直至 `cli-ux` slice。
2. **Golden**：`docs/baseline/2026-06-09/skill-runtime/run-all.sh` 6/6 pass 作为
   equivalence 门禁（lib 级；非 CLI 字节对账）。
3. **Divergence**：SQLite `IndexDb` 未迁移；`storage::MemoryDocumentStore` 为
   in-shadow 占位。

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | products/skill-runtime/intents/*.intent verified | pass（observe）|
| Golden ≥5 | `run-all.sh` 6 scripts | pass（2026-06-09 实跑）|
| cargo test | `cargo test -p skill-runtime -p storage` | pass |

## Approval

- **Approved by**: PROJ-4 slice-delivery cutover stage (`pipeline stage complete cutover --confirm`)
- **Date**: 2026-06-09
