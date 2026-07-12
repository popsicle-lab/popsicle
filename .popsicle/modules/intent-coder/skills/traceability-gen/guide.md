# traceability-gen 使用指南

把 `migration/traceability.md` 从**手工表格**升级为**机器派生**产物（feedback #12/#21/S5）。
它交叉 task frontmatter（`migrates_from` / `equivalence`）× `baseline.yaml`（golden 状态）× intents，
产出覆盖矩阵并**自动暴露缺口**（哪个 legacy rpc 没 golden / 没 acceptance intent）。

```
prd-writer（task.migrates_from + equivalence）┐
equivalence-baseline（baseline.yaml goldens） ├→ traceability-gen（本 skill）→ migration/traceability.md
intents（related_intents）                    ┘
```

## 为什么要它

#12 指出：覆盖关系（task↔crate↔rpc↔intent↔golden）此前只在人工维护的 `migration/traceability.md` 里，
无法回答「哪些 task 覆盖 store-engine」「哪个 rpc 没有 acceptance intent」。#21 指出 traceability 全手工。
把它交给机器派生后，缺口不再依赖人记得去查。

## 定位

| 做 | 不做 |
|---|---|
| 从 task frontmatter 逐行派生 | 手写矩阵行 |
| golden 状态读 `baseline.yaml` | 臆造 pass |
| 自动检出四类缺口 | 只列已覆盖、掩盖缺口 |
| 输出拟写入 `migration/traceability.md` 的 diff | 直接改写而不说明 |

## 四类缺口（核心产出）

1. **缺 golden**：`migrates_from` 非空但 `equivalence.golden_id` 空。
2. **golden 未对齐**：task 指向的 `golden_id` 在 `baseline.yaml` 中缺失或 `status!=pass`。
3. **缺 intent**：`migrates_from` 非空但 `related_intents` 空（无 acceptance 断言）。
4. **孤儿 golden**：`baseline.yaml` 有 golden 但无 task 的 `equivalence` 指向它。

## 与 gate 的关系

`ref_resolvable` gate（ROADMAP §10）校验 intents 的 `realized_by` 可解析；traceability-gen 更进一步，
校验 **task↔golden↔intent 三方对齐**。二者互补：gate 拦引用悬空，traceability-gen 暴露覆盖缺口。
建议在 `living-docs` 阶段跑本 skill 刷新矩阵。
