---
task_id: T-AS-0005
slug: extract-and-guard-total
title: "我 doc extract 抽不到条目或 guard 报未知类型时怎么排查"
journey_stage: troubleshooting
audience: ["ai-coding-agent"]
task_type: 排错指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: 2026-06-09
intent_kind: diagnose
involved_features: ["extractor", "guard-doc-checks"]
prerequisites:
  - "doc extract 或 doc check 已返回异常 / 空结果"
limits:
  - "本 task 覆盖 extractor 提取 + guard 未知类型容错；不含 upstream 判定"
related_intents:
  - "acceptance.intent#ExtractPreservesKind"
  - "invariants.intent#EvaluateGuard"
related_next_tasks:
  - T-AS-0006
fact_cite:
  - "legacy extractor.rs:10,69,125 (extract_user_stories/test_cases/bugs)"
  - "legacy guard.rs:92-95 (unknown guard → InvalidSkillDef)"
  - "unsafe-risk-report § extractor (19 production unwrap)"
---

# 我 doc extract 抽不到条目或 guard 报未知类型时怎么排查

---

## 本 task 可解答

- "doc extract 抽不到 story/bug/testcase 怎么排查？"
- "guard 收到不认识的类型会 panic 还是明确报错？"
- "extractor 的正则匹配失败时返回空还是崩？"

---

## 前提与限制

**你需要先**：`doc extract` 返回空 Vec，或 `doc check` 报 guard 错误。

**本 task 受以下限制**：覆盖 extractor 提取 kind 保持 + guard 未知类型容错（`EvaluateGuard`）；`upstream_approved` 判定属 skill-runtime 注入。

---

## 完成路径

1. **确认提取函数对应 kind**：`extract_user_stories`→story、`extract_test_cases`→testcase、`extract_bugs`→bug（`extractor.rs:10,69,125`）。kind 不会被改写。
2. **空结果不是崩**：源文档无匹配章节时 `find_iter` 返回空、提取产物为空 Vec —— 这是正常空结果，不是错误。
3. **guard 未知类型必须明确报错**：`check_guard` 收到非 `has_sections`/`checklist_complete`/`upstream_approved` 的字符串，返回 `InvalidSkillDef`，**绝不 panic**（`guard.rs:92-95`）。
4. **留意 extractor 的 unwrap 风险**：legacy `extractor.rs` 有 19 处 production unwrap（`Regex::new().unwrap()` + post-find `.captures().unwrap()`）；迁移到 crates/artifact-system 时改 total（不 unwrap），由 rfc 定方案。

---

## 可观察的成功标志

提取产物 kind 与函数对应；无匹配时返回空 Vec 不崩；guard 收到未知类型返回 InvalidSkillDef 错误（非 panic）。

形式化定义：见 [`acceptance.intent#ExtractPreservesKind`](../../intents/acceptance.intent) 与 [`invariants.intent#EvaluateGuard`](../../intents/invariants.intent)（后者为 guard 全函数性不变量，受 `safety UnknownGuardIsInvalid` 守护，Z3 verified）。

---

## Related Next Tasks

- **[T-AS-0006 - 我把 work_item 改名 task_chunk_entity 后历史 kind/字段不丢](../lifecycle/T-AS-0006-workitem-to-taskchunk-rename.md)** — 提取产物实体的重命名

---

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] Related Next Tasks ≥ 1

---

`Decision-Ref: PDR-001`
