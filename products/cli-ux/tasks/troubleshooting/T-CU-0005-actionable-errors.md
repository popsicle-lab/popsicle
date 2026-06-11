---
task_id: T-CU-0005
slug: actionable-errors
title: "我遇到 guard lock not-found 错误时知道怎么修"
journey_stage: troubleshooting
audience: ["ai-coding-agent", "maintainer"]
task_type: 排错指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: diagnose
involved_features: ["errors", "guard", "spec-lock"]
prerequisites:
  - "CLI 返回错误"
limits:
  - "不要求保留 legacy 错误文案字节"
related_intents:
  - "acceptance.intent#ErrorsAreActionable"
related_next_tasks:
  - T-CU-0004
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § CLI 子命令清单"
---

# 我遇到 guard lock not-found 错误时知道怎么修

## 本 task 可解答

- "guard failed 后下一步该改哪个文档？"
- "spec locked by run 时我该查哪个 run？"
- "not found 错误要告诉我哪个 id 不存在吗？"

## 前提与限制

你已经收到 CLI 错误。新 CLI 锁定 actionable 结构，不锁 legacy 文案字节。

## 完成路径

1. CLI 返回非零 exit code。
2. 错误输出包含 category：guard / lock / not-found / invalid-args。
3. 错误输出包含对象 id 或 stage 名。
4. 错误输出包含下一步命令或修复提示。

## 可观察的成功标志

agent 不需要猜测，即可根据错误输出定位文档、run、spec 或命令参数。

## Related Next Tasks

- T-CU-0004

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
