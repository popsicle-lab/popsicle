# intent-lang vs Lean vs TLA+ 对比分析

## 同一个例子：银行转账

### intent-lang（18 行核心代码）

```intent
intent TransferSafe(sender: Account, receiver: Account, amount: Int) {
  require amount > 0
  require sender.balance >= amount
  ensure sender.balance' == sender.balance - amount
  ensure receiver.balance' == receiver.balance + amount
  invariant sender.balance' >= 0
}

theorem TransferPreservesTotal {
  forall s: Account, r: Account, a: Int,
    TransferSafe(s, r, a) ==>
      s.balance' + r.balance' == s.balance + r.balance
}
```

### Lean 4（~40 行）

```lean
structure Account where
  balance : Int
  owner : String
  active : Bool

-- 必须写出实现
def transfer (sender receiver : Account) (amount : Int) : Account × Account :=
  ( { sender with balance := sender.balance - amount },
    { receiver with balance := receiver.balance + amount } )

-- 必须分离定义 pre/post
def transferPre (...) : Prop := amount > 0 ∧ sender.balance ≥ amount ∧ ...
def transferPost (...) : Prop := sender'.balance = sender.balance - amount ∧ ...

-- 必须手写证明策略
theorem transfer_correct (...) (h : transferPre ...) :
    ... transferPost ... := by
  unfold transferPre at h; unfold transfer transferPost; simp; omega

theorem transfer_preserves_total (...) :
    s'.balance + r'.balance = ... := by
  unfold transfer; simp; omega
```

### TLA+（~35 行）

```tla
VARIABLES balances, locked

Transfer(sender, receiver, amount) ==
    /\ sender /= receiver
    /\ amount > 0
    /\ balances[sender] >= amount
    /\ balances' = [balances EXCEPT
        ![sender] = balances[sender] - amount,
        ![receiver] = balances[receiver] + amount]
    /\ UNCHANGED locked

BalanceNonNegative == \A a \in Accounts : balances[a] >= 0

-- 总额守恒在 TLA+ 中需要手动实现求和（较繁琐）
```

---

## 核心差异对比

| 维度 | intent-lang | Lean 4 | TLA+ |
|------|-------------|--------|------|
| **核心范式** | 意图声明 | 定理证明 | 状态机建模 |
| **用户写什么** | 只写条件 | 实现 + 条件 + 证明 | 状态转换 + 性质 |
| **验证方式** | SMT 自动验证 | 交互式证明 (tactic) | 有界模型检查 |
| **需要写实现吗** | ❌ 不需要 | ✅ 必须 | ⚠️ 需要写状态转换 |
| **需要写证明吗** | ❌ 自动 | ✅ 手写 tactic | ❌ 自动穷举 |
| **保证强度** | 无界（SMT） | 最强（完全证明） | 有界（状态空间内） |
| **primed 变量** | ✅ `x'` | ❌ 需要手动传参 | ✅ `x'` |
| **反例** | ✅ 自动生成 | ❌ 证明失败无反例 | ✅ 自动生成 trace |
| **学习曲线** | 低 | 高（需学类型论） | 中（需学时序逻辑） |
| **适合谁** | 想快速验证意图 | 需要最高保证 | 并发/分布式系统 |

---

## 各自的优势场景

### intent-lang 最适合
- "我想快速验证这个业务逻辑对不对"
- 不想学证明策略，不想写实现
- 配合 LLM 从自然语言直接生成可验证的 spec

### Lean 最适合
- 数学定理证明（纯数学）
- 需要最高保证的安全关键系统
- 愿意投入时间写详细证明

### TLA+ 最适合
- 并发协议（Paxos, Raft）
- 分布式系统设计验证
- 需要查看执行 trace 理解系统行为

---

## intent-lang 的设计借鉴

| 借鉴来源 | 借鉴了什么 | 改进了什么 |
|----------|-----------|-----------|
| **TLA+** | primed 变量 `x'` 表示新状态 | 不需要写完整状态转换，只声明条件 |
| **Dafny** | require/ensure/invariant 关键字 | 去掉了实现代码，纯声明式 |
| **Lean** | 定理声明 | 证明自动化（SMT），无需手写 tactic |
| **Alloy** | 声明式风格 | 无界验证（SMT vs SAT 有界） |
