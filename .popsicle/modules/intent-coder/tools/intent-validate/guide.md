# intent-validate 使用指南

把 intent-lang 的形式化验证（parse → typecheck → VC 生成 → Z3 求解）封装成一个
popsicle tool，供 `intent-consistency-check` skill 和 CI 闸门调用。

## 何时用

- 写完/改完任意 `.intent` 文件后，验证「意图自身是否自洽」（require/ensure/invariant/safety 之间不打架）。
- CI 中作为「意图一致性闸门」：任一 VC 失败则 `exit 1`，阻断合并。
- 在写实现代码之前——先证明需求不自相矛盾，再动手。

## 参数

| 参数 | 必填 | 默认 | 说明 |
|---|---|---|---|
| `path` | 是 | — | `.intent` 文件或目录路径（相对仓库根或绝对路径；目录会递归枚举 `*.intent`） |
| `format` | 否 | `json` | `json` 给 skill/CI 解析；`text` 给人读 |
| `include_asis` | 否 | 空 | 传 `--include-asis` 时一并验证 `@asis` 遗留意图 |
| `merge` | 否 | 空 | `merge=true`：**整目录一个 program**（#14）——把每个产品 `intents/*.intent` 合并成一份跑单次 `intent check`，消除跨文件 `realized_by` 的 W0010、获得整程序作用域验证。默认 per-file（行为不变）。 |

## 退出码

| exit | 含义 |
|---|---|
| `0` | 全部 VC `verified` 或合理 `skipped`（如 struct-typed theorem 尚未实现、`@asis` 默认跳过）；且（path 在 `products/` 下时）合并 goal 追溯通过 |
| `1` | 解析错误、类型错误，或任一 VC `failed` / `unknown` / `error`；或合并后存在孤儿 goal / 未知 `realized_by` 引用 |
| `127` | 环境缺失：找不到 `intent` 可执行文件（安装 v0.1.1+ release 或 DMG 捆绑版） |

## JSON 输出结构

```json
{
  "file": "auth.intent",
  "diagnostics": [{ "level": "error|warning|info", "code": "", "message": "", "line": 0, "col": 0 }],
  "results": [{ "name": "", "kind": "intent|theorem", "status": "verified|failed|unknown|skipped|error", "detail": null, "track": "primary|asis-skipped" }],
  "ok": true
}
```

- `ok` 为顶层结论：true = 可放行。
- `results[].status == "failed"` 时，`detail` 含 Z3 反例文本——这就是「最短反例」，直接喂给人或 LLM 修 spec。
- `status == "skipped"` 不计入失败（当前 intent-lang 不支持 struct-typed `forall` theorem，会标 skipped；`@asis` 也默认 skip）。
- 当 `path` 指向目录时，tool 会按路径排序逐个输出每个 `.intent` 文件的原生结果；
  任一文件非 0 则整体非 0。`format=text` 适合人工报告，`format=json` 适合上层 skill
  逐段解析。

## 合并 Goal 追溯闸（popsicle 内置）

当通过 `popsicle tool run intent-validate` 调用且 `path` 落在 `products/` 下时，CLI 在
per-file Z3 **全部通过**后还会：

1. 合并每个 product 的 `intents/*.intent`
2. 要求每个 `goal` 的 `realized_by` 非空，且引用已声明的 safety/intent/theorem

失败时额外输出 `E_GOAL_UNLINKED` / `E_GOAL_UNKNOWN_REF` 并 **exit 1**。单文件
`intent check` 对跨文件 `realized_by` 仅 W0010 warning——合并闸才是交付标准。

### 关于跨文件 `realized_by` 的 W0010 噪音（feedback #14）

`contracts.intent` 的 `goal.realized_by` 引用 `acceptance.intent` / `invariants.intent` 里的
声明时，**per-file** `intent check` 恒报 `W0010: realized_by references unknown declaration`，
即便整体 `status: ok`、`exit 0`。这是**预期噪音，不是错误**，原因与处置：

- **成因（上游限制）**：`intent` 二进制的 `check` 子命令只接受**单个文件**（`file: PathBuf`），
  没有「整个产品 `intents/` 目录当一个 program」的多文件解析模式。所以逐文件校验时，
  跨文件符号自然「未知」。
- **谁才是权威**：popsicle 的**合并 Goal 追溯闸**（上一节）会把每个 product 的 `intents/*.intent`
  合并后校验 `realized_by`——**这**才是交付判据。合并闸通过 = `realized_by` 正确，W0010 可忽略。
- **判读**：只要 `intent-validate` 整体 `exit 0` 且无 `E_GOAL_*`，per-file 的 W0010 行**不必处理**。
  不要为了消 W0010 去把跨文件声明塞回单文件（那会违反「一文件一作用域」规则，见 §四条硬规则）。
- **W0010 本身无害**：只要整体 `exit 0` 且无 `E_GOAL_*`，per-file 的 W0010 **不必处理**——
  合并 goal 追溯闸已权威校验 `realized_by`。这是最常见、最安全的判读。
- **整目录一个 program（`merge=true`，慎用）**：`popsicle tool run intent-validate
  path=products/<p>/intents merge=true` 把该产品 `intents/*.intent` 合并成单个 program 跑一次
  `intent check`，跨文件 `realized_by` 同作用域解析，**不再有 W0010**。⚠️ 但这是**更严格**的
  整程序验证：合并后每条 `safety` 声明会**无条件合并进所有 intent**（见 §四条硬规则 规则 2），
  于是**未约束该不变量后态的操作 intent 会因自由变量 FAIL**——这不是 bug，是整程序语义的真实结果。
  因此 `merge=true` 是「有意识地检查跨文件交互」的诊断模式，**不是**盲目的消噪开关；默认 per-file
  行为不变。若产品刻意把 acceptance / invariants 分文件以隔离作用域，per-file 才是其设计意图。

## 与 intent-lang 能力边界

intent-lang 是 Hoare 逻辑 + SMT，**只验证逻辑一致性**，不处理时间/时序/性能。
形如「P95 ≤ 90s」「5 秒内响应」这类约束**不要**写进 `.intent`——它们属于
task 文件的「可观察的成功标志」，由 benchmark / e2e 测试守护。详见仓库 `ROADMAP.md` 决策 D2。
