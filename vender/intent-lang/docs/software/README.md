# 软件开发场景

> intent-lang 如何将 PRD 到代码之间的断裂链变成形式化验证链。

---

## 痛点：从 PRD 到代码的断裂

```
PRD              设计文档           代码              测试             生产
"余额不足        (可能过时)        if bal >= amt     3个测试用例      用户投诉：
 不能转账"                         { ... }                          "余额为0
                                                                    竟然能转账"
```

问题根源：**每个环节之间没有形式化链接**。PRD 写了"余额不足不能转账"，但：
1. 什么叫"不足"？等于 0 算不算？（PRD 歧义）
2. 代码是否完全实现了 PRD 的意思？（无法验证）
3. 测试覆盖了所有边界吗？（几乎不可能）

---

## 解决方案：4 阶段验证链

### Phase 1：PRD → L1 业务意图

```
PRD: "转账金额必须大于零，余额不足时拒绝，保证资金安全"
                    │
                    │ LLM 翻译 + 人工审查
                    ▼
intent TransferSafe(sender: Account, receiver: Account, amount: Int) {
  require amount > 0                    ← "金额大于零"
  require sender.balance >= amount      ← "余额不足拒绝"
  ensure sender.balance' == sender.balance - amount
  ensure receiver.balance' == receiver.balance + amount
  invariant sender.balance' >= 0        ← "资金安全"
}
```

**关键价值：形式化过程暴露 PRD 的遗漏。**

```bash
$ intent check --audit-prd

  ⚠️ PRD 未覆盖的边界情况:
    1. sender == receiver 时是否允许？
    2. amount 是否有上限？
    3. receiver 账户冻结时怎么办？
    4. 并发转账的 TOCTOU 风险？

  → 反馈给产品经理补充 PRD
```

### Phase 2：L1 → L2 系统意图（API 契约）

业务意图不关心 HTTP/认证，系统意图要关心：

```intent
intent POST_Transfer(req: TransferRequest) -> TransferResponse {
  require req.auth.valid                    // 认证
  require req.idempotency_key.unique        // 幂等

  ensure response.status == 200 ==>
    TransferSafe(account(req.sender_id),    // 成功 → 满足业务意图
                 account(req.receiver_id), req.amount)

  ensure response.status == 400 ==>
    unchanged(all_accounts)                 // 失败 → 无副作用
}

refines TransferSafe by POST_Transfer when response.status == 200
```

SMT 自动验证：API 层是否正确满足业务层。

### Phase 3：L2 → L3 组件意图

```intent
intent DeductBalance(account_id: String, amount: Int) {
  require amount > 0
  require db.get(account_id).balance >= amount
  ensure db.get(account_id).balance' == db.get(account_id).balance - amount
  ensure db.get(account_id).version' == db.get(account_id).version + 1  // 乐观锁
  ensure failed ==> unchanged(db)                                        // 原子性
}
```

### Phase 4：从意图生成工程产物

#### 测试用例

```bash
$ intent test-gen TransferSafe --format pytest
```

```python
def test_transfer_normal():           # require 满足
    assert transfer(100, 50).balance == 50

def test_transfer_exact_balance():    # 边界: balance == amount
    assert transfer(50, 50).balance == 0

def test_transfer_insufficient():     # require 违反
    with pytest.raises(InsufficientBalance):
        transfer(30, 50)

def test_transfer_zero():             # require 违反
    with pytest.raises(InvalidAmount):
        transfer(100, 0)
```

#### 运行时断言

```bash
$ intent export TransferSafe --format rust-assert
```

```rust
fn transfer(sender: &mut Account, receiver: &mut Account, amount: i64) -> Result<()> {
    // --- require (auto-generated) ---
    assert!(amount > 0);
    assert!(sender.balance >= amount);
    let old_s = sender.balance;
    let old_r = receiver.balance;

    // ... your implementation ...

    // --- ensure (auto-generated) ---
    assert_eq!(sender.balance, old_s - amount);
    assert_eq!(receiver.balance, old_r + amount);
    assert!(sender.balance >= 0);  // invariant
    Ok(())
}
```

#### API 契约

```bash
$ intent export POST_Transfer --format openapi
```

```yaml
paths:
  /transfer:
    post:
      x-intent: POST_Transfer
      requestBody:
        content:
          application/json:
            schema:
              properties:
                amount: { type: integer, minimum: 1 }
      responses:
        200: { description: "Transfer successful" }
        400: { description: "Bad request (no state mutation)" }
        401: { description: "Unauthorized" }
```

#### CI/CD 集成

```yaml
# .github/workflows/intent-check.yml
on: [push, pull_request]
jobs:
  verify:
    steps:
      - run: intent check specs/                # 验证意图
      - run: intent check --refinement specs/    # 验证精化
      - run: intent coverage --tests tests/      # 测试覆盖率
      - run: intent audit --prd docs/prd.md      # PRD 覆盖率
```

---

## 完整链路图

```
PRD (自然语言)
  │ ① LLM 翻译 → 暴露遗漏 → 反馈产品经理
  ▼
L1 业务意图 ──── SMT ✅
  │ ② refines
  ▼
L2 系统意图 ──── SMT ✅
  │ ③ refines
  ▼
L3 组件意图 ──── SMT ✅
  │ ④ 生成
  ├→ 测试用例    ├→ 运行时断言    ├→ API 契约    └→ CI 验证
```

---

## 有无 intent-lang 的对比

| 阶段 | 传统方式 | intent-lang |
|------|---------|-------------|
| PRD 评审 | 人肉发现遗漏 | **SMT 暴露边界情况** |
| 设计评审 | review 文档 | **验证 L2 refines L1** |
| 编码 | 凭理解写 | **生成骨架 + 断言** |
| 测试 | 手写用例 | **自动生成边界测试** |
| Code Review | 人肉检查 | **CI 自动验证** |
| 上线后 | 用户投诉 | **运行时断言告警** |
| 需求变更 | 全链路人肉 | **显示哪些意图被破坏** |
