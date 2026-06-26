# PDR-002: Telemetry Run 聚合报告与 Weekly 反馈闭环

> **Status**: Accepted
> **Date**: 2026-06-26
> **Target Product**: `telemetry`
> **Decision Type**: Product Decision Record (PDR)
> **Supersedes**: —
> **Related ADRs**: ADR-002（待 adr-writer）
> **Related Journey**: daily-ops / lifecycle
> **Source Issue**: PROJ-73

---

## Decision Context

### 触发因素

Phase A–C 已建立 WAL span、OTLP 导出与 Agent 上报约定，但 **缺少把 span 读回为可读结论的路径**。维护者无法快速回答：某 run 各 stage 耗时、doc check 失败次数、Agent 是否上报 score。PDR-001「AI Feedback Monitoring」表尚未可执行。

### 多角色辩论摘要

**参与角色**: PM, ENGLD, OPS

**用户置信度**: 4/5

**关键分歧**:
- 报告是否写入 artifact → **否**，CLI stdout + 活文档摘要（延续 sidecar 策略）
- 是否 gate stage complete → **否**，fail-open 不变

**核心事实引用**:
- PROJ-72 Phase C 完成；`SPAN_SCHEMA.md` / `AGENT_TELEMETRY.md`
- `doc-sync-weekly` pipeline 已有 `product-context` target

### 备选方案

| 方案 | 提案者 | 否决理由 |
|------|--------|---------|
| 仅依赖 OTLP 仪表盘 | OPS | 无 endpoint 时不可用；本地 first |
| 报告写入 state.db | ENGLD | 污染事务索引 |
| 每 run 生成 artifact md | PM | 与 sidecar 策略冲突；git 噪音 |

---

## Decision

1. **本地聚合**：`popsicle tool run telemetry action=report run=<run_id>`（或 `path=` 扫描目录）从 WAL 生成 **JSON/text** 报告，exit 0，fail-open。
2. **报告内容（MVP）**：stage 时间线、`doc.check` 统计、Agent span 覆盖（gen_ai / score / decision）、关联 `issue_key` / `pipeline` / `skill`。
3. **跨 run 摘要（P1）**：可选 `action=report scope=recent limit=N` 按 pipeline/skill 聚合 median duration 与 check 失败率。
4. **Weekly 闭环**：`doc-sync-weekly` / `living-doc-author --target product-context` 在 `docs/PROJECT_CONTEXT.md` §现在状态 写入一行 telemetry 健康摘要；PDR-001 Monitoring 表转为可执行检查项。
5. **受众**：仓库维护者 / PM 读报告改 pipeline 与 skill guide；Agent 间接消费更新后的活文档。

---

## Consequences

### Task File Updates

#### 新增 Tasks

- [ ] `products/telemetry/tasks/daily-ops/T-TEL-0004-run-telemetry-report.md`

#### 修改 Tasks

| Task ID | 修改说明 |
|---|---|
| T-TEL-0002 | `related_next_tasks` 增加 T-TEL-0004 |

### PRODUCT.md Top-Level Updates

- [ ] Tasks Catalog 增加 T-TEL-0004
- [ ] 用户入口增加 `action=report`

### Intent File Updates

| 文件 | 变更 |
|------|------|
| `acceptance.intent` | 新增 Block 4 `TelemetryReportFailOpen` |
| `contracts.intent` | ADR-002 解锁 `TelemetryReportSink`（可选 P1） |

### Code Updates (informational — feature-delivery PROJ-74)

- `crates/telemetry/src/report.rs` — WAL → RunReport 聚合
- `crates/telemetry/src/lib.rs` — `action=report` 分支
- `products/telemetry/REPORT_SCHEMA.md` — JSON 字段表

---

## Intent Impact

| 层 | 变更 |
|----|------|
| `acceptance.intent` | Block 4 种子 |
| `invariants.intent` | 无（沿用 FailOpen） |

---

## AI Feedback Monitoring

| 度量 | 阈值 | 时间窗 | 回滚 |
|------|------|--------|------|
| `action=report` 导致主命令非零 exit | 0 次 | 每 release | 禁用 report action |
| report 读取 WAL 阻塞 > 5s（大文件） | 0 次 P95 | 每 sprint | 限流 / 分页 |

---

## Approval

- **Status**: Accepted
- **Approval date**: 2026-06-26
