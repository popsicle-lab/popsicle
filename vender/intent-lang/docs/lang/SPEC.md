# intent-lang 语法规范

> 本文档是 intent-lang 的权威语法参考。

---

## 1. 程序结构

```ebnf
program     ::= declaration*
declaration ::= import_decl | type_decl | enum_decl | function_decl
              | intent_decl | safety_decl | theorem_decl | axiom_decl
```

---

## 2. 类型系统

### 2.1 内置类型

| 类型 | 说明 | 示例 |
|------|------|------|
| `Int` | 任意精度整数 | `42`, `-1` |
| `Bool` | 布尔值 | `true`, `false` |
| `String` | 字符串 | `"hello"` |
| `Seq<T>` | 有序序列 | `Seq<Int>` |
| `Set<T>` | 无序集合 | `Set<String>` |

### 2.2 结构体

```intent
type Account {
  balance: Int
  owner: String
  active: Bool
}
```

```ebnf
type_decl   ::= "type" IDENT type_params? "{" field_list "}"
type_params ::= "<" IDENT ("," IDENT)* ">"
field_list  ::= (IDENT ":" type_expr)*
```

### 2.3 枚举

```intent
enum Role { Admin, Editor, Viewer }
```

```ebnf
enum_decl ::= "enum" IDENT "{" IDENT ("," IDENT)* "}"
```

### 2.4 泛型

```intent
type Pair<A, B> {
  first: A
  second: B
}
```

---

## 3. 意图声明

核心构造。声明一个操作应满足的条件，不写实现。

```intent
intent TransferSafe(sender: Account, receiver: Account, amount: Int) {
  require amount > 0                                    // 前置条件
  require sender.balance >= amount
  ensure sender.balance' == sender.balance - amount      // 后置条件
  ensure receiver.balance' == receiver.balance + amount
  invariant sender.balance' >= 0                         // 不变量
}
```

```ebnf
intent_decl ::= annotation* "intent" IDENT "(" param_list ")" "{" clause* "}"
clause      ::= "require" expr | "ensure" expr | "invariant" expr
```

### 语义

| 子句 | 含义 | 何时必须为真 |
|------|------|------------|
| `require P` | 前置条件 | 调用前 |
| `ensure Q` | 后置条件 | 执行后 |
| `invariant I` | 不变量 | 执行前后都必须 |

**验证条件**：`(∧ require_i) ∧ (∧ invariant_j) → (∧ ensure_k) ∧ (∧ invariant_j')`

### Primed 变量

`x'` 表示变量执行后的新值。同时支持 `after(x)` 作为等价别名，方便 LLM 生成和阅读。

```intent
// 以下两种写法等价：
ensure sender.balance' == sender.balance - amount
ensure after(sender.balance) == sender.balance - amount
```

规则：

- 只能出现在 `ensure` 和 `invariant` 中
- 类型与 `x` 相同
- 支持嵌套字段：`account.balance'` 或 `after(account.balance)`
- `after(...)` 在 AST 层面脱糖为 primed 形式

---

## 4. 安全规则

全局不变量，自动附加到同作用域所有 intent 的验证条件中。

```intent
safety HomeSafety(home: Home) {
  invariant !home.occupied ==> home.frontDoor.locked
  invariant home.frontDoor.open ==> !home.frontDoor.locked
}
```

```ebnf
safety_decl ::= "safety" IDENT "(" param_list ")" "{" invariant_clause* "}"
```

---

## 5. 定理

需要被 SMT solver 证明的性质。

```intent
theorem TransferPreservesTotal {
  forall s: Account, r: Account, a: Int,
    TransferSafe(s, r, a) ==>
      s.balance' + r.balance' == s.balance + r.balance
}
```

```ebnf
theorem_decl ::= "theorem" IDENT "{" expr "}"
```

---

## 6. 公理

向 SMT solver 注入领域知识。被无条件假设为真。

```intent
axiom temp_monotonic {
  forall t: Thermostat,
    t.mode == Heat && t.target > t.temperature ==>
      t.temperature' > t.temperature
}
```

```ebnf
axiom_decl ::= "axiom" IDENT "{" expr "}"
```

> ⚠️ 错误的公理会导致验证不可靠 (unsound)。公理必须经过领域专家审核。

---

## 6.5 业务目标 (Goal — RFC A1)

`goal` 块声明业务层意图——为什么这套规则存在。
与 `safety`/`intent` 同级，是一等语法节点（不是注释）。

```intent
goal "用户余额永远满足透支底线" {
  rationale: "防止账户被透支放任，符合监管要求"
  stakeholder: ["finance", "compliance"]
  measure: "审计日志中无 balance < -overdraftLimit 的快照"
  realized_by: [BalanceSafety, Transfer, Refund]
}
```

```ebnf
goal_decl  ::= "goal" STRING "{" goal_field+ "}"
goal_field ::= "rationale"   ":" STRING
             | "stakeholder" ":" "[" STRING ("," STRING)* "]"
             | "measure"     ":" STRING
             | "realized_by" ":" "[" IDENT  ("," IDENT)*  "]"
```

**类型检查**：`realized_by` 中的每个标识符必须是已声明的
`safety` / `intent` / `theorem` 名字；否则发出 `W0010`（warning，
不阻断 CI，但会出现在 `intent.consistency_report` 中）。

**用途**：`intent impact` 通过 `realized_by` 反查"修改某条 intent
影响哪些目标 / 哪些 stakeholder"，见 `docs/protocol/artifacts.md`。

---

## 6.6 完备性维度 (Coverage — RFC A3)

`coverage` 块声明应该被规则覆盖的领域维度笛卡尔积。

```intent
coverage "permission-matrix" {
  dimensions: {
    role: [Admin, Editor, Viewer, Guest]
    sensitivity: [Public, Internal, Confidential, Secret]
    user_state: [authenticated, banned]
  }
}
```

```ebnf
coverage_decl ::= "coverage" STRING "{" "dimensions" ":" "{" dim+ "}" "}"
dim           ::= IDENT ":" "[" expr ("," expr)* "]"
```

**类型检查**：维度值是 *opaque labels*（域内符号），
**故意不做** 类型检查。它们只用于 `intent coverage` 工具的语法级
见证扫描——在每条 safety/intent 子句的文本中查找字面值出现。

**为什么这样设计**：完备性是 *沟通工具*，不是证明工具——
让 reviewer 看到"哪些组合根本没人提及"已经有足够价值。
完整理由见 `docs/lang/DECISIONS.md` 决策 8。

---

## 6.7 时态标注 (As-is / To-be — RFC A2)

复用现有 `@annotation` 系统区分意图的生命周期：

```intent
@asis
intent LegacyV1(...) { ... }   // 老代码现在的实际行为

@tobe
intent NewV2(...) { ... }      // 新承诺的目标行为
```

| 标注 | 含义 | `intent check` 默认 |
|------|------|---------------------|
| `@asis` | 老代码的实然 | **跳过**（除非 `--include-asis`） |
| `@tobe` | 新承诺的应然 | 验证 |
| 无标注  | 当前规则 | 验证 |

迁移场景的典型用法：把老逻辑翻译成 `@asis` 一次性入库，
再为新承诺写 `@tobe`，让 review 同时看到"现状"与"目标"。

---

## 7. 纯函数

辅助定义，无副作用。

```intent
function max(a: Int, b: Int) -> Int {
  if a >= b then a else b
}
```

```ebnf
function_decl ::= "function" IDENT "(" param_list ")" "->" type_expr "{" expr "}"
```

---

## 8. 模块系统

### 8.1 导入

intent-lang 有两种导入：**插件导入** 和 **文件导入**。

```intent
// 插件导入：加载类型 + safety + axiom + 函数
import smarthome
import finance.currency

// 文件导入：仅引入符号（类型、函数、intent 等），不注入 safety/axiom
import "./types/order.intent"
import "./common/helpers.intent"
```

```ebnf
import_decl  ::= "import" import_path
import_path  ::= module_path | STRING_LITERAL
module_path  ::= IDENT ("." IDENT)*
```

**两者的区别：**

| | 插件导入 (`import smarthome`) | 文件导入 (`import "./foo.intent"`) |
|---|---|---|
| 来源 | 已安装的插件包 | 项目内的 `.intent` 文件 |
| 类型/函数 | ✅ 引入 | ✅ 引入 |
| safety 规则 | ✅ 自动合并到验证条件 | ❌ 不合并（需在文件中显式声明） |
| axiom | ✅ 注入 SMT | ❌ 不注入 |
| 场景 | 领域基础设施（物理约束、行业规则） | 项目内代码组织 |

### 8.2 限定名

当多个导入引入同名类型时，使用限定名消歧：

```intent
import finance      // 定义了 Account
import user         // 也定义了 Account

// 用模块前缀消歧
intent Checkout(wallet: finance.Account, profile: user.Account) {
  require wallet.balance >= price
  require profile.authenticated
}
```

文件导入的限定名默认为文件名（不含扩展名），可用 `as` 自定义别名：

```intent
// 默认：前缀为文件名
import "./domains/order.intent"          // 前缀 order
import "./domains/payment.intent"        // 前缀 payment

// 两个文件名相同时，用 as 消歧
import "./domains/payment/types.intent" as payment
import "./domains/user/types.intent" as user

intent ProcessOrder(o: order.Order, wallet: payment.Account, profile: user.Account) {
  require p.amount == o.total
  require profile.authenticated
}
```

```ebnf
import_decl    ::= "import" import_path ("as" IDENT)?
import_path    ::= module_path | STRING_LITERAL
module_path    ::= IDENT ("." IDENT)*
type_expr      ::= qualified_type | IDENT | IDENT "<" type_expr ("," type_expr)* ">"
qualified_type ::= module_name "." IDENT
module_name    ::= IDENT
```

**规则：**

- 插件导入的限定名前缀为最后一段路径：`import finance.currency` → 前缀 `currency`
- 文件导入的限定名默认为文件名（不含 `.intent`），可用 `as` 覆盖
- 无冲突时，可直接使用类型名：`Account`
- 存在同名冲突时，必须使用限定名，否则报编译错误
- 限定名始终可用，即使无冲突

### 8.3 注解

```intent
@source("PRD-2024-Q1", section: "3.2.1")
@priority(P0)
intent PaymentSafe(...) { ... }
```

```ebnf
annotation  ::= "@" IDENT "(" annotation_args? ")"
```

---

## 9. 表达式

```ebnf
expr ::= literal | IDENT | IDENT "'"
       | "after" "(" expr ")"
       | expr "." IDENT | expr "." IDENT "'"
       | "!" expr | "-" expr
       | expr binop expr
       | "if" expr "then" expr "else" expr
       | "forall" typed_vars "," expr
       | "exists" typed_vars "," expr
       | IDENT "(" expr_list ")"
       | expr "[" expr "]"
       | "(" expr ")"

binop ::= "==" | "!=" | "<" | "<=" | ">" | ">="
        | "+" | "-" | "*" | "/" | "%"
        | "&&" | "||" | "==>"
```

### 优先级（低 → 高）

| 级别 | 运算符 | 结合性 |
|------|--------|--------|
| 1 | `==>` | 右 |
| 2 | `\|\|` | 左 |
| 3 | `&&` | 左 |
| 4 | `==`, `!=` | 无 |
| 5 | `<`, `<=`, `>`, `>=` | 无 |
| 6 | `+`, `-` | 左 |
| 7 | `*`, `/`, `%` | 左 |
| 8 | `!`, `-` (一元) | 前缀 |
| 9 | `.`, `'`, `[]` | 后缀 |

---

## 10. SMT 编码策略

### 类型映射

| intent-lang | SMT-LIB2 |
|-------------|----------|
| `Int` | `Int` |
| `Bool` | `Bool` |
| `String` | `String` |
| struct | `(declare-datatype ...)` |
| enum | `(declare-datatype ...)` |
| `Seq<T>` | `(Array Int T)` |

### Intent 验证编码（反证法）

```smt2
(assert R1)              ; require
(assert R2)
(assert V1)              ; invariant (旧状态)
(assert (not (and E1 E2 V1_prime)))  ; 否定 ensure + invariant'
(check-sat)
; unsat → 验证通过 | sat → 反例
```

### Theorem 验证编码

```smt2
(assert (not <theorem_body>))
(check-sat)
```

---

## 11. 错误报告

采用 rustc 风格的诊断信息：

```
error[E0001]: type mismatch
  --> transfer.intent:12:11
   |
12 |   require amount > "zero"
   |                     ^^^^^^ expected Int, found String

error[V0001]: verification failed
  --> transfer.intent:49:3
   |
49 |   ensure sender.balance' == sender.balance - amount - 1
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot be verified
   |
   = counterexample:
       sender.balance = 100, amount = 10
     expected: 149 == 160 (off-by-one)
```

### 隐式安全规则展示

当 intent 验证时，作用域内的 `safety` 规则被隐式合并到验证条件中。
为避免黑盒效果（LLM 和用户看不到实际验证了什么），报告中应展示完整约束：

```
info[V0010]: verification context
  --> smarthome.intent:25:1
   |
25 | intent SetTemperature(...) { ... }
   |
   = applied safety rules:
     - HomeSafety.invariant[1]: !home.occupied ==> home.frontDoor.locked
     - HomeSafety.invariant[2]: home.frontDoor.open ==> !home.frontDoor.locked
   = effective verification condition:
     (require_1 ∧ require_2 ∧ safety_1 ∧ safety_2) → (ensure_1 ∧ safety_1' ∧ safety_2')
```

这使得 LLM 可以理解失败原因，也让用户能审计验证过程。
