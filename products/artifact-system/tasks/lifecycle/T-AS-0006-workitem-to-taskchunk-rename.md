---
task_id: T-AS-0006
slug: workitem-to-taskchunk-rename
title: "我把 work_item 改名 task_chunk_entity 后历史 kind 和字段不丢"
journey_stage: lifecycle
audience: ["ai-coding-agent", "maintainer"]
task_type: 迁移指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: 2026-06-09
intent_kind: migrate
involved_features: ["task-chunk-entity"]
prerequisites:
  - "理解 legacy work_item 的 kind + JSON fields blob 结构"
limits:
  - "本 task 只覆盖重命名 + 字段保持，不改 kind 语义集合"
related_intents:
  - "acceptance.intent#RenameWorkItemToTaskChunk"
related_next_tasks:
  - T-AS-0001
fact_cite:
  - "legacy work_item.rs:42,112-117 (kind + fields JSON blob)"
  - "migration/progress.md:12 (work_item → task_chunk_entity 重命名要求)"
  - "product-debate 5415991a § Phase 4 表 1 T-A6"
---

# 我把 work_item 改名 task_chunk_entity 后历史 kind 和字段不丢

---

## 本 task 可解答

- "work_item 改名 task_chunk 后我历史数据的 kind 会丢吗？"
- "task_chunk 的 fields JSON blob 重命名后还在吗？"
- "bug/story/testcase 三种 kind 重命名后语义保持吗？"

---

## 前提与限制

**你需要先**：理解 legacy `WorkItem` = 统一 `kind`（bug/story/testcase）+ JSON `fields` blob（`work_item.rs:42,112-117`），0 单测。

**本 task 受以下限制**：只做**重命名**（`work_item` → `task_chunk_entity`）+ 字段保持；不改 kind 枚举集合、不改 fields schema（migration/progress.md:12）。

---

## 完成路径

1. **重命名实体**：`WorkItem` → `TaskChunk`（文件 `work_item.rs` → `task_chunk_entity.rs`）。
2. **保持 kind 集合**：bug / story / testcase 三值不增不减、语义不变。
3. **保持 fields blob**：JSON `fields` 字段逐键保留，重命名不触碰内容。
4. **验证不丢**：构造含 kind+fields 的实例，重命名前后序列化 diff = 0（acceptance intent `RenameWorkItemToTaskChunk`）。

---

## 可观察的成功标志

重命名后任一历史实例的 kind 与 fields 逐键相等；序列化往返 diff = 0。

形式化定义：见 [`acceptance.intent#RenameWorkItemToTaskChunk`](../../intents/acceptance.intent)（intent-check 期由 invariants 重定位至 acceptance 以避免 cross-type havoc，Z3 verified）。

---

## Related Next Tasks

- **[T-AS-0001 - 我第一次读懂 artifact-system 怎么生产/读回一份文档制品](../onboarding/T-AS-0001-document-lifecycle-primer.md)** — 回到实体模型全貌

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
