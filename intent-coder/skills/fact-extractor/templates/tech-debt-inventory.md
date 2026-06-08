# Tech-Debt Inventory — {project_name}

> `{slug}.fact-extraction-report.md` 的配套文件。记录代码库中的**显式债务标记**：TODO、FIXME、HACK、XXX、NOTE 注释 + deprecated API 使用 + 死代码候选。

> **范围规则**：只记代码库**自己标**为债务的条目。**推断**出来的债务（如「这段看起来很复杂」）**不**记——那是发表观点。

---

## 标记计数

| 标记 | 数量 | 来源 |
|---|---|---|
| `TODO` | {n} | `rg -i 'TODO:'` |
| `FIXME` | {n} | `rg -i 'FIXME:'` |
| `HACK` | {n} | `rg -i 'HACK:'` |
| `XXX` | {n} | `rg -i 'XXX:'` |
| `NOTE` | {n} | `rg -i 'NOTE:'` |

---

## TODO / FIXME（含年龄）

> 按年龄降序。年龄 = 注释首次引入以来的月数（`git log --diff-filter=A -S "TODO: …"`）。> 12 个月 = 应升级为 GitHub issue + ADR 的候选。

| File:Line | 标记 | 注释 | 作者 | 首次出现 | 年龄 |
|---|---|---|---|---|---|
| `src/payment/process.rs:108` | TODO | `validate amount > 0 before going to prod` | {alice} | {2023-04-12} | {24mo} |
| `src/auth/login.rs:55` | FIXME | `token TTL hard-coded — make configurable` | {bob} | {2024-01-08} | {16mo} |
| `src/cache/tiered.rs:88` | HACK | `holding mutex across await — fix when tokio fixes #5454` | {carol} | {2025-02-20} | {3mo} |

> 如果 git blame 不可用，省略 作者 / 首次出现 / 年龄 列，并把本节标 `[reduced fidelity —— git history unavailable]`。

---

## Deprecated API 使用

> 内部代码调用了被标 `#[deprecated]`（Rust）、`@deprecated`（TS/Java）等的 API。

| 调用方（file:line）| Deprecated API | 建议替代 | 自版本 |
|---|---|---|---|
| `src/auth/legacy.rs:42` | `crypto::md5_hash` | `crypto::sha256_hash` | 0.3.0 |

---

## 死代码候选

> 被编译器 / linter 标记为未使用的条目。**只**记来自 build 的信号（`cargo build` 警告、`rustc -W dead_code`、ESLint `unused-vars` 等）。**不**记臆测的死代码。

| 条目 | File:Line | 工具 | 置信度 |
|---|---|---|---|
| `pub fn old_login` | `src/auth/legacy.rs:8` | rustc dead_code | medium（仍是 pub —— 可能有外部使用者）|

---

## 禁用的测试

> 被标 `#[ignore]`（Rust）、`it.skip` / `xit`（TS）、`@unittest.skip`（Python）、`t.Skip`（Go）的测试。

| 测试 | File:Line | 跳过原因（来自注释）| 自何时跳过 |
|---|---|---|---|
| `test_payment_idempotency` | `tests/payment.rs:142` | "flaky — CI 重试 5 次，见 #421" | {2024-09} |

---

## 配置异味

> 大概不该硬编码的魔法数 / URL / 路径。基于简单 grep 启发式检测；**附置信度**。

| File:Line | 构造 | 置信度 |
|---|---|---|
| `src/payment/gateway.rs:8` | `https://api.stripe.com/v1`（硬编码 URL）| high —— 应进配置 |
| `src/cache/tiered.rs:14` | `LIMIT = 1024`（无注释）| low —— 可能是有意 |

---

## 编译警告（快照）

> 抽取时捕获。完整输出如相关则放 appendix B。

| 警告 | 数量 | 最严重的文件 |
|---|---|---|
| `unused_variables` | 12 | `src/codegen/emit.rs` |
| `dead_code` | 4 | `src/auth/legacy.rs` |
| `clippy::too_many_arguments` | 6 | `src/payment/process.rs` |

---

## 按模块汇总

| 模块 | TODO | FIXME | HACK | Deprecated 调用 | 死代码 |
|---|---|---|---|---|---|
| `auth` | 3 | 2 | 1 | 1 | 1 |
| `payment` | 4 | 1 | 0 | 0 | 0 |
| ... | ... | ... | ... | ... | ... |

---

## Extraction Checklist

- [ ] 每条 TODO/FIXME 都有 file:line、标记、注释、年龄
- [ ] 若 git history 不可用，年龄列显示 `?` 且本节标 `[reduced fidelity]`
- [ ] Deprecated API 章节已填，或写 `(none found)`
- [ ] 死代码章节**只**来自编译器 / linter 输出，不含臆测
- [ ] 禁用测试章节即使为空也已审阅
- [ ] Build 警告快照在场（或写 `(build did not run)`）
- [ ] 按模块汇总的合计与各分节匹配
- [ ] 没有句子含 "should be done"、"ought to fix"、"is bad"（发表观点检查）
