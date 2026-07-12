# Doc Architecture Charter — {repo_name}

> **Promoted to**: `docs/CHARTER.md`，骨架铺好后
> **Status**: 基础性——修改本 charter 需要它自己的 ADR

本 charter 定义本仓库**文档如何组织、如何编写、如何变更**的不可妥协规则。每个贡献者——人或 AI agent——在动 `docs/`、`products/*/PRODUCT.md`、`products/*/ARCHITECTURE.md` 或任何决策文件之前，都要先读这份。

---

## 文档体系的四条铁律

1. **活文档没有版本号** —— 只有 `Last-Updated` 和 `Last-Decision-Ref`。它们永远代表「现在」。过期内容就地修正；不写历史叙事。
2. **决策档案只追加** —— ADR/PDR 文件在 `Status: Accepted` 之后永不修改。错的决策通过写一份**新**决策（标注 Supersedes）来纠正。
3. **每次活文档编辑必须引用一个 Decision ID**（除错别字 / 链接 / 措辞修复外）。改 `PRODUCT.md` 或 `ARCHITECTURE.md` 的 PR 由 CI 强制。
4. **一次变更可能波及多份活文档** —— 触发它的 ADR/PDR 的 `Consequences` 章节**必须**列出所有被它强制更新的活文档段落；PR 必须在一次提交里全部更新。

---

## Layer Map

文档体系分 7 层，由**它们约束什么**和**多久变一次**来区分。

| Layer | 文档 | 约束对象 | 变更频率 | Owner | intent-lang |
|---|---|---|---|---|---|
| L0 | `docs/CHARTER.md` | 产品存在的理由；绝对底线 | 一年级 | 创始人 / 架构委员会 | 仅自然语言 |
| L1 | `docs/invariants/*.intent`, `products/*/intents/invariants.intent` | 领域自然律 | 一季度 | PM + 架构师 | ✅ 核心 |
| L2 | `products/*/PRODUCT.md` + `acceptance.intent` | 用户可见行为 | 一月 | PM | ✅ acceptance 片段 |
| L3 | `products/*/decisions/{adr,pdr}/*.md` | 为什么这么选 | 决策时定，永不修改 | 架构师 / PM | ❌ |
| L4 | `products/*/ARCHITECTURE.md` + `contracts.intent` | 模块如何实现 | 提案期可变，落地即冻结 | 技术 lead | ✅ contracts 片段 |
| L5 | `migration/slices/*.md`、变更 PR | 一次具体变更 | 一次性 | 开发者 | ✅ 以 diff 形式 |
| L6 | `crates/`、`src/`、`tests/` | 机器行为 | 持续 | AI + 人 | — |

> 必须避免的滥用：一份 "PRD" 把 L1 invariant、L2 product spec、L4 contract 混在一份文档里，是大项目文档腐烂 #1 的成因。每一层变更频率不同；混层强行把所有层拉到最高频率。

---

## Per-Product 4-Piece Set

每一个 product（在 `products/<name>/` 之下）正好有 4 类制品。每一类有定义好的 audience、定义好的更新方式、**以及定义好的「不准在这里写什么」清单**。

| 制品 | 视角 | Audience | 更新方式 | 不准在这里写什么 |
|---|---|---|---|---|
| `PRODUCT.md` | 商业 | PM、销售、客户成功、AI | 直接编辑（小改）或 PDR 触发（大改）| 实现细节、技术选型理由 |
| `ARCHITECTURE.md` | 实现 | 工程师、AI | 直接编辑（小改）或 ADR 触发（大改）| 商业策略、定价、客户分层 |
| `intents/*.intent` | 形式化 | LLM、Z3、CI | 跟随 PRODUCT.md / ARCHITECTURE.md 的变化 | 自然语言叙述、理由（这些放进 PDR/ADR）|
| `decisions/{adr,pdr}/*` | 历史 | 任何追溯决策的人 | 只追加 | "当前状态"描述（放活文档里）|

---

## 三层 Intent 体系

| 作用域 | 路径 | Owner | 触发 |
|---|---|---|---|
| **跨 product（全局）** | `docs/invariants/*.intent` | charter 级别的决定 | "宪法级" PDR（罕见）|
| **单个 product** | `products/<name>/intents/invariants.intent` | product team | product 内部的 PDR |
| **单个 feature / acceptance** | `products/<name>/intents/acceptance.intent` | product team | product 内部的 PDR |
| **单个 contract / 模块 API** | `products/<name>/intents/contracts.intent` | 架构师 | product 内部的 ADR |

> 规则：每份 PDR/ADR 的 `Intent Impact` 章节必须指出它修改的是哪一层 intent。CI 拒绝缺这一项的决策。

---

## 提案 & 决策生命周期

### 技术侧（RFC → ADR）

```
┌─────────┐    accept    ┌──────────┐
│Proposed │ ───────────► │ Accepted │ （不可变）
└─────────┘              └────┬─────┘
     │                        │ supersede
     │ reject                 ▼
     ▼                   ┌─────────────┐
┌──────────┐             │ Superseded  │
│ Rejected │             └─────────────┘
└──────────┘
```

### Product 侧（PRFC → PDR）

```
┌──────────┐  ready   ┌──────────┐  accept  ┌──────────┐
│Exploring │ ───────► │Proposed  │ ───────► │ Accepted │
└──────────┘          └──────────┘          └────┬─────┘
     │ deadline             │                    │ supersede
     │ exceeded             │ reject             ▼
     ▼                      ▼              ┌─────────────┐
┌──────────┐          ┌──────────┐         │ Superseded  │
│ Rejected │          │ Rejected │         └─────────────┘
└──────────┘          └──────────┘
```

> **为什么 product 侧多了 `Exploring` 而技术侧没有**：技术 RFC 是作者已经知道方向、在备选方案之间选择时提交的。产品提案常常起源于「我们到底要不要做这件事」，需要一段研究期（用户访谈、数据分析、A/B 测试）才能写出真正的提案。`Exploring` 带一个 `Decision-Deadline:` 字段——到期必须进入 `Proposed` 或 `Rejected`。这避免两种失败模式：(a) 强行让胚胎期想法以 `Proposed` 起步，污染提案队列；(b) 让探索性工作完全在档案之外发生，丢失审计轨迹。

---

## RFC 持久归宿（feedback #15）

RFC 是 Layer Map 的 **L4 一等层**，**不能**只活在 `.popsicle/artifacts/<run>/`（流水线临时产物）。spec 链跑完，RFC 必须落到产品树的持久位置，否则 ADR 的 `Related RFC` 会指向不存在的文件、活文档失去可发现性。

规则：
1. **落盘路径**：`rfc-writer` / spec 链收尾时把 RFC 落成
   `products/<p>/proposals/<lifecycle>/RFC-NNNN-{slug}.md`——`proposed/` 阶段在 `proposed/`，
   关联 ADR Accepted 后移到 `accepted/` **原地保留**（不再 move，避免破坏链接与 git 历史）。
2. **编号**：`RFC-NNNN` 按 `products/<p>/proposals/` 现有最大号 +1，product 内递增。
3. **双向回链**：RFC frontmatter `realized_by_adr` ↔ ADR 的 `Related RFC`，用**相对路径**互指
   （链接修复属铁律 3 豁免）。
4. **溯源**：RFC frontmatter 带 `source_artifact`（临时产物出处）与 `legacy_pin`（锚定的 legacy commit）。

---

## Decision 作用域：product 级 vs 仓库级（feedback #16）

决策不都是 product-scoped。共享一个 legacy Cargo workspace 的 monorepo 里，"沿用包结构""gRPC 契约约定""共享 crate 归属"等是**仓库级**决策，塞进某个 product 是概念错位。

| 作用域 | 路径 | 编号 | 用于 |
|---|---|---|---|
| **product 级** | `products/<p>/decisions/{adr,pdr}/` | `ADR-NNNN` / `PDR-NNNN`（各 product 从 1 起）| 单 product 的技术/产品决策 |
| **仓库级/跨产品** | `docs/decisions/{adr,pdr}/` | `ADR-G-NNNN` / `PDR-G-NNNN` | 跨多个 product、共享基础设施的决策 |
| **Charter 修订** | `docs/decisions/cadr/` | `CADR-NNNN` | 修改本 charter 自身 |

> 跨产品决策的 `Consequences` **必须**逐一列出受影响的 product；各受影响 `PRODUCT.md` / `ARCHITECTURE.md` 反链回该 `ADR-G-`。

---

## proposals/ 与 decisions/ 的真相源（feedback #17）

两个目录同时存在时，**谁是真相源**必须无歧义，否则跨目录 move 文件会破坏链接与 git 历史可读性。

规则：
1. **Accepted 后 `decisions/` 为准**。`proposals/` 里的对应项**原地加状态标记**（`Status: Accepted → 见 decisions/adr/ADR-NNNN`），**不 move、不删**——保留探索轨迹与 git 历史。
2. **RFC 是唯一例外**：RFC 作为 L4 活的技术设计，`accepted/` 原地保留（见上一节），由 ADR 固化"决策"部分；RFC 记"怎么设计"，ADR 记"定了什么、为什么"。
3. **迁移期收敛**：对"冻结 / 等价保留"这类无真实方案空间的决策，**直接写 ADR**，不走 RFC→ADR 双文档自我复述，避免 RFC↔ADR↔ARCHITECTURE 三处重复同一套模块边界导致 drift。

---

## 活文档中的禁用短语

如果某份活文档（`PRODUCT.md`、`ARCHITECTURE.md`）含有以下任一短语，PR 评审不通过：

- "We originally used X..."
- "Previously, ..."
- "曾经..." / "之前..."
- "We migrated from X to Y because..."
- "Was: X. Now: Y."

这些短语描述**历史**，而历史是 ADR / PDR / `git log` 的活。活文档**只用现在时**。

✅ `"Replication: Multi-Paxos [ADR-0015]"`
❌ `"We initially used Raft, then changed to Multi-Paxos in 2026 to reduce cross-region latency"`

---

## 反模式（以及如何检测）

| 反模式 | 检测方式 | 缓解 |
|---|---|---|
| **活文档当 wiki** —— 任何人都加章节，结构腐烂 | PR 改 PRODUCT.md / ARCHITECTURE.md 时做模板符合性检查 | 每份活文档有固定的顶层大纲；加新章节需要 charter 修订 |
| **ADR/PDR 当日记** —— 日常 standup 笔记被记成决策 | 评审者在 merge 前问「这件事一年后还重要吗」 | `decisions/` 目录的 PR 需要 2 个评审者；琐碎信息进 issue / PR description |
| **每个微服务一个 product** —— 一个 product 被切成 20 个 `products/` 目录 | 仓库级检查：一个 product 有 > 5 个子 product 就要拆 | Product 边界画在客户能识别的位置，不是代码模块边界 |
| **Roadmap 当愿望清单** —— "Committed Roadmap" 列了一堆没 PDR 的想法 | grep `PRODUCT.md` 的 Roadmap 章节，找没有 `[PDR-XXXX]` 的条目 | 没有 Decision ID 的 roadmap 条目无效；想法住在 `proposals/exploring/` |
| **Doc–code drift** —— 活文档变陈旧 | CI 检查 `Last-Updated` 字段；N 天后告警；改 `crates/<X>/` 的 PR 必须确认 `products/<X>/ARCHITECTURE.md` 是否要更新 | linter；PR 模板里显式提问 |

---

## 迁移序列（把本 charter 套到一份遗留项目上）

> 复刻来源讨论 §七 的序列；不要重发明。

| 步骤 | 动作 | 何时完成 |
|---|---|---|
| 0 | 为每个 product 铺空的 4 件套骨架 | 本 skill（`project-init`）|
| 1 | 为每份 `PRODUCT.md` + `ARCHITECTURE.md` 写当前快照（含 `[TBD: needs archaeology]` 标记）| 首切片的 PM / 技术 lead，约 2-3 周 |
| 2 | 懒回填 ADR/PDR —— 只在撞到 `[TBD]` 或新决策触及老地盘时回填；`Status: Reconstructed` | 持续 |
| 3 | 从某个固定的 cutover date 起：每次活文档 PR 都需要 Decision ID | CI 强制 |
| 4 | 加 intent 层 —— 每份 PDR/ADR 长出 `Intent Impact` 章节 | charter 落地后，与 step 3 并行 |
| 5 | acceptance/contracts intent 按 product 长起来；首切片产出 playbook | 持续 |

---

## Charter 自指

本 charter 本身就是一份活文档。修改它需要一类特殊的决策：**CADR**（Charter Amendment Decision Record），位于 `docs/decisions/cadr/`。CADR 与其它决策文件一样受四条铁律约束。
