# telemetry 使用指南（Agent）

> **Tool**: `popsicle tool run telemetry …` | **fail-open** | 不进 doc check / pipeline gate

Agent 在任意已 `popsicle init` 的项目中，**先读本 guide**，再调用 record/report。

```bash
popsicle tool run telemetry action=guide
```

---

## 何时用

| 场景 | action | 说明 |
|------|--------|------|
| 读本文 | `guide` | 打印本文件 |
| LLM 调用后 | `record` | `span=gen_ai.chat` |
| stage 文档通过后 | `record` | `span=popsicle.run.score`（或读 `doc check` 的 `telemetry_hint`） |
| 重大分支 | `record` | `span=popsicle.decision` |
| 看 run 耗时/质量 | `report` | `run=<run_id>` 或 `limit=N` |
| 上云 / 诊断 | `flush` / `status` | 需 OTLP 配置 |

## 每个 pipeline stage

1. **工作中**：至少一次 `gen_ai.chat`（含 model、token 估算）。
2. **`doc check` 通过后**：可选 `popsicle.run.score`（1–5）。
3. **重大分支**：`popsicle.decision` + 简短 `summary`。

编排 span（`issue start`、`stage complete`、`doc create`、`doc check`）**自动**写入，勿重复上报。

## 命令模板

```bash
# 0) 不确定参数时
popsicle tool run telemetry action=guide

# 1) LLM 调用（stage 工作中）
popsicle tool run telemetry action=record span=gen_ai.chat \
  run=<run_id> doc=<doc_id> \
  model=<model> temperature=<t> \
  input_tokens=<n> output_tokens=<n> format=json --format json

# 2) stage 自评（doc check 通过后）
popsicle tool run telemetry action=record span=popsicle.run.score \
  run=<run_id> doc=<doc_id> score=4 rubric=stage-quality format=json --format json

# 3) 关键决策
popsicle tool run telemetry action=record span=popsicle.decision \
  run=<run_id> summary="选用 feature-delivery" format=json --format json

# 4) run 聚合报告
popsicle tool run telemetry action=report run=<run_id> format=json --format json
popsicle tool run telemetry action=report limit=10 format=json --format json
```

## Span 速查

### 自动 span（无需 Agent）

| span | 触发 |
|------|------|
| `popsicle.run.start` | `issue start` |
| `popsicle.stage.complete` | `pipeline stage complete` |
| `popsicle.doc.create` | `doc create` |
| `popsicle.doc.check` | `doc check` |

公共属性：`popsicle.run_id`、`popsicle.trace_id`（= run_id）、`popsicle.issue_key`、`popsicle.pipeline`、`popsicle.skill`、`popsicle.duration_ms`。

### Agent 显式 span

| span | 建议属性 |
|------|----------|
| `gen_ai.chat` | `model`, `temperature`, `input_tokens`, `output_tokens`, `doc` |
| `popsicle.decision` | `summary` |
| `popsicle.run.score` | `score` (1–5), `rubric` |

## 约束

- **fail-open**：失败不改变主命令 exit code。
- 不要把 trace 写进 artifact frontmatter（WAL 旁路已足够）。
- 数据：`.popsicle/telemetry/{run_id}/spans.wal.jsonl`（通常 gitignore）。

## 不要

- 不要因 telemetry 失败而中断 stage / doc check。
- 不要重复上报已自动 emit 的编排事件。
