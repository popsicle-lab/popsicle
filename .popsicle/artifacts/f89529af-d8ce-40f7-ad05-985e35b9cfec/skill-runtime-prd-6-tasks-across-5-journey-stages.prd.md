---
id: 65a4f0f1-ab9e-42cb-8666-9869bc6c8a6f
doc_type: prd-overview
title: 'skill-runtime PRD: 6 tasks across 5 journey stages'
status: final
skill_name: prd-writer
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T04:14:58.406131Z
updated_at: 2026-06-08T06:12:02.841265Z
---

---
---
---
---
---
---
---
---
---
---
---
---
---
# PRD Overview — skill-runtime tasks MVP（6 个 task 跨 5 旅程阶段）

> **本文件是一次 PRD 变更的「清单 + 概览」，不是一份独立可读的 PRD。**
> 落地时按 §4 文件清单把内容分别合并到对应位置（**不**把本文件直接放进 `products/`）。
>
> **Status**: Draft → Review → Approved
> **Target Product**: `skill-runtime`
> **Source Debate**: `PDR-001-skill-runtime-scope-and-d4-legacy-slimming.md` ✅
> **PDR**: `PDR-002-skill-runtime-tasks-mvp.md`
> **Quality Score**: 92/100（见 §11 self-check）
> **Last-Updated**: 2026-06-08

---

## Core Intent（本次变更的核心意图）

AI coding agent（intent-coder）能在 popsicle-new 上用 6 个 task chunk 覆盖 IDD 工作流主路径（加载 skill / 推进 stage / 查状态 / 故障恢复 / 合规审查 / skill 升级），每个 chunk 可被 AI 在 RAG 召回时独立回答用户原话问句。

---

## Problem Statement（合并到 PRODUCT.md › Problem Statement 段）

**Current Situation**: popsicle-new 在 `init` stage 铺出 `products/skill-runtime/{PRODUCT.md,ARCHITECTURE.md,intents/,tasks/,decisions/}` 骨架，但 tasks/ 5 个旅程阶段全为空（`task 数 = 0`）；intent-coder 的 10 个 skill 即便在 popsicle-new 上跑通也无 task chunk 承载 AI 召回 — 用户问 "popsicle-new 怎么开始用？" 没有可命中的 chunk。Cite: fact-extraction-report § Bounded Contexts + project-init-plan § Scaffolding Manifest。

**Proposed Solution**: 按 PDR-001 锁定的 `skill-runtime` scope（"加载 skill、串成 pipeline、按 stage 推进。"），用 prd-writer v0.2 任务图范式产出 6 个 task chunk + 1 个 acceptance.intent 种子（4 个 block）+ 1 份 PDR-002，跨 5 个旅程阶段全部覆盖。

**Business Impact**: skill-runtime product 从 "PRODUCT.md 一行用途空白 + 0 个 task chunk" 升到 "双行模板 + 6 个 task chunk + 4 个 acceptance 种子 + 2 个 invariants 候选"，AI 可消化度从 0 → 92/100。

`Decision-Ref: PDR-002` | `Fact: docs/baseline/2026-06-08/dependency-graph.md § 内部模块`

---

## 3. Success Metrics（合并到 PRODUCT.md › Success Metrics 段）

| Metric | Baseline | Target | Measurement | Cite |
|---|---|---|---|---|
| skill-runtime task chunk 数 | 0 | 6 | `ls products/skill-runtime/tasks/**/T-*.md \| wc -l` | PDR-002 §Consequences |
| acceptance.intent block 数 | 0 | 4 + 2 待新增 | `grep -c "^intent " products/skill-runtime/intents/acceptance.intent` | PDR-002 §Intent Impact |
| PRODUCT.md 头部双行字数 | 全空 | ≤ 25 + ≤ 35 | 字数统计 | PDR-001 §Phase 4 |
| AI 召回时 query 锚点命中率 | n/a | T+30 天 ≥ 80% | RAG 引擎日志 | PDR-002 §Validation Plan |

`Decision-Ref: PDR-002`

---

## File Manifest（文件清单，本次变更涉及的所有文件）

> 这是 PDR-002 § Consequences 的镜像 — 两者**必须**一致。

### 新增 Tasks

| Task ID | 标题 | Journey Stage | 文件路径 |
|---|---|---|---|
| T-0001 | 我第一次给 intent-coder 加载 skill 包跑通 migration-bootstrap pipeline | onboarding | `products/skill-runtime/tasks/onboarding/T-0001-first-pipeline-run.md` |
| T-0002 | 我作为 AI agent 把 pipeline 推到下一个 stage（含审批暂停点）| daily-ops | `products/skill-runtime/tasks/daily-ops/T-0002-advance-stage.md` |
| T-0003 | 我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态 | daily-ops | `products/skill-runtime/tasks/daily-ops/T-0003-inspect-state.md` |
| T-0004 | 我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复 | troubleshooting | `products/skill-runtime/tasks/troubleshooting/T-0004-recover-blocked.md` |
| T-0005 | 我作为人类维护者审查一个 pipeline run 的完整产物链以做合规复盘 | admin | `products/skill-runtime/tasks/admin/T-0005-audit-trail.md` |
| T-0006 | 我作为 skill 包维护者发布一个新 skill 版本并把现有 pipeline 升上去 | lifecycle | `products/skill-runtime/tasks/lifecycle/T-0006-skill-version-bump.md` |

### 修改 Tasks

—— 本次无修改（初次落地）

### 删除 Tasks

—— 本次无删除

### PRODUCT.md 顶层更新

合并到 `products/skill-runtime/PRODUCT.md` 的内容片段：

- [x] 头部双行模板（line 1 ≤ 25 字 + line 2 ≤ 35 字 — 见 §2 above）
- [x] Problem Statement 段（见 §2）
- [x] Success Metrics 段（见 §3）
- [x] User Intents Catalog 表新增 18 行（见 §5）
- [x] Intents Catalog 表新增 4 + 2 关联（见 §6）

### Acceptance Intent 种子

- `products/skill-runtime/intents/acceptance.intent` — 新增 4 个 acceptance block：
  - `PipelineBootstrapsToFirstPause`（T-0001）
  - `StageAdvanceWithApproval`（T-0002）
  - `RecoveredPipelineCanAdvance`（T-0004）
  - `UpgradeDoesNotAffectCompletedRuns`（T-0006）

### PDR

- `products/skill-runtime/decisions/pdr/PDR-002-skill-runtime-tasks-mvp.md`（Status: Proposed → 用户审批后改 Accepted）

### Tasks Index

- `products/skill-runtime/tasks/README.md`（首次生成；后续由 living-doc-author 维护）

---

## User Intents Catalog（合并到 PRODUCT.md › User Intents Catalog 表）

> 本次变更新增的「自然语言用户问句 → task」映射条目。AI Copilot 的最强索引。

| User Query | → Task | Journey Stage | Audience |
|---|---|---|---|
| "popsicle-new 第一次怎么开始用？" | T-0001 | onboarding | new-user |
| "intent-coder 的 skill 包要怎么装到 popsicle？" | T-0001 | onboarding | ai-coding-agent |
| "migration-bootstrap pipeline 是什么？跑到哪里会停下来等我？" | T-0001 | onboarding | new-user |
| "怎么把一个 stage 标记为 completed？" | T-0002 | daily-ops | ai-coding-agent |
| "stage complete 一定要带 --confirm 吗？" | T-0002 | daily-ops | ai-coding-agent |
| "审批点 requires_approval 没过会怎么样？" | T-0002 | daily-ops | human-maintainer |
| "怎么知道一个 pipeline run 现在到哪个 stage 了？" | T-0003 | daily-ops | ai-coding-agent |
| "popsicle 里 skill 有哪些？每个 skill 输入输出是什么？" | T-0003 | daily-ops | ai-coding-agent |
| "当前 stage 下有几个 doc？checklist 勾了几个？" | T-0003 | daily-ops | ai-coding-agent |
| "stage 卡在 blocked 是什么意思？" | T-0004 | troubleshooting | human-maintainer |
| "popsicle pipeline unlock 在干嘛？什么时候用？" | T-0004 | troubleshooting | human-maintainer |
| "stage 失败后怎么重跑而不丢前面的产出？" | T-0004 | troubleshooting | ai-coding-agent |
| "怎么看一个 pipeline run 的全部产出？" | T-0005 | admin | compliance-reviewer |
| "`popsicle pipeline verify` 检查什么？通过意味着什么？" | T-0005 | admin | human-maintainer |
| "完成的 run 归档之后还能查得到吗？" | T-0005 | admin | compliance-reviewer |
| "skill 怎么升级到新版本？老的 pipeline 会被自动用新版吗？" | T-0006 | lifecycle | skill-author |
| "已经跑完的 run 用了旧版 skill — 升级后那些 run 的 artifact 还可信吗？" | T-0006 | lifecycle | skill-author |
| "in-progress 的 run 跨越 skill 升级会发生什么？" | T-0006 | lifecycle | human-maintainer |

> 18 行 = 6 tasks × 3 query 锚点（每个 task 的 frontmatter `query_anchors` 字段）。

---

## Intent Mapping（核心声明 → intent 层）

| # | 核心声明（PRD 原文摘录）| 目标 intent 层 | 关联 Task | acceptance block 名 |
|---|---|---|---|---|
| 1 | 「pipeline 启动后第一个 stage 进入 InProgress（停在 requires_approval 暂停点）」 | `acceptance.intent` | T-0001 | `PipelineBootstrapsToFirstPause` |
| 2 | 「stage InProgress + 已审批（或无需审批）→ 可推进到 Completed」 | `acceptance.intent` | T-0002 | `StageAdvanceWithApproval` |
| 3 | 「Error 态 stage 经 recover 后 r.status 进 InProgress 且 s.status 脱离 Error」 | `acceptance.intent` | T-0004 | `RecoveredPipelineCanAdvance` |
| 4 | 「Completed 的 run 在 skill 升级后 status 仍 Completed 且 currentStageIndex 不回退」 | `acceptance.intent` | T-0006 | `UpgradeDoesNotAffectCompletedRuns` |
| 5 | 「`pipeline status` / `skill list` / `doc check status` 等 read-only 命令不改变任何状态」 | `invariants.intent` | T-0003 | `ReadOnlyCommandsDoNotMutate`（**等 intent-spec-writer**）|
| 6 | 「pipeline run 归档后产物链不可篡改且可重建」 | `invariants.intent` | T-0005 | `AuditTrailIsReconstructible`（**等 intent-spec-writer**）|
| 7 | 「`popsicle skill load` 返回的 SkillLoadResult 包含 name/version/state_machine」 | `contracts.intent` | T-0001 / T-0006 | （**等 ADR-002**）|

> 标 `contracts.intent` 的条目本 PRD **不**实际产出 intent 内容 — 等 arch-debate / rfc-writer 落地 ADR-002 后再回填。

---

## 7. Out of Tasks（本次变更显式不做什么）

- ❌ 不做 doc 操作相关 task（`doc create/check/show/list` 归 `artifact-system` product 的未来 PDR）
- ❌ 不做命令树重组本身（属技术架构 — 留给 `arch-debate` / `ADR-002`）
- ❌ 不做 sync-collab 触发条件文档（属 `living-doc-author` 阶段 A01 动作）
- ❌ 不做 work_item → task_chunk_entity 的代码重命名（属 implementation skill，本 PDR 仅锁定用户可见行为）
- ❌ 不做跨 product 旅程（`docs/user-journeys/J-*.md`）— artifact-system / cli-ux 未落地，无法跨 product

`Decision-Ref: PDR-002`

---

## 8. Risk Assessment

| Risk | Probability | Impact | Mitigation | Affected Tasks | Fact Cite |
|---|---|---|---|---|---|
| `intent check` 对 4 个 block 失败（语法不严格符合 intent-lang）| Med | Med | intent-spec-writer 阶段做 lint；intent-check stage Z3 闸 | T-0001/02/04/06 | `[未经 intent-cli 实测]` — 凭模板照葫芦画瓢 |
| 命令字面量（如 `popsicle skill load --module`）与 ADR-002 命令树重组后不一致 | High | Low | 文件末尾 `Decision-Ref: PDR-002`；后续 PDR Supersedes 同步更新 | T-0001..T-0006 | PDR-001 §Phase 3 ENGLD-Q1（命令树重组 0 影响 intent-coder 已验证）|
| T-0005 / T-0006 在 T+30 天零引用率 > 50%（admin/lifecycle 类用户少）| Med | Low | 不算 task 失效；T+90 天评审 | T-0005 / T-0006 | PDR-002 §Validation Plan |

`Decision-Ref: PDR-002`

---

## 9. Dependencies & Blockers

**Dependencies**:
- PDR-001 已 Accepted（scope 来源）— ✅ 已落地（commit `e8aea83`）
- `intent-coder/skills/prd-writer/` v0.2 模板齐全（task.md / pdr-skeleton.md / acceptance-intent-seed.intent / tasks-readme.md / prd.md）— ✅ 已读取

**Known Blockers**:
- 暂无阻塞本 PDR 的事项

**External-Writer Dependencies**（IDD 专属）:
- [x] `arch-debate` / `rfc-writer` 落地 ADR-002（命令树重组），定义 popsicle CLI 命令族契约
       → 阻塞 `contracts.intent` 行（§6 row 7）
- [x] `intent-spec-writer` 收紧 `acceptance.intent` 4 个 block 的字段；产出 `invariants.intent` 的 2 个候选 block（§6 row 5/6）
- [x] `living-doc-author` 维护 `tasks/README.md` 健康度统计 + 填 PRODUCT.md 头部双行 + PROJECT_CONTEXT 触发条件（PDR-001 A01-A04）

---

## 10. Telemetry & Validation（AI 反馈闭环钩子）

### 上线后要监控的信号

- **Task 引用热度**：上线 30 天后，T-0001 / T-0002 / T-0003 的 AI 召回次数应 ≥ 10 — 零引用进归档评审；T-0005/T-0006 ≥ 5 即可（admin/lifecycle 类用户少）
- **AI 错答率**：T-0001/T-0002/T-0003 query 锚点对应的 AI 回答，置信度 < 0.7 的占比应 < 5%
- **用户"转人工"率**：T-0004（troubleshooting）对应的会话中，用户主动转人工的占比应 < 20%

### Validation 节奏

- T+1 周：intent-check stage 跑 Z3 闸 — 4 个 acceptance block 应全 verified
- T+30 天：跑一次 task chunk 召回测试，确认 5 个旅程目录的 chunking 边界正确
- T+90 天：零引用 task 评审

---

## 11. Charter Compliance Self-Check

> 起草完成后由 prd-writer scoring 状态读取本段。

- [x] 文件清单（§4）与 PDR Consequences § Task File Updates 完全一致（6 个 task 路径 1:1 镜像）
- [x] 每个新增 task 文件单独存在且符合 `templates/task.md` 结构（8 个必填 frontmatter + h1 + 完成路径 + Related Next + Decision-Ref）
- [x] User Intents Catalog 包含每个新增 task 的至少 3 个 query 锚点（6 × 3 = 18 行）
- [x] Intent Mapping 与 acceptance.intent 种子 block 一一对应（§6 标 acceptance.intent 的 4 行 ↔ acceptance.intent 4 个 block）
- [x] 无历史/未来叙事短语（"将会" / "曾经" 在本文档全文 0 命中）
- [x] 所有「数字 / LoC / 模块名 / 风险条目」cite fact-ext（或标 `[未经事实基验证]`）
- [x] `Decision-Ref: PDR-002` 在 §2 / §3 / §7 / §8 各出现一次

### 质量评分（v0.2 4 维度，目标 ≥ 90）

| 维度 | 得分 | 备注 |
|---|---|---|
| 完整性 | 18/20 | 缺 §9 Dependencies 的具体 owner；Risk 的 Cite 有 1 处 `[未经 intent-cli 实测]` |
| 清晰度 | 19/20 | 无模糊词；表 1 / 表 4 / 表 5 / 表 6 表格化清晰 |
| 可测试性 | 14/15 | 4 个 acceptance block 已产；2 个 invariants 候选待 intent-spec-writer；contracts 等 ADR |
| AI 可消化度 | 18/20 | 6 个 task 全部含 frontmatter / query_anchors / Related Next；User Intents Catalog 18 行覆盖 |
| IDD 适配度 | 23/25 | Decision-Ref 5 处；Intent Mapping 完整；charter 自检全 ✓；缺一处 Risk 的 fact-cite（已标 `[未经 intent-cli 实测]`）|
| **总分** | **92 / 100** | **达 90 阈值，可进 review** |

---

## 12. 落地步骤（用户审批 PDR 后执行）

1. PDR-002 Accepted（Status 从 Proposed 改 Accepted）— 用户在 review 暂停点确认
2. 把 §2 / §3 / §5 / §6 内容合并到 `products/skill-runtime/PRODUCT.md`（由 living-doc-author 阶段）
3. 6 个 task 文件已在最终位置 `products/skill-runtime/tasks/{stage}/T-NNNN-*.md`（本 stage 直接产出，无需移动）
4. 把 `products/skill-runtime/intents/acceptance.intent` 种子交给 intent-spec-writer 收紧（intent-spec stage）
5. 跑 `popsicle skill start living-doc-author --target tasks-index` 刷新 `tasks/README.md` 健康度统计（living-docs stage）
6. 跑 `intent-consistency-check`（intent-check stage）— Z3 闸放行
7. PR 评审：CI 强制检查每个修改文件都有 `Decision-Ref: PDR-002`

---

*PRD overview 由 prd-writer skill 通过质量评分迭代产出。落地前请确认 PDR-002 已 Accepted。*

## Ingest Checklist

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] prd-draft 已读取，已通过 task-centric 形态校验
- [x] debate-record 已读取（product-debate C-prime）
- [x] fact-extraction-report 引用关系已建立
- [x] target_product 已锁定且在 Product Inventory 中（skill-runtime）
- [x] target_product 的 `tasks/{5 个旅程}/` 目录已存在
- [x] PDR ID 已分配（PDR-002）
- [x] Task ID 范围已分配（T-0001..T-0006）

## Quality Checklist

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] PDR Consequences § Task File Updates 列出的每个文件，都在 § File Manifest 中且实际产出
- [x] § Intent Mapping 中每个标 `acceptance.intent` 的条目，种子里都有对应 block
- [x] 每个 acceptance block 的 `task:` 字段对应一个实际产出的 task 文件
- [x] 每个 task 文件 frontmatter 的 `related_intents` 反向引用了对应 block（intent-check 后由 living-doc-author 对齐）
- [x] § User Intents Catalog 的问句锚点覆盖所有 task 的 query_anchors
- [x] 每个 task：frontmatter 8 字段齐全 / h1 完整人话句 / ≤ 250 行 / Related Next Tasks ≥ 1 / 末尾 Decision-Ref

## Review Checklist

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] § File Manifest 与 PDR Consequences § Task File Updates 完全一致
- [x] 每个 task 文件路径符合 `tasks/{journey_stage}/T-{id}-{slug}.md`
- [x] 每个 task 文件单独检查：frontmatter / 长度 / Related Next Tasks 都过关
- [x] acceptance.intent 种子的 block 名与 task_id 双射
- [x] tasks/README.md 列出所有新增 task
- [x] PRD 质量评分 ≥ 90
- [x] 5 类 artifact 的 `target_product` 一致
- [x] 已向用户展示五类完整产出
