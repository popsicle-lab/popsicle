---
agent_context: [Project preferences]
- 界面 / Agent 语言：简体中文
- 产品目录：`products/`
- ADR：`products/<product>/decisions/adr/`
- PDR：`products/<product>/decisions/pdr/`
- Pipeline 审批：delegate-dangerous（危险操作需审批（其余代批））
- 非危险 `requires_approval` 阶段可由 agent 代批完成；危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。
doc_type: shadow-implementer
id: doc-74
pipeline_run_id: 00000027-0000-4027-8002-27000000000027
status: active
title: test
version: 1
---

# test
