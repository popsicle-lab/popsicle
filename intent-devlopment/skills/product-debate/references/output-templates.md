# 产出物模板

辩论结束后生成三份 artifact。模板文件分别落在 `templates/` 目录下；本文件解释
**为什么这样设计**以及**填写要点**。

---

## 一、prd-draft（PRD 草稿）

文件：`templates/prd-draft.md`

**定位**：辩论收敛的忠实摘要，**不是终态 PRD**。质量门槛由下游 `prd-writer`
把守。

**填写要点**：

1. `Status` 字段固定为 `Draft (from product-debate)`，让 prd-writer 知道这是
   辩论产出，不是用户直接写的
2. 「核心声明 → intent 层」表必填——这是 IDD 工作流的硬约束，缺这张表的草稿在
   prd-writer 阶段会被退回
3. 所有数字 / LoC / 模块名都要 cite fact-extraction-report 的具体章节
4. 草稿用「现在时」写，能直接粘进 PRODUCT.md
5. 末尾「辩论记录」段保留辩论元信息（参与角色、置信度、投票结果），让 PDR 起草
   时能引用辩论作为决策依据

**不准写的内容**：
- 架构选型细节（数据库、协议等）—— 标 `[ADR 候选：技术方案待定]`，留给 arch-debate
- 用「将来时」写的需求 —— 改成现在时
- 没在辩论中讨论过的内容 —— 草稿是辩论摘要，不是发挥

---

## 二、debate-record（辩论纪要）

文件：`templates/debate-record.md`

**定位**：永久存档，作为「为什么这样决定」的证据。下游 PDR 起草时**必须引用本
文件**——这是 charter 「每次活文档编辑必须引用 Decision-Ref」的源头证据。

**填写要点**：

1. 完整记录 4 个 Phase 的发言摘要（不必逐字，但要保留各角色的关键论据）
2. 「关键分歧」列表必填——这是日后回头看「我们当时为什么没选 B」的核心证据
3. 「用户决策」段显式记录：
   - 用户是接受 PM 推荐 ✅
   - 还是覆盖了 PM 推荐 ⚠️
   - 覆盖时记录用户的理由
4. 「下游 skill 接驳建议」给出明确指向：通常是 prd-writer，但如果触及架构 / charter，
   先指向 arch-debate / CADR

---

## 三、decision-matrix（决策矩阵）

文件：`templates/decision-matrix.md`

**定位**：候选方案 × 评估维度的打分。当候选方案 ≥ 2 个时生成；单方案讨论可省略。

**填写要点**：

1. 评估维度**由参与辩论的角色提出**，不是预设的——PM 在 Phase 4 收敛时让每个
   角色提名 1-2 个自己关注的维度，去重后形成矩阵列
2. 权重由 PM 决定，但每个权重要写一句话理由（「用户价值权重高，因为本辩论的
   起点是 UXR 提的用户痛点」）
3. 每个方案在每个维度上的评分要附一句话依据，**禁止只打星不解释**
4. 综合推荐**可以**与最高加权分数方案不一致——比如用户最终决策覆盖了打分结果。
   此时在「综合推荐」一栏注明覆盖理由。

---

## 产出物生成规则

1. **prd-draft 和 debate-record 必出**，每次辩论必须生成
2. **decision-matrix** 在候选方案 ≥ 2 时必出，单方案讨论可省略
3. 所有产出**使用中文**（除非用户指定其他语言）
4. 三份 artifact 的 `Topic` 字段**必须一致**——这是 prd-writer 用来找辩论上下文
   的索引键
5. 三份 artifact 都引用同一个 `target_product`（来自 setup 时的绑定）—— 不允许
   一份辩论横跨多个 product 的产出

---

## 产出归档位置

三份 artifact 默认落在 popsicle 的 run 目录：

```
.popsicle/artifacts/<run-id>/
├── <slug>.product-debate.md     # debate-record
├── <slug>.prd-draft.md
└── <slug>.decision-matrix.md
```

辩论结束后，建议把这些文件移动到 product 的 proposals 目录：

```bash
mv .popsicle/artifacts/<run-id>/<slug>.* products/<target_product>/proposals/exploring/
```

理由：辩论产物在 PDR Accepted 之前都是 **Exploring** 状态——根据 charter，
product 侧的提案生命周期是 Exploring → Proposed → Accepted/Rejected。辩论产出
天然属于「Exploring」桶。
