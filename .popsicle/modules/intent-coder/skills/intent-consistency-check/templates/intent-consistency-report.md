---
artifact: intent-consistency-report
slug: {slug}
generated_by: intent-consistency-check
mode: observe              # observe | gate —— skill 始终 observe；gate 是 CI 用 tool exit code 实现
last_updated: {date}
intent_files_checked: 0
vcs_total: 0
vcs_verified: 0
vcs_failed: 0
vcs_skipped: 0
overall: pass              # pass | has-failures
consecutive_clean_runs: 0  # 截至本次，连续 overall=pass 的次数（读上次报告 +1；非 pass 归零）
gate_ready: false          # consecutive_clean_runs >= 3 且本次 overall=pass 时为 true
query_anchors:
  - "我的 .intent 自洽吗？"
  - "哪条意图验证失败了，反例是什么？"
  - "为什么有的意图被 skip 了？"
---

# Intent 一致性报告 — {slug}

> 由 `intent-consistency-check` skill 调用 `intent-validate` tool（intent-lang + Z3）生成。
> **只验证逻辑一致性**（require / ensure / invariant / safety 之间不打架），
> 不覆盖时间 / 性能 / 运行时约束——那些由测试守护，见各 task「可观察的成功标志」。

## Summary

| 指标 | 数 |
|---|---|
| 检查的 `.intent` 文件 | 0 |
| VC 总数 | 0 |
| ✅ verified | 0 |
| ❌ failed | 0 |
| ⚠️ skipped | 0 |
| 总体结论 | PASS / HAS-FAILURES |
| 模式 | observe（Phase 0：报告但不阻断 pipeline）|

一句话结论：……

## Per-File Results

> 每个 `.intent` 文件一行。`exit` 是 `intent check` 的退出码（0 = 全通过/合理跳过）。

| 文件 | verified | failed | skipped | exit | 结论 |
|---|---|---|---|---|---|
| products/auth/intents/invariants.intent | 0 | 0 | 0 | 0 | ✅ |

## Goal Trace（合并 realized_by）

> `popsicle tool run intent-validate path=products/<product>/intents` 在 per-file Z3 通过后
> 自动合并该 product 下全部 `.intent` 并检查 L4 goal 追溯。

| 指标 | 值 |
|---|---|
| 检查的 product | {product} |
| 孤儿 goal（realized_by 空）| 0 |
| 缺失 goal（contracts 存在但零 goal）| 0 |
| 未知引用 | 0 |
| goal 追溯结论 | PASS / FAIL |

> **硬门禁**：`contracts.intent` 存在时合并程序须 ≥1 个 `goal`；否则 `E_PRODUCT_MISSING_GOALS`。
> goal 总数 = 0 且 product 非 trivial → 结论必须 **FAIL**，不可 complete intent-check stage。

失败明细（逐条抄 tool 输出）：

```
（粘贴 E_PRODUCT_MISSING_GOALS / E_GOAL_UNLINKED / E_GOAL_UNKNOWN_REF）
```

## Failures

> 每个 `status == failed` 的 VC：名字 + 文件 + Z3 反例**原文** + 处置建议。
> 没有失败则写「（无）」。**禁止臆造反例**——只抄 tool 实际输出。

### （模板）`<VC 名>` — `<文件>`

- **kind**: intent / theorem
- **反例**（Z3 原样输出）：

```
<counterexample 原文，逐字粘贴>
```

- **怎么读这个反例**：Z3 给的是一组让 goal 不成立的变量赋值。逐字段对照
  intent 的 require/ensure：找出哪个 primed 字段取了「不该有」的值。常见根因——
  ① 漏了 `ensure x' == x`（无 frame，字段自由漂移）；② safety 用了 unprimed
  只约束旧态；③ require 太弱没排除非法前态。**把「哪个字段=什么值违反了哪条约束」
  写成一句话**，再选处置。
- **处置建议**（四选一）：
  - 修 spec（require/ensure 写错了，或漏了 frame `ensure x' == x`）
  - 这是预期收紧（需要新 PDR 记录决策变更）
  - 降级为测试断言（约束本不该进 `.intent`，违反 D2 能力边界）
  - 待 intent-lang 支持（记入 ROADMAP，暂挂）

## Skipped

> `status == skipped` 不计入失败。列出原因，便于追踪工具能力边界。
> 常见原因：struct-typed `forall` theorem 尚未实现；`@asis` 遗留意图默认跳过。

| VC | 文件 | 原因 |
|---|---|---|
| — | — | — |

## Disposition

- **observe（本 skill 的行为）**：即使存在 failed，skill **不阻断** pipeline；
  每个 failed 必须在 Failures 中列出并指派负责人跟进。
- **gate（不是 skill 状态，是 CI 行为）**：在 CI 里跑 `intent-validate` tool，
  靠它的 **exit code ≠ 0** 阻断合并（tool FAIL 时 exit 1）。skill 只负责出报告 +
  判断「现在该不该开这个 CI 闸」。
- 跟进项：
  - [ ] ……

## Gate Readiness（observe → gate 退出判据）

> 何时把 CI 从「跑了看看」升级为「红了就拦」。判据要量化、可复算，避免拍脑袋开闸。

| 项 | 值 |
|---|---|
| 本次 overall | pass / has-failures |
| consecutive_clean_runs（含本次）| 0 |
| 升级阈值 N | 3 |
| **gate_ready** | false |

**判据**：`consecutive_clean_runs >= N(=3)` 且本次 `overall == pass` → `gate_ready = true`。
含义：连续 3 次（含本次）对全量 `.intent` 跑验证都 0 failed/unknown，说明 spec 已稳定，
开 CI 硬闸不会天天误伤。任一次出现 failed/unknown → 计数归零，回到 observe 攒数。

**计数怎么算**：读取上一次 `*.intent-consistency-report.md` 的 `consecutive_clean_runs`：
本次 pass 则 `+1`，本次 has-failures 则置 `0`。首次运行从 0 起算。

**达到 gate_ready 后的开闸动作**（CI 配置建议）：

```yaml
# .github/workflows/intent-gate.yml（或等价 CI step）
- name: intent consistency gate
  run: |
    popsicle tool run intent-validate path=products format=text
    # tool 对任一 FAILED 返回 exit 1 → 此 step 失败 → 阻断合并
```

> 注意：`popsicle tool run` 在被调 tool 非零退出时会一并非零退出，正好用作闸门。
> skipped（struct-forall theorem 未实现）**不算** failed，不会误拦。

---

## 检查清单（提交前勾完）

- [ ] 枚举了项目内**所有** `.intent` 文件（products/*/intents/、docs/invariants/）
- [ ] 每个文件的结果都来自 tool 的**真实输出**，未臆造
- [ ] 所有 failed 的反例已逐字粘贴，且每条都写了「哪个字段=什么值违反哪条约束」
- [ ] 所有 skipped 已注明原因
- [ ] frontmatter 计数与正文一致
- [ ] 已读取上次报告并算出 consecutive_clean_runs 与 gate_ready
- [ ] 若 gate_ready=true，已在 Gate Readiness 给出 CI 开闸建议
