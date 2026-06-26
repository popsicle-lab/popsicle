# PDR-001: Telemetry Agent 观测旁路

> **Status**: Proposed
> **Date**: 2026-06-26
> **Target Product**: `telemetry`
> **Decision Type**: Product Decision Record (PDR)
> **Supersedes**: —
> **Related ADRs**: （待 arch-debate → ADR-001）
> **Related Journey**: —

---

## Decision Context

### 触发因素

Popsicle 编排主平面（issue → pipeline → doc → guard）不托管 LLM；Agent 在 IDE 外执行。缺少结构化观测数据，无法聚合 pipeline 耗时、Agent 质量或 OTel 标准 trace，也无法为未来工作流调优提供依据。

### 多角色辩论摘要

**参与角色**: PM, UXR, SEC, ENGLD, OPS

**用户置信度**: 4/5

**关键分歧**:
- trace 是否进 artifact frontmatter vs sidecar：→ sidecar JSONL + OTel span（artifact 仅语义决策摘要）
- 是否进 doc check：→ 否，fail-open 旁路

**核心事实引用**:
- doc-140 product-debate（PROJ-67 run）
- ADR-007 cli-ux IO shell；ADR-013 session JSON 与 state.db 分离

### 备选方案

| 方案 | 提案者 | 否决理由 |
|------|--------|---------|
| 观测写入 state.db | OPS | 污染事务索引；度量失败不应影响主流程 |
| Cursor hook 自动采集 | ENGLD | IDE 绑定；用户明确不要 |
| 挂在 cli-ux product | PM | 横切平台能力，独立 bounded context |

---

## Decision

采用独立 product **telemetry**：`.popsicle/telemetry/{run_id}/` WAL + OTLP 异步导出；OpenTelemetry GenAI 语义；**fail-open**（度量失败不改变主命令 exit code）；不进 doc check / pipeline gate。

---

## Consequences

### Task File Updates (required by this PDR)

#### 新增 Tasks

- [ ] `products/telemetry/tasks/admin/T-TEL-0001-configure-otlp-endpoint.md`
- [ ] `products/telemetry/tasks/daily-ops/T-TEL-0002-agent-telemetry-record.md`
- [ ] `products/telemetry/tasks/daily-ops/T-TEL-0003-orchestration-auto-span.md`

### PRODUCT.md Top-Level Updates

- [ ] `products/telemetry/PRODUCT.md` — 替换 `[TBD]` 为正式一行用途与 Tasks Catalog

### Intent File Updates

| 文件 | 变更 |
|------|------|
| `products/telemetry/intents/acceptance.intent` | 新增 3 block（种子 → intent-spec-writer 收紧） |
| `products/telemetry/intents/invariants.intent` | 新增 FailOpen 不变量种子 |
| `products/telemetry/intents/contracts.intent` | 待 ADR-001 解锁 |

### Code Updates (informational)

- `crates/telemetry/` — WAL、OTLP exporter、redaction（feature-delivery）
- `crates/cli-ux/` — `tool run telemetry`、编排 inject 点
- `~/.popsicle/otel.yaml` 或 workspace `.popsicle/otel.yaml`

---

## Intent Impact

| 层 | 变更 |
|----|------|
| `products/telemetry/intents/acceptance.intent` | 新增 |
| `products/telemetry/intents/invariants.intent` | 新增 |
| `docs/invariants/` | 无（fail-open 为 product 级） |

---

## AI Feedback Monitoring

| 度量 | 阈值 | 时间窗 | 回滚 |
|------|------|--------|------|
| 主命令因 telemetry 非零 exit | 0 次 | 每 release | 禁用 exporter 默认 |
| WAL 写入导致 stage complete 失败 | 0 次 | 每 sprint | hotfix fail-open |

---

## Approval

- **Status**: Proposed
