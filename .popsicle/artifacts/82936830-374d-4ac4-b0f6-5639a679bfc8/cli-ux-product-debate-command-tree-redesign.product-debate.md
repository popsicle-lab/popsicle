---
id: 01f55623-cd9a-4585-a7ef-23eace9dcb7d
doc_type: product-debate-record
title: cli-ux product debate command tree redesign
status: final
skill_name: product-debate
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:29:02.000000Z
updated_at: 2026-06-09T10:29:02.000000Z
---

# cli-ux Product Debate — command tree redesign

## Setup Checklist

- [x] 讨论主题已用一句话表达完毕
- [x] 目标 product 已绑定（cli-ux）
- [x] fact-extraction-report 存在状态已记录
- [x] 角色阵容确定（PM / UXR / ENGLD / DEV）
- [x] 用户置信度已设置（4：新设计优先，事实约束强）
- [x] 已向用户展示完整 setup 摘要并取得 `start` 确认

## Topic

cli-ux 如何把已切流的 `skill-runtime` 与 `artifact-system` 暴露成新的 popsicle CLI，而不是完整复刻 legacy 的 22 个子命令。

## Participants

| 角色 | 关注点 |
|---|---|
| PM | 命令树是否服务 IDD 主路径 |
| UXR | AI agent 与人类能否快速理解下一步 |
| ENGLD | 是否避免把旧 core 逻辑搬进 CLI |
| DEV | 实现范围、依赖方向、测试可行性 |

## Phase 1

共识：legacy CLI 是事实输入，不是目标契约。`api-contracts.md § Bounded Context：popsicle-cli` 证明 legacy 有 22 个子命令，其中 `doc` / `pipeline` / `skill` / `prompt` 是 IDD 主路径最相关入口，`checklist` / `item` / `sync` 属裁剪候选。

用户决策：不做完整 observable baseline；按新设计生产 cli-ux spec。

## Phase 2

候选方案：

| 方案 | 描述 | 结论 |
|---|---|---|
| A：legacy-compatible CLI | 保持 22 命令树并逐步换内部实现 | 否决：会把未设计好的旧产品面固化 |
| B：minimal IDD shell | 保留主路径命令，drop 重复/非主流程命令 | 采纳为基础 |
| C：domain-grouped shell | `runtime` / `artifact` / `admin` 三大命令组 | 部分采纳：内部边界用 domain，外部保持熟悉名 |

## Phase 3

角色评审：

| 角色 | 评审 |
|---|---|
| PM | B 能让 PRODUCT 一行用途收敛：把 IDD runtime/artifact 暴露给 agent |
| UXR | 保留 `skill` / `pipeline` / `doc` 名称，避免用户重新学习太多 |
| ENGLD | CLI 必须是 IO shell，业务逻辑只调用 `skill-runtime` / `artifact-system` / `storage` |
| DEV | `checklist` 与 `doc check` 重复，`item` 与 task_chunk 方向重复，`sync` 非 IDD 主流程 |

## Phase 4

最终决策：

1. cli-ux 保留单 binary `popsicle`。
2. MVP 保留并重接：`init`、`module`、`tool`、`skill`、`pipeline`、`spec`、`issue`、`namespace`、`doc`、`prompt`、`admin`、`git`、`memory`、`context`、`registry`、`completions`。
3. MVP drop：`checklist`（并入 `doc check`）、`item`（由 task_chunk/doc 派生路径替代）、`sync`（非 IDD 主流程）。
4. `migrate` 变为 `admin migrate`，不作为顶层命令。
5. CLI crate 不持有业务状态机、guard、document parser；只做 argv/format/error/file IO shell。

## Decision

采纳 B-prime：minimal IDD shell + domain boundary。旧 CLI 命令清单进入 PDR 的 disposition 表；新 spec 的 acceptance intent 锁定“命令调用后的状态副作用与输出结构”，不锁 legacy byte parity。

## Phase Coverage

- [x] Phase 1 已完成，有用户痛点 + 目标用户 + 约束清单
- [x] Phase 2 已产出 2-3 个差异化候选方案
- [x] Phase 3 全部角色已发表评审意见
- [x] Phase 4 已收敛到推荐方案 + 用户最终决策
- [x] 至少 4 个用户交互点（本次由用户连续指令给出方向：不做 baseline、按新设计迁移）

## Output Checklist

- [x] product-debate-record 含 Phase 1-4 全部小结
- [x] prd-draft 输入要点已列出
- [x] decision-matrix 可由本记录表格派生
- [x] 三份 artifact 的 Topic 一致
- [x] 已展示三份产出并取得 `approve` 确认
