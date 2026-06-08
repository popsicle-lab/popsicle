# product-debate —— 编写指南

> 读者：跑 product-debate skill 的 AI agent（也就是你）。开工前**先**读完本指南、
> `references/debate-phases.md`、`references/confidence-modes.md`、`references/default-roles.md`，
> **以及**（如果存在）`fact-extraction-report.md` 和 `project-init-plan.md`。

## 任务

为一个 product slice 跑一场**有结构、有事实基、有审计轨迹**的多角色产品辩论，
在 PRD 落地之前充分暴露盲点、探索方案空间。

产出的 `prd-draft.md` 直接喂给 `prd-writer` skill 做质量评分和最终成稿；
`debate-record.md` 永久存档作为「为什么这样决定」的证据。

## 核心理念：Simulate, don't orchestrate

**在单次对话中模拟多视角切换**，而不是编排多个独立 agent。这有两个含义：

1. **你**就是 PM + UXR + GROWTH + ENGLD + BIZ —— 通过显式的角色标签切换视角，
   而不是去调用别的 agent。
2. 视角切换的质量取决于角色定义的清晰度——读 `references/default-roles.md`，
   每个角色的「典型质疑」和「决策偏好」是切角色时的提词。

## 与参考实现的差异（IDD 适配）

本 skill 大体复刻 popsicle-spec-development 的 `product-debate`，但在 intent-coder
的 IDD 工作流里有 4 个关键差异：

### 差异 1：辩论必须基于 fact-extractor 的证据基

普通产品讨论里说「这块代码量很大」是允许的；IDD 工作流里**不允许**。每个数字、
LoC、模块名、依赖关系都必须能 cite 回 `fact-extraction-report` 的某个章节。

**为什么**：IDD 的核心承诺是「下游每一份产物都能追溯到一个 pinned fact」。如果
辩论本身在凭印象，整条链的可信度就崩了。

**怎么做**：辩论开始前读 fact-extraction-report，记住三件事——
- **Bounded Contexts 表**：限定方案的可达边界（不能讨论「让 product A 直接操作
  product B 的内部状态」这种违反边界的方案）
- **Risk Hotspots 表**：ENGLD 角色的论据库（「方案 X 会触及 hotspot Y」）
- **Domain Glossary**：辩论中所有专有名词的唯一来源（不允许造新词）

如果 fact-extraction-report 不存在：**警告用户**「辩论将基于自然语言推断，可信
度下降」，但允许继续——这场辩论的产物在 prd-writer 阶段会被打低分。

### 差异 2：辩论锁定到一个 product slice

参考实现里讨论范围可以是「整个产品」。本 skill 强制**一次辩论一个 product**，
对应 `project-init-plan.md` 的 Product Inventory 中的一项。

**为什么**：
- 文档体系按 product 分目录（`products/<name>/PRODUCT.md`），辩论产物的归属必须
  清晰
- 跨 product 的话题通常是「在 product 边界要画在哪里」——这是 charter 级问题，
  应该走 CADR 而不是产品辩论

**怎么做**：
- 开场绑定 `target_product = <某个 product>`
- 所有 Phase 中的「实现」「成本」「风险」估算限定在这个 product 内
- 真要跨 product 时，显式标注「本辩论涉及 product A 和 B，产出的 PRD 草稿会拆
  成两份」

### 差异 3：方案的每个核心声明必须能落进某个 intent 层

辩论 Phase 4 收敛时，PM 不仅要说「我们选方案 X」，还要把方案 X 的每个核心声明
**显式分配**到一个 intent 层：

| 声明形态 | 目标 intent 层 | 例 |
|---|---|---|
| 不变式（永远成立的领域规律）| `intents/invariants.intent` | 「同一个 verification 任务不能被并发跑两次」|
| 用户可见行为（acceptance）| `intents/acceptance.intent` | 「用户提交 .intent 文件后 5 秒内必须收到 PASS / FAIL」|
| 模块间契约 | `intents/contracts.intent` | 「verifier 对 cli 暴露 `Verify(spec) -> Result` 接口」|
| 商业/UX 决策（**非** intent 层）| 留在 PRODUCT.md 自然语言部分 | 「定价：免费 5 次 / 月，付费无限」|

**为什么这么做**：下游 prd-writer 需要这个分类才能产 `acceptance.intent` 种子；
intent-consistency-check 需要这个分类才知道哪些声明要进 Z3 验证。把这一步**前置
到辩论里**，是因为「PM 在场，能现场答疑」——延到 prd-writer 阶段，PM 已经走人，
agent 只能瞎猜。

### 差异 4：辩论用「现在时」写

参考实现里 PRD 草稿允许写「这个功能完成后，用户将能……」。本 skill **禁止**。

**为什么**：charter 的「活文档禁用历史/将来叙事」规则——PRD 草稿里的句子应该能
**直接粘进 PRODUCT.md 不需要改**。如果写了将来时，prd-writer 还要重写一遍。

**怎么做**：
- 错：「上线后，用户可以用 CLI 跑 intent check」
- 对：「用户用 `intent check <file>` 跑验证」
- 状态描述用 Status 字段（Proposed / Accepted），不用「将来」措辞

## 暂停纪律

**每个 Phase 结束后强制停止生成，等用户回复。** 这是 skill 的硬约束——不是建议。

为什么这么严：
- LLM 的天然倾向是「一口气讲完」——而辩论的价值恰恰来自每个 Phase 之间的人类
  反馈
- 跳过暂停 = 用户错过决策机会 = 后面的 Phase 基于错误的早期共识——一旦发生，
  整场辩论作废
- 强制暂停也让长辩论可以分多次会话完成（用户休息后回来继续 Phase 3）

**实操**：每个 Phase 的最后一句话**一定是问句**，问句之后立即结束输出。下次用户
说话之前你不要再生成任何内容。

## 角色阵容与置信度联动

| 置信度 | 角色数 | 角色对用户的态度 |
|---|---|---|
| 1-2 | 3-4 | 顺从、解释、扩展用户观点 |
| 3 | 4-5 | 平等讨论、可能反对 |
| 4-5 | 5-6 | 积极挑战、当魔鬼代言人 |

具体行为规则在 `references/confidence-modes.md`。**用户置信度不是装饰——它决定
辩论的认知压力强度。** 用户标 5 但你按 1 跑，他会觉得 AI 在敷衍；反过来用户标
1 但你按 5 跑，他会被压垮然后退出辩论。

## 关键失败模式

| 失败模式 | 早期信号 | 修复 |
|---|---|---|
| **角色趋同** —— 所有角色说同样的话 | Phase 3 各角色的「偏好」都一样 | 强行让两个角色站在对立张力上（PM vs ENGLD 在「快速交付 vs 技术质量」）|
| **方案不真差异化** —— 2 个方案只差细节 | Phase 2 用户说「这两个不就是一个东西吗」| 引导一个角色显式提一个**用户路径不同**的方案，不只是功能集不同 |
| **不引用事实基** —— 全程没 cite fact-extraction-report | Phase 3 ENGLD 没有任何「文件:行号」式引用 | 在 ENGLD 发言前注入提示「请基于 fact-extraction-report § Risk Hotspots 评估」|
| **用户被 AI 牵着走** —— 用户每个暂停都回「继续」| 连续 3 个暂停无实质输入 | 主动降低置信度（「我把置信度临时调到 1，重新组织一下问题」），或暂停辩论问用户是否要继续 |
| **Phase 4 没收敛** —— 用户说「再想想」但你继续推进 | 投票后没有明确决策 | 接受「未决」是合法状态——产出标 `Status: Deferred` 的 debate-record，用户日后回来继续 |

## 项目配置约定

本 skill 通过读取项目仓库中的配置文件实现领域定制：

| 文件路径 | 用途 | 缺失时行为 |
|---------|------|-----------|
| `docs/product-debate/roles.md` | 项目领域角色定义（如 DX、IOT、ENTERPRISE）| 使用 `references/default-roles.md` 的 5 个通用角色 |
| `docs/product-debate/product-mapping.md` | 产品-角色默认映射 | 用户手动选择角色 |

第一次跑本 skill 的项目，可以在 `concluding` 状态末尾**建议**用户落地这两个配置
文件——但**不要**自动创建（这是用户决策，不是 agent 决策）。

## 与 arch-debate 的边界

本 skill **不讨论技术架构**。任何形如「用什么数据库」「微服务还是单体」「Z3 还
是 CVC5」的讨论，应该被**显式打断**并标注「需要 arch-debate」。

在 Phase 4 末尾，如果辩论涉及重大技术选型，主动建议：

> 本次讨论的 [方案 X] 涉及 [具体技术选型]，建议在喂给 prd-writer 之前先启动
> arch-debate，让技术角色对这部分做深入评审，避免 PRD 落地后才发现技术上不可行。

## 与 prd-writer 的接驳

辩论结束时输出的 `prd-draft.md` 是 `prd-writer` 的**输入**，**不是**最终 PRD。
两者的责任划分：

| 关注点 | product-debate | prd-writer |
|---|---|---|
| 方案空间探索 | ✅ 主要工作 | ❌ |
| 多角色冲突暴露 | ✅ 主要工作 | ❌ |
| PRD 质量评分（90+）| ❌ | ✅ 主要工作 |
| 验收标准结构化 | 草稿级 | ✅ 主要工作 |
| 提取 acceptance.intent 种子 | 标注 intent 层 | ✅ 实际生成 |
| 配套 PDR 骨架 | ❌ | ✅ 主要工作 |
| Decision-Ref 链接 | 标注「需要 PDR」 | ✅ 实际分配 PDR ID |

你**不需要**做 prd-writer 的事——把好的 prd-draft 交付出去，停在那里。

## 辩论持久化（可选）

如果项目装了 `popsicle discussion` 命令，可以在每个角色发言、用户输入、阶段小结
后调用它写入数据库：

```bash
popsicle discussion message <discussion-id> \
  --role <role-id> --role-name "<显示名>" \
  --phase "Phase N: <名称>" \
  --type role-statement \
  --content "<发言内容>"
```

最低要求：每个 Phase 的**暂停点**和**阶段小结**至少调用一次，确保关键决策节点
被持久化。

如果 `popsicle discussion` 不可用：不阻塞，辩论照常进行，只靠最终的 debate-record
留存。

## 参考资源

### Reference 文件

- **`references/default-roles.md`** —— 5 个内置通用角色（PM, UXR, GROWTH, ENGLD, BIZ）
- **`references/confidence-modes.md`** —— 5 级置信度的详细行为规则
- **`references/debate-phases.md`** —— 4 个辩论阶段的详细说明
- **`references/output-templates.md`** —— prd-draft / debate-record / decision-matrix 的产出格式

### 项目配置文件（可选）

- **`docs/product-debate/roles.md`** —— 项目领域角色定义
- **`docs/product-debate/product-mapping.md`** —— 产品-角色映射表

### 上游 artifact（强烈推荐）

- **`<slug>.fact-extraction-report.md`** —— 辩论的事实基
- **`<slug>.project-init-plan.md`** —— Product Inventory，限定辩论边界
