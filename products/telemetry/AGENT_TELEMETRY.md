# Agent Telemetry 约定（Pipeline 内）

> **Agent 操作**：`popsicle tool run telemetry action=guide`（源文件：`intent-coder/tools/telemetry/guide.md`）
>
> **Spec 维护**（dogfood，Agent 运行时不必读）：完整字段表见 [`SPAN_SCHEMA.md`](SPAN_SCHEMA.md)

## 每个 pipeline stage 结束前

1. **工作中**：至少一次 `gen_ai.chat`（含 model / token 估算）。
2. **`doc check` 通过后**：可选 `popsicle.run.score`（1–5）。
3. **重大分支**：`popsicle.decision` + 简短 `summary`。

## 示例（替换 `<run_id>` / `<doc_id>`）

```bash
popsicle tool run telemetry action=guide

popsicle tool run telemetry action=record span=gen_ai.chat \
  run=<run_id> doc=<doc_id> model=composer-2.5-fast \
  input_tokens=1200 output_tokens=400 format=json --format json

popsicle tool run telemetry action=record span=popsicle.run.score \
  run=<run_id> doc=<doc_id> score=4 rubric=stage-quality format=json --format json
```

## 不要

- 不要把 trace 写进 artifact frontmatter（旁路 WAL 已足够）
- 不要因 telemetry 失败而中断 stage / doc check
