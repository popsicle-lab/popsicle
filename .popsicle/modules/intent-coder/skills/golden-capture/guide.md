# golden-capture 使用指南

补上 pipeline 缺失的一环：**为 rewrite 型迁移起 pinned legacy、录真实输出为可回放 fixture**
（feedback #18/S5）。`equivalence-baseline` 的 prompt 要求「实跑 legacy vs new 差分」，但
pipeline 从不提供跑 legacy 的机制——verbatim 平移下这退化成 characterization 快照。本 skill
把「跑 legacy」显式化，只服务 **rewrite** 切片。

```
fact-extractor（behavior 事实 + golden_candidate）
    → golden-capture（本 skill，起 legacy 录 fixture）        ← 仅 rewrite
    → equivalence-baseline（用 fixture 做 legacy↔new 差分）
    → cutover-author
```

## 何时用 / 何时不用

| migration_mode | 用 golden-capture？ | golden 性质 |
|---|---|---|
| `verbatim`（逐字节平移）| **不用** | characterization（快照自证，等价平凡）|
| `rewrite`（真重写）| **用** | legacy 录制 + new 回放的**差分** |

先读切片 `task` / `equivalence-report` 的 `migration_mode`。verbatim 直接跳过本 skill。

## 定位

| 做 | 不做 |
|---|---|
| 起 **pinned** legacy submodule 录真实 I/O | 用 new 代码充当 legacy（那是自证）|
| 每条 fixture 溯源到一条 behavior 事实 | 凭空造 fixture / 编 exit code |
| 记录真实 exit + 内容 sha256（可复算）| 手写「pass」而不跑 |
| 写清 new 回放 + 归一化 + 差分判定 | 替代 equivalence 的对账（那是下游）|

## 输入 / 输出

- 输入：`<slug>.facts.yaml`（`kind: behavior` 且 `golden_candidate` 非空的事实）、`legacy/<name>` submodule。
- 输出：`<slug>.golden-capture-manifest.yaml`（机器可复算索引）+ `<slug>.golden-capture-plan.md`（实跑证据）
  + `docs/baseline/<date>/<slice>/fixtures/<golden_id>.*`（fixture 快照）。

## 硬规则

1. **pin 必须真实**：`legacy_pin != REPLACE_WITH_REAL_LEGACY_PIN` 且 == `git -C legacy/<name> rev-parse HEAD`。
2. **不臆造**：录不到就登记原因，交 cutover 作 divergence + ADR。
3. **可复算**：每条 fixture 带 `stdout_sha256`，equivalence 回放时复算比对，防事后篡改。
4. **隔离**：legacy 构建/运行与 new 完全隔离（独立 target/端口/只读目录）。

## 与 gate 的关系

equivalence / cutover 的机验 gate（见 ROADMAP §10）校验 `summary.golden_pass` 可复算、`legacy_pin` 真实。
本 skill 产出的 manifest 让那些 gate 有据可依：rewrite 切片没有 golden-capture-manifest 就无法通过真正的差分对账。
