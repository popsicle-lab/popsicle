# Run Report Schema — Phase D

> **Product**: telemetry | **PDR**: PDR-002 | **Task**: T-TEL-0004

`popsicle tool run telemetry action=report` 的 JSON 输出约定（MVP）。

## 顶层

| 字段 | 类型 | 说明 |
|------|------|------|
| `run_id` | string | 目标 run（单 run 模式） |
| `issue_key` | string? | 从 span 属性推断 |
| `pipeline` | string? | 从 span 属性推断 |
| `span_count` | number | WAL 行数 |
| `stages` | array | stage 时间线 |
| `doc_checks` | object | doc check 汇总 |
| `agent_coverage` | object | Agent 显式 span 覆盖 |
| `status` | string | `ok` / `degraded` |

## stages[]

| 字段 | 说明 |
|------|------|
| `name` | `popsicle.stage` 属性 |
| `skill` | `popsicle.skill` |
| `completed` | 是否有 `popsicle.stage.complete` |
| `duration_ms` | 相邻 span 或 stage complete 上的 `popsicle.duration_ms` |

## doc_checks

| 字段 | 说明 |
|------|------|
| `total` | `popsicle.doc.check` span 数 |
| `passed` | `popsicle.doc_check.passed=true` 计数 |
| `failed` | 其余 |
| `by_skill` | `{ skill: { passed, failed } }` |

## agent_coverage

| 字段 | 说明 |
|------|------|
| `gen_ai_chat` | 是否有 `gen_ai.chat` |
| `run_score` | 是否有 `popsicle.run.score` |
| `decision` | 是否有 `popsicle.decision` |

## 约束

- fail-open；WAL 不可读时 `status: degraded` 但 exit 0
- 不进 `doc check` / pipeline gate
- 报告默认 **stdout**；weekly 摘要由 `living-doc-author` 写入 `docs/PROJECT_CONTEXT.md`

详见 [`tasks/daily-ops/T-TEL-0004-run-telemetry-report.md`](tasks/daily-ops/T-TEL-0004-run-telemetry-report.md)。
