# equivalence-baseline 使用指南

为单个 slice 建立 **legacy vs new 等价性基线**，是 Strangler Fig 切流前的硬门禁。

```
shadow-implementer（代码 + coverage）
    → equivalence-baseline（本 skill）
    → cutover-author
```

## 切流门禁（与 CONTRIBUTING §4 对齐）

满足其一即可进入 cutover：

1. **≥5 条 golden** 同输入 legacy/new **diff 为空**
2. 或：未通过的项均在 **Divergence** 表中有 **Accepted ADR** 登记

## 产物布局

```
docs/baseline/<YYYY-MM-DD>/<slice>/
  README.md
  golden-001-*.sh          # 或 rust integration test
  fixtures/
```

## traceability

本 skill 起草 `migration/traceability.md` 行；**cutover ADR Accepted 后**
由 cutover-author 正式写入并标 `in-shadow` → `cutover-done`。

## 常见 divergence

| 现象 | 处理 |
|---|---|
| intent 要求 `body' == body`，legacy `trim_start` | 登记 divergence + 切流 ADR 说明「新语义为准」|
| extractor 简化实现 vs legacy regex | golden 比「kind + title 集合」而非字节 |
| 尚无 cli-ux | golden 直接调 lib API，CLI 对账留给 slice-3 |

## 红线

- 不臆造 pass——必须实跑脚本
- 不把 divergence 静默吞掉
- 不修改 `.intent`（修 spec 走 intent-spec-writer + PDR）

## Agent 观测

verify 阶段若使用 LLM，先 `popsicle tool run telemetry action=guide` 再上报；`doc check` 通过后 **必须**打 `popsicle.run.score`（见 JSON `telemetry_hint`），并确认 `agent_coverage.gaps` 无本 stage 的 doc。
