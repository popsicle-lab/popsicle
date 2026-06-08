# Requirements-Style Examples

> **需求建模风格的示例集合**——和 `examples/basics/` 不同，这里的示例**不描述如何转账、如何登录**，而描述**业务到底要保证什么**。

---

## 风格差异

`examples/basics/` 风格（混合 / 程序规格风格）：

```intent
intent Login(user: User, passwordCorrect: Bool) {
  ensure passwordCorrect ==> user.authenticated' == true
  ensure !passwordCorrect ==> user.loginAttempts' == user.loginAttempts + 1
  // ... 描述了"登录这个动作"如何改变状态
}
```

`examples/requirements/` 风格（需求建模风格）：

```intent
// 先声明业务安全规则（最高优先级，永不违反）
safety AuthenticatedUserCannotBeLocked {
  forall u: User, u.authenticated ==> !u.locked
}

safety LockoutAfterFiveFailures {
  forall u: User, u.loginAttempts >= 5 ==> u.locked
}

// 业务定理：从规则集合可以推导的高阶性质
theorem LockedUserStaysLockedUntilReset {
  forall u: User, u.locked ==> u.loginAttempts >= 5
}
```

**关键区别：**
- 前者描述"动作的前后断言"——更接近程序规格
- 后者描述"业务世界的恒久规则"——更接近需求建模

需求建模风格不强调"操作"，强调**世界的状态约束**。这才是 intent-lang 真正想做的事。

---

## 何时用哪种风格？

| 场景 | 推荐风格 |
|---|---|
| PRD/RFC 评审，要钉死业务规则 | requirements |
| API 契约描述（前后置条件） | basics |
| 验证多个业务规则之间无矛盾 | requirements |
| 描述某个具体函数的输入输出 | basics |
| 给 PM/业务方看的"我们承诺什么" | requirements |
| 给开发者看的"实现要满足什么" | basics |

**实际项目中两者并用**：先用 requirements 风格钉死最高层不变量，再用 basics 风格描述具体 intent，最后用 theorem 验证两者一致。

---

## 示例清单

| 文件 | 演示什么 |
|---|---|
| [billing.intent](billing.intent) | 计费/账户领域的纯需求建模：goal + safety + theorem + coverage |
| [access-control.intent](access-control.intent) | 权限模型 + @asis/@tobe 双轨意图 + 完整 goal/coverage 演示 |

---

## 新工具链快速上手 (RFC 实现版)

这两个示例同时展示了 RFC 落地的全部新语法（goal / @asis / @tobe / coverage）
与全部新工具命令。在仓库根目录构建后试：

```bash
cargo build --release
export PATH=$PWD/target/release:$PATH

intent check     examples/requirements/billing.intent
intent check     examples/requirements/access-control.intent --include-asis
intent coverage  examples/requirements/access-control.intent
intent testspec  examples/requirements/billing.intent
intent explain   examples/requirements/billing.intent Transfer

# 机读模式（schema 见 docs/protocol/artifacts.md）：
intent --format json check examples/requirements/billing.intent | jq

# diff / impact 需要两个版本：
cp examples/requirements/billing.intent /tmp/v1.intent
sed 's/amount > 0/amount > 10/' /tmp/v1.intent > /tmp/v2.intent
intent diff   /tmp/v1.intent /tmp/v2.intent
intent impact /tmp/v1.intent /tmp/v2.intent
```

每条命令都接受全局 `--format json|text`；JSON schema 由
[`docs/protocol/artifacts.md`](../../docs/protocol/artifacts.md) 锁定。

