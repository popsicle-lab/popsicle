---
# ============================================================
# Task Frontmatter（必填字段不可省略；可选字段无内容时删除整行）
# ============================================================
task_id: T-0001                       # 必填 — product 内 4 位 0 填充递增，永不变
slug: first-time-verify               # 必填 — kebab-case，可改（不影响 task_id 引用）
title: "我第一次跑 intent check 直到拿到 PASS"   # 必填 — 完整人话句子，第一人称
journey_stage: onboarding             # 必填 — 5 选 1: onboarding / daily-ops / troubleshooting / admin / lifecycle
audience: ["end-user", "new-user"]    # 必填 — 数组；可选值见 docs/glossary.md § Audience
task_type: 操作指南                   # 必填 — 操作指南 / 概念解释 / 故障排查 / 决策参考 / 配置任务
decision_ref: PDR-0042                # 必填 — 引入或最近修改本 task 的 PDR
last_updated: 2026-05-13              # 必填 — YYYY-MM-DD

# ---- 可选字段（不需要时整行删除）----
intent_kind: verify                   # 用户意图大类（verify / diagnose / share / configure / ...）
involved_features: ["cli", "verifier-api"]
prerequisites:
  - "user.has(.intent_file_local)"
  - "user.installed(intent_cli, >= 0.2)"
limits:
  - "免费额度 5 次 / 月"
related_intents:
  - "acceptance.intent#T-0001-response-under-5s"
  - "invariants.intent#no-concurrent-verification"
related_next_tasks:
  - T-0002
  - T-0010
fact_cite:
  - "fact-extraction-report § Bounded Contexts row 2"
last_verified: ~                      # 由 living-doc-author / CI 填；未验证时为 ~
---

# {title}

> ⚠️ 本 task 是 `products/<product>/PRODUCT.md` 的展开篇章。文件路径
> `products/<product>/tasks/{journey_stage}/T-{nnnn}-{slug}.md` 是引用契约——
> 引用本 task 时**必须用 `task_id`（T-{nnnn}）**，不要用 slug（slug 会改）。

---

## 本 task 可解答

> 列出 3-5 个**自然语言用户问句**。这是 AI 召回时的 query 锚点。
> 写问句时用用户原话，不用产品术语。

- 怎么第一次跑 `intent check`？
- 跑完没拿到 PASS / FAIL 怎么办？
- 5 秒内没结果是不是卡死了？

---

## 前提与限制

> 用结构化字段陈述（而不是埋在散文段落里）。本节内容来自 frontmatter 的
> `prerequisites` 和 `limits`，正文用人话再解释一遍。

**你需要先**：

- 本地已有一个 `.intent` 文件（用 `intent init <name>` 可以生成示例）
- 已经装好 `intent` CLI（`brew install intent-cli` 或 `cargo install intent-cli`）

**本 task 受以下限制**：

- 免费账号每月有 **5 次** 验证额度，超出后下一个月 1 号重置
- CLI 版本需要 ≥ 0.2，旧版没有 `check` 子命令

---

## 完成路径

> 用「步骤 1 / 2 / 3」式编号清单写。每步含**用户动作** + **可观察结果**。
> 不要在步骤里塞「如果……则……」分支——分支去 `troubleshooting/` 目录的 task。

1. **执行命令**：在 `.intent` 文件所在目录运行

   ```bash
   intent check my-spec.intent
   ```

2. **CLI 提示「Uploading spec…」**：约 0.5 秒内出现，确认与 verifier 服务已建连

3. **CLI 返回 PASS 或 FAIL**：5 秒内必返回

   - 看到 **`✓ PASS`** → 你的 spec 通过了所有 invariants 和 acceptance
   - 看到 **`✗ FAIL: <invariant-name>`** → 进入 [T-0002](../onboarding/T-0002-handle-fail-counterexample.md) 看反例

> 5 秒上限是硬约束，由 `acceptance.intent#T-0001-response-under-5s` 形式化。

---

## 可观察的成功标志

> 用一句话陈述「task 完成时用户能看到什么」。这句话**必须对应**到 frontmatter
> `related_intents` 里的某个 acceptance block。

用户在终端看到 `✓ PASS` 字样，**且**该输出在 5 秒内出现。

形式化定义：见 [`acceptance.intent#T-0001-response-under-5s`](../../intents/acceptance.intent)

---

## Related Next Tasks

> 这是给 AI Copilot 的「追问路径」。用户完成本 task 后**最可能**继续问什么？
> 至少列 1 个（叶子 task 例外）。引用用 task_id。

- **[T-0002 - 拿到 FAIL 后看反例并调整 spec](../onboarding/T-0002-handle-fail-counterexample.md)**
  —— 上一步如果出 FAIL，进这里
- **[T-0010 - 一次性批量验证一组 .intent](../daily-ops/T-0010-batch-verify.md)**
  —— 首次验证通过后的日常使用

---

## 反向引用（informational, 由 living-doc-author 维护）

> 本节列出**引用了本 task 的其它资源**。手工维护容易腐烂，建议交给
> `living-doc-author` skill 重跑时刷新。

- `PDR-0042-verifier-mvp.md` § Consequences › Living Doc Updates
- `products/cli/tasks/daily-ops/T-0011-share-spec-with-team.md` § Related Next Tasks
- 暂无其它反向引用

---

## Charter Compliance

> 本节由 prd-writer skill 自动检查；扣分项见 `references/quality-rubric.md`。

- [ ] frontmatter 8 个必填字段齐全
- [ ] 文件长度 ≤ 250 行（硬上限）/ ≤ 150 行（软建议）
- [ ] title 是完整人话句子（非「xxx 功能」式标题）
- [ ] 「本 task 可解答」有 3-5 个用户原话问句
- [ ] 完成路径无「如果……则……」分支（分支走另开 task）
- [ ] Related Next Tasks ≥ 1 个（叶子 task 例外，需在 frontmatter 加 `is_leaf: true`）
- [ ] 文件末尾有 `Decision-Ref: {decision_ref}` 行

---

`Decision-Ref: PDR-0042`
