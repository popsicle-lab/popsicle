# Span Schema — Agent & 编排观测

> **Product**: telemetry | **PDR**: PDR-001 | **Task**: T-TEL-0002

编排 span 由 cli-ux / skill-runtime **自动**写入；本节约定 **Agent 显式**上报字段。

## 自动 span（无需 Agent 调用）

| span | 触发 |
|------|------|
| `popsicle.run.start` | `issue start` → `PipelineSession::start` |
| `popsicle.stage.complete` | `pipeline stage complete` |
| `popsicle.doc.create` | `doc create` |
| `popsicle.doc.check` | `doc check` |

公共属性：`popsicle.run_id`、`popsicle.trace_id`（= run_id）、`popsicle.issue_key`、`popsicle.pipeline`、`popsicle.skill`（stage/doc）、`popsicle.duration_ms`（相邻事件间隔）。

## Agent 显式 span

| span | 何时 | 建议属性 |
|------|------|----------|
| `gen_ai.chat` | 每个 stage 内至少一次 LLM 调用后 | `model`, `temperature`, `input_tokens`, `output_tokens`, `doc` |
| `popsicle.decision` | 架构/流程分支决策 | `summary` |
| `popsicle.run.score` | stage 文档 `doc check` 通过后 | `score` (1–5), `rubric` |

## 命令模板

```bash
# LLM 调用（stage 工作中）
popsicle tool run telemetry action=record span=gen_ai.chat \
  run=<run_id> doc=<doc_id> \
  model=<model> temperature=<t> \
  input_tokens=<n> output_tokens=<n> format=json --format json

# stage 完成自评（doc check 通过后）
popsicle tool run telemetry action=record span=popsicle.run.score \
  run=<run_id> doc=<doc_id> score=4 rubric=spec-clarity format=json --format json

# 关键决策
popsicle tool run telemetry action=record span=popsicle.decision \
  run=<run_id> summary="选用 feature-delivery" format=json --format json
```

## 约束

- fail-open：失败不改变主命令 exit code
- 不进 `doc check` / pipeline gate
- 敏感 key（token/password 等）WAL 层 redact

详见 [`tasks/daily-ops/T-TEL-0002-agent-telemetry-record.md`](tasks/daily-ops/T-TEL-0002-agent-telemetry-record.md)。
