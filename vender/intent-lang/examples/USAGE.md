# intent-lang 使用指南

## 完整用例演示

### 用例 1：编写意图 → 自动验证

用户编写 `transfer.intent` 文件（见 `examples/basics/transfer.intent`），然后运行：

```bash
$ intent check examples/basics/transfer.intent

  Checking transfer.intent...

  ✅ type Account                          — well-formed
  ✅ intent TransferSafe                   — verified
  ✅ theorem TransferPreservesTotal        — proved
  ❌ intent TransferBuggy                  — FAILED

     Counterexample found:
       sender.balance = 100
       receiver.balance = 50
       amount = 10

       Expected: sender.balance' + receiver.balance' == sender.balance + receiver.balance
       Got:      (100 - 10 - 1) + (50 + 10) = 149 ≠ 160

     --> examples/basics/transfer.intent:49:3
      |
   49 |   ensure sender.balance' == sender.balance - amount - 1
      |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ off-by-one error
      |

  Results: 3 passed, 1 failed
```

**解读**：
- `TransferSafe` 的所有条件逻辑自洽 → ✅
- 定理 `TransferPreservesTotal` 可由 `TransferSafe` 推导出 → ✅
- `TransferBuggy` 多扣了 1，Z3 找到反例 → ❌

---

### 用例 2：自然语言 → 意图代码（LLM 辅助）

```bash
$ intent generate "用户登录时，密码错误超过5次应该锁定账户"

  🤖 Generating intent from natural language...

  Generated:

  ┌─────────────────────────────────────────────────┐
  │ intent LoginWithLockout(user: User, pwOk: Bool) │
  │ {                                               │
  │   require !user.locked                          │
  │                                                 │
  │   ensure !pwOk ==>                              │
  │     user.loginAttempts' == user.loginAttempts + 1│
  │                                                 │
  │   ensure pwOk ==>                               │
  │     user.loginAttempts' == 0                    │
  │                                                 │
  │   ensure user.loginAttempts' >= 5 ==>           │
  │     user.locked' == true                        │
  │ }                                               │
  └─────────────────────────────────────────────────┘

  🔍 Auto-verifying... ✅ Verified

  Save to file? [y/N] y
  Saved to login_lockout.intent
```

---

### 用例 3：交互式探索

```bash
$ intent check examples/basics/auth.intent --explain

  ✅ intent Login — verified

     What was proven:
     1. If user is not locked and password is correct
        → user becomes authenticated, attempts reset to 0
     2. If user is not locked and password is wrong
        → attempts increment by 1
     3. If attempts reach 5 → account locks
     4. All branches are mutually consistent (no contradictions)

  ✅ intent AccessResource — verified

     What was proven:
     1. Authenticated users can access public resources
     2. Owners can access their own private resources
     3. Admins can access all resources
     4. Unauthenticated users are always rejected

  ✅ theorem LockedUserCannotLogin — proved

     Proof sketch:
     Login requires !user.locked (line 22)
     Therefore: user.locked ==> Login precondition fails
     QED
```

---

### 用例 4：格式化

```bash
$ intent fmt examples/basics/transfer.intent
  Formatted 1 file
```

---

## 核心概念速查

```
┌─────────────────────────────────────────────────────────┐
│                    intent-lang 核心概念                    │
├──────────┬──────────────────────────────────────────────┤
│ type     │ 定义数据结构                                   │
│ enum     │ 定义枚举类型                                   │
│ intent   │ 声明一个意图（What, not How）                   │
│ require  │ 前置条件：调用前必须为真                         │
│ ensure   │ 后置条件：执行后必须为真                         │
│ invariant│ 不变量：前后都必须为真                           │
│ x'       │ 变量 x 的"新值"（执行后的状态）                  │
│ theorem  │ 需要被证明的更高层性质                           │
│ forall   │ "对所有 x 都成立..."                           │
│ exists   │ "存在某个 x 使得..."                           │
│ ==>      │ 逻辑蕴含："如果...那么..."                      │
│ function │ 纯辅助函数（无副作用）                           │
└──────────┴──────────────────────────────────────────────┘
```

---

## 与其他工具的对比

| 特性 | intent-lang | Dafny | TLA+ | Alloy |
|------|-------------|-------|------|-------|
| 意图声明语法 | ✅ 原生 | 部分 | ❌ | ❌ |
| 自动 SMT 验证 | ✅ | ✅ | ❌ (模型检查) | ❌ (SAT) |
| LLM 辅助生成 | ✅ | ❌ | ❌ | ❌ |
| 学习曲线 | 低 | 中 | 高 | 中 |
| Primed 变量 | ✅ | ❌ | ✅ | ❌ |
| Web Playground | 计划中 | ✅ | ✅ | ❌ |
