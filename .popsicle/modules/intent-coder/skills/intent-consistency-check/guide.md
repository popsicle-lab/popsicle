# intent-consistency-check 使用指南

把 intent-lang 的形式化验证接进 IDD 工作流：枚举项目里所有 `.intent` 文件，
逐个跑 Z3 一致性检查，汇总成一份可追溯的报告。

这是 intent-coder「intent → 机器验证」闭环的**验证端**。上游是
`prd-writer`（产出意图种子）与 `intent-spec-writer`（把种子正式化为合法 `.intent`）；
底层执行器是 `intent-validate` tool。

## 定位：observe vs gate（谁来阻断）

| 角色 | 实现者 | 行为 |
|---|---|---|
| **observe** | 本 skill | 跑验证、出报告、列出失败 + 反例，但**不阻断** pipeline |
| **gate** | **CI**（不是 skill）| 在 CI 跑 `intent-validate` tool，靠它 **exit code ≠ 0** 拦合并 |

关键认知：**gate 不是 skill 的某个状态，而是 CI 的一个 step**。tool 在任一 VC failed
时 `exit 1`，CI step 随之失败——这就是硬闸。skill 永远只做 observe，外加判断
「现在该不该让 CI 开这个闸」。这样分工干净：skill 读不到 exit code、也不该假装能拦。

### observe → gate 退出判据（连续 N 次零失败）

先 observe 是刻意的：让团队先看到「形式化验证能抓到什么」，建立信任，再收紧成闸门。
过早 gate 会在 spec 尚未适配真实语法时制造噪音、引发抵触。量化的升级判据：

- 报告 frontmatter 维护 `consecutive_clean_runs`：本次 `overall=pass` 则在上次基础
  上 `+1`，出现任何 failed/unknown 则归 `0`。
- 当 `consecutive_clean_runs >= 3` 且本次 pass → `gate_ready = true`：spec 已稳定，
  开 CI 硬闸不会天天误伤。
- 达到后在 CI 增加一步（exit code 即闸门，skipped 不算失败、不会误拦）：

```yaml
- name: intent consistency gate
  run: popsicle tool run intent-validate path=products format=text
  # 任一 FAILED → tool exit 1 → step 失败 → 阻断合并
```

判据写进报告的 `Gate Readiness` 段，可复算、不拍脑袋。

## 调用底层 tool

```bash
# 一次性安装 tool（装到 .popsicle/tools/）
popsicle tool install ./tools/intent-validate

# 验证单个文件，拿机器可解析的 JSON
popsicle tool run intent-validate path=products/auth/intents/invariants.intent format=json

# 人读输出
popsicle tool run intent-validate path=products/auth/intents/invariants.intent format=text
```

两个必须知道的坑：

1. **双层 JSON**：`popsicle tool run --format json` 把 tool 的 stdout 再包一层
   `{exit_code, stdout, stderr}`。intent-lang 的 JSON 在**内层** `stdout` 字段里。
   想省事就用 `format=text` 调 tool，或直接调 `intent --format json check <file>`。
2. **exit code 即结论**：intent check 在「任一 VC failed / 文件不合法」时 `exit 1`，
   于是 `popsicle tool run` 也会以错误退出。observe 模式下这是**预期数据**，
   不要当成 skill 崩溃——捕获 JSON、记进报告、继续。

## 怎么读 intent-lang 的结果

`results[].status` 的语义：

| status | 含义 | 计入失败？ |
|---|---|---|
| `verified` | Z3 证明该 VC 成立 | 否（这是目标）|
| `failed` | Z3 找到反例，`detail` 含反例原文 | **是** |
| `unknown` | Z3 无法判定（超时/不可判定理论）| **是**（需人看）|
| `skipped` | 工具暂不支持（struct-typed theorem）或 `@asis` 默认跳过（迁移时用 `include_asis=true` 让其参与，见下）| 否 |

报告里 failed 的反例必须**逐字粘贴**，那就是给人/LLM 修 spec 的最短线索。

## @asis 意图的处置（迁移，#20）

`@asis`（legacy 实然）默认 `asis-skipped`——连 Z3 都不进，等于带语法的注释。迁移切片里
「legacy 到底做了什么」恰是最该锁定的，别让它形式化真空。做法：

1. **让 @asis 也进 Z3（自洽性）**：
   ```bash
   popsicle tool run intent-validate path=products/<p>/intents include_asis=true format=text
   ```
   `include_asis=true`（popsicle CLI 归一为 intent-lang 的 `--include-asis`）使 `@asis` 参与验证
   （不再 `asis-skipped`）；失败同样记进报告。此前 CLI 会吞掉该参数，现已修复贯通。
2. **产出 @asis↔@tobe 分叉报告**：逐条对照同名/对应能力的 `@asis` 与 `@tobe`，显式列
   「保留（等价）/ 有意分叉（+理由+ADR）」。**有意分叉必须对应一条 divergence + ADR**，
   不要在 `@tobe` 里悄悄改。写进报告模板的「@asis ↔ @tobe 分叉」段。
3. **边界**：让 `@asis` **默认**进 Z3、或**自动**生成分叉报告，属**上游 intent-lang** 能力
   （记 ROADMAP）；当前靠 `include_asis` + 本段人工对照。

## 能力边界（决策 D2）

intent-lang 只验证**逻辑一致性**，不处理时间 / 时序 / 性能 / 概率 / 运行时事实。
形如「P95 ≤ 90s」「5 秒内响应」「密码不出现在日志」这类约束**不属于** `.intent`，
应写进 task 文件的「可观察的成功标志」，由 benchmark / e2e / 单元测试守护。
本 skill 若发现这类约束被塞进 `.intent`，应在报告中标记为违反 D2。

## Goal 追溯闸（realized_by）

Z3 通过后，`popsicle tool run intent-validate path=products/...` 还会对**合并后的**
每个 product 跑 goal 追溯检查：

| 代码 | 含义 |
|---|---|
| `E_GOAL_UNLINKED` | `contracts.intent` 中某 goal 的 `realized_by` 为空 |
| `E_GOAL_UNKNOWN_REF` | `realized_by` 引用了合并程序中不存在的符号 |
| `E_PRODUCT_MISSING_GOALS` | 存在 `contracts.intent` 但合并程序中**零个** `goal` 块 |

任一命中 → tool **exit 1**（与 VC failed 同等，可进 CI gate）。**零 goal 亦 FAIL**——
不可仅因 per-file Z3 全绿就 complete intent-spec / intent-check stage。

## 写 .intent 的四条硬规则（来自 dogfood 发现）

0. **关键字（别写错）**：不变量**子句**是 `invariant`（用在 intent {} 或
   `safety {}` 内）；`safety` 是**顶层声明** `safety 名(参) { invariant ... }`，
   不能当子句写进 intent。intent {} 内合法子句只有 `require` / `ensure` /
   `invariant`（把 `safety` 写进去会报 expected require/ensure/invariant）。
1. **后态用 primed 变量**：`invariant` 子句要约束操作**之后**的状态，
   必须显式写 `x'`。写成 unprimed 只验旧态，会假通过（vcgen 的真实行为）。
2. **一个文件 = 一个验证作用域**：vcgen 把每条 `safety` 声明无条件合并进文件内
   所有 intent，且靠**参数名**绑定。所以一个 `.intent` 文件只放共享同一组
   不变量的操作；不相关操作放到各自的文件，否则自由变量会让无关 intent 误判 FAIL。
3. **无 frame 假设**：intent-lang **不**默认「未提及字段不变」。要声明某操作不改
   某字段，必须显式 `ensure x' == x`；否则该 primed 字段自由，一旦被
   `invariant`（含 `safety` 声明内的）约束就会必然 FAIL。
4. **纯 require+ensure = trivial verified**：只有 `invariant` 子句产生
   验证目标（goals），`ensure` 只是假设。所以孤立的「操作规约」（acceptance 那种）
   不会被证伪——真正的一致性验证来自 invariants 里 `safety` 声明 + 完整 ensure 的组合。
   报告里要清楚区分「trivial verified（操作规约）」与「真正验证了不变量」。
