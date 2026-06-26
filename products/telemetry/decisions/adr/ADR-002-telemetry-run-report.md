# ADR-002 · Telemetry Run 聚合报告

> **Status**: Accepted
> **Date**: 2026-06-26
> **Product**: telemetry
> **Source-Issue**: PROJ-73
> **Source-PDR**: PDR-002-telemetry-run-report-feedback.md
> **Supersedes**: —

## Context

PDR-002 要求从 WAL 生成可读 run 报告并接入 weekly 活文档。ADR-001 已定 sidecar 写路径；读路径需对称且不污染主平面。

## Decision

1. **`crates/telemetry/src/report.rs`**：只读解析 `.popsicle/telemetry/{run_id}/spans.wal.jsonl`，聚合为 RunReport（字段见 `REPORT_SCHEMA.md`）。
2. **CLI**：`popsicle tool run telemetry action=report run=<id>` 或 `path=` + 可选 `limit` / `scope=recent`；输出 JSON 或 text 到 stdout。
3. **Fail-open**：WAL 缺失/解析错误 → `status: degraded`，**exit 0**；不读 state.db、不进 doc check。
4. **Weekly**：`doc-sync-weekly` 经 `living-doc-author --target product-context` 刷新 `docs/PROJECT_CONTEXT.md` §现在状态 telemetry 一行（delivery 实现时落地）。
5. **OTLP**：report 与 OTLP 互补；有 endpoint 时云端仍可按 `popsicle.issue_key` 聚合（T-TEL-0003）。

## Alternatives

| 方案 | 否决 |
|------|------|
| 独立 telemetry-report crate | 无边界收益 |
| 报告写入 artifact | 违反 sidecar 策略 |
| OTLP-only 查询 | 离线不可用 |

## Consequences

- `products/telemetry/ARCHITECTURE.md` File Manifest 增加 report.rs
- `acceptance.intent#TelemetryReportFailOpen` 解锁 delivery
- feature-delivery Issue（PROJ-74 建议）实现代码

## File Manifest

| Path | Change |
|------|--------|
| `crates/telemetry/src/report.rs` | 新建 |
| `crates/telemetry/src/lib.rs` | `action=report` |
| `products/telemetry/REPORT_SCHEMA.md` | JSON 契约 |
| `products/telemetry/ARCHITECTURE.md` | Phase D spec 完成标记 |

## Intent Impact

| Goal | Status |
|------|--------|
| `TelemetryReportFailOpen` | unlocked |

## Approval

- **Status**: Accepted
- **Approval date**: 2026-06-26
