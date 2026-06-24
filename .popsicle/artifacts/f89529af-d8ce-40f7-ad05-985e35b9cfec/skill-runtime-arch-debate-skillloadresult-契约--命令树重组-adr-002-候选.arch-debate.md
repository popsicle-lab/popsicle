---
id: c02c41a4-caec-4b7f-b422-307b7cb656b6
doc_type: arch-debate-record
title: 'skill-runtime arch-debate: SkillLoadResult 契约 + 命令树重组（ADR-002 候选）'
status: final
skill_name: arch-debate
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T07:05:48.511859Z
updated_at: 2026-06-08T07:22:59.167989Z
---

---
artifact: arch-debate-record
slug: skill-runtime-skillloadresult-and-command-tree
topic: "skill-runtime 对 AI agent 的稳定技术契约：SkillLoadResult 字段/版本语义 + 命令树重组（每 product ≤7 公开命令）"
participants: [ARCH, SEC, PERF, OPS, DEV]
confidence: 3
date: 2026-06-08
query_anchors:
  - "SkillLoadResult 当时为什么定成 name/version/state_machine 这三个字段？"
  - "命令树重组哪些方案被否了，理由是什么？"
  - "这个决策走 ADR-002 还是要 CADR？"
---

# 架构辩论纪要 — skill-runtime-skillloadresult-and-command-tree

> 由 `arch-debate` skill 生成。本纪要是技术决策的**审计轨迹**，供 rfc-writer /
> adr-writer 追溯论据，也供后人理解「当时为什么这么选」。

## Topic

定义 skill-runtime 对 AI coding agent 暴露的两条核心技术契约并裁决其走 ADR 还是 CADR：
(1) `popsicle skill load` 返回的 `SkillLoadResult{name, version, state_machine}` 字段与版本语义；
(2) 命令树从「admin/sync/extract/checklist/migrate/prompt 全混在一棵树」重组为「每 product ≤ 7 公开命令」的边界划分。
来源：prd-overview §6 Intent Mapping 第 7 行 `[contracts.intent 候选 → 等 ADR-002]` + product-debate CON-SR-01。

## Participants

| 角色 | 立场速写 |
|---|---|
| ARCH | 主持。关注契约可演进性与命令树边界是否与 product 划分同构。 |
| SEC | 关注契约 schema 校验闭环（CON-SR-01 当前仅「部分验证」）、命令重组是否扩大可被误用的表面。 |
| PERF | 关注 `skill load` 是否进热路径、命令解析开销。fact 基线显示性能非瓶颈。 |
| OPS | 关注命令重组的可观测性/回滚、对 6 个 task chunk 写死命令字面量的兼容冲击。 |
| DEV | 关注实现成本与技术债：guard 已 hardcode（不走 trait 扩展），命令重命名波及面。 |

用户置信度：3/5

## Phase 1 — 技术问题 + 质量属性（NFR）

### 要解决的问题

skill-runtime 需要对外（主要用户 = AI coding agent）冻结两条稳定契约：

1. **SkillLoadResult 契约**：`popsicle skill load` 返回结构必须含 `name / version / state_machine`，
   且三者的语义、版本兼容规则、schema 校验闭环要定死（CON-SR-01 当前仅「部分验证」= 只做了 schema 校验，无版本语义）。
2. **命令树重组边界**：把现状「不可数命令混在一棵 popsicle CLI 树」收敛为「每个 product 暴露 ≤ 7 个公开命令」，
   并明确 skill-runtime 这一 product 的公开命令集（候选：`skill load/run` + `pipeline start/stage/status` + 恢复类）。

### 硬约束

- **HC-1（ADR-001）**：intent-coder 是 popsicle-new 内部消费者，CLI 命令变动算 internal API 调整；
  命令树重组「0 影响 intent-coder」已在 PDR-001 §Phase 3 ENGLD-Q1 验证。→ 重组**不**触发外部破坏性变更。
- **HC-2（状态机不变量）**：skill 状态机仅允许 `{pending → in_progress → completed/blocked}` 转移
  （product-debate CON 表 INV）。SkillLoadResult.state_machine 必须能表达且只能表达这组转移。
- **HC-3（命令字面量兼容）**：PRD §8 列为 High 风险——6 个 task chunk 写死的命令字面量
  （如 `popsicle skill load --module`）须与重组后命令树**逐字一致**，否则 RAG 召回的答案会失效。
- **HC-4（charter Layer Map）**：触及 charter「四条铁律 / Layer Map」的决策须走 **CADR**；普通架构决策走 **ADR-002**。

### 质量属性优先级（ARCH 提议排序，待用户确认）

> P1 最高。理由全部 cite 事实基。

1. **可演进性（Evolvability）** — 契约要能加字段而不破坏既有 agent 调用；命令树要能随 product 增加而扩展。
2. **安全/契约完整性（Contract Integrity）** — CON-SR-01 必须从「部分验证」补齐到「schema + 版本语义」双闭环。
3. **可观测性/可运维（OPS）** — 命令重组要可回滚、有迁移期 alias。
4. **性能（PERF）** — 最低优先级：fact 基线 `unsafe=0`、`skill load` 不在高频热路径，时延非瓶颈（属能力边界 D2，不进 `.intent`，由 benchmark 守护）。

### 事实基引用

- **F-1**：`SkillDef` 含 inputs / artifacts / workflow（状态机 + guard）/ hooks（`crates/popsicle-core/src/model/skill.rs:11`，fact-extraction-report § Domain Glossary）。
- **F-2**：CON-SR-01「skill load 必须返回 SkillLoadResult 含 name/version/state_machine」当前可验证性 = **部分（仅 schema 校验）**（product-debate-record CON 表）。
- **F-3**：命令树现状 = 「admin/sync/extract/checklist/migrate/prompt 全混在一棵 popsicle CLI 树，主路径命令数不可数」；目标 = 每 product ≤ 7 公开命令（product-debate-record 度量表）。
- **F-4**：`guard` 已 **hardcode** 一组类型、**不**通过 trait 扩展（`crates/popsicle-core/src/engine/guard.rs:26`）——影响 state_machine 契约的可扩展性论证（DEV Phase 3 待评）。

### Phase 1 小结

- **共识（草案）**：议题双契约清晰；HC-1~HC-4 四条硬约束已锚定事实基；NFR 草案排序 = 可演进性 > 契约完整性 > 可观测性 > 性能。
- **待用户决定（暂停点 #1）**：见下方问题。
- **悬而未决**：SkillLoadResult 的 `version` 到底是「skill 包语义版本」还是「state_machine schema 版本」——留到 Phase 2/3 方案里展开。

## Phase 2 — 方案发散

> 3 个**实质差异化**方案，差异轴 = ①`version` 语义 ②`state_machine` 载体 ③命令树投影方式。

### 方案 A —「内嵌薄契约」（thin in-process，提案者 ARCH）

- **核心**：`skill load` 同步反序列化 `SkillDef.workflow`（F-1）直接产出内存 `SkillLoadResult`；
  `state_machine` = 内嵌数据结构（就是 workflow 的 states+transitions），`version` = **skill 包 YAML frontmatter 的 semver**。
- **模块边界**：`commands/skill.rs(load) → core::model::SkillDef(serde) → SkillLoadResult(DTO)`。不新增模块。
- **数据流**：`YAML 文件 → serde 反序列化 → 校验状态机闭合性({pending→in_progress→completed/blocked}, HC-2) → 返回`。
- **命令树**：静态扁平，skill-runtime product 固定暴露 `skill load/run` + `pipeline start/stage/status` + `pipeline recover`（=6，满足 F-3 ≤7）。
- **取舍**：实现成本最低（复用现有 serde + DTO 层）；但 version 把「包升级」和「schema 兼容」绑死——加一个 state 字段就得 bump 包版本（可演进性弱）。

### 方案 B —「schema 与包版本解耦 + 名词分组命令树」（registry-backed，提案者 DATA）

- **核心**：把 `state_machine` 的 **schema 单独版本化**，存进已存在的 `registry` 模块（fact: registry/ 有 module/tool/package 注册，15 pub）；
  `SkillLoadResult.version` = **state_machine schema 版本**（非包版本），包版本另存 metadata。
- **模块边界**：`commands/skill.rs(load) → registry::state_machine_schema(version 解析) → core::model::SkillDef → SkillLoadResult`。
- **数据流**：`load → registry 查 schema 版本 → 按该 schema 校验 workflow → 返回(schema_version, name, pkg_version)`。
- **命令树**：**noun-first 分组**——`popsicle skill <load|run>`、`popsicle pipeline <start|stage|status|recover>` 每个名词组 = 一个 product 边界（天然映射 product 划分，F-3）。
- **取舍**：契约完整性最强（schema 独立演进、向后兼容可判定）；但引入 registry 依赖、迁移期要双版本字段，DEV 成本中。

### 方案 C —「能力清单动态投影」（capability-manifest，提案者 DEV）

- **核心**：`skill load` 返回一个 **capability manifest**（name + 双版本{pkg_semver, schema_rev} + state_machine + 暴露的命令谓词集）；
  命令树由 manifest **动态投影**——每个 product 的 ≤7 命令集从 manifest 派生而非硬编码。
- **模块边界**：新增 `engine::capability_projector`；`commands/* → projector(manifest) → 动态命令注册`。
- **数据流**：`load → 构造 manifest → projector 按 manifest 投影出命令树 → CLI 注册`。
- **取舍**：可演进性最高（加命令/加 state 全靠 manifest 驱动，零命令码改动）；但与现状冲突最大——guard 已 hardcode（F-4），动态投影要先把 guard 抽成可配置，技术债与 HC-3（命令字面量逐字兼容）风险都最高。

### Phase 2 小结

- **共识**：三方案在「version 语义」上正交（A=包版本 / B=schema 版本 / C=双版本），决定了下游 `contracts.intent` 形态。
- **关键分歧（预告 Phase 3）**：B vs C 在「是否值得为可演进性引入 registry/projector 复杂度」上对立；A 是保守基线。
- **待用户决定（暂停点 #2）**：见下方问题——决定哪些方案带入 Phase 3 多角色评审。


## Phase 3 — 多角色评审

> 评审 A（内嵌薄契约）vs B（schema 解耦）。评分对齐 Phase 1 已确认 NFR 排序：可演进性 > 契约完整性 > 可观测性 > 性能。

| 视角 | 方案 A（内嵌薄契约） | 方案 B（schema 解耦 + 名词分组） |
|---|---|---|
| **SEC** 威胁模型 | state_machine 内嵌、无独立校验点；损坏/恶意 skill 包可改 transitions 绕过 HC-2。CON-SR-01 停在「部分验证」。**弱** | schema 独立版本化可加签名/版本校验，CON-SR-01 升到「schema+版本」双闭环。威胁面更小。**强（偏好 B）** |
| **PERF** 容量/时延 | load 零额外查询，最快 | load 多一次 registry schema 查询；非热路径（NFR P4），可忽略。**中立，轻偏 A 但不否决 B** |
| **OPS** 可观测/回滚 | 命令扁平、回滚=改 YAML；但加 state 要 bump 包版本 → 升级耦合 | noun-first 命令树按 product 分组、日志更可观测；迁移期需 schema 版本 alias + 双版本兼容窗口。**偏 B（对齐 NFR P3）** |
| **DATA** 一致性/迁移 | version 单轨、无迁移成本，但无法表达「schema 兼容但包不同」 | schema 独立 → 向后兼容可判定，一致性最强；代价 = 维护 pkg↔schema 版本映射表。**强偏好 B** |
| **DEV** 成本/技术债 | 复用 serde+DTO，~0 新模块，最省 | 引入 registry 依赖 + 双版本字段，中成本。cite risk hotspot：`agent` 模块为 commit 热点 top1（26/yr），`registry` 非热点 → 改动相对安全。guard hardcode（F-4）对 A/B 均不阻塞（仅 C 受影响） |

### 关键分歧

- **D-1（契约载体）**：SEC / DATA / OPS 倾向 **B**（契约完整性 + 可观测性，对齐 NFR P2/P3）vs PERF / DEV 轻偏 **A**（成本/时延，但二者是 NFR 最低优先级）。
  → 按已确认 NFR 排序，B 在前三维（可演进性 / 契约完整性 / 可观测性）占优，A 仅在最低维（性能/成本）占优。
- **D-2（version 语义）**：A 的「包版本」语义无法满足「schema 兼容但包升级」场景；B 的「schema 版本」可判定向后兼容——这是 `contracts.intent` 能否表达「向后兼容」的前提。

### Phase 3 小结

- **倾向**：综合 5 角色 + NFR 排序，**B 占优**；A 作为「若本切片要极限压成本」的退路。
- **悬置**：B 的迁移期是否需要在 SkillLoadResult 同时返回 `pkg_version` 和 `schema_version`（双字段过渡），还是只暴露 `schema_version`——影响 contracts.intent 字段数。
- **待用户决定（暂停点 #3）**：见下方问题。


## Phase 4 — 收敛与决策

### ARCH 综合

采用**方案 B（schema 解耦 + noun-first 命令树）**，`SkillLoadResult` 取**双字段过渡**形态
`{name, pkg_version, schema_version, state_machine}`。理由：对齐 Phase 1 已确认 NFR 排序——
B 在可演进性 / 契约完整性 / 可观测性三维占优；双字段满足 HC-3（task chunk 命令字面量逐字兼容）的迁移期需求。

### 角色投票

| 角色 | 票 | 备注 |
|---|---|---|
| SEC | ✅ B | schema 独立校验补齐 CON-SR-01 双闭环 |
| DATA | ✅ B | 向后兼容可判定（B 原提案者）|
| OPS | ✅ B | noun-first 可观测 + 双字段过渡可回滚 |
| PERF | 接受（不否决）| 多一次 registry 查询，非热路径，NFR P4 |
| DEV | 接受 B | registry 非 commit 热点，成本可控；guard hardcode 不阻塞 |

### 用户最终决策

**选 B + 双字段 `{pkg_version, schema_version}`**（用户暂停点 #3 确认）。
该决策**与多数角色意见一致**（未覆盖多数），无需冷静期。

### 声明 → intent 层 / ADR-CADR 标注（IDD 纪律 2/3）

| # | 核心声明（现在时）| intent 层 | ADR / CADR | 关联 |
|---|---|---|---|---|
| C-1 | `popsicle skill load` 返回 `SkillLoadResult{name, pkg_version, schema_version, state_machine}` | `contracts.intent` | **ADR-002 候选** | PRD §6 row7；CON-SR-01 |
| C-2 | `state_machine` 只表达且仅表达 `{pending → in_progress → completed/blocked}` 转移 | `invariants.intent` | ADR-002 候选 | HC-2；PRD §6 row5 |
| C-3 | `schema_version` 独立于 `pkg_version`，向后兼容时 `schema_version` 不变 | `contracts.intent` | **ADR-002 候选** | D-2 |
| C-4 | 命令树按 noun-first 分组，skill-runtime product 暴露 ≤ 7 个公开命令 | （不进 .intent，写 ARCHITECTURE.md）| **ADR-002 候选** | F-3；ADR-001 保护 |
| C-5 | `skill load` 时延目标（能力边界 D2）| **不进 .intent** | 写 RFC 质量属性目标，benchmark 守护 | NFR P4 |

### CADR 判定

命令树重组属 **internal API 调整**（ADR-001 已确立 intent-coder 是内部消费者，重组 0 外部破坏，PDR-001 §Phase3 ENGLD-Q1 已验证），
**不触 charter 四条铁律 / Layer Map** → 全部走 **ADR-002**，**无需 CADR**。

### Phase 4 小结

- **共识**：B + 双字段，5 角色一致或接受。
- **下一步**：rfc-writer 据 C-1~C-5 落 RFC + contracts 种子 + ADR-002 骨架；intent-spec-writer 收紧 C-2 invariant。


## Decision

skill-runtime 采用方案 B：`popsicle skill load` 返回 `SkillLoadResult{name, pkg_version, schema_version, state_machine}`；`schema_version` 独立于包版本并在向后兼容时保持不变；`state_machine` 仅表达 `{pending → in_progress → completed/blocked}`；命令树按 noun-first 分组，每 product 暴露 ≤ 7 个公开命令。全部决策走 ADR-002，无需 CADR。

## 关键分歧

- **D-1（契约载体）**：SEC/DATA/OPS 偏 B（契约完整性+可观测性）vs PERF/DEV 轻偏 A（成本/时延）→ 按 NFR 排序 B 占优，收敛到 B。
- **D-2（version 语义）**：A「包版本」无法表达「schema 兼容但包升级」vs B「schema 版本」可判定向后兼容 → 选 B 双字段过渡。

## 用户决策点

- [x] 暂停点 #1（Phase 1）：NFR 排序「可演进性>契约完整性>可观测性>性能」—— 用户认可。
- [x] 暂停点 #2（Phase 2）：评审范围 A+B，C 列未来演进附注 —— 用户认可。
- [x] 暂停点 #3（Phase 3）：选 B + 双字段 {pkg_version, schema_version} —— 用户拍板。
- [x] 暂停点 #4（Phase 4）：最终决策 B + 双字段，与多数角色一致，无需冷静期。

## 下游接驳建议

- rfc-writer：把本纪要 + rfc-draft 打磨成正式 RFC + `contracts.intent` 种子 + ADR-002 骨架。
- 需要 CADR 的条目（如有）：先走 charter 修订。

## Output Checklist

- [x] Phase 1-4 小结齐全
- [x] 关键分歧与各方立场已记录
- [x] 用户决策点已显式记录（含覆盖情况）
- [x] 每个数字/模块名引用可追溯到事实基
- [x] Topic 与另两份 artifact 一致

## Setup Checklist

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] 技术议题已用一句话表达（来自 prd § Intent Mapping 的 [ADR 候选] 行）
- [x] 边界已绑定（skill-runtime / SkillLoadResult + 命令树）
- [x] 事实基状态已记录（已读 api-contracts + risk）
- [x] 技术角色阵容确定（ARCH + DATA + DEV + SEC + PERF + OPS）
- [x] 用户置信度已设置
- [x] 已展示 setup 摘要并取得 `start` 确认

## Phase Coverage

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] Phase 1：技术问题 + NFR 优先级 + 硬约束已明确
- [x] Phase 2：3 个差异化架构方案，各含模块边界/数据流
- [x] Phase 3：全部角色（含 SEC / PERF / OPS）已评审
- [x] Phase 4：收敛到方案 B + 双字段 + 用户决策 + 每个声明标了 intent 层/ADR
- [x] 至少 4 个用户交互暂停点
