---
task_id: T-AS-0001
slug: document-lifecycle-primer
title: "我第一次读懂 artifact-system 怎么生产和读回一份文档制品"
journey_stage: onboarding
audience: ["ai-coding-agent", "new-user"]
task_type: 概念入门
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: 2026-06-09
intent_kind: understand
involved_features: ["document-model", "markdown-engine"]
prerequisites:
  - "popsicle-new 仓库已 clone（含 submodule legacy/popsicle）"
limits:
  - "本 task 只讲概念，不含具体 CLI 验收（见 T-AS-0002）"
related_intents:
  - "acceptance.intent#DocumentRoundTrips"
related_next_tasks:
  - T-AS-0002
  - T-AS-0003
fact_cite:
  - "fact-extraction-report b27c5ea6 § Bounded Contexts"
  - "docs/baseline/2026-06-09/api-contracts.md § model/document"
---

# 我第一次读懂 artifact-system 怎么生产和读回一份文档制品

---

## 本 task 可解答

- "artifact-system 怎么把一份文档从内存写到磁盘又读回来？"
- "Document 的 frontmatter 和 body 分别是什么？"
- "一份文档制品的生命周期 active→final 是怎么流转的？"

---

## 前提与限制

**你需要先**：理解 popsicle 的实体层级 Namespace→Spec→Issue→PipelineRun→**Document**（文档制品是 run 的产出）。

**本 task 受以下限制**：只讲 Document 模型与 markdown 引擎的概念；具体「存盘可还原」验收在 [T-AS-0002](../daily-ops/T-AS-0002-doc-roundtrip.md)。

---

## 关键概念

1. **Document = frontmatter + body**（`model/document.rs:9`）：frontmatter 是 YAML 元数据（id/doc_type/title/status/version 等），body 是 Markdown 正文。
2. **status 是 String 不是状态机**（`model/document.rs:13`）：取值 `"active"`（创建时）/ `"final"`（stage 完成时）。Document **没有**独立状态机——状态由 stage 完成驱动。
3. **revision 链**（`model/document.rs:85-105`）：改一版会 `version + 1` 并把 `parent_id` 链到上一版。
4. **markdown 引擎是 6 个纯函数**（`engine/markdown.rs`，382 行）：按标题抽 section / upsert section / 提取 summary / 提取 tags / 判定模板占位等。

---

## 可观察的成功标志

读完后你能说出：Document 由 frontmatter+body 组成、status 是 active/final 两值字符串、改版自增 version 且链 parent。

形式化承接：见 [`acceptance.intent#DocumentRoundTrips`](../../intents/acceptance.intent)（由 T-AS-0002 验收）。

---

## Related Next Tasks

- **[T-AS-0002 - 我 doc create 后确认存盘能一字不差还原](../daily-ops/T-AS-0002-doc-roundtrip.md)** — 把概念落到可验收行为
- **[T-AS-0003 - 我 doc check 看章节齐不齐 + checklist 勾完没](../daily-ops/T-AS-0003-doc-check-guard.md)** — 文档怎么被校验

---

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] Related Next Tasks ≥ 1

---

`Decision-Ref: PDR-001`
