---
task_id: T-AS-0003
slug: doc-check-guard
title: "我 doc check 看章节齐不齐和 checklist 勾完没"
journey_stage: daily-ops
audience: ["ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: 2026-06-09
intent_kind: validate
involved_features: ["guard-doc-checks"]
prerequisites:
  - "已有一份 active 文档"
limits:
  - "本 task 只覆盖纯文档校验 has_sections/checklist_complete；upstream_approved 属 skill-runtime 注入"
related_intents:
  - "acceptance.intent#GuardChecklistCompleteIffNoUnchecked"
related_next_tasks:
  - T-AS-0004
  - T-AS-0005
fact_cite:
  - "legacy guard.rs:79-90,173-266 (has_sections/checklist_complete/count_checkboxes)"
  - "product-debate 5415991a § Phase 4 表 1 T-A3"
---

# 我 doc check 看章节齐不齐和 checklist 勾完没

---

## 本 task 可解答

- "doc check 怎么告诉我哪个章节还是模板占位？"
- "checklist 还差几个没勾，guard 会判失败吗？"
- "has_sections 和 checklist_complete 分别校验什么？"

---

## 前提与限制

**你需要先**：有一份 active 文档，已绑定 skill 的 guard 字符串。

**本 task 受以下限制**：只覆盖**纯文档校验**（`has_sections` / `checklist_complete`）；`upstream_approved`（依赖 pipeline/run）由 skill-runtime 注入，不在本 product 行为内（product-debate 方案 C）。

---

## 完成路径

1. **运行校验**：`popsicle doc check --doc <id>`（或 stage 转换时自动触发 `check_guard`，`guard.rs:26`）。
2. **看 has_sections 结果**：缺失或仍是模板占位的 H2 章节会判失败（`guard.rs:173-213`）。
3. **看 checklist_complete 结果**：目标 section 里仍有 `- [ ]` 未勾即判失败；全部 `- [x]` 才过（`guard.rs:217-266`，`count_checkboxes` `guard.rs:79-90`）。
4. **修正后重跑**：补齐章节 / 勾完 checklist，重跑直到 GuardResult.passed = true。

---

## 可观察的成功标志

未勾完时 guard 判失败并指出差几个；全勾后判通过。GuardResult.passed 当且仅当 checkedBoxes == totalBoxes。

形式化定义：见 [`acceptance.intent#GuardChecklistCompleteIffNoUnchecked`](../../intents/acceptance.intent)。

---

## Related Next Tasks

- **[T-AS-0004 - 我 prompt 装配让最相关文档给全文、次要给摘要](T-AS-0004-prompt-context-assembly.md)** — 文档过 guard 后怎么被装配
- **[T-AS-0005 - 我 doc extract 抽不到条目 / guard 报未知类型时怎么排查](../troubleshooting/T-AS-0005-extract-and-guard-total.md)** — guard 收到未知类型时

---

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1

---

`Decision-Ref: PDR-001`
