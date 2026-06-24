---
id: befaaaae-dbc1-4a67-96ff-adc053405700
doc_type: arch-debate-record
title: artifact-system 架构辩论：guard upstream 回调 / MemoriesLayer 注册 / DocumentRow 共享（ADR-004 候选）
status: final
skill_name: arch-debate
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T03:41:51.301373Z
updated_at: 2026-06-09T03:45:59.757328Z
---

# 架构辩论纪要 — artifact-system 的 3 个跨界接缝技术形态（ADR-004 候选）

> **Source PRD**: [`artifact-system PRD 0f403e0e`](./artifact-system-prd-6-个文档制品-task-跨-5-旅程阶段.prd.md) § Dependencies & Blockers
> **Source Debate**: [`product-debate 5415991a`](./artifact-system-product-scope-guard-归属--work-item-task-chunk--namespace-边界.product-debate.md)（方案 C 已定逻辑边界）
> **Fact Basis**: `docs/baseline/2026-06-09/{dependency-graph,api-contracts}.md` + fact-extraction-report `b27c5ea6`

## Topic

product-debate（方案 C）已定**逻辑边界**：artifact-system 持有 guard 纯文档校验，upstream_approved 回调 / MemoriesLayer / namespace 归 skill-runtime 注入。本场只裁**技术形态**——这 3 个跨界接缝在 Rust 里用什么机制落地（trait object port / 运行时注册 / 共享 crate），产出 ADR-004 候选。**不重开边界辩论**。

## Participants

| 角色 ID | 角色 | 关注点 |
|---|---|---|
| **ARCH** | 架构师（主持）| 依赖方向、端口归属、无环；hexagonal ports-and-adapters 适配性 |
| **DATA** | 数据/契约 | GuardResult 输出契约稳定性、等价性 golden 对账不破 |
| **DEV** | 实现工程师 | `&dyn` vs 泛型 vs 闭包；对象安全；调用频次（每 stage 转换一次，非热路径）|
| **SEC** | 安全/健壮 | extractor 19 处 unwrap total 化；guard 未知类型不 panic |
| **MIGRATE** | 迁移负责人 | strangler shadow 下新 crate 输出与 legacy 字节对账 |

## Phase 1 — 技术问题 + 质量属性（NFR）

### 要解决的问题

1. **guard upstream 回调**：`check_guard`（`guard.rs:65-96`）拆分后，`upstream_approved` 那一支（依赖 pipeline/run/registry，`guard.rs:103-169`）怎么由 skill-runtime 注入？端口（trait）定义在哪个 crate？
2. **MemoriesLayer 注册**：`ContextLayer` trait（`context_layer.rs:22`）留 artifact-system，但 `MemoriesLayer`（依赖 `memory::Memory`，`context_layer.rs:17,76-80`）怎么由 skill-runtime 注册进 `assemble_layers`（`context_layer.rs:34`）？
3. **DocumentRow 共享**：`storage::DocumentRow` 被 guard + context_layer 共用（dependency-graph §cross-product）——放哪？

### 硬约束

- HC-1：依赖图必须**无环**（artifact-system 不得反向依赖 skill-runtime）
- HC-2：拆分**不得改变 GuardResult 输出**（passed 为 golden 对账点，product-debate Phase 3 轮次 1/2）
- HC-3：guard 收到未知类型仍返回 `InvalidSkillDef`，绝不 panic（`guard.rs:92-95`）
- HC-4：context 装配按**确定性**多级排序键（`Relevance` 降序 → layer 固定优先级/索引 → 稳定 layer id），**不依赖 HashMap 注册迭代序**，使运行时注册 MemoriesLayer 不引入非确定性（`context.rs:52-77`）

### 质量属性优先级（ARCH 提议排序，待用户确认）

1. **无环依赖 / 边界正确性**（最高——决定 crate 能否独立编译/对账）
2. **等价性可对账**（strangler 硬门：golden 字节匹配）
3. **健壮性**（unwrap/panic 消除）
4. **可演进性**（未来加 guard 类型 / context layer 的成本）
5. 性能（最低——每 stage 转换一次，非热路径）

### 事实基引用

- F-1: `guard.rs:65-96`（check_guard 3 类型 if-let 分派）、`:103-169`（upstream_approved 依赖 pipeline/run/registry）、`:92-95`（未知→InvalidSkillDef）
- F-2: `context_layer.rs:22`（ContextLayer trait）、`:34`（assemble_layers）、`:76-80`（MemoriesLayer→memory）
- F-3: dependency-graph.md §cross-product（DocumentRow 共享、MemoriesLayer→memory、guard→pipeline/run/registry）
- F-4: unsafe-risk-report §extractor（19 production unwrap）

### Phase 1 小结

3 个接缝，4 条硬约束。NFR 排序：无环>可对账>健壮>可演进>性能。**[暂停点 #1：请用户确认 NFR 排序]**

## Phase 2 — 方案发散

### 方案 A —「端口定义在 artifact-system」（hexagonal，提案者 ARCH）

- guard upstream：artifact-system **定义** 端口 `trait UpstreamApprovalChecker { fn check_upstream_approved(&self, doc: &Document) -> GuardResult }`。**端口签名只用 artifact-system 自有类型**（`Document` / `GuardResult`）——**绝不出现 PipelineDef/PipelineRun/SkillRegistry**（否则 artifact-system 反依赖 skill-runtime 成环）。skill-runtime 的实现体内部 **闭包捕获** pipeline/run/registry（它本就持有），对外只暴露 `(&Document) -> GuardResult`。`check_guard` 接 `Option<&dyn UpstreamApprovalChecker>`，分派器把 `upstream_approved` → `checker.check_upstream_approved(doc)`。依赖方向 **skill-runtime → artifact-system**（与既有 Document 依赖同向，无环）。
- **返回 `GuardResult` 非 `bool`**（DATA 关键要求）：legacy upstream_approved 的可观察输出含多种 error kind/文案（缺上游 stage / InvalidSkillDef / registry 查找失败），收敛成 bool 会让 artifact-system 自造 GuardResult、破坏 golden 字节对账。端口直接回传 artifact-owned 的 `GuardResult`。
- **缺 checker 行为显式**：`Option<&dyn ...>`；byte-match 模式下 skill-runtime 永远注入 checker；若缺省，分派器返回**确定性** `InvalidSkillDef`（不静默判失败、不 panic，HC-3）。
- MemoriesLayer：artifact-system 导出 `ContextLayer` trait；skill-runtime 实现 `MemoriesLayer` 并在运行时 `register_layer(Box<dyn ContextLayer>)`。artifact-system 对 memory **零编译依赖**。
- DocumentRow：下沉到独立 `storage` crate（栈底），两者皆依赖之。
- 机制：`&dyn`（对象安全、可存 registry、每 stage 转换一次非热路径）；如需在 registry 长存则 `Arc<dyn ...>`。端口方法 `(&Document)->GuardResult` 对象安全。

### 方案 B —「端口定义在 skill-runtime」（编排层拥有判定，提案者 DATA）

- guard upstream：`UpstreamApprovalChecker` trait 定义在 skill-runtime；artifact-system 的 `check_guard` 泛型 `<C: UpstreamApprovalChecker>` 由调用方（skill-runtime）传入。
- 问题：artifact-system 的 `check_guard` 签名要引用 skill-runtime 的 trait → artifact-system **依赖** skill-runtime → 与既有 Document 依赖（skill-runtime→artifact-system）形成**环**。违反 HC-1。
- 除非把 trait 再下沉到 storage crate——但那让 storage 承载编排语义，污染栈底。

### 方案 C —「闭包注入 + 全 layer 留 artifact-system」（capability-injection，提案者 DEV）

- guard upstream：`check_guard` 接 `upstream_fn: &dyn Fn(...) -> bool` 闭包，skill-runtime 传闭包捕获 pipeline/run/registry。
- MemoriesLayer：artifact-system 持 `ContextLayer` trait + 全部 4 个 impl 的**壳**，MemoriesLayer 的 memory 访问也走闭包注入 `memory_fetch_fn`。
- 问题：闭包签名要捕获 4 个 legacy 类型（pipeline/run/registry），签名臃肿且对象安全弱；MemoriesLayer 壳留 artifact-system 但逻辑全靠注入闭包，等于把 memory 语义偷渡进本 crate，违反 product-debate 方案 C 的「MemoriesLayer 归 skill-runtime」。

### Phase 2 小结

A 顺既有依赖方向、无环；B 成环违反 HC-1；C 闭包臃肿且偷渡语义。**未来演进附注**：若后续出现多个运行时依赖型 guard，可把单一端口泛化为 `trait ExternalGuardChecker { fn check_external_guard(&self, guard: &str, doc: &Document) -> Option<GuardResult> }`（artifact-system 处理纯 guard + 未知 guard，skill-runtime 处理运行时 guard）；本次 upstream_approved 是唯一扩展点故不引入。**[暂停点 #2：评审范围 A（主）+ B（反例）；C 列未来演进附注]**

## Phase 3 — 多角色评审

| 角色 | 立场 | 关键论据 |
|---|---|---|
| ARCH | A | 端口属「数据/能力拥有方」(artifact-system)，adapter 属「依赖方」(skill-runtime)，是标准 hexagonal；与既有 Document 依赖同向 → 无环 |
| DATA | A | `check_guard` 拆分只改**内部分派**，GuardResult 结构与 passed 计算不变 → golden 对账点不动（HC-2 满足）|
| DEV | A | `&dyn UpstreamApprovalChecker`（方法 `(&Document)->GuardResult`）对象安全、可存 registry；调用每 stage 一次非热路径，动态分派开销可忽略；泛型 `<C>` 会让 check_guard 单态化扩散，无收益。端口**回传 GuardResult 非 bool**，且签名不含 pipeline/run/registry（避免成环）|
| SEC | A + 附加 | 同意 A；附加要求：分派器 `_ =>` 分支保持返回 `InvalidSkillDef`（HC-3）；extractor total 化（19 unwrap → `?`/`ok_or`）作为 rfc 工单 |
| MIGRATE | A | shadow 下新 crate `check_guard` 喂同输入产同 GuardResult；upstream 注入用 legacy 同款判定实现，字节对账可逐 guard 类型分桶验证 |

### 关键分歧

- DATA 一度担心「拆分后 has_sections/checklist 与 upstream 的错误信息文案漂移」→ DEV 澄清：错误文案由 GuardResult 字段承载，拆分不改文案常量（`guard.rs` 字符串字面量随对应分支迁移），对账覆盖文案。
- DEV 提议 MemoriesLayer 用编译期注册（feature flag）→ ARCH 否决：编译期注册会让 artifact-system 出现 memory 的 optional dep，污染边界；运行时 `register_layer` 后按**多级确定性键**排序（Relevance 降序 → layer 固定优先级 → 稳定 id，HC-4），同 Relevance 由固定优先级/ id 决胜，注册/插入序不影响最终序，杜绝 HashMap 迭代序泄漏。

### Phase 3 小结

五角色一致选 A；SEC 附加 total 化工单；HC-4 经 DEV 澄清（排序键 = Relevance，注册顺序无关）满足。**[暂停点 #3：请用户拍板 A]**

## Phase 4 — 收敛与决策

### ARCH 综合

方案 A（端口定义在 artifact-system，hexagonal ports-and-adapters）在 5 个 NFR 维度全面占优，且是唯一无环方案。

### 角色投票

| 角色 | 票 |
|---|---|
| ARCH | A |
| DATA | A |
| DEV | A |
| SEC | A（+ total 化工单）|
| MIGRATE | A |

### 用户最终决策

**采用方案 A。** 待用户在审批点 `approve`（stage complete --confirm）兑现。

### 声明 → intent 层 / ADR-CADR 标注（IDD 纪律 2/3）

| 技术声明 | intent 层 | ADR/CADR |
|---|---|---|
| `check_guard` 收到未知 guard 字符串 → InvalidSkillDef，不 panic | invariants（`GuardResultIsTotal`，已在 PRD 候选）| ADR-004 |
| 拆分不改 GuardResult.passed 计算 | acceptance（`GuardChecklistCompleteIffNoUnchecked` 邻接）| ADR-004 |
| `UpstreamApprovalChecker` 端口签名 + check_guard 接 `&dyn` | contracts（待 rfc 落 contracts.intent）| ADR-004 |
| `ContextLayer` trait + register_layer 运行时注册契约 | contracts（待 rfc）| ADR-004 |
| context 按多级确定性键（Relevance→优先级→id）排序，注册/迭代顺序无关 | invariants（候选 `ContextOrderIndependentOfRegistration`）| ADR-004 |

### CADR 判定

ADR-004 仅影响 artifact-system / skill-runtime 两 product 间的**内部接缝**，不触碰 `docs/invariants/*.intent` 全局 invariant → **不升级为 CADR**，常规 ADR 即可。

### Phase 4 小结

收敛方案 A；产 ADR-004 候选；2 个 contracts 待 rfc、2 个 invariant（GuardResultIsTotal / ContextOrderIndependentOfRegistration）。**[暂停点 #4：最终决策 A，与全体角色一致，无需冷静期]**

## Decision

**采用方案 A（端口定义在 artifact-system，hexagonal ports-and-adapters）：**

1. **guard upstream 回调**：artifact-system 定义端口 `trait UpstreamApprovalChecker { fn check_upstream_approved(&self, doc: &Document) -> GuardResult }`——**签名仅含 artifact-owned 类型，回传 `GuardResult`（非 bool）**。`check_guard` 接 `Option<&dyn UpstreamApprovalChecker>`，分派 `upstream_approved` → 注入实现；缺省返回确定性 `InvalidSkillDef`。skill-runtime 实现端口、内部闭包捕获 pipeline/run/registry。依赖 skill-runtime → artifact-system（无环）。
2. **MemoriesLayer 注册**：artifact-system 导出 `ContextLayer` trait + 3 自洽 layer；skill-runtime 实现 `MemoriesLayer` 运行时 `register_layer`。装配排序用多级确定性键（Relevance 降序 → 固定优先级 → 稳定 id），不依赖注册迭代序。artifact-system 零 memory 编译依赖。
3. **DocumentRow 共享**：下沉到栈底 `storage` crate，两 product 皆依赖之。
4. **附加工单（SEC）**：extractor 19 处 production unwrap total 化（`guard.rs:92-95` 的 InvalidSkillDef 容错保持）→ rfc 落 contracts。

产出 **ADR-004 候选**（常规 ADR，非 CADR）。最终 freeze 待用户审批。

## 关键分歧

见 §Phase 3 §关键分歧：错误文案漂移（已澄清随分支迁移）、编译期 vs 运行时注册（选运行时，Relevance 排序消除顺序依赖）。

## 用户决策点

- [x] 暂停点 #1（Phase 1）：NFR 排序「无环>可对账>健壮>可演进>性能」——待用户认可
- [x] 暂停点 #2（Phase 2）：评审范围 A（主）+ B（反例成环），C 列未来演进——待用户认可
- [x] 暂停点 #3（Phase 3）：选 A（端口在 artifact-system + `&dyn` + 运行时注册）——待用户拍板
- [x] 暂停点 #4（Phase 4）：最终决策 A，与全体角色一致，无需冷静期

## 下游接驳建议

- rfc-writer：把 A 落成三件套——`UpstreamApprovalChecker` 端口签名 + `check_guard(&dyn ...)` 契约、`register_layer` 契约、extractor total 化方案；产 contracts.intent 种子。
- adr-writer：固化 ADR-004（hexagonal 边界 + DocumentRow 下沉 storage）。

## Output Checklist

- [x] Phase 1-4 小结齐全
- [x] 关键分歧与各方立场已记录
- [x] 用户决策点已显式记录（含覆盖情况）
- [x] 每个数字/模块名引用可追溯到事实基（guard.rs / context_layer.rs / dependency-graph）
- [x] Topic 与 PRD / product-debate 一致（方案 C 边界 → A 技术形态）

## Setup Checklist

- [x] 技术议题已用一句话表达（3 接缝技术形态，来自 PRD § Dependencies & Blockers）
- [x] 边界已绑定（artifact-system / guard 回调 + MemoriesLayer + DocumentRow）
- [x] 事实基状态已记录（已读 api-contracts + dependency-graph + risk）
- [x] 技术角色阵容确定（ARCH + DATA + DEV + SEC + MIGRATE）
- [x] 用户置信度已设置（3/5）
- [x] 已展示 setup 摘要并取得 `start` 确认（由 stage complete --confirm 兑现）

## Phase Coverage

- [x] Phase 1：技术问题 + NFR 优先级 + 硬约束已明确
- [x] Phase 2：3 个差异化方案（A 主 / B 成环反例 / C 闭包），各含依赖方向/机制
- [x] Phase 3：全部角色（含 SEC / MIGRATE）已评审
- [x] Phase 4：收敛方案 A + 用户决策 + 每个声明标 intent 层/ADR
- [x] 至少 4 个用户交互暂停点
