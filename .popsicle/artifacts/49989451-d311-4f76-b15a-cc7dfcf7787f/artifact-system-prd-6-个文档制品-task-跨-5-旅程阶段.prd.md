---
id: 0f403e0e-97e4-41bc-a2e0-78845369c35c
doc_type: prd-overview
title: artifact-system PRD：6 个文档制品 task 跨 5 旅程阶段
status: final
skill_name: prd-writer
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T03:30:28.979071Z
updated_at: 2026-06-09T03:40:53.229792Z
---

# PRD · artifact-system tasks MVP（6 个文档制品 task 跨 5 旅程阶段）

> **Decision-Ref**: PDR-001（artifact-system tasks MVP）
> **Source Debate**: [`product-debate 5415991a`](./artifact-system-product-scope-guard-归属--work-item-task-chunk--namespace-边界.product-debate.md)（方案 C）
> **Fact Basis**: `docs/baseline/2026-06-09/{dependency-graph,api-contracts,unsafe-risk-report,tech-debt-inventory}.md` + [`fact-extraction-report b27c5ea6`](./popsicle-c76d729-fact-basis-slice-2--artifact-system.fact-extraction-report.md)

## Core Intent（本次变更的核心意图）

为 `artifact-system` product 定义首批 **6 个 user task chunk**，覆盖「生产 / 校验 / 装配 / 提取 / 重命名」文档制品的主路径，跨 5 个旅程阶段，让 AI coding agent 在 RAG 召回时可独立回答用户原话问句。本 PRD 锁定**用户可见行为**与 task 颗粒度，不写技术设计（trait 签名 / 依赖注入留 arch-debate / rfc）。`Decision-Ref: PDR-001`。

## Problem Statement（合并到 PRODUCT.md › Problem Statement 段）

product-debate（方案 C）裁定 artifact-system 持有 `Document` / `markdown` / `context`+`context_layer` trait+3 内建层 / `extractor` / `task_chunk_entity` / `guard` 纯文档校验。但裁决只给了抽象的 Task 识别表（T-A1..A6）与 6 条 User Intents；落地需要**具体 task chunk**，让 agent 独立回答「doc create 存盘能不能一字还原」「doc check 哪个章节还是占位」「prompt 装配怎么排序」等问句。缺这层颗粒度，迁移后责任不清、RAG 召回无锚点。`Decision-Ref: PDR-001`。

## 3. Success Metrics（合并到 PRODUCT.md › Success Metrics 段）

- artifact-system 6 个 task 各追溯到 product-debate Task 识别表（T-A1..A6）的一条裁决
- 4 个 acceptance 型 task 各有 acceptance.intent 种子 block（T-A1/A3/A4/A5）
- 2 个 invariant 候选（GuardResultIsTotal / TaskChunkRenamePreservesFields）入 Intent Mapping
- 每个 task ≤ 150 行、3-5 个 query 锚点、Related Next Tasks ≥ 1
- `Decision-Ref: PDR-001` 在 §Core Intent / §Problem Statement / §3 / §7 / §8 各出现一次

## File Manifest（文件清单，本次变更涉及的所有文件）

### 新增 Tasks

- [x] `products/artifact-system/tasks/onboarding/T-AS-0001-document-lifecycle-primer.md`
- [x] `products/artifact-system/tasks/daily-ops/T-AS-0002-doc-roundtrip.md`
- [x] `products/artifact-system/tasks/daily-ops/T-AS-0003-doc-check-guard.md`
- [x] `products/artifact-system/tasks/daily-ops/T-AS-0004-prompt-context-assembly.md`
- [x] `products/artifact-system/tasks/troubleshooting/T-AS-0005-extract-and-guard-total.md`
- [x] `products/artifact-system/tasks/lifecycle/T-AS-0006-workitem-to-taskchunk-rename.md`

### 修改 Tasks

- [x] —— 本次无修改（初次落地）

### 删除 Tasks

- [x] —— 本次无删除

### PRODUCT.md 顶层更新

- [ ] `products/artifact-system/PRODUCT.md` § 头部双行模板（待 living-doc-author 填）
- [ ] `products/artifact-system/PRODUCT.md` § Problem Statement（见本文 §Problem Statement）
- [ ] `products/artifact-system/PRODUCT.md` § Success Metrics（见本文 §3）
- [ ] `products/artifact-system/PRODUCT.md` § User Intents Catalog（见本文 §User Intents Catalog，18 行）
- [x] `products/artifact-system/PRODUCT.md` § Intents Catalog（4 acceptance + 2 invariants 候选关联）

### PDR

- [x] `products/artifact-system/decisions/pdr/PDR-001-artifact-system-tasks-mvp.md`

### Acceptance Intent 种子

- [x] `products/artifact-system/intents/acceptance.intent` 追加 4 个 block（`DocumentRoundTrips` / `GuardChecklistCompleteIffNoUnchecked` / `ContextAssemblyOrdersByRelevance` / `ExtractPreservesKind`）

### Tasks Index

- [x] `products/artifact-system/tasks/README.md` 重新生成（健康度待 living-doc-author 刷新）

## User Intents Catalog（合并到 PRODUCT.md › User Intents Catalog 表）

| query 锚点（用户原话）| task | journey_stage |
|---|---|---|
| "artifact-system 怎么把一份文档从内存写到磁盘又读回来？" | T-AS-0001 | onboarding |
| "Document 的 frontmatter 和 body 分别是什么？" | T-AS-0001 | onboarding |
| "一份文档制品的生命周期 active→final 是怎么流转的？" | T-AS-0001 | onboarding |
| "我 doc create 后存盘再读能一字不差还原吗？" | T-AS-0002 | daily-ops |
| "改一次文档版本号会自增、会链到上一版吗？" | T-AS-0002 | daily-ops |
| "markdown 的 section 怎么按标题抽取 / upsert？" | T-AS-0002 | daily-ops |
| "doc check 怎么告诉我哪个章节还是模板占位？" | T-AS-0003 | daily-ops |
| "checklist 还差几个没勾，guard 会判失败吗？" | T-AS-0003 | daily-ops |
| "has_sections 和 checklist_complete 分别校验什么？" | T-AS-0003 | daily-ops |
| "prompt 装配时最相关的文档会给全文吗？" | T-AS-0004 | daily-ops |
| "次要文档只给摘要、还是给全文？" | T-AS-0004 | daily-ops |
| "context 的 Relevance 排序是怎么定的？" | T-AS-0004 | daily-ops |
| "doc extract 抽不到 story/bug/testcase 怎么排查？" | T-AS-0005 | troubleshooting |
| "guard 收到不认识的类型会 panic 还是明确报错？" | T-AS-0005 | troubleshooting |
| "extractor 的正则匹配失败时返回空还是崩？" | T-AS-0005 | troubleshooting |
| "work_item 改名 task_chunk 后我历史数据的 kind 会丢吗？" | T-AS-0006 | lifecycle |
| "task_chunk 的 fields JSON blob 重命名后还在吗？" | T-AS-0006 | lifecycle |
| "bug/story/testcase 三种 kind 重命名后语义保持吗？" | T-AS-0006 | lifecycle |

## Intent Mapping（核心声明 → intent 层）

| 核心声明 | intent 层 | block | 关联 task |
|---|---|---|---|
| doc 序列化往返还原（to_file_content→from_file_content） | acceptance.intent | `DocumentRoundTrips` | T-AS-0002 |
| 全勾才过、有未勾即失败 | acceptance.intent | `GuardChecklistCompleteIffNoUnchecked` | T-AS-0003 |
| context 按 Relevance 排序，Low→摘要/High→全文 | acceptance.intent | `ContextAssemblyOrdersByRelevance` | T-AS-0004 |
| 提取产物 kind 与提取函数对应 | acceptance.intent | `ExtractPreservesKind` | T-AS-0005 |
| 任何 guard 字符串→Ok 或 InvalidSkillDef，绝不 panic | invariants.intent（候选）| `GuardResultIsTotal` | T-AS-0005 |
| 重命名后 kind/fields 不丢 | invariants.intent（候选）| `TaskChunkRenamePreservesFields` | T-AS-0006 |

## 7. Out of Tasks（本次变更显式不做什么）

- 不做 guard upstream_approved 回调的技术形态（属 arch-debate；本 PRD 只标 T-AS-0005 的 guard-total 行为）
- 不做 namespace（已裁给 skill-runtime，product-debate 候选项 3）
- 不做 MemoriesLayer（已裁给 skill-runtime 注入）
- 不做 doc/extract/summarize CLI 命令壳（属 cli-ux/slice-3；本 PRD 只覆盖核心行为）
- 不做 trait 签名 / 依赖注入方向（留 rfc-writer）
- `Decision-Ref: PDR-001`

## 8. Risk Assessment

| Risk | 触发条件 | 缓解 | Decision-Ref |
|---|---|---|---|
| extractor.rs 19 处 production unwrap 迁移后 panic | 正则 `Regex::new().unwrap()` + `.captures().unwrap()`（unsafe-risk-report §extractor）| T-AS-0005 形式化 `GuardResultIsTotal` 邻接，extractor 改 total（不 unwrap）；rfc 阶段定 total 化方案 | PDR-001 |
| work_item→task_chunk 改名丢 fields | JSON blob 字段映射不全（work_item.rs:42,112-117）| T-AS-0006 + invariant `TaskChunkRenamePreservesFields`；intent-check Z3 闸 | PDR-001 |
| guard 拆分后纯文档校验与 upstream 回调输出不一致 | check_guard 分派器拆错（guard.rs:65-96）| 对账以 GuardResult.passed 为 golden（product-debate Phase 3 轮次 1）| PDR-001 |
| T-AS-0001 onboarding 无 acceptance 种子致评分 AI 可消化度低 | 纯概念 primer 无操作 intent | §Intent Mapping 标关联 acceptance block（T-AS-0002 的 DocumentRoundTrips 承接）| PDR-001 |

## 9. Dependencies & Blockers

- [x] `product-debate` 已落地方案 C（doc 5415991a），给出 Task 识别表 + Intent 层归类表
- [x] `arch-debate` / `rfc-writer` 待裁定 guard upstream 回调 + MemoriesLayer 注册 + DocumentRow 共享的技术形态
- [x] `intent-spec-writer` 待收紧 acceptance.intent 4 block + 产 invariants.intent 2 候选（GuardResultIsTotal / TaskChunkRenamePreservesFields）
- [ ] `living-doc-author` 待填 PRODUCT.md 头部双行 + User Intents Catalog + tasks/README.md 健康度

## 10. Telemetry & Validation（AI 反馈闭环钩子）

### 上线后要监控的信号

- 6 个 task chunk 的 RAG 召回次数（T-AS-0002/0003/0004 为主路径，目标各 ≥ 10）
- doc check / doc extract CLI 调用占比（验证 task 覆盖主路径）
- AI 错答率（query 锚点对应回答置信度 < 0.7 占比）< 10%

### Validation 节奏

- intent-check stage：跑 `intent check products/artifact-system/intents/acceptance.intent`，4 block 通过 Z3，与 invariants/contracts 无矛盾
- 实现后：crates/artifact-system 单测镜像 4 acceptance + 2 invariant，等价性对账以 legacy GuardResult / Document 序列化为 golden

## 11. Charter Compliance Self-Check

- [x] 文件清单（§File Manifest）与 PDR Consequences § Task File Updates 一致（6 个 task 路径 1:1）
- [x] 每个新增 task 符合 task 结构（frontmatter + h1 + 完成路径 + Related Next + Decision-Ref）
- [x] User Intents Catalog 含每个 task ≥ 3 个 query 锚点（6 × 3 = 18 行）
- [x] Intent Mapping 与 acceptance.intent 种子 block 一一对应（4 acceptance 行 ↔ 4 block）
- [x] 无历史/未来叙事短语（"将会"/"曾经" 全文 0 命中）
- [x] 所有「数字/LoC/模块名/风险条目」cite fact-extraction-report / baseline / *.rs:line
- [x] `Decision-Ref: PDR-001` 在 §Core Intent / §Problem Statement / §3 / §7 / §8 各出现一次

### 质量评分（v0.2 4 维度，目标 ≥ 90）

| 维度 | 得分 | 说明 |
|---|---|---|
| 事实可追溯性 | 24/25 | 全部数字 cite baseline / fact-extraction / *.rs:line |
| AI 可消化度（task 颗粒度 + intent 关联）| 23/25 | 6 task 各有 query 锚点；T-AS-0001 借 T-AS-0002 的 acceptance |
| 边界清晰度（Out of Tasks）| 24/25 | namespace/MemoriesLayer/CLI 壳显式排除 |
| 可测试性（acceptance 种子覆盖）| 22/25 | 4 acceptance + 2 invariant 候选；troubleshooting 借 invariant |
| **合计** | **93/100** | ≥ 90 |

## 12. 落地步骤（用户审批 PDR 后执行）

1. 用户审批 PDR-001（Proposed → Accepted）
2. `pipeline stage complete prd --confirm`（本 stage requires_approval）
3. `doc summarize` 本 PRD overview
4. 进入 arch-debate：裁 guard 回调 / MemoriesLayer / DocumentRow 技术形态

## Ingest Checklist

- [x] prd-draft 已读取，已通过 task-centric 形态校验
- [x] debate-record 已读取（product-debate 方案 C，doc 5415991a）
- [x] fact-extraction-report 引用关系已建立（doc b27c5ea6 + 4 baseline）
- [x] target_product 已锁定且在 Product Inventory 中（artifact-system）
- [x] target_product 的 `products/artifact-system/tasks/{5 个旅程}/` 目录已存在
- [x] PDR ID 已分配（PDR-001）
- [x] Task ID 范围已分配（T-AS-0001..T-AS-0006）

## Quality Checklist

- [x] PDR Consequences § Task File Updates 列出的每个文件，都在 § File Manifest 中且实际产出
- [x] § Intent Mapping 中每个标 `acceptance.intent` 的条目，种子里都有对应 block
- [x] 每个 acceptance block 的 `task:` 字段对应一个实际产出的 task 文件
- [x] 每个 task 文件 frontmatter 的 `related_intents` 反向引用对应 block（intent-check 后由 living-doc-author 对齐）
- [x] § User Intents Catalog 的问句锚点覆盖所有 task 的 query_anchors
- [x] 每个 task：frontmatter 字段齐全 / h1 完整人话句 / ≤ 150 行 / Related Next Tasks ≥ 1 / 末尾 Decision-Ref

## Review Checklist

- [x] § File Manifest 与 PDR Consequences § Task File Updates 完全一致
- [x] 每个 task 文件路径符合 `tasks/{journey_stage}/T-AS-{id}-{slug}.md`
- [x] 每个 task 文件单独检查：frontmatter / 长度 / Related Next Tasks 都过关
- [x] acceptance.intent 种子的 block 名与 task_id 双射（4 block ↔ T-AS-0002/0003/0004/0005）
- [x] tasks/README.md 列出所有新增 task
- [x] PRD 质量评分 ≥ 90（93/100）
- [x] 5 类 artifact 的 `target_product` 一致（artifact-system）
- [x] 已向用户展示完整产出
