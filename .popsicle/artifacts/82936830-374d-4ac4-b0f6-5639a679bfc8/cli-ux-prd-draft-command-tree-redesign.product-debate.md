---
id: 5de2ad9f-6b47-455e-be38-a4bae1cdf1a1
doc_type: product-debate-record
title: cli-ux prd draft command tree redesign
status: final
skill_name: product-debate
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:29:24.244947Z
updated_at: 2026-06-09T10:29:24.244947Z
---

# cli-ux PRD Draft Summary — command tree redesign

## Setup Checklist

- [x] 讨论主题已用一句话表达完毕
- [x] 目标 product 已绑定（cli-ux）
- [x] fact-extraction-report 存在状态已记录
- [x] 角色阵容确定（PM / UXR / ENGLD / DEV）
- [x] 用户置信度已设置（4）
- [x] 已向用户展示完整 setup 摘要并取得 `start` 确认

## Topic

把 legacy `popsicle` CLI 重组为新设计驱动的 IDD 命令壳。

## Participants

| 角色 | 贡献 |
|---|---|
| PM | 收敛命令树 disposition |
| UXR | 保持 agent 可预测的下一步体验 |
| ENGLD | 约束业务逻辑下沉 |
| DEV | 约束实现 MVP |

## Phase 1

问题定义：legacy 22 个子命令不是全部产品承诺；cli-ux 只保留 IDD 主路径需要的命令壳。

## Phase 2

候选方案 A/B/C 与主 debate record 一致，采纳 B-prime。

## Phase 3

评审重点：命令名保留熟悉入口，但实现不复制 legacy core；重复命令与非主流程命令裁剪。

## Phase 4

最终 PRD draft 输入：

| Task | 旅程 | 用户问题 | Intent 层 |
|---|---|---|---|
| T-CU-0001 | onboarding | 我第一次初始化 popsicle-new 并看到下一步 | acceptance |
| T-CU-0002 | daily-ops | 我创建 issue 并启动 pipeline run | acceptance |
| T-CU-0003 | daily-ops | 我创建/查看/校验 stage 文档 | acceptance |
| T-CU-0004 | daily-ops | 我查询 pipeline 状态并推进 stage | acceptance + invariants |
| T-CU-0005 | troubleshooting | 我遇到 guard/lock/not-found 错误时知道怎么修 | acceptance |
| T-CU-0006 | admin | 我执行低频 admin migrate/reinit | acceptance |
| T-CU-0007 | lifecycle | 我确认旧命令 checklist/item/sync 被移除或延后 | invariants |

## Decision

PRD writer 以这 7 个 task 起草 cli-ux 任务图。CLI byte parity 不作为验收；验收聚焦状态副作用、JSON 字段稳定性、错误可诊断性、命令树 disposition。

## Phase Coverage

- [x] Phase 1 已完成，有用户痛点 + 目标用户 + 约束清单
- [x] Phase 2 已产出 2-3 个差异化候选方案
- [x] Phase 3 全部角色已发表评审意见
- [x] Phase 4 已收敛到推荐方案 + 用户最终决策
- [x] 至少 4 个用户交互点

## Output Checklist

- [x] product-debate-record 含 Phase 1-4 全部小结
- [x] prd-draft 输入要点已列出
- [x] decision-matrix 可由本记录表格派生
- [x] 三份 artifact 的 Topic 一致
- [x] 已展示三份产出并取得 `approve` 确认
