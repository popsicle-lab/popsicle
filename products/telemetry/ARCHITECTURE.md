# Architecture: telemetry

> **Layer**: L4（实现视角）
> **Audience**: 工程师、AI agent
> **Status**: MVP 已交付（PROJ-68）
> **Last-Updated**: 2026-06-26
> **Last-Decision-Ref**: ADR-002

## 责任边界

telemetry 拥有旁路观测数据平面：WAL、OTLP exporter、redaction、OTel attribute schema。

它不拥有：issue/pipeline FSM（skill-runtime）、doc/guard（artifact-system）、argv 解析（cli-ux）。cli-ux / skill-runtime 通过 **TelemetrySink 端口** inject。

## 模块图

```
cli-ux / skill-runtime
  └─ inject → crates/telemetry/ (TelemetrySink)
                ├─ WAL (.popsicle/telemetry/{run_id}/)
                ├─ report (action=report, ADR-002)
                └─ OTLP exporter (HTTP/protobuf + 后台 flush, fail-open)
```

## File Manifest

| 路径 | 责任 | ADR | 状态 |
|------|------|-----|------|
| `crates/telemetry/src/lib.rs` | record_span、run_tool、fail-open | ADR-001 | ✅ |
| `crates/telemetry/src/wal.rs` | spans.wal.jsonl append + redaction + read | ADR-001 | ✅ |
| `crates/telemetry/src/config.rs` | `.popsicle/otel.yaml` 加载 | ADR-001 | ✅ |
| `crates/telemetry/src/otlp.rs` | WAL → SpanData → OTLP http/protobuf | ADR-001 | ✅ |
| `crates/telemetry/src/background.rs` | 后台 batch flush | ADR-001 | ✅ |
| `crates/telemetry/src/export.rs` | flush/status、增量导出 | ADR-001 | ✅ |
| `crates/telemetry/src/report.rs` | WAL → RunReport / RecentReport | ADR-002 | ✅ |
| `crates/skill-runtime/src/session_span.rs` | SessionSpanSink + session 生命周期 emit | ADR-001 | ✅ |
| `crates/cli-ux/src/telemetry_bridge.rs` | SessionSpanSink → WAL；report stdout | ADR-001/002 | ✅ |
| `crates/cli-ux/src/project_context.rs` | `refresh_telemetry_health_row` | ADR-002 | ✅ |
| `products/telemetry/SPAN_SCHEMA.md` | span 字段表 | PDR-001 | ✅ |
| `products/telemetry/REPORT_SCHEMA.md` | report JSON 契约 | ADR-002 | ✅ |
| `products/telemetry/AGENT_TELEMETRY.md` | Agent 约定 | PDR-001 | ✅ |
| `AGENTS.md` Workflow Rule 12 | 全局 Agent 门禁 | PDR-001 | ✅ |
| `intent-coder/tools/telemetry/guide.md` | Agent 权威操作指南（action=guide） | PDR-001 | ✅ |
| `intent-coder/tools/telemetry/tool.yaml` | guide shell 回退 + 参数文档 | PDR-001 | ✅ |
| `crates/cli-ux/tests/telemetry_tool.rs` | guide 集成测试 | PDR-001 | ✅ |
| `intent-coder/skills/*/guide.md` | Agent 观测小节（指向 action=guide） | PDR-001 | ✅ |
| `crates/cli-ux` doc check | `telemetry_hint` 字段 | PDR-002 | ✅ |

## P2 待办（未排期 — 详见 PDR-002 §Decision.3–4）

> 产品动机已 Accepted（PDR-002）；实现细节在此维护。开工前再建 task（T-TEL-0005+）与 Issue。

### 报告与分析（telemetry）

| 项 | 说明 | 关联 |
|----|------|------|
| `scope=recent` 显式参数 | 与无 `run=` 行为对齐并写入 CLI 帮助 | ADR-002 |
| `summary.by_pipeline` / `by_skill` | 跨 run median `duration_ms`、doc_check 失败率 | PDR-002 §3 |
| `path=` 扫描 | ADR 已写、代码未实现 | ADR-002 |
| `REPORT_SCHEMA.md` 扩展 | 上述 summary 字段 | ADR-002 |
| 大 WAL 限流/分页 | PDR-002 Monitoring：P95 读 WAL > 5s | PDR-002 |

### Weekly 反馈闭环

| 项 | 说明 | 关联 |
|----|------|------|
| `doc-sync-weekly` 自动刷新 | `living-doc-author --target product-context` 内调 `refresh_telemetry_health_row` | PDR-002 §4 |
| Monitoring 可执行化 | PDR-001/002 AI Feedback Monitoring 表 → CI 或 weekly 脚本 | PDR-001/002 |
| `contracts.intent` `TelemetryReportSink` | 可选收紧（report fail-open 形式化） | PDR-002 Consequences |

### 跨 product（cli-ux UI）

| 项 | 说明 | 关联 |
|----|------|------|
| UI run timeline | `popsicle ui` 只读展示 stage 时间线；数据来自 `action=report` 或 WAL | 消费方：`products/cli-ux/` |

### 产品未决（见 PRODUCT.md Open Questions）

- MCP `telemetry.record` 是否与 `tool run` 并存

---

> Decision-Ref: ADR-002 · Phase A–D 已交付（PROJ-70–74）
