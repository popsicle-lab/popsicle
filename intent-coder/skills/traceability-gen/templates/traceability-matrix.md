---
artifact: traceability-matrix
slug: {slug}
generated_by: traceability-gen
product: {product}
last_updated: {date}
derived_from:
  - "products/{product}/tasks/**/*.md (frontmatter: migrates_from, equivalence)"
  - "docs/baseline/**/baseline.yaml (goldens[].status)"
query_anchors:
  - "哪些 task 覆盖了这个 legacy crate/rpc？"
  - "哪个 rpc 还没有 golden / acceptance intent？"
---

# 迁移覆盖矩阵 — {product}

> 由 `traceability-gen` **机器派生**（feedback #12/#21）。每行追回 task frontmatter，
> 数字读 `baseline.yaml`。**不要手写行**——手写即失去可复算性。拟覆盖/更新
> `migration/traceability.md`。

## Sources

- Task 数（迁移 task，`migrates_from` 非空）：0
- baseline.yaml：`docs/baseline/{date}/{slice}/baseline.yaml`
- 派生时间：{date}

## Coverage Matrix

| task_id | legacy 源（migrates_from）| golden_id | golden 状态 | intent | 模式 |
|---|---|---|---|---|---|
| T-ST-0001 | `crates/store-engine/src/lib.rs#L1-120` / `rpc:SchemaApi.CreateTable` | G-ST-API-001 | pass | `acceptance.intent#…` | verbatim |

## Gaps（机器检出，交人跟进）

> 有则逐条列，无则写「（无缺口）」。这是本 skill 的核心产出。

| 类型 | 对象 | 说明 |
|---|---|---|
| 缺 golden | T-ST-0003 | `migrates_from` 有值但 `equivalence.golden_id` 空 |
| golden 未对齐 | G-ST-API-004 | task 指向但 baseline.yaml 中 status=fail/缺失 |
| 缺 intent | T-ST-0005 | `migrates_from` 有值但 `related_intents` 空 |
| 孤儿 golden | G-ST-API-009 | baseline 有但无 task equivalence 指向 |

## Traceability Checklist

- [ ] 每行都追回了某个 task 的 frontmatter 字段（无手写行）
- [ ] golden 状态读自 `baseline.yaml`，读不到的标 `?` 并进 Gaps
- [ ] 四类缺口（缺 golden / 未对齐 / 缺 intent / 孤儿 golden）都跑过
- [ ] 已注明将覆盖/更新 `migration/traceability.md` 的哪些行
