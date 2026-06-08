# RFC: intent-lang 作为"需求建模语言"的定位收敛与能力补全

Status: Draft
Created: 2026-05-06
Context: 来自一系列关于"用 intent-lang + popsicle 落地 IDD 并迁移大型遗留项目"的方法论讨论
Related: docs/lang/DECISIONS.md, docs/lang/LLM.md

---

## 1. Motivation

intent-lang 当前的文档（README、DECISIONS、LLM.md）在不同位置呈现了三种潜在定位：
- **契约语言**（contract language）：描述模块对外承诺
- **程序规格语言**（program specification language，类 Dafny）：描述算法行为
- **需求建模语言**（requirements modeling language）：描述业务意图与不变量

经过深入讨论，作者明确表态：**intent-lang 的真实定位是"描述需求到意图，并验证意图之间无悖论"**——即第三种。

但当前语言设计、文档、示例仍偏向前两种。这个定位错位会导致：
- 用户预期混乱（以为可以生成代码）
- 与 Dafny / TLA+ / Z3 SMT-LIB 的差异化不清晰
- 错过最大价值场景（需求侧的一致性 + 完备性验证）

本 RFC 提议将定位收敛到"需求建模"，并列出由此产生的语言层 / 工具链 / 文档层改动需求。

---

## 2. 定位收敛后的核心主张

> intent-lang 是一种**需求建模 DSL**，用于：
> 1. 把自然语言需求和领域不变量翻译成机器可读、可验证的形式
> 2. 通过 SMT 验证多份需求/意图之间**无逻辑矛盾**
> 3. 检测需求集合的**完备性**（是否覆盖应有场景）
> 4. 作为 LLM 生成代码、生成测试、解释决策的**可信锚点**

它**不是**：
- 实现语言（不生成可执行代码）
- 程序规格语言（不验证算法实现是否正确——那是 Dafny 的事）
- 通用契约语言（不直接约束 API 调用，而是约束业务规则）

它在 IDD 体系中位于：
```
人类需求（自然语言）
   ↓ 翻译
intent-lang（机器可读意图 + Z3 可验证）
   ↓ 派生
测试规格 / API 契约 / 实现代码（由其他工具生成）
   ↓ 实现
代码（与 intent 双向追溯）
```

---

## 3. 由定位收敛产生的改动需求

### 3.1 语言层

#### A1. `goal` 提升为 L1 一等公民
- 当前 `invariant` / `precondition` / `postcondition` 是核心
- 需要新增 `goal` 块作为最高层概念，承载"为什么"
- `goal` 携带 `rationale`（业务理由）、`stakeholder`（受益方）、`measure`（如何度量达成）
- 一个 `goal` 可以由多条 `invariant` 实现，关系显式声明

```
goal "用户余额永不为负" {
    rationale: "金融监管硬要求"
    stakeholder: "compliance, finance"
    measure: "线上未发生 negative balance 事件"
    realized_by: [account.balance_non_negative]
}

invariant account.balance_non_negative {
    forall a in accounts: a.balance >= 0
}
```

#### A2. `as-is` 与 `to-be` 的语义区分
- 迁移场景需要描述"老代码实际是什么样"（as-is），不能与"我们希望它是什么样"（to-be）混淆
- 提议：annotation 形式 `@asis` / `@tobe`，或独立块 `legacy { ... }`
- as-is intent 不参与主 invariants 库的一致性验证，但参与"老代码合规性"验证

#### A3. 完备性约束声明
- 当前 Z3 验证一致性（无矛盾），但不验证完备性（无遗漏）
- 提议：`coverage` 块声明"应该覆盖的场景维度"
- 工具检测是否每个维度都有对应 invariant

```
coverage "支付场景" {
    dimensions: [
        currency: [CNY, USD, EUR],
        amount: [zero, positive, max_int],
        account_state: [normal, frozen, closed]
    ]
    require: forall combo: exists invariant covering combo
}
```

#### A4. intent diff 语义
- 比 git diff 更高阶
- 能识别一次修改是：放宽约束 / 收紧约束 / 改变契约 / 添加新约束
- 不同类型对应不同影响等级（放宽 → 向后兼容；收紧 → 可能 breaking）
- 由工具实现（不一定是语言层），但需要语言提供稳定的 AST 锚点

---

### 3.2 工具链

#### B1. 一致性验证（已有，需强化）
- 当前 Z3 验证主要是 LLM.md 提到的能力
- 强化：报告应明确指出哪两条 intent 冲突 + 反例
- 集成到 popsicle 的 decision-gatekeeper Skill

#### B2. 完备性检查（新）
- 配合 A3 的 `coverage` 块
- 输出：未覆盖维度组合清单
- 注意：完备性永远不可能 100%，工具应给"已知未覆盖"清单供人工审

#### B3. 测试规格生成（新）
- 从 intent 派生测试场景描述（自然语言 / 表格）
- 不生成可执行测试代码——那是 popsicle Skill 的事
- 提供稳定的"intent → 场景"映射 API，供下游工具调用

#### B4. 影响分析（新）
- 输入：intent diff
- 输出：受影响的下游 intent / 测试规格 / 标注代码
- 是 popsicle Skill `intent-impact-analyzer` 的底层能力

#### B5. LLM 反向解释（新或强化）
- 输入：一条 intent
- 输出：自然语言解释 + 典型违反例 + 典型满足例
- 用于人工审查环节，让 PM/业务方能审查 intent 是否真的反映需求

---

### 3.3 文档层

#### C1. 重写 README 的"是什么"段落
- 明确写"需求建模 DSL，不是程序规格语言"
- 给出与 Dafny / TLA+ / Z3 SMT-LIB 的差异化矩阵

#### C2. 新增 docs/lang/POSITIONING.md
- 详细说明 intent-lang 在 IDD 体系中的位置
- 边界声明：什么不该用 intent-lang 写（实现细节、UI 行为、性能优化策略等）

#### C3. 重组 examples/
- 当前 examples（transfer.intent、smarthome.intent）偏程序规格风格
- 增加纯需求建模风格的例子：仅 goal + invariant，不含 procedural 细节
- 增加迁移场景的 as-is / to-be 对比例子

#### C4. LLM.md 更新
- 当前重点是"如何让 LLM 生成 intent"
- 补充：如何让 LLM 生成 testspec / 解释 intent / 检测漏洞
- 反作弊原则：intent 生成的 LLM 不应同时生成验证测试

---

### 3.4 与 popsicle 的协作接口

#### D1. Artifact schema
- 与 popsicle Skill 之间通过哪些 artifact_type 通信
- 提议：`intent.diff`、`intent.consistency_report`、`intent.coverage_report`、`testspec.draft`

#### D2. CLI 命令稳定化
- popsicle Skill 通过 shell 调用 intent-lang CLI
- 需要稳定的、版本化的 CLI 接口（如 `intent verify --format json`）

---

## 4. 优先级建议

遵循"先 dogfood，再设计"的纪律。在第一个真实迁移场景跑通前，**只做最小改动**：

### Phase 0（即刻）：定位收敛的文档改动
- C1 README "是什么"重写
- C2 新增 POSITIONING.md
- 这些是零代码风险的，先把定位钉死

### Phase 1（迁移启动后 1-2 月）：dogfood 期，只改报错
- 不动语言层
- 不加新工具
- 只观察：写真实需求时哪些场景表达不出来

### Phase 2（第一个场景跑通后）：补最痛的 1-2 个语言特性
- 预测：A1（goal）和 A2（as-is/to-be）会先变痛
- 但实际优先级以 dogfood 反馈为准

### Phase 3（持续）：完备性、影响分析等高级能力
- A3、B2、B4 在多场景迁移中价值才能凸显
- 提前做风险高，过度设计概率大

---

## 5. 反模式 / 克制清单

1. **不要往 intent-lang 加实现语义**——任何"如何执行"的语法都偏离需求建模定位
2. **不要重做 Dafny**——程序规格验证不是 intent-lang 的目标
3. **不要追求语言完备性**——能表达 80% 真实业务需求就够，剩下 20% 用自然语言加注释
4. **不要让 LLM 生成 intent 完全闭环**——必须有人工 review 环节（adversarial-reviewer）
5. **不要把 intent-lang 设计成"AI 优先"**——人类可读优先，AI 友好是副产品

---

## 6. Open Questions

1. `goal` 的 `realized_by` 是双向引用还是单向？双向便于追溯但增加同步成本
2. `coverage` 的维度声明是 ad-hoc 还是基于类型系统？
3. as-is intent 是用 annotation、独立块还是独立文件后缀（`.legacy.intent`）？
4. intent diff 工具是 intent-lang 仓库内置还是 popsicle Skill 调用？
5. 是否需要"intent 的 intent"——元层级声明 intent 文件本身的目的（项目级 README intent）？

这些问题留待第一个迁移场景的真实使用反馈后回答。

---

## 7. 历史背景

本 RFC 来自一系列关于以下主题的讨论：
- 意图驱动开发（IDD）方法论
- intent-lang 的定位（需求建模 vs 契约语言 vs 程序语言）
- LLM 生成 intent 与生成测试代码的反合谋设计
- Z3 在 LLM 时代的角色（最终裁判 vs 辅助工具）
- 大型项目向 IDD 框架的 Strangler Fig 迁移策略

完整讨论档案保存在用户本地：
- `idd-intent-lang-discussion.md`
- `idd-doc-migration-discussion.md`
- `idd-new-repo-migration-discussion.md`

姊妹 RFC（popsicle 侧）：
- `popsicle/docs/rfc-idd-migration-gaps.md`
