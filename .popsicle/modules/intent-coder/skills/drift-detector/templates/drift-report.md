---
artifact: drift-report
slug: {slug}
generated_by: drift-detector
product: {product}
last_updated: {date}
old_pin: "<facts.yaml meta.legacy_pin>"
new_pin: "<git -C legacy/<name> rev-parse HEAD>"
query_anchors:
  - "legacy 更新后，哪些 intent/task/golden 过期了？"
  - "事实基还准吗？"
---

# Legacy 漂移报告 — {product}

> 由 `drift-detector` 产出（feedback #13）。在新 pin 上重跑 fact-extractor 抽取，
> 按 `fact_id` + `source` diff 旧 `facts.yaml`，把「事实基快照」变成「活契约」。

## Pin Delta

| 项 | 值 |
|---|---|
| 旧 pin（facts.yaml）| `<old sha>` |
| 新 pin（submodule HEAD）| `<new sha>` |
| legacy 变更范围 | `git -C legacy/<name> diff --stat <old>..<new>` 摘要 |

> 新旧相同 → 无漂移，其余段写「（无）」。

## Fact Diff

> 按 `fact_id` + `source` 机械 diff，勿凭印象。

| 类型 | fact_id | source（旧→新）| 说明 |
|---|---|---|---|
| 新增 | F-ST-API-012 | — → `legacy@<new>:proto/schema.proto#L88` | legacy 新加 rpc |
| 删除 | F-ST-API-003 | `legacy@<old>:…#L40` → — | legacy 删除，引用它的下游失效 |
| 变更 | F-ST-BEH-001 | signature/evidence 变 | 行为可能变，golden 需复核 |

## Downstream Impact

> 用 fact_id 反查受影响的 intent / task / golden，给建议动作。

| 受影响对象 | 引用的 fact | 建议动作 |
|---|---|---|
| `acceptance.intent#…` | F-ST-API-003 | 失效并移除 / 改指新 fact |
| T-ST-0004 (`migrates_from`) | F-ST-API-003 | 关闭或改绑 |
| G-ST-API-001 | F-ST-BEH-001 | rewrite：重录 fixture；verbatim：重确认 characterization |
| （新增覆盖）| F-ST-API-012 | 起一个新 task + golden 覆盖新 rpc |

## Drift Checklist

- [ ] 旧/新 pin 均已确认（新 pin 真实）
- [ ] 新 pin 上重跑了与 fact-extractor 一致的抽取命令
- [ ] diff 按 `fact_id` + `source` 机械得出（新增/删除/变更三类）
- [ ] 每条 diff 都反查了下游 intent/task/golden 并给了动作
- [ ] 报告可直接驱动一轮增量（更新/新增覆盖 → task）
