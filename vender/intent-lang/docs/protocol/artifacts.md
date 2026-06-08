# intent-lang 工件协议（artifact protocol）

> 本文档锁定 `intent` CLI 在 `--format json` 模式下输出的所有结构。
> 这是上游消费者（popsicle skill、CI、IDE 插件、LLM agent）的稳定契约。
>
> **变更纪律**：新增字段是次要版本；删除/重命名字段是主要版本。
> 任何修改必须同时更新本文与 CLI；review 时把 schema diff 与代码 diff 一并审查。

所有顶层 JSON 对象都包含 `kind` 字段，便于消费者识别。
所有时间字段使用 ISO 8601；所有路径使用项目相对 POSIX 路径。

---

## `intent.consistency_report`

**来源**：`intent --format json check <file> [--include-asis]`

**用途**：CI 门禁、popsicle `idd-verify` skill 的输入。

```json
{
  "kind": "intent.consistency_report",
  "file": "examples/requirements/billing.intent",
  "include_asis": false,
  "diagnostics": [
    {
      "severity": "warning",
      "code": "W0010",
      "message": "goal references unresolved symbol: FooBar",
      "span": { "start": 123, "end": 130 }
    }
  ],
  "results": [
    {
      "name": "Transfer",
      "kind": "intent",
      "status": "verified",
      "lifecycle": "tobe",
      "detail": null
    },
    {
      "name": "LegacyV1",
      "kind": "intent",
      "status": "skipped",
      "lifecycle": "asis",
      "detail": "legacy track; pass --include-asis to verify"
    }
  ],
  "summary": { "verified": 1, "violated": 0, "skipped": 1, "errors": 0 }
}
```

**字段语义**

| 字段 | 类型 | 说明 |
|------|------|------|
| `results[].kind` | `intent` \| `safety` \| `theorem` | 验证目标种类 |
| `results[].status` | `verified` \| `violated` \| `skipped` \| `error` | 单条 VC 结果 |
| `results[].lifecycle` | `tobe` \| `asis` \| `current` | 来自 `@tobe`/`@asis` 标注（无标注 = `current`） |
| `results[].detail` | string \| null | 反例 / 跳过原因 / 错误信息（与 status 配套） |

CI 退出码：`summary.violated > 0 \|\| summary.errors > 0` → 非零。

---

## `intent.coverage_report`

**来源**：`intent --format json coverage <file>`

**用途**：评审者扫"哪些维度组合还没被任何规则提到"。

```json
{
  "kind": "intent.coverage_report",
  "file": "examples/requirements/access-control.intent",
  "coverages": [
    {
      "name": "permission-matrix",
      "total_combinations": 64,
      "covered": 17,
      "uncovered": [
        { "role": "Guest", "sensitivity": "Secret", "user_state": "authenticated", "resource_state": "archived" }
      ]
    }
  ]
}
```

**说明**：`covered` 用 *语法级见证* 判定（参见 DECISIONS.md 决策 8）。
未覆盖意味着"这个组合的字面值没在任何 safety/intent 子句中出现"。
人工 review 时可能裁定"已被通用规则覆盖"，这是 *沟通工具* 不是 *证明工具*。

---

## `intent.diff`

**来源**：`intent --format json diff <old> <new>`

**用途**：PR 评审、popsicle `spec-diff` skill 输入、breaking-change 通知。

```json
{
  "kind": "intent.diff",
  "old_file": "billing.v1.intent",
  "new_file": "billing.v2.intent",
  "added": [{ "name": "PartialRefund", "kind": "intent" }],
  "removed": [],
  "modified": [
    {
      "name": "Transfer",
      "kind": "intent",
      "classification": "tightened",
      "added_clauses":   ["require (amount <= sender.dailyLimit)"],
      "removed_clauses": []
    }
  ],
  "summary": { "added": 1, "removed": 0, "modified": 1, "potentially_breaking": 0 }
}
```

**`classification` 取值**

| 值 | 含义 | 兼容性 |
|----|------|--------|
| `loosened` | new require ⊆ old, new ensure ⊇ old | 一般向后兼容 |
| `tightened` | new require ⊇ old, new ensure ⊆ old | 调用方可能违反 |
| `reshaped` | 既不严格放宽也不严格收紧 | 视作 *潜在破坏* |
| `renamed`  | 仅形如重命名（reserved，v2 启用） | — |

`potentially_breaking` 计入 `tightened` 与 `reshaped`。
v1 使用语法启发式，v2 引入 Z3 自动判别（详见决策 10）。

---

## `intent.impact`

**来源**：`intent --format json impact <old> <new>`

**用途**：PR 描述自动填充"本次变更影响的目标 / stakeholder / 维度"。

```json
{
  "kind": "intent.impact",
  "diff": { "...": "intent.diff 嵌入" },
  "affected_goals":     ["用户余额永远满足透支底线"],
  "affected_stakeholders": ["finance", "compliance"],
  "affected_coverages": ["transfer-domain", "overdraft-policy"]
}
```

影响分析当前基于：
- `goal.realized_by` 反向索引
- `coverage.dimensions[*].values` 与变更子句字面文本的交集

未来扩展：跨文件 import 链、跨 spec 层引用。

---

## `testspec.draft`

**来源**：`intent --format json testspec <file>`

**用途**：交给 *独立 LLM* 生成测试用例（**反勾结原则**：生成 intent 的
LLM 与生成 testspec 的 LLM 必须不是同一个；详见 LLM.md）。

```json
{
  "kind": "testspec.draft",
  "file": "examples/requirements/billing.intent",
  "intents": [
    {
      "name": "Transfer",
      "lifecycle": "tobe",
      "params": [{ "name": "amount", "type": "Int" }],
      "rows": [
        { "label": "happy-path",         "given": "(amount > 0) && ...", "expect": "balance' == ..." },
        { "label": "violates require[0]", "given": "!((amount > 0))",     "expect": "behavior unspecified — caller error" }
      ]
    }
  ]
}
```

`rows` 是 *草稿*：覆盖每条 require 的违反路径 + 一个 happy-path。
下游必须把它转成具体语言的测试代码 *并独立验证*；不允许把 rows 当作直接断言。

---

## `intent.explanation`

**来源**：`intent --format json explain <file> <name>`

**用途**：自动生成 PR/RFC 的"自然语言版"声明、给非工程 stakeholder 看。

```json
{
  "kind": "intent.explanation",
  "name": "Transfer",
  "subject_kind": "intent",
  "summary": "Intent `Transfer` describes the operation that takes ...",
  "clauses": [
    { "category": "precondition", "raw": "(amount > 0)", "naturalized": "(amount > 0)" }
  ],
  "examples": {
    "satisfying": "(run `intent check` and inspect Z3 model — automated witnesses pending)",
    "violating":  "(violation witness requires running with --counterexample flag)"
  }
}
```

`naturalized` 使用 *模板* 翻译（替换 `==>` → "implies"，`&&` → "and"，等等）。
不调用 LLM —— 这是 anti-collusion 设计：解释器与生成器不能是同一系统。

---

## 退出码总表

| 命令 | 退出码 0 | 退出码 1 |
|------|---------|---------|
| `check` | 全部 verified/skipped | 任一 violated 或 error |
| `parse` | 解析成功 | 词法/语法/类型错误 |
| `coverage` | 计算成功 | 文件无法解析 |
| `diff` / `impact` | 计算成功 | 任一文件无法解析 |
| `testspec` / `explain` | 生成成功 | 文件无法解析 / 名字不存在 |

`--format json` 模式下，错误也写入 stdout 的 JSON 对象（kind=`intent.error`），
而不是 stderr，方便 pipeline 消费。
