---
task_id: T-CU-0008
slug: self-host-workflow-provenance
title: "我用 popsicle-new 自己的二进制跑 workflow 并确认来源"
journey_stage: lifecycle
audience: ["ai-agent", "maintainer"]
task_type: 验收任务
decision_ref: PDR-002
last_updated: 2026-06-10
last_verified: ~
intent_kind: validate
involved_features: ["workflow-cli", "binary-provenance", "self-hosting"]
prerequisites:
  - "popsicle-new has been built locally"
limits:
  - "full legacy CLI byte parity is out of scope"
related_intents:
  - "acceptance.intent#SelfHostedWorkflowSmokePasses"
  - "acceptance.intent#BinaryProvenanceVisible"
  - "self-hosting-invariants.intent#WorkflowSmokeDoesNotDependOnParentBinary"
related_next_tasks:
  - T-CU-0004
  - T-CU-0007
fact_cite:
  - "PROJ-9 dogfood failure"
---

# 我用 popsicle-new 自己的二进制跑 workflow 并确认来源

## 本 task 可解答

- "这次 workflow 到底是不是 `popsicle-new/target/debug/popsicle` 跑的？"
- "`popsicle-new` 自己能不能 create issue / start run / create doc / complete stage？"
- "二进制来源和 workspace root 怎么看？"

## 前提与限制

本 task 只覆盖 self-hosting MVP，不覆盖 legacy 22 个命令的字节级兼容。

## 完成路径

1. 运行 `./target/debug/popsicle doctor --format json`。
2. 创建 smoke issue，并启动一个最小 pipeline run。
3. 调用 `pipeline next`、`doc create`、`pipeline stage complete`、`pipeline status`。
4. 确认所有命令由当前 workspace 的 binary 执行。

## 可观察的成功标志

`./target/debug/popsicle` 完成 workflow smoke，且 doctor 输出包含 executable path、workspace root 和 current-workspace binary match。

## Related Next Tasks

- T-CU-0004
- T-CU-0007

Decision-Ref: PDR-002
