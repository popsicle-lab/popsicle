---
task_id: T-AS-0004
slug: prompt-context-assembly
title: "我 prompt 装配让最相关文档给全文、次要文档给摘要"
journey_stage: daily-ops
audience: ["ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: 2026-06-09
intent_kind: assemble
involved_features: ["context-assembly", "context-layer"]
prerequisites:
  - "已有若干 final 文档可供召回"
limits:
  - "本 task 覆盖 context 装配 + 3 内建层；MemoriesLayer 由 skill-runtime 注入，不在本 product"
related_intents:
  - "acceptance.intent#ContextAssemblyOrdersByRelevance"
related_next_tasks:
  - T-AS-0005
fact_cite:
  - "legacy context.rs:52-77 (assemble_input_context, Relevance 排序)"
  - "legacy context_layer.rs:22,34 (ContextLayer trait, assemble_layers)"
---

# 我 prompt 装配让最相关文档给全文、次要文档给摘要

---

## 本 task 可解答

- "prompt 装配时最相关的文档会给全文吗？"
- "次要文档只给摘要、还是给全文？"
- "context 的 Relevance 排序是怎么定的？"

---

## 前提与限制

**你需要先**：run 内有若干 final 文档（可被 UpstreamDocsLayer 召回）。

**本 task 受以下限制**：覆盖 `context` 装配 + `ContextLayer` trait + 3 内建层（ProjectContext / HistoricalRefs / UpstreamDocs）；`MemoriesLayer` 依赖 memory，由 skill-runtime 注册（product-debate 方案 C）。

---

## 完成路径

1. **触发装配**：`popsicle prompt <skill> --run <id> --related`（内部 `assemble_input_context` + `assemble_layers`，`context.rs`/`context_layer.rs:34`）。
2. **理解排序**：按 `Relevance` 排序（`context.rs:52-77`）。
3. **理解粒度**：`Low` → 只取 `extract_summary`（摘要）；`Medium` → sections 或全文；`High` → 全文。
4. **读输出**：返回的 full_prompt 中高相关文档为全文、低相关为摘要。

---

## 可观察的成功标志

装配输出里 High 相关文档为全文，Low 相关文档仅摘要；顺序按 Relevance 稳定。

形式化定义：见 [`acceptance.intent#ContextAssemblyOrdersByRelevance`](../../intents/acceptance.intent)。

---

## Related Next Tasks

- **[T-AS-0005 - 我 doc extract 抽不到条目 / guard 报未知类型时怎么排查](../troubleshooting/T-AS-0005-extract-and-guard-total.md)** — 装配/提取异常排查

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
