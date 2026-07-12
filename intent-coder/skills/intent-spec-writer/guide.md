# intent-spec-writer 使用指南

把 prd-writer 的 **acceptance / invariants 种子** 与 adr-writer 解锁后的
**contracts handoff** 收紧成可合并、可被 Z3 验证的**正式 intent-lang**。这是
「intent → 机器验证」闭环里承上启下的一棒：

```
product-debate → prd-writer →（acceptance/invariants 种子）→ intent-spec-writer
arch-debate → rfc-writer → adr-writer →（contracts unlocked）↗
intent-spec-writer →（正式 .intent）→ intent-consistency-check（Z3 闸）
```

## 为什么内置（决策 D3）

种子通常是合法 intent-lang 骨架（prd-writer v0.2+），contracts handoff 通常是
goal/工单形态；「能不能真喂给 Z3、合并进现有 `.intent` 不打架」这一步直接决定闭环
能否成立——是 dogfood 必经环节，不外包。
本 skill 做的不是「发明语义」，而是**规范化 + 分层 + 查冲突 + 跑通**，保持薄。

## 五件事

1. **分层归位**：把种子里每条内容按 PRD § Intent Mapping 归到正确的层——
   - 操作后态 → `acceptance.intent`（require/ensure）
   - 跨操作保持型不变量 → `invariants.intent`（safety + primed）
   - 模块接口契约 → `contracts.intent`（goal；ADR Accepted 后可收紧）
2. **剥离 D2 约束**：时间 / 性能 / 运行时事实 / 概率**不进** `.intent`，
   登记到对应 task 的「可观察的成功标志」，由测试守护。
3. **四规则审查**：见下。
4. **去重查冲突**：与目标产品现有 `.intent` 比对，复用已有 type、避免重名/矛盾。
5. **回填 `realized_by`**：合并 acceptance / invariants 后，在 `contracts.intent`
   每个 `goal` 上补 `realized_by: [SafetyOrIntent, ...]`，指向**已声明**的
   safety / intent / theorem 名（跨文件引用；单文件 `intent check` 会 W0010，合并后须通过）。
6. **自验**：交付前用 `intent check` + 合并 goal 追溯闸（经 `intent-validate` tool）跑出 exit 0。
   - 存在 `contracts.intent` 时须 ≥1 个 `goal`（否则 `E_PRODUCT_MISSING_GOALS`）

## intent-lang 四条硬写法规则（来自 dogfood，违反必出错）

0. **关键字**：不变量**子句**是 `invariant`（intent {} 或 `safety {}` 内）；
   `safety 名(参) { invariant ... }` 是**顶层声明**，只在 invariants.intent，
   不能当子句写进 intent。intent {} 内合法子句：`require` / `ensure` / `invariant`。
1. **后态用 primed `x'`**：`invariant` 要约束操作**之后**的状态必须写 primed；
   unprimed 只验旧态 = 假通过。
2. **一个文件 = 一个验证作用域**：每条 `safety` 声明被无条件合并进文件内所有 intent，
   且靠**参数名**绑定。→ `acceptance.intent` 只放操作 intent，保持型不变量进
   `invariants.intent`；不相关操作分文件。
3. **无 frame 假设**：不默认「未提及字段不变」。要声明不改某字段必须显式
   `ensure x' == x`。
4. **纯 require+ensure = trivial verified**：`ensure` 只是假设，只有 `invariant`
   子句产生验证目标。acceptance 操作规约属 trivial verified（合法、可跑，但不
   被证伪）；真正的不变量验证靠 invariants 的 `safety` 声明 + 完整 ensure。

## 迁移等价方法论：@asis → @tobe（feedback #13）

intent-lang + Z3 只证**逻辑自洽**，**不证「与 legacy 等价」**。迁移场景里自顶向下
**设计**出来的 intent 断言的是「我以为 legacy 做什么」，未与 legacy 对齐——等价闭环没合上。
做迁移 spec 时按下面走，别一上来全写 `@tobe`：

1. **@asis 优先**：先从 fact-extractor 的 `facts.yaml` `behavior:` 事实（收割的 legacy
   测试 / 可观测 I/O）抽 `@asis` intent，**如实刻画 legacy 实然行为**（含怪癖 / panic 语义）。
   `@asis` 默认被 Z3 skip（不阻断），但把「legacy 到底做什么」显式写进档案。
2. **逐条决定「保留 vs 改进」**：每条 `@asis` 行为要么升为 `@tobe`（保留、成为验证目标），
   要么显式记为「改进」（新行为，链到 ADR 说明为什么偏离 legacy）。等价 vs 改进在每条行为上**显式**。
3. **intent 当等价 oracle**：acceptance intent 部分从 legacy 收割的测试 / 追踪 I/O 生成，
   使其可证「这就是 legacy 做的」；新实现再满足**同一** intent + golden diff（行为闸）。
4. **golden 骨架从 facts.yaml 自动派生**：`api:` 事实 → 「调 legacy rpc 录输出」的对账骨架，
   new 侧回放比对。task frontmatter 的 `equivalence.golden_id` 与 `facts.yaml` 的
   `golden_candidate` 呼应，形成 task ↔ golden ↔ intent 的等价三元组。
5. **漂移检测**：换 pinned commit 重跑 fact-extractor → diff `facts.yaml` 的 fact_id/source，
   即得「哪些 intent/task/golden 因 legacy 变动而过期」——事实基从一次性快照变活契约。

> **快车道设想（`migration-preserve`，尚未内置为 pipeline）**：对「保留 legacy 行为」为主、
> 无重设计需求的切片，理想链路是 facts → `@asis` 捕获 → golden 骨架 → 冻结 ADR → shadow →
> equivalence 闸，**跳过**为「重设计」准备的 debate→arch→rfc 链。目前无专用 `@asis`-capture /
> golden-skeleton skill，先用本方法论在 `migration-slice-spec` + `migration-slice-delivery`
> 里手动执行；待相关 skill 落地后再固化为独立 pipeline（见 ROADMAP）。

## 能力边界提醒

- intent-lang 不支持聚合（`count`/`where`）；集合基数 / 双实体唯一性只能写成
  struct-forall `theorem`，当前会被 `skipped`（仅声明意图，等 intent-lang 支持）。
- 报告里必须区分「trivial verified（操作规约）」与「真正验证了不变量」，
  别让一片绿色 ✅ 误导成「全都被证明了」。

## 产物

- `{slug}.acceptance-formal.intent`：收紧后的 acceptance 增量，可直接 intent check。
- `{slug}.intent-spec-report.md`：分层归位 / 剥离清单 / 四规则审查 / 验证结果 /
  冲突检查 / 合并计划。

合并：按报告「合并计划」追加或就地更新
`products/{target_product}/intents/{acceptance,invariants,contracts}.intent`，再跑
`intent-consistency-check` 做 Z3 闸。

### `realized_by` 回填纪律

- **时机**：acceptance / invariants 增量合并进产品目录**之后**（否则引用的符号还不存在）。
- **方向**：L4 `goal` → L1 `safety` / L2 `@tobe intent`（可混列；见 billing 示例）。
- **映射依据**：ADR finalization report「收紧工单」+ PRD Intent Mapping 行——每个 goal
  至少链到 1 条 safety 或 intent；契约 2/3 优先链 invariants，用户可见行为链 acceptance。
- **验收**：`popsicle tool run intent-validate path=products/<product>/intents format=text`
  在 Z3 通过后，合并 goal 追溯闸亦须 exit 0（孤儿 goal / 未知引用 → exit 1）。
