# intent-spec-writer 使用指南

把 prd-writer 的 **acceptance-intent 种子** 收紧成可合并、可被 Z3 验证的
**正式 intent-lang**。这是「intent → 机器验证」闭环里承上启下的一棒：

```
product-debate → prd-writer →（种子）→ intent-spec-writer →（正式 .intent）→ intent-consistency-check（Z3 闸）
```

## 为什么内置（决策 D3）

种子已经是合法 intent-lang 骨架（prd-writer v0.2+），但「能不能真喂给 Z3、合并进
现有 `.intent` 不打架」这一步直接决定闭环能否成立——是 dogfood 必经环节，不外包。
本 skill 做的不是「发明语义」，而是**规范化 + 分层 + 查冲突 + 跑通**，保持薄。

## 五件事

1. **分层归位**：把种子里每条内容按 PRD § Intent Mapping 归到正确的层——
   - 操作后态 → `acceptance.intent`（require/ensure）
   - 跨操作保持型不变量 → `invariants.intent`（safety + primed）
   - 模块接口契约 → `contracts.intent`（goal + `[Awaiting ADR]`）
2. **剥离 D2 约束**：时间 / 性能 / 运行时事实 / 概率**不进** `.intent`，
   登记到对应 task 的「可观察的成功标志」，由测试守护。
3. **四规则审查**：见下。
4. **去重查冲突**：与目标产品现有 `.intent` 比对，复用已有 type、避免重名/矛盾。
5. **自验**：交付前用 `intent check`（经 `intent-validate` tool）跑出 exit 0。

## intent-lang 四条硬写法规则（来自 dogfood，违反必出错）

1. **后态用 primed `x'`**：safety/invariant 要约束操作**之后**的状态必须写 primed；
   unprimed 只验旧态 = 假通过。
2. **一个文件 = 一个验证作用域**：每条 `safety` 被无条件合并进文件内所有 intent，
   且靠**参数名**绑定。→ `acceptance.intent` 只放操作 intent，保持型不变量进
   `invariants.intent`；不相关操作分文件。
3. **无 frame 假设**：不默认「未提及字段不变」。要声明不改某字段必须显式
   `ensure x' == x`。
4. **纯 require+ensure = trivial verified**：`ensure` 只是假设，只有 `invariant`/
   `safety` 产生验证目标。acceptance 操作规约属 trivial verified（合法、可跑，但不
   被证伪）；真正的不变量验证靠 invariants 的 safety + 完整 ensure。

## 能力边界提醒

- intent-lang 不支持聚合（`count`/`where`）；集合基数 / 双实体唯一性只能写成
  struct-forall `theorem`，当前会被 `skipped`（仅声明意图，等 intent-lang 支持）。
- 报告里必须区分「trivial verified（操作规约）」与「真正验证了不变量」，
  别让一片绿色 ✅ 误导成「全都被证明了」。

## 产物

- `{slug}.acceptance-formal.intent`：收紧后的 acceptance 增量，可直接 intent check。
- `{slug}.intent-spec-report.md`：分层归位 / 剥离清单 / 四规则审查 / 验证结果 /
  冲突检查 / 合并计划。

合并：按报告「合并计划」追加到 `products/{target_product}/intents/acceptance.intent`，
再跑 `intent-consistency-check` 做 Z3 闸。
