---
id: 0f32b0fd-2279-4cf9-9f4a-e4f6959af1a8
doc_type: project-init-plan
title: popsicle-new slice-2 = artifact-system bootstrap 计划（复用既有骨架）
status: final
skill_name: project-init
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T02:37:25.951158Z
updated_at: 2026-06-09T02:48:15.762102Z
---

# Project Init Plan — popsicle-new（slice-2 = artifact-system）

> **Status**: draft → review → approved
> **Created**: 2026-06-09
> **Author**: copilot agent (Claude Opus 4.8) + 人类审批者 @curtiseng

本计划**不是首次 bootstrap**。仓库骨架已在 slice-1（PROJ-1，run `f89529af`）铺好——三个 product
的 4 件套目录、`docs/`、`migration/`、`legacy/popsicle` submodule 均已存在并提交。本 run 的 init
只做一件事：**把第二切片 pin 到 `artifact-system`**，复用既有骨架，scaffolding 对本 run 为 no-op。
product 命名沿用已批准的 inventory，不新增、不重命名。

> 与 slice-1 的 project-init-plan（doc `8ef92146`）配套阅读。本文件只覆盖「slice 选择 + 复用既有
> 骨架」的增量决策；仓库身份 / 模块依赖等已固化字段在此重申以自洽，但来源是既有骨架。

---

## Repository Identity

| 字段 | 值 | 来源 |
|---|---|---|
| 仓库名 | `popsicle-new` | 既有（slice-1 已定）|
| 本地路径 | `/Users/narwal/Workspace/github/popsicle/popsicle-new` | 已验证 |
| Issue key 前缀 | `PROJ`（用于 PROJ-1/2/3 …）| 既有 |
| 默认 agent target | `copilot`（本会话工作环境；slice-1 为 cursor，互不冲突）| 既有 |
| License | `MIT`（与 legacy popsicle 一致，无兼容性问题）| 既有 |
| 初始分支 | `main` | 既有 |

---

## Product Inventory

> 沿用 slice-1 已批准的 3-product inventory；**本 run 不改动命名**。唯一变化是 `status` 列：
> 本切片把 `artifact-system` 从 `scaffold-only` 提升为 **slice（本切片）**。

| # | Product | 一行用途 | 估计 LoC（legacy 源） | 来源（legacy 路径，pinned c76d729）| 状态 |
|---|---|---|---|---|---|
| 1 | `skill-runtime` | Skill 状态机 + Pipeline DAG + Run/Spec ledger + Hook 总线 + 注册表 | 已迁（slice-1）| `popsicle-core/src/{model,engine/hooks,registry,memory}/` | done（slice-1，in-shadow）|
| 2 | `artifact-system` | Document 实体 + Markdown 智能编辑 + Guard + Context 装配 + WorkItem 提取 + frontmatter | ~2,276（6 模块实测）| `popsicle-core/src/{model/document, engine/{markdown,guard,context,context_layer,extractor}}.rs` | **slice（本切片）** |
| 3 | `cli-ux` | popsicle CLI 框架 + 命令族 + Bootstrap | `[TBD: needs archaeology]` | `popsicle-cli/` + `popsicle-core/src/commands/` | scaffold-only |

**对照硬规则的校验**：
- [x] 每个名字客户能识别（不是 `core` / `utils` / `common`）—— artifact / document / context 客户可识别
- [x] 数量在 3-7 之间（=3）
- [x] 没有万能的 "shared" product
- [x] 本切片 product（`artifact-system`）的 legacy 来源已逐模块核实存在（见 Legacy Source 表）

---

## Legacy Source

| 字段 | 值 |
|---|---|
| 仓库 URL | `https://github.com/popsicle-lab/popsicle.git` |
| Submodule 路径 | `legacy/popsicle` |
| Pinned commit SHA | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（`v0.4.0-19-gc76d729`）|
| Pin 理由 | 沿用 slice-1 的 pin；同一 SHA 保证两切片考古基线一致 |
| License | `MIT` —— 与新仓库兼容 |
| 多仓库？| 否（单 submodule）|

**本切片 archaeology 范围（逐模块已核实存在于 pinned SHA）**：

| legacy 模块 | 行数 | 责任 |
|---|---|---|
| `popsicle-core/src/model/document.rs` | 191 | Document 实体 + frontmatter |
| `popsicle-core/src/engine/markdown.rs` | 382 | Markdown 智能编辑 |
| `popsicle-core/src/engine/guard.rs` | 858 | Guard（章节/checklist 校验闸）|
| `popsicle-core/src/engine/context.rs` | 244 | Context 装配 |
| `popsicle-core/src/engine/context_layer.rs` | 236 | Context 分层 |
| `popsicle-core/src/engine/extractor.rs` | 365 | WorkItem 提取 |

> 另含 `namespace` 实体 + `task_chunk_entity`（旧 `work_item` 重命名）+ doc/extract/summarize 命令族，
> 具体落点由 facts 阶段（fact-extractor）考古确认；边界由 **PDR-001** 锁定。

---

## First Migration Slice

**选中**：`artifact-system`（slice-2）

**理由**：
1. **slice-1（skill-runtime）已 in-shadow**，其反向依赖点已稳定——artifact-system 可安全跟进。
2. **它是 intent-coder 文档体系最紧耦合的引擎**：Markdown / Guard / Context 都直接喂 LLM prompt，迁好后下游 cli-ux 的命令族才有依附点。
3. **范围明确、边界已锁**：6 个 legacy 模块 + namespace + task_chunk，PDR-001 已界定边界，archaeology 目标清晰。

**考虑过的替代**：

- **`cli-ux`**：用户直接可见，反馈直观；但它聚合 artifact-system + skill-runtime 的能力，先迁会产生大量向下空依赖。否决：应作为最后一个切片（slice-3）。
- **继续深做 `skill-runtime`**（补 T-0003/T-0005 等）：价值有限且非切片级里程碑；留作 skill-runtime 后续 task，不占第二切片名额。

**「本切片」具体意味着什么**：
- 推进期间，只有 `products/artifact-system/` 被填进非骨架内容（PRODUCT / ARCHITECTURE / 三层 intent / 首批 ADR/PDR）。
- `cli-ux` 仍停在 `[TBD: needs archaeology]`，等 slice-3。
- artifact-system 完整复用 slice-1 的 playbook（fact-extractor → … → intent-check → living-doc → crates/artifact-system 实现 → 等价性对账）。

---

## Module Dependencies

| 模块 | 来源 | 用途 | 状态 |
|---|---|---|---|
| `intent-coder` | 本地装（slice-1 已激活，v0.3.0）| 迁移工具箱（含 .intent + Z3 能力）| ✅ 已装（active）|
| `intent-validate` tool | `intent-coder/tools/intent-validate/` | intent check + Z3 封装 | 在 intent-spec 阶段前确认可用 |

> 本切片不引入新模块依赖；沿用 slice-1 既有装配。所有路径在 slice-1 批准时已验证存在。

---

## Scaffolding Manifest

> **本 run 的 scaffolding 是 no-op**：下列目录/文件已在 slice-1 bootstrap 时创建并提交。
> 本切片不创建任何新骨架文件——只在下游阶段（facts/prd/…）**填充** `products/artifact-system/`
> 既有 stub。清单之外不创建，清单之内（已存在）不重复创建。

### 已存在、本切片将被填充的目标（artifact-system 4 件套）

```
products/artifact-system/                    # 已存在（slice-1 铺）
├── PRODUCT.md                               # 现 [TBD] —— 由 prd-writer 填
├── ARCHITECTURE.md                          # 现 [TBD] —— 由 arch-debate/rfc 填
├── tasks/                                   # 5 旅程阶段目录已存在
│   ├── README.md / onboarding/ / daily-ops/ / troubleshooting/ / admin/ / lifecycle/
├── intents/
│   ├── invariants.intent                    # 现空 stub —— 由 intent-spec-writer 填
│   ├── contracts.intent                     # 现空 stub
│   └── acceptance.intent                    # 现空 stub
├── decisions/{adr,pdr}/                      # 现仅 .gitkeep
└── proposals/{exploring,proposed,accepted,rejected}/
```

### 本切片不触碰

```
docs/                                        # 已存在（slice-1）
migration/                                   # 已存在；progress.md row 2 状态由 not-started → in-progress
legacy/popsicle/                             # 已 pin c76d729，只读考古
products/skill-runtime/                      # slice-1 资产，不动
products/cli-ux/                             # slice-3 骨架，本切片不动
```

> ⚠️ 5 个旅程阶段目录（onboarding / daily-ops / troubleshooting / admin / lifecycle）在 slice-1
> 已为 artifact-system 铺齐，本切片复用，不新增第 6 个。

---

## 风险 & 未决问题

| 风险 / 问题 | 缓解 / 负责人 |
|---|---|
| **重跑 migration-bootstrap 的 init 与 slice-1 重叠**（init/debate 部分冗余）| 本 init 显式声明为「复用既有骨架」、scaffolding no-op；实质考古集中在 facts 阶段 |
| **`task_chunk_entity`（旧 `work_item`）重命名边界** | 由 facts 阶段考古 legacy `extractor.rs` + model 确认；PDR-001 锁定边界，命名变更记入 traceability |
| **artifact-system 与 skill-runtime 的接口面**（Context 装配是否依赖 skill ledger）| facts 阶段标注跨 product 依赖；若发现强耦合，product-debate 阶段裁决归属 |
| **guard.rs 858 行是最大单模块** | archaeology 时优先拆其职责（章节校验 / checklist 完成度 / 模板占位拒绝），避免一次性迁整块 |
| **doc/extract/summarize 命令族归属** | 命令壳属 cli-ux（slice-3），但其核心逻辑属 artifact-system；facts 阶段标注「壳 vs 核」边界 |

---

## Plan Checklist

- [x] Repository Identity 表全填满（无占位符；均为既有已固化值）
- [x] Product Inventory 有 3-7 条，每条客户能识别（=3，沿用 slice-1）
- [x] 每个 product 条目有状态（slice / scaffold-only / done）和一行用途
- [x] 正好一个 product 标 **slice**（`artifact-system`）
- [x] Legacy Source 章节有 URL + pinned SHA + license 检查（c76d729；6 模块逐一核实存在）
- [x] First Migration Slice 章节列了 ≥1 个替代并说明否决理由（cli-ux / 继续 skill-runtime）
- [x] Module Dependencies —— 每条路径都已验证存在（沿用 slice-1 既装）
- [x] Scaffolding Manifest 已穷举（本 run 为 no-op；列明既有目标 + 不触碰项）
- [x] **v0.2 任务图：artifact-system 的 5 个旅程阶段目录已存在（slice-1 铺）**
- [x] **v0.2 任务图：`docs/user-journeys/` 全局层已存在（slice-1 铺）**
- [x] 风险 & 未决问题 —— 每个已知顾虑都已记录

---

## Survey Checklist

- [x] Legacy 仓库 URL + pinning commit SHA 已记录（c76d729，legacy/popsicle submodule）
- [x] License 兼容性已检查（MIT，与既有一致）
- [x] 3-7 个 product 命名完毕，每个有一行用途（沿用 slice-1 inventory）
- [x] 每个 product 名通过「客户可识别」测试
- [x] 首个迁移切片已选定（本切片 = artifact-system），且至少考虑过一个替代（cli-ux）
- [x] 新仓库名 + issue key 前缀已定（popsicle-new / PROJ）
- [x] 用户额外提供的 writer 模块依赖路径已验证存在（intent-coder，slice-1 已装）
- [x] fact-extraction-report 缺失：本切片显式同意先做 init survey，实质考古由下一阶段 facts 产出

## Scaffolding Verification

> 本 run scaffolding 为 no-op；以下逐项核验「既有骨架已就位」。

- [x] artifact-system 4 件套目录在磁盘上（PRODUCT/ARCHITECTURE/tasks/intents/decisions/proposals）
- [x] 本 run 未创建任何新骨架文件（只会在下游阶段填充既有 stub）
- [x] `legacy/popsicle/.git` 存在（submodule 正确 pin）
- [x] `git -C legacy/popsicle rev-parse HEAD` == c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1
- [x] `popsicle module list` 列出 intent-coder（slice-1 已装 active）
- [x] artifact-system 的 intents/*.intent 仍为空 stub（等下游填，未被本 run 编造）
- [x] `git status` 无意外改动（本 run 仅产出 init 文档本身）
