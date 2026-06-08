# intent-lang 与大模型 (LLM)

> intent-lang 可能是目前最适合 LLM 生成的形式化语言。

---

## 为什么 LLM 友好？

| 特性 | 原因 |
|------|------|
| **关键字少** | 7 个核心词，LLM 只需学一个模板 |
| **接近自然语言** | `require amount > 0` 几乎就是英语 |
| **不需要写实现** | LLM 最容易在算法实现上犯错，intent-lang 不需要 |
| **结构固定** | 每个 intent：名字 → 参数 → require → ensure → invariant |
| **短** | 一个 intent 通常 5-10 行 |

### 与其他形式化语言的生成难度对比

| 语言 | 生成难度 | 验证能力 | 自我修正 |
|------|---------|---------|---------|
| Python | ⭐⭐ | ❌ 无 | ❌ 难 |
| Rust | ⭐⭐⭐ | ⭐⭐ 类型 | ⚠️ 一般 |
| Dafny | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ SMT | ⭐⭐⭐ |
| TLA+ | ⭐⭐⭐⭐ | ⭐⭐⭐ 模型检查 | ⭐⭐ |
| Lean 4 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ |
| **intent-lang** | **⭐⭐** | **⭐⭐⭐⭐** | **⭐⭐⭐⭐** |

**独一无二的组合：低生成难度 + 高验证能力 + 高自我修正能力。**

---

## 核心机制：LLM 犯错，Z3 兜底

```
LLM 生成的代码 → SMT 验证 → 通过？
                              │
                ┌──────────────┼──────────────┐
                ▼              ▼              ▼
              ✅ 安全         ❌ 反例        ⚠️ timeout
              直接用          反馈给 LLM      人工审查
                             重新生成
```

### 示例：闭环修正

```intent
// LLM 第 1 轮（有 bug：忘了检查余额）
intent TransferSafe(sender: Account, amount: Int) {
  require amount > 0
  ensure sender.balance' == sender.balance - amount
  invariant sender.balance' >= 0
}
```

```bash
$ intent check
  ❌ Counterexample: sender.balance = 3, amount = 10
     sender.balance' = -7, violates invariant
```

```intent
// LLM 第 2 轮（看到反例，自动修正）
+ require sender.balance >= amount
```

```bash
$ intent check → ✅ verified
```

**为什么其他语言做不到这个闭环？**

| 语言 | LLM 犯错后 |
|------|-----------|
| Python | 编译通过，运行时才出错，可能根本不被发现 |
| Lean 4 | 错误信息晦涩（proof state），LLM 很难自我修正 |
| TLA+ | 输出长 trace，LLM 难以解析 |
| **intent-lang** | **反例是 `variable=value`，LLM 一看就懂** |

---

## LLM-Friendly 设计原则

| 原则 | 说明 |
|------|------|
| 语法接近自然语言 | `require balance >= amount`，不是 `/\ balance >= amount` |
| 结构固定可预测 | LLM 只需学一个模板 |
| 不需要写 How | 避开 LLM 最容易犯错的算法实现 |
| 错误反馈可机器解读 | 反例是结构化的 `variable = value` |
| 验证结果二元 | ✅ 或 ❌，没有"大部分对"的灰色地带 |
| 安全网兜底 | 最坏是"验证失败"，不是"静默出错" |

---

## 待改进的点

| 问题 | 描述 | 改进 | 状态 |
|------|------|------|----|
| `x'` 语义 | LLM 可能不理解 primed 变量 | 新增 `after(x)` 别名 | ✅ 已更新至 SPEC |
| `::` 分隔符 | `forall x: T :: P(x)` 不常见 | 改为 `forall x: T, P(x)` | ✅ 已更新至 SPEC |
| 隐式 safety | LLM 看不到但会影响验证 | 报告中展示完整约束和合并的 safety 规则 | ✅ 已更新至 SPEC + PLAN |
| 训练数据 | 新语言，LLM 没见过 | M4 增加 few-shot 示例库构建任务（50+ 示例） | ✅ 已更新至 PLAN |

---

## 新工具链：LLM 工作流（RFC C4）

intent-lang 的 CLI 现在提供 5 条与 LLM 协作的稳定工作流，每条都以
**JSON artifact** 为接口（schema 见 [`docs/protocol/artifacts.md`](../protocol/artifacts.md)）。

### 1. 起草 → 验证（draft → check）

```
人类草稿 / LLM 草稿 → *.intent
  └─ intent check --format json → intent.consistency_report
       ├─ verified: 进入 review
       └─ violated: 把反例反馈给 LLM 让它重写
```

LLM 看到 Z3 反例（结构化 `variable = value`）后能精准修正子句，
比看人话错误信息收敛快得多。

### 2. 完备性扫描（coverage scan）

```
*.intent + coverage 块
  └─ intent coverage --format json → intent.coverage_report
       └─ uncovered: 提示 LLM "下面这些组合可能漏了，补一条 safety 或确认无关"
```

### 3. 测试草稿（testspec draft）

```
*.intent → intent testspec --format json → testspec.draft
  └─ 喂给 *另一个* LLM 转成具体测试代码（Rust/TS/Python）
```

> ⚠️ **反勾结原则（anti-collusion）**：生成 `*.intent` 的 LLM 与
> 把 `testspec.draft` 转成测试代码的 LLM **必须不是同一个会话/同一个上下文**。
> 否则它会偷偷修正一致的 bug —— 测试和 intent 双向作弊。
>
> 实操：在 popsicle skill 里用两个独立的 sub-agent 调用，
> 或在 CI 里跑两个不同 prompt 模板。

### 4. 变更影响（diff + impact）

```
old.intent + new.intent
  ├─ intent diff --format json   → intent.diff
  └─ intent impact --format json → intent.impact
       └─ affected_goals/stakeholders/coverages 自动写入 PR 描述
```

LLM 看到 `classification=tightened` 时能自动起草迁移说明；
看到 `reshaped` 时应当主动要求人工 review。

### 5. 解释（explain）

```
*.intent + 名字 → intent explain --format json → intent.explanation
  └─ 给非工程 stakeholder 看的自然语言版
```

`naturalized` 字段是 *模板* 输出（`==>` → "implies"），
**故意不用 LLM 翻译**：避免 LLM 在解释阶段二次解释、丢失原意。

---

## 反勾结原则总结

| 阶段 | 由谁产出 | 由谁验证 |
|------|---------|---------|
| `*.intent` 草稿 | LLM A 或人类 | Z3（机械） |
| 测试代码 | LLM B（必须 ≠ A） | CI runner |
| 自然语言解释 | 模板（不是 LLM） | 人类 stakeholder |
| 迁移文档 | 模板 + LLM C | 人类 reviewer |

**核心**：在任意环节插入 *机械验证* 或 *独立 LLM*，
避免一条因果链上有同一个智能体既写答案又判分。
