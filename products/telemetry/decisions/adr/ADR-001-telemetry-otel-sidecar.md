# ADR-001 · Telemetry OTel 旁路数据平面

> **Status**: Accepted
> **Date**: 2026-06-26
> **Product**: telemetry
> **Source-Issue**: PROJ-67
> **Source-PDR**: PDR-001-telemetry-agent-observability.md

## Context

PDR-001 定 telemetry 为独立 product。需固化 crate 边界、WAL/OTLP、fail-open 与跨 product inject 端口，对齐 OpenTelemetry GenAI semantic conventions。

## Decision

1. **新 crate `crates/telemetry/`**：WAL append、OTLP HTTP/protobuf exporter、attribute redaction、配置加载（`.popsicle/otel.yaml` / `~/.popsicle/otel.yaml`）。
2. **存储路径**：`.popsicle/telemetry/{run_id}/spans.wal.jsonl` + `export.state.json`；**不入** `state.db`。
3. **TelemetrySink 端口**（contracts.intent）：同步 `record_span(envelope) -> Result<RecordOutcome, Never>` — 错误变 `RecordOutcome::Degraded`；**永不**向上 panic。
4. **cli-ux inject**：`issue start`、`doc create`、`pipeline stage complete`、`tool run telemetry` 调用 Sink；skill-runtime 可选 pipeline session span（P1）。
5. **OTLP**：HTTP/protobuf；batch flush 后台线程；网络失败仅写 `export.state.json`。
6. **Git**：建议 gitignore `.popsicle/telemetry/`（含潜在 secrets）。
7. **doc check / stage guard**：不读取 telemetry 路径。

## Alternatives

| 方案 | 否决 |
|------|------|
| telemetry 全在 cli-ux | ADR-007 违反 |
| WAL 在 SQLite | 与 ADR-013 索引态混责 |
| Cursor hook | 用户明确排除 |

## Consequences

- `products/telemetry/intents/contracts.intent` 解锁 `TelemetrySinkFailOpen`
- `products/telemetry/ARCHITECTURE.md` File Manifest 固化
- `crates/cli-ux` 依赖 `telemetry` crate

## File Manifest

| Path | Change |
|------|--------|
| `crates/telemetry/` | 新建 lib |
| `crates/cli-ux/src/telemetry_tool.rs` 或 workspace 内等价 | `tool run telemetry` |
| `crates/cli-ux/Cargo.toml` | dep telemetry |
| `.gitignore` | optional `.popsicle/telemetry/` |
| `products/telemetry/intents/contracts.intent` | 收紧 |

## Intent Impact

| Goal | Status |
|------|--------|
| `TelemetrySinkFailOpen` | unlocked |

## Approval

- **Status**: Accepted
- **Approval date**: 2026-06-26
