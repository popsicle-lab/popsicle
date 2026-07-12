# drift-detector 使用指南

让事实基从「一次性快照」变「活契约」（feedback #13/S5）。legacy 是 pinned submodule；它前进后，
`facts.yaml` 里部分 fact 会过期，连带引用这些 fact 的 intent/task/golden 也过期。本 skill 在**新 pin**
上重跑 fact-extractor 的抽取，按 `fact_id` + `source` **机械 diff**，把漂移结构化暴露。

```
fact-extractor（旧 pin 的 facts.yaml = 基线）
    → drift-detector（本 skill，新 pin 重跑 + diff）
    → migration-preserve / feature-spec（把「更新/新增覆盖」变成增量 task）
```

## 三类 diff → 下游动作

| diff | 含义 | 下游动作 |
|---|---|---|
| **新增 fact** | legacy 加了 API/行为 | 新 task + golden 覆盖 |
| **删除 fact** | legacy 删了 | 引用它的 intent/task/golden **失效**，关闭或改绑 |
| **变更 fact** | 同 fact_id，source/签名/evidence 变 | golden 复核：rewrite 重录、verbatim 重确认 |

## 定位

| 做 | 不做 |
|---|---|
| 新 pin 上重跑一致的抽取命令 | 凭印象猜哪里变了 |
| 按 fact_id + source 机械 diff | 只看 git log 不落到 fact |
| 用 fact_id 反查受影响下游 | 只报 legacy 变更、不连下游 |
| 输出可驱动增量的动作清单 | 停在「发现了漂移」 |

## 与 facts.yaml 的契约

依赖 fact-extractor 的 `facts.yaml` 每条带稳定 `fact_id` 与可复验 `source: legacy@<sha>:<path>#L`
（feedback #11）。没有这个结构化事实基，漂移检测无从做起——这也是 #11 把 facts.yaml 定为唯一真相源的回报。
建议在每次 bump legacy pin 前先跑本 skill。
