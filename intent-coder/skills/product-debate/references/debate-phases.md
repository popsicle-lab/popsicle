# 辩论阶段详细说明

辩论共分 4 个阶段，每阶段有明确的参与角色、活动内容、暂停点和产出。

> **强制暂停规则**（来自 skill.yaml）：每个 Phase 结束后立即停止生成，等用户回复。
> 详细规则参见 guide.md 的「暂停纪律」一节。

---

## Phase 1: 用户需求与问题定义

**目标**：确保所有参与者对用户痛点、目标用户和业务目标有共同理解

**主要角色**：PM（主持）+ UXR + BIZ

**活动内容**：

1. PM 陈述对问题的理解，列出：
   - 目标用户画像和核心场景
   - 用户痛点和需求假设
   - 产品目标和成功指标
   - 已知的约束条件（资源、时间、技术）

2. UXR 补充：
   - 已有的用户研究发现或行为数据（**cite 来源**：调研记录 / 数据报告 /
     fact-extraction-report 中的某项观察）
   - 类似产品的用户反馈模式
   - 潜在的用户心智模型和期望

3. BIZ 补充：
   - 商业上下文和市场环境
   - 竞品格局和差异化机会
   - 合规、法务等约束

**IDD 专属步骤**：在 PM 发言前，**先朗读** fact-extraction-report 的 Bounded
Contexts row（如果存在），让所有人共享同一份事实底图：

```
**[PM]**:
> 在我们开始之前，先把事实基对齐一下：
> - 目标 product = `<name>`（来自 project-init-plan §Product Inventory）
> - 该 product 当前 LoC = N（fact-extraction-report § Bounded Contexts row K）
> - 高 churn 模块 = M（fact-extraction-report § Risk Hotspots row K）
> 大家辩论时请基于这些事实，不要凭印象。
```

如果 fact-extraction-report 不存在，PM 应明确说：

```
**[PM]**:
> 警告：本次辩论缺少 fact-extraction-report 作为事实基，所有「现状」类陈述
> 都将标 `[未经事实基验证]`。结论交给 prd-writer 后，质量评分会被扣分。
```

**暂停点**（1 次）：
```
🎤 Phase 1 暂停 — 等待你的输入
📋 问题定义如上。已识别的关键约束：
   1. [约束 1]
   2. [约束 2]
   3. [约束 3]
❓ [根据置信度调整的提问]
```

**阶段产出**：
- 用户痛点陈述（1-2 句话）
- 目标用户画像
- 约束清单（必须满足 / 最好满足 / 可以取舍）
- 成功指标（如何判断方案是好的）

---

## Phase 2: 方案发散与初评

**目标**：产出 2-3 个**差异化**的候选产品方案

**主要角色**：2-3 位角色分别提出不同方案

**活动内容**：

1. 角色分配：
   - 选择 2-3 位风格偏好不同的角色，每人独立提出一个方案
   - 方案之间应有**实质性差异**（不同产品策略 / 不同用户路径 / 不同商业模式）

2. 每个方案包含：
   - 方案名称和一句话概述
   - 核心用户流程描述
   - 关键功能列表和优先级
   - 商业模式 / 变现策略
   - 自评优势和劣势

3. 初步交叉评论：
   - 每位提案者简短评论其他方案的最大优势和最大风险

**方案差异化指导**：

确保方案之间存在真正的策略差异，而非细节差异。典型差异维度：

| 维度 | 方案 A 方向 | 方案 B 方向 |
|------|-----------|-----------|
| 目标用户 | 大众市场 | 细分垂直市场 |
| MVP 范围 | 功能精简、快速验证 | 功能完整、体验打磨 |
| 增长策略 | 产品驱动增长（PLG）| 销售驱动增长（SLG）|
| 变现模式 | 订阅制 | 交易抽佣 / 一次性付费 |
| 竞争策略 | 差异化创新 | 快速跟进 + 本土化 |
| **实现路径** *(IDD 专属)* | **复用现有 module** | **新建 product slice** |

**IDD 专属维度（实现路径）**：在迁移期工作流里，「方案是否新增 product / 是否
影响 charter」本身就是一个关键差异维度。强制至少一个方案考虑「最小侵入式做法」
（不新增 product，只在现有 slice 内迭代）。

**暂停点**（1 次）：
```
🎤 Phase 2 暂停 — 等待你的输入
📋 当前候选方案：
   方案 A: [一句话]
   方案 B: [一句话]
   方案 C: [一句话]（如有）
❓ [根据置信度调整的提问]
```

**阶段产出**：
- 2-3 个结构化候选方案
- 各方案自评优劣势
- 初步交叉评论

---

## Phase 3: 多角色辩论

**目标**：从多维度深度评审每个候选方案，暴露风险和盲点

**主要角色**：全部参与角色（除已提方案的角色外，所有角色都做评审）

**活动内容**：

1. 逐角色评审：
   - 每位评审角色从自己的视角评估所有候选方案
   - 指出各方案在自己关注领域的优劣
   - 给出明确的偏好排序及理由

2. 评审格式：
   ```
   **[角色ID - 角色名称]** 评审:

   方案 A:
   - ✅ [优势]
   - ⚠️ [风险/不足]（**附 cite**：fact-extraction-report § X / 行业基准 / ……）

   方案 B:
   - ✅ [优势]
   - ⚠️ [风险/不足]

   **偏好**: 方案 [X]，因为 [理由]
   ```

3. 辩论环节：
   - 观点冲突时涉及的角色互相回应
   - 利用角色间的内在张力推动深入讨论
   - 不回避分歧，明确记录分歧点及各方理由

**IDD 专属环节**：ENGLD 角色在评审「实现成本」类条目时**必须**附带 cite。无 cite
的成本估算被标记为「直觉估算，可信度低」。

**暂停点**（1-2 次）：

每 1-2 个角色评审后暂停：
```
🎤 Phase 3 暂停 — 等待你的输入
📋 [角色 X] 评审完毕。主要观点：[要点]
❓ [根据置信度调整的提问]
```

**阶段产出**：
- 每位角色的评审意见
- 识别出的关键分歧点
- 各方案综合优劣势对照

---

## Phase 4: 收敛与决策

**目标**：形成最终推荐方案，产出 prd-draft / debate-record / decision-matrix

**主要角色**：PM（主持收敛）+ 全部角色投票

**活动内容**：

1. PM 综合所有评审意见：
   - 梳理各角色的偏好和理由
   - 识别共识点和分歧点
   - 提出综合推荐（可以是某方案 + 其他方案的局部优点）

2. 角色投票：
   - 每位角色投票：**支持** / **有保留地支持** / **反对**
   - 反对者**必须**说明具体顾虑和接受条件

3. 用户最终决策：
   - 向用户展示投票结果和分歧
   - 请用户做最终决定
   - 对于「用户推翻多数角色意见」的情况，标注为「用户决策覆盖」

4. **IDD 专属（v0.2 强化）：Task 识别 + Intent 层归类（PM 强制执行）**

   **Step 4a — Task 识别**：PM 输出一张「用户意图 → task → 旅程阶段」表。这是
   v0.2 任务图范式的核心步骤——不识别 task 就没有 PRD 主体。

   | 候选 ID | 用户原话标题 | Journey Stage | Audience |
   |---|---|---|---|
   | TBD-1 | 「我第一次跑 intent check 直到拿到 PASS」 | onboarding | new-user |
   | TBD-2 | 「拿到 FAIL 后看反例并调整 spec」 | onboarding | new-user |
   | TBD-3 | 「一次性校验一组 .intent」 | daily-ops | end-user |
   | TBD-4 | 「验证超时怎么办」 | troubleshooting | end-user |

   **Task 识别准则**（PM 必背）：
   - **能用一句完整用户原话讲出来的是 task**；讲不出（只能讲模块名 / 功能名）
     的是 feature，不要列上去
   - 5 个旅程阶段固定：`onboarding` / `daily-ops` / `troubleshooting` / `admin` /
     `lifecycle`。按优先级判定：
     1. 首次成功旅程 → onboarding
     2. 异常排查 → troubleshooting
     3. 组织/配额/权限/审计 → admin
     4. 终止/迁出/续费 → lifecycle
     5. 其它 → daily-ops
   - 完成路径含大量 if-else 分支 → 拆出去到 `troubleshooting/` 单独成 task
   - 单个 task 估计 > 250 行 → 它是 epic，拆成多个

   **task_id 是占位符**——本辩论用 `TBD-1 / TBD-2`，由下游 prd-writer 在分配阶
   段替换为 `T-XXXX`（product 内全局递增 4 位 0 填充）。

   **Step 4b — Intent 层归类**：PM 输出「核心声明 → intent 层 → 关联 task」表：

   | 核心声明 | 目标 intent 层 | 关联 Task | 后续 PDR/ADR |
   |---|---|---|---|
   | 「用户提交 .intent 后 5 秒内必须收到 PASS/FAIL」| `acceptance.intent` | TBD-1 | PDR-XXXX |
   | 「拿到 FAIL 时必须含可读最短反例」| `acceptance.intent` | TBD-2 | PDR-XXXX |
   | 「同一个验证任务不能被并发跑两次」| `invariants.intent` | —— (跨 task) | PDR-XXXX |
   | 「verifier 暴露 `Verify(spec) -> Result` 接口」| `contracts.intent` | TBD-1 / TBD-3 | **ADR-XXXX（建议跑 arch-debate）** |
   | 「定价：免费 5 次 / 月」| —— 留在 task frontmatter `limits` 字段 | TBD-1 等 | PDR-XXXX |

   声明里凡是含「永远 / 不可 / 必须 / 至多 / 至少 / 5 秒内 / 100%」之类**可形
   式化的硬约束**，都倾向于落进 intent 层。模糊声明（「用户体验良好」「快」）
   留在自然语言部分。

   **Step 4c — User Intents Catalog 起草**：PM 把每个 task 的「本 task 可解答」
   问句汇总成一张 query 锚点表——这是 AI Copilot 的核心索引，PRD 顶层直接复
   用。每个 task 至少 3 个用户原话问句。

5. 产出生成：
   - 基于最终决策生成三份 artifact（见 `references/output-templates.md`）
   - prd-draft 必须是 task-centric 形态（含 §4 Tasks / §5 User Intents Catalog
     / §6 Intent Mapping），下游 prd-writer 会强制校验

**暂停点**（1 次）：
```
🎤 Phase 4 最终确认 — 等待你的输入
📋 投票结果：
   方案 A: [N] 票支持 — [角色列表]
   方案 B: [N] 票支持 — [角色列表]
   分歧: [角色 X] 强烈反对方案 A，因为 [原因]
🏆 PM 推荐: 方案 [X]，采纳方案 [Y] 的 [具体部分]

📋 Task 识别（待你确认）：
   TBD-1 [onboarding]      「我第一次跑 intent check 直到拿到 PASS」
   TBD-2 [onboarding]      「拿到 FAIL 后看反例并调整 spec」
   TBD-3 [daily-ops]       「一次性校验一组 .intent」
   TBD-4 [troubleshooting] 「验证超时怎么办」

📋 Intent 层归类（待你确认）：
   - 声明 1（5 秒响应）→ acceptance.intent → TBD-1
   - 声明 2（最短反例）→ acceptance.intent → TBD-2
   - 声明 3（无并发）→ invariants.intent → 跨 task
   - 声明 4（Verifier 接口）→ contracts.intent（先跑 arch-debate）→ TBD-1/3

❓ 你同意 task 划分和 intent 归类吗？需要合并/拆分 task 吗？
```

**阶段产出**：
- 最终推荐方案（含修正和折中）
- prd-draft.md（Phase 4 末尾生成，进入 concluding 状态时落盘）
- debate-record.md
- decision-matrix.md

---

## 阶段小结格式

每个 Phase 结束时生成阶段小结：

```markdown
---
### Phase [N] 小结

**共识**:
- [共识点 1]
- [共识点 2]

**用户决定**:
- [用户做出的关键决定]

**悬而未决**:
- [待下阶段讨论的问题]

---
```

## 异常处理

**讨论陷入僵局时**：
- PM 角色介入，将分歧拆分为更小的子问题
- 对子问题逐一达成共识
- 如仍无法收敛，标记为「需要更多数据验证」并列出所需调研

**用户长时间不回应暂停点时**：
- **不要**假设用户「继续」就推进——本 skill 的策略与参考实现不同：
- 给用户 24 小时窗口（或会话级别的「下一次回到本辩论」）
- 在 debate-record 中标注「此暂停点用户未参与」
- 不要伪造用户决策

**讨论偏离主题时**：
- PM 角色提醒并拉回主线
- 偏离话题记录为「后续讨论项」

**辩论触及 charter 锁定内容时**：
- **立即停止 Phase 推进**
- 在 debate-record 中标注「触及 charter，需要 CADR」
- 建议用户暂停本辩论，先发起 CADR 流程
