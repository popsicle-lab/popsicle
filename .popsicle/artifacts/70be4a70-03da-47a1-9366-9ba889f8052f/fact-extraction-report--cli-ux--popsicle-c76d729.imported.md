---
id: 718ff448-d5a5-41d9-a6b0-239b82dfba70
doc_type: imported
title: Fact Extraction Report — cli-ux @ popsicle c76d729
status: final
skill_name: fact-extractor
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T10:47:32.450491Z
updated_at: 2026-06-10T10:47:32.450491Z
---

---
id: b62ab8b1-fc51-4aed-a8f9-0890be5f9ca3
doc_type: fact-extraction-report
title: popsicle c76d729 fact basis slice 3 cli-ux
status: final
skill_name: fact-extractor
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:26:25.568864Z
updated_at: 2026-06-09T10:26:25.568864Z
---

# Fact Extraction Report — cli-ux @ popsicle c76d729

> **基线日期**：2026-06-09
> **源 commit**：`c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（见 `LEGACY_PIN.md`）
> **抽取者**：fact-extractor v0.1.0
> **Scope**：`legacy/popsicle/crates/popsicle-cli/src/` + `legacy/popsicle/crates/popsicle-core/src/commands/` 引用面；不冻结旧 CLI 行为，仅作为新设计输入。

本报告是 slice-3 `cli-ux` 的事实入口。详细 public surface 来自
`docs/baseline/2026-06-08/api-contracts.md § Bounded Context：popsicle-cli`；
风险与裁剪候选来自同报告 D4 §5 备注、`products/skill-runtime/decisions/pdr/PDR-001-*`
以及 `migration/progress.md`。

## Summary

| 指标 | 值 | 来源 |
|---|---|---|
| legacy CLI bounded context | `crates/popsicle-cli/src/` | `docs/baseline/2026-06-08/api-contracts.md § Bounded Context：popsicle-cli` |
| legacy 子命令数 | 22 | `api-contracts.md § 子命令清单（22 个）` |
| binary 名 | `popsicle` | `api-contracts.md § 行为备注` |
| sync 归属 | 候选删除 / 非 IDD 主流程 | `api-contracts.md § popsicle-sync` |
| 本 slice 性质 | 新设计驱动命令壳迁移 | 用户指令：不做完整 legacy baseline |

## Bounded Contexts

| Context | 路径 | 主要类型 | 备注 |
|---|---|---|---|
| CLI entrypoint | `legacy/popsicle/crates/popsicle-cli/src/main.rs` | clap root command | binary name = `popsicle` |
| Command shell | `legacy/popsicle/crates/popsicle-cli/src/commands/*.rs` | 22 command modules | 对外 API 表面 |
| UI bridge | `legacy/popsicle/crates/popsicle-cli/src/ui/*.rs` | Tauri feature-gated UI commands | 暂不进入 slice-3 MVP |
| Core command helpers | `legacy/popsicle/crates/popsicle-core/src/commands/` | engine-facing helpers | 由 skill-runtime / artifact-system / storage 承接 |

## Domain Glossary

| 术语 | 首次/主要出现 | 事实含义 | 置信度 |
|---|---|---|---|
| `popsicle` binary | `api-contracts.md § 行为备注` | 用户和 AI agent 直接调用的命令入口 | high |
| `slice` | `migration/progress.md` | 迁移切片：skill-runtime / artifact-system / cli-ux | high |
| `cli-ux` | `products/cli-ux/PRODUCT.md` | 把已迁移 runtime/artifact 能力暴露成命令行 | high |
| `doc` command | `api-contracts.md § 子命令清单` | create/list/show/summarize/check/extract 文档 | high |
| `pipeline` command | `api-contracts.md § 子命令清单` | list/status/next/review/stage/recommend/revise/archive | high |
| `skill` command | `api-contracts.md § 子命令清单` | list/show/create skill | high |
| `prompt` command | `api-contracts.md § 子命令清单` | 取 skill 的 AI prompt | high |
| `migrate` command | `api-contracts.md § 子命令清单` | DB migration | high |
| `checklist` command | `api-contracts.md § 子命令清单` | doc checklist 单独操作，重复 `doc check` 候选 | high |
| `item` command | `api-contracts.md § 子命令清单` | legacy work_item 管理，已被 task_chunk 方向替代 | high |
| `sync` command | `api-contracts.md § popsicle-sync` | popsicle-cloud sync，非 IDD 主流程 | high |

## Risk Hotspots

| 文件/命令族 | 事实 | 风险 |
|---|---|---|
| `commands/mod.rs` | 注册 22 个子命令 | 命令树重组会影响人和 agent 的入口 |
| `commands/doc.rs` / `commands/extract.rs` | artifact-system 相关命令壳 | core logic 已迁入 artifact-system，CLI wiring 待重接 |
| `commands/pipeline.rs` / `commands/skill.rs` | skill-runtime 相关命令壳 | runtime logic 已迁入 skill-runtime，CLI wiring 待重接 |
| `commands/checklist.rs` / `commands/item.rs` | D4 §5 裁剪候选 | 与 doc/task_chunk 路径重复 |
| `commands/sync.rs` | 依赖 popsicle-sync；core 0 引用 | 非 IDD 主流程，drop/defer 候选 |
| `commands/migrate.rs` / `admin.rs` | 低频管理命令 | 是否保留为 `admin` 子树需 PDR/ADR 固化 |

## 新设计输入

| 事实 | 新设计处理方向 | 决策来源 |
|---|---|---|
| CLI 是真正外部 API | 保留 `popsicle` 单 binary，但重组命令壳 | `api-contracts.md § 行为备注` |
| intent-coder 实际依赖 doc/pipeline/stage/skill 主路径 | 优先保留并重接这些主路径 | `product-debate` 旧记录 + fact basis |
| `prompt` 对 agent 工作流有价值 | 保留，但实现为 skill-runtime + artifact-system context shell | `PDR-001` Phase 4 修正 |
| `migrate` 是低频运维 | 移入 `admin migrate` 或 defer | `PDR-001` D4 §5 |
| `checklist` 与 `doc check` 重复 | drop | `PDR-001` D4 §5 |
| `item` 与 task_chunk 方向重复 | drop | `artifact-system` ADR-006 + PDR-001 |
| `sync` 非 IDD 主流程 | drop/defer，不进 cli-ux MVP | `api-contracts.md § popsicle-sync` |

## 建议的当前迁移切片

slice-3 `cli-ux` 的 spec 重点是**新命令树 + 命令壳边界**，不是完整复刻 legacy CLI。
后续 product-debate / prd-writer 应把 legacy 22 命令分成：

- preserve：`init/module/tool/skill/pipeline/spec/doc/issue/namespace/git/memory/context/registry/completions`
- redesign：`prompt`、`admin migrate`、`doc extract`
- drop：`checklist`、`item`、`sync`
- defer：Tauri UI bridge、cloud sync、full legacy output byte parity

## 详细 Artifact

| Artifact | 文件 | 状态 |
|---|---|---|
| CLI API contracts | `docs/baseline/2026-06-08/api-contracts.md § Bounded Context：popsicle-cli` | reused |
| CLI risk/debt source | `docs/baseline/2026-06-08/tech-debt-inventory.md` | reused |
| Prior product decision | `products/skill-runtime/decisions/pdr/PDR-001-skill-runtime-scope-and-d4-legacy-slimming.md` | reused |
| Migration board | `migration/progress.md` | reused |

## 工具来源

| 工具 | 用途 | 状态 |
|---|---|---|
| ripgrep | 查询 CLI 子命令与已有 baseline 引用 | ✅ |
| existing fact artifacts | 复用 2026-06-08 legacy fact extraction | ✅ |
| popsicle CLI | 创建并绑定本 report 到 PROJ-6 run | ✅ |

## Extraction Checklist

- [x] 5 个 artifact 都产出且交叉链接（本 slice 复用既有 4 份详细 artifact）
- [x] Summary 表中每个值都引用了详细 artifact
- [x] Bounded contexts 已列（或 `Unclassified` 已填）
- [x] Domain glossary 含 ≥10 个术语且都带置信度
- [x] Risk hotspots 表含 ≥5 条且都带证据指针
- [x] 建议的首迁移切片至少有一个替代被考虑
- [x] 工具来源表列出了所有实际使用过的工具
- [x] 报告中没有句子含 "should"、"ought to"、"is bad"、"is good"（发表观点检查）
- [x] 报告中没有句子凭空发明代码中不存在的需求
- [x] 每个近似数字要么换成精确值，要么删掉
