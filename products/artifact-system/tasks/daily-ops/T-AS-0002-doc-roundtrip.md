---
task_id: T-AS-0002
slug: doc-roundtrip
title: "我 doc create 后确认 frontmatter+body 存盘能一字不差还原"
journey_stage: daily-ops
audience: ["ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: 2026-06-09
intent_kind: produce
involved_features: ["document-model", "markdown-engine"]
prerequisites:
  - "已有一个 active 的 pipeline run"
limits:
  - "本 task 只覆盖 doc 序列化往返，不含 guard 校验（见 T-AS-0003）"
related_intents:
  - "acceptance.intent#DocumentRoundTrips"
related_next_tasks:
  - T-AS-0003
fact_cite:
  - "docs/baseline/2026-06-09/api-contracts.md § model/document (to_file_content/from_file_content)"
  - "legacy document.rs:85-114"
---

# 我 doc create 后确认 frontmatter+body 存盘能一字不差还原

---

## 本 task 可解答

- "我 doc create 后存盘再读能一字不差还原吗？"
- "改一次文档版本号会自增、会链到上一版吗？"
- "markdown 的 section 怎么按标题抽取 / upsert？"

---

## 前提与限制

**你需要先**：有一个 active run（`popsicle pipeline status` 非空）。

**本 task 受以下限制**：只验收 Document 序列化往返 + revision 链；guard 校验在 [T-AS-0003](T-AS-0003-doc-check-guard.md)。

---

## 完成路径

1. **创建文档**：`popsicle doc create <skill> --title "<t>" --run <run-id>` → 返回 doc id + 磁盘文件路径。
2. **写正文**：编辑该文件 body（frontmatter 行 1-N 保留）。
3. **读回校验往返**：CLI 读盘 `from_file_content` 解析出的 Document，其 body 与磁盘 body 逐字相等（`document.rs:108,114`）。
4. **改版验证 revision**：触发一次 `new_revision`，确认 `version` +1 且 `parent_id` 指向上一版 id（`document.rs:85-105`）。

---

## 可观察的成功标志

存盘的文件再读回，body 一字不差；改版后 version 自增、parent 链正确。

形式化定义：见 [`acceptance.intent#DocumentRoundTrips`](../../intents/acceptance.intent)。

---

## Related Next Tasks

- **[T-AS-0003 - 我 doc check 看章节齐不齐 + checklist 勾完没](T-AS-0003-doc-check-guard.md)** — 文档存好后怎么过 guard

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
