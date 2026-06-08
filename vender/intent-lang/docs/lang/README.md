# intent-lang 语言概览

> 5 分钟了解 intent-lang —— 一门声明"意图"而非"实现"的形式化语言。

---

## 一句话定义

**intent-lang** 是一门通用的意图声明语言：你只说"要什么"，系统自动证明"对不对"。

```intent
intent TransferSafe(sender: Account, receiver: Account, amount: Int) {
  require amount > 0                                   // 前置条件
  require sender.balance >= amount
  ensure sender.balance' == sender.balance - amount     // 后置条件
  ensure receiver.balance' == receiver.balance + amount
  invariant sender.balance' >= 0                        // 不变量
}
```

```
$ intent check transfer.intent
  ✅ intent TransferSafe — verified
```

不需要写实现，不需要写证明，不需要写测试 —— SMT solver (Z3) 自动验证。

---

## 核心概念（7 个关键字）

| 关键字 | 含义 | 示例 |
|--------|------|------|
| `type` | 定义数据结构 | `type Account { balance: Int }` |
| `intent` | 声明一个意图 | `intent TransferSafe(...) { ... }` |
| `require` | 前置条件（调用前必须满足） | `require amount > 0` |
| `ensure` | 后置条件（执行后必须满足） | `ensure balance' == balance - amount` |
| `invariant` | 不变量（前后都必须满足） | `invariant balance' >= 0` |
| `theorem` | 需要被证明的更高层性质 | `theorem TotalPreserved { ... }` |
| `safety` | 全局约束（所有 intent 必须满足） | `safety NeverOverdraft { ... }` |

### Primed 变量 `x'` / `after(x)`

`x'`（或等价的 `after(x)`）表示变量执行后的新值：

```intent
// 以下两种写法等价：
ensure sender.balance' == sender.balance - amount
ensure after(sender.balance) == sender.balance - amount
//     ^^^^^^^^^^^^^^^^         ^^^^^^^^^^^^^^
//     新值（执行后）               旧值（执行前）
```

> `after(x)` 别名对 LLM 更友好——LLM 可能不理解 prime 记号的语义。

---

## 为什么不用现有工具？

| | intent-lang | Dafny | TLA+ | Lean 4 |
|---|---|---|---|---|
| **你写什么** | 只写条件 | 实现 + 条件 | 状态机 | 实现 + 证明 |
| **需要写证明？** | ❌ 自动 | ❌ 自动 | ❌ 穷举 | ✅ 手写 |
| **需要写实现？** | ❌ | ✅ 必须 | ⚠️ 状态转换 | ✅ 必须 |
| **LLM 可生成？** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐ |
| **学习曲线** | 低 | 中 | 高 | 高 |

> 详细对比见 [examples/comparison/](../../examples/comparison/COMPARISON.md)

---

## 三个特色能力

### 1. LLM 辅助生成

自然语言 → intent 代码 → SMT 自动验证。LLM 犯的错，Z3 兜底。

```bash
$ intent generate "用户登录失败5次后锁定账户"
  🤖 Generated intent... 🔍 Auto-verifying... ✅ Verified
```

> 详见 [LLM 友好性分析](LLM.md)

### 2. 领域插件

核心语言不变，通过插件适配不同领域：

```intent
import smarthome    // 智能家居插件：Device, Light, Sensor...
import finance      // 金融插件：Account, Ledger, Currency...
```

> 详见 [插件系统](../architecture/PLUGINS.md)

### 3. 多层意图精化

从业务需求到代码实现，每层都有验证：

```
L1 业务意图 → L2 系统意图 → L3 组件意图
     │ refines    │ refines    │
     └── SMT ✅ ──┘── SMT ✅ ──┘
```

> 软件开发场景详见 [docs/software/](../software/)
> 智能家居场景详见 [docs/smarthome/](../smarthome/)

---

## 技术栈

| 用途 | 技术 |
|------|------|
| 实现语言 | Rust |
| 解析器 | `logos` (lexer) + 手写递归下降 |
| 验证引擎 | Z3 SMT solver (SMT-LIB2) |
| LLM 集成 | OpenAI / Anthropic API |
| 未来 | WASM 编译 → Web Playground |

---

## 下一步阅读

| 想了解... | 阅读 |
|-----------|------|
| 完整语法规范 | [SPEC.md](SPEC.md) |
| 为什么这样设计 | [DECISIONS.md](DECISIONS.md) |
| 与大模型的关系 | [LLM.md](LLM.md) |
| 软件开发怎么用 | [docs/software/](../software/) |
| 智能家居怎么用 | [docs/smarthome/](../smarthome/) |
| 插件怎么写 | [docs/architecture/PLUGINS.md](../architecture/PLUGINS.md) |
| 意图怎么执行 | [docs/architecture/EXECUTION.md](../architecture/EXECUTION.md) |
