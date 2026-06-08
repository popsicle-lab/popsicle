# RFC: intent-lang 作为 AI 时代 PRD 的真相层 —— 整合架构与语言扩展建议

Status: Draft
Created: 2026-05-08
Context: 来自一次"AI 时代产品文档应如何组织"的方法论讨论。以 SaaS billing 模块为案例，对比了传统叙事式 PRD 与结构化 PRD，并提出把 intent-lang 嵌入 PRD 体系作为可机器验证的"真相层"。
Related:
- docs/rfc-idd-positioning-and-improvements.md（姊妹 RFC，定位收敛）
- docs/lang/POSITIONING.md
- docs/software/README.md
- examples/requirements/billing.intent
- examples/requirements/access-control.intent

---

## 1. Motivation

`rfc-idd-positioning-and-improvements.md` 已将 intent-lang 的定位收敛为"需求建模 DSL"，回答了**它是什么 / 不是什么**。但在真实落地时，团队还会问下一层问题：

> 我们已经有 PRD（产品需求文档）、ADR、API 文档、Runbook 等一整套叙事性文档体系，intent-lang 应该**怎样**嵌入进去？写在哪里？谁来写？谁来 review？怎么和 PRD 双向同步？

进入 AI 时代后这个问题变得更紧迫——文档不再只给人看，AI Agent 也是主要消费者。一份既能给人读、又能给 AI 用、还能被 SMT 验证的文档体系，长什么样子？

本 RFC 提议：

1. **以 intent-lang 为真相层**，重新组织 AI 时代 PRD 的物理结构。
2. **明确人写部分与机器派生部分的边界**，让叙事 PRD、API 契约、测试规格、运行时断言尽可能从 `*.intent` 派生。
3. **由整合实践反推出 5 条语言/工具扩展建议**，让 intent-lang 真正胜任真相层职责。

本 RFC 与已有的 `rfc-idd-positioning-and-improvements.md` 关系如下：

| RFC | 解决的问题 |
|---|---|
| 既有：定位收敛 RFC | intent-lang **是什么 / 不是什么** |
| 本 RFC：PRD 整合 RFC | intent-lang **如何嵌入产品文档体系** |

两者一起回答"在 AI 时代，一支团队应该如何写需求"。

---

## 2. AI 时代 PRD 的核心矛盾

### 2.1 从单读者到双读者

| 维度 | 传统叙事 PRD | AI 时代 PRD |
|---|---|---|
| 读者 | 人类 | 人 + AI Agent |
| 检索 | 目录 / 全文搜索 | 向量检索 + 上下文注入 |
| 单位 | 章节 | 语义原子块（chunk） |
| 一致性 | 人工 review | 可机器验证 |
| 更新 | 滞后于代码 | 与代码同生命周期 |
| 形式 | 自然语言 | 自然语言 + 结构化元数据 + 可执行片段 |

### 2.2 真相缺口

即使采用结构化 PRD（带 frontmatter、术语表、契约、Gherkin 场景），仍然存在一个机器永远无法自行检查的层：**业务规则之间的逻辑自洽**。

```
人类语言层  ─►  契约 / 测试层  ─►  代码层
   (PRD)         (OpenAPI/BDD)      (实现)

   ↑ 有歧义       ↑ 能跑             ↑ 能跑
                  但不能证明
                  规则之间不矛盾
```

OpenAPI 校验形状、Gherkin 校验路径、单测校验实现，但**没有任何工具校验"PRD 写的 5 条业务规则放一起是否自洽"**。intent-lang 正好坐这一层。

### 2.3 四个组织原则

支持双读者的文档体系应满足：

1. **分层（Layered）**：按信息密度分层（入口 / 能力图 / 契约 / 决策 / 示例），AI 按需加载。
2. **原子化（Atomic）**：每个语义块自包含，可被独立检索和拼接。
3. **结构化元数据（Structured Metadata）**：frontmatter 让机器先读懂"这是什么、什么时候适用"。
4. **单一事实源（SSOT + Derivation）**：核心规则只在一处写一次，叙事/契约/测试都是派生物。

intent-lang 的产物（`.intent` 文件）天然适合做原则 4 中的"那一处"。

---

## 3. 真相层主张

> **`*.intent` 文件持有业务规则的真相；其他文档（叙事 PRD、API 契约、测试、运行时断言）尽可能从 intent 派生。**

| 层 | 形式 | 主要工具 | 验证者 |
|---|---|---|---|
| 叙事层 | Markdown PRD、ADR、Runbook | 人脑 | 人 |
| **真相层** | **`*.intent`** | **`intent check` (Z3)** | **机器** |
| 契约层 | OpenAPI、JSON Schema | OpenAPI lint | 机器 |
| 行为层 | Gherkin / pytest | CI | 机器 |
| 代码层 | TS / Rust | 类型检查 + 测试 | 机器 |

这与 `POSITIONING.md` 的"intent 是所有下游行为的可信锚点"完全一致。本 RFC 把这个主张从概念落实到目录结构与工作流。

---

## 4. 推荐的目录骨架

以 SaaS billing 模块为例：

```
billing/
├── AGENTS.md                     # AI 入口（人写，CI 校验链接活性）
│
├── intents/                      ★ SSOT —— 真相层
│   ├── _domain.intent            # 共享类型 / enum / 全局术语
│   ├── plan-catalog.intent       # CAP-001
│   ├── subscription.intent       # CAP-002, 003
│   ├── plan-change.intent        # CAP-004
│   ├── invoice.intent            # CAP-005
│   ├── refund.intent             # CAP-007
│   └── cross-cutting.intent      # 跨能力守恒定理
│
├── docs/                         # 叙事层 —— 大多由 intent 派生
│   ├── PRD.md                    # ← intent explain --as prd
│   ├── glossary.md               # ← intent ontology --extract
│   ├── api/openapi.yaml          # ← intent export --format openapi
│   ├── decisions/                # 人写（Why）
│   │   └── ADR-002-second-precision.md
│   ├── concepts/                 # 人写（心智模型）
│   ├── runbooks/                 # 人写
│   ├── anti-patterns.md          # 人写
│   └── _open-questions.md        # 人写
│
├── tests/
│   ├── generated/                # ← intent testspec
│   └── manual/                   # 人写补充
│
└── .intent/
    ├── coverage.lock             # CI 守门：覆盖率不许下降
    └── impact.lock               # CI 守门：未声明的破坏性变更被拒
```

### 4.1 关键纪律

1. **任何业务规则的"权威定义"只能在 `intents/` 出现一次**。叙事文档要引用必须用 ID（`@CAP-004` / `@theorem.MoneyConservation`）。
2. **`docs/PRD.md` 是只读派生物**。要改业务规则就改 intent，叙事会自动重生成。
3. **ADR / Concepts / Runbooks / Anti-patterns 是人写**。intent-lang 不管"为什么这样做"，只管"我们承诺什么"。
4. **CI 守护两道闸**：一致性（intent check）+ 覆盖率不回退（coverage.lock）。

### 4.2 与现有 examples 的对应关系

`examples/requirements/billing.intent` 和 `access-control.intent` 已经是这个骨架的雏形（用了 `goal` / `@tobe` / `@asis` / `coverage` / `realized_by`）。本 RFC 主张把它们升级为"真实模块的 SSOT 母本"，配套生成叙事 PRD。

---

## 5. 工作流：从 PRD 到上线

```
PM 写 PRD.md / FRD
   │ ① LLM + 人工 翻译为 .intent
   ▼
intents/*.intent  ←── SSOT，所有人 review 这一层
   │ ② intent check
   ▼
✅ 一致 / ⚠️ 反例（修正 PRD 或修正 intent）
   │
   ├── intent coverage  ──▶ 维度未覆盖清单 → 反馈 PM 补 PRD
   ├── intent testspec  ──▶ tests/generated/*.feature
   ├── intent export    ──▶ docs/api/openapi.yaml
   ├── intent explain   ──▶ docs/PRD.md
   ├── intent diff v1 v2 ─▶ ChangeLog 自动分类（破坏 / 兼容 / 加严）
   └── intent impact    ──▶ "改这条 intent 影响：3 条下游 / 12 测试 / 5 端点 / 28 处代码"
```

### 5.1 PRD 评审会的"SMT 把关"

传统：PM 念稿，工程师靠记忆指出"这条好像和上次说的不一致？"
intent 化：评审屏幕上跑 `intent check`，**Z3 直接给反例**——一条具体的 subscription/account 状态触发矛盾。

### 5.2 PR 守门

```yaml
# .github/workflows/billing-pr.yml
on: pull_request
paths: ['billing/**']
jobs:
  intent-gate:
    steps:
      - run: intent check         intents/
      - run: intent diff   --base main intents/
      - run: intent impact --base main intents/
      - run: intent coverage --baseline .intent/coverage.lock intents/
```

把"PRD 评审"从一次性会议**变成持续运行的 gate**。

---

## 6. 边界声明（intent-lang 不该做的事）

按 `POSITIONING.md` 的纪律，下面这些**不要**塞进 `*.intent`：

| 议题 | 为什么不塞 | 推荐工具 |
|---|---|---|
| 幂等键的 TTL / 回放语义 | 过程性、依赖外部存储 | Idempotency-Key spec + 集成测试 |
| 支付网关重试退避策略 | 时序行为 | TLA+ / 状态机 DSL |
| 税收金额的具体数值 | 数据问题，不是逻辑 | TaxJar / Avalara + 数据验证 |
| 金额舍入的实现细节 | 精度细节属算法 | Dafny + property test |
| 限流 / 并发安全 | 资源调度 | TLA+ / Jepsen |
| UI 文案合规 | 不是逻辑断言 | 法务审稿 |

**审稿原则**：看到 PR 里有人在 `*.intent` 写"重试 3 次"或"前端 toast 一条提示"——退回。

---

## 7. 由整合实践提出的语言 / 工具扩展建议

下面 5 条来自把 billing 模块按上述骨架走通时遇到的真实摩擦。优先级以 dogfood 反馈为准，本节只列**问题与建议形态**，不锁定实现节奏。

### E1. `narrative` 块 —— 让 intent 反向喂叙事 PRD

**问题**：`intent explain --as prd` 生成的文本千篇一律，缺乏业务温度。

**建议**：允许在 intent 内附带 `narrative` 元数据，作为生成 PRD 段落的"叙事种子"。

```intent
@tobe
intent ChangePlan(...) {
  narrative {
    summary: "用户在订阅期内切换到不同档位计划"
    user_story: "作为团队管理员，我升级到 Pro，立即解锁高级功能"
    when_to_use: ["从定价页", "从账单中心", "从产品内升级提示"]
  }
  require ...
  ensure ...
}
```

CI 守护 narrative 字段非空——把"人写 PRD"的工作量挪到 intent 文件里，且强制和规则同步更新。

**和既有 RFC 的关系**：是 A1 (`goal`) 的下沉版本——`goal` 描述"为什么需要"，`narrative` 描述"怎么讲给读者"。

### E2. `refines` 关键字 —— 多级精化的形式化

**问题**：`docs/software/README.md` 描述了 L1（业务）→ L2（API）→ L3（组件）的 4 级精化链，但语言层目前没有显式连接两级 intent 的语法。

**建议**：

```intent
intent POST_ChangePlan(req: HttpRequest, res: HttpResponse) {
  require req.auth.valid
  require req.idempotencyKey.unique
  ensure  res.status == 200 ==> ChangePlan(...)
  ensure  res.status >= 400 ==> unchanged(allSubscriptions)
} refines ChangePlan when res.status == 200
```

`intent check --refinement` 时，Z3 验证：在 `when` 条件成立的所有状态下，子 intent 的后置条件 ⊨ 父 intent 的后置条件。

**收益**：
- 业务层（L1）和 API 层（L2）的"是否一致"从口头评审变成机器证明
- ChangeLog 可以分级别报告（"破坏了 L1"远比"破坏了 L2"严重）

### E3. `source` 元数据 —— 双向追溯

**问题**：intent → 代码的链路在 `docs/software/README.md` 已设想好（`x-intent` 注解），但 intent → PRD/Figma/Issue 的反向链接还缺。

**建议**：

```intent
@tobe
intent ChangePlan(...) {
  source {
    prd:    "billing/PRD.md#plan-change"
    figma:  "https://figma.com/file/xxx?node=42"
    issue:  "LIN-1234"
    rfc:    "docs/rfc-prd-integration-and-language-extensions.md"
  }
  ...
}
```

`intent impact` 报告时附 source 链接；PR review 时一键跳转到原始上下文。

### E4. `goal` 的关系查询能力

**问题**：当前 `goal` 块的 `realized_by` 是手维护的清单，缺乏工具支持，容易腐烂。

**建议**：工具侧加几个查询命令：

```bash
$ intent goals --orphan       # 哪些 safety/intent 没有任何 goal 覆盖
$ intent goals --unrealized   # 哪些 goal 的 realized_by 全为空
$ intent goals --diagram      # 输出 goal → safety → intent 的依赖图（mermaid）
```

`--diagram` 输出可直接嵌入 PRD，让"为什么这条规则存在"可视化。

**和既有 RFC 的关系**：是 A1（`goal` 一等公民）和 B4（影响分析）的衔接。

### E5. 把 `coverage` 从"字面文本扫描"升级为"可证伪关系"

**问题**：现状的 `coverage` 像维度笛卡尔积 + 字面文本扫描，但它**不**是 SMT 级别的完备性，容易给团队"覆盖了"的错觉。

**建议**：让 `coverage` 显式声明每个组合应被哪些规则覆盖，并允许显式豁免：

```intent
coverage "plan-change-matrix" {
  dimensions {
    direction:       [Upgrade, Downgrade, SameTier]
    cadence_change:  [same, monthly_to_yearly, yearly_to_monthly]
    state:           [Trialing, Active, PastDue]
    payment_method:  [valid, invalid]
  }

  must_be_handled_by: [ChangePlan]

  exempt {
    when: direction == SameTier && cadence_change == same
    rationale: "无意义组合：没改任何东西"
  }
}
```

工具能给出"这 3×3×3×2 = 54 个组合中，去掉 6 个豁免后还剩 48，其中 3 个没被 ChangePlan 的任何 require/ensure 显式触达"。

**和既有 RFC 的关系**：是 A3（完备性约束声明）和 B2（完备性检查）的演进版。

---

## 8. 优先级与实施路径

延续既有 RFC 的"先 dogfood 再设计"纪律：

### Phase 0（即刻）：文档落地
- 将本 RFC 收入 `docs/`
- 在 `docs/software/README.md` 顶部加链接，引导读者从"4 阶段验证链"读到"PRD 整合架构"
- 在 `examples/requirements/billing.intent` 顶部加链接，标注它是本 RFC 的样本实现

### Phase 1（在 1 个真实模块上 dogfood，不动语言层）
- 选一个内部小模块（`billing` 或 `access-control`），按第 4 节骨架完整跑一遍
- 只用现有语法（`goal` / `@tobe` / `@asis` / `coverage` / `theorem`）
- 重点观察哪条 E1–E5 建议先变痛

### Phase 2（按痛点节奏补语言/工具特性）
- 大概率优先级：E3 (source 元数据) > E1 (narrative) > E2 (refines) > E4 (goal 查询) > E5 (coverage 升级)
- 但实际优先级以 dogfood 反馈为准

### Phase 3（外部化）
- 当 1–2 个内部模块跑顺后，沉淀为公共 examples/templates
- 推动其他团队迁移

---

## 9. 反模式 / 克制清单

延续既有 RFC 的纪律，本 RFC 额外补充几条 PRD 整合视角的克制：

1. **不要让 intent-lang 长成 PRD 的全部** —— ADR、Runbook、Concepts、Anti-patterns 仍是人写的叙事产物，intent 只占真相层。
2. **不要追求 PRD.md 100% 由 intent 生成** —— 人类需要有"为什么"的叙事空间，机器派生产物的目标是覆盖 70–80%，剩余靠人补。
3. **不要把 narrative 写成营销文案** —— narrative 是"中立的、面向同事的"叙事，不是给客户的卖点。
4. **不要在 `*.intent` 里塞实现细节、UI 行为、运维策略** —— 见第 6 节边界表。
5. **不要让 `coverage` 声明的维度无限膨胀** —— 维度爆炸时优先拆分能力（capability），而不是堆维度。
6. **不要在 PR 里只看叙事 PRD 的 diff** —— 叙事是派生物，diff 没意义；review 必须落到 `*.intent` 的 diff。

---

## 10. Open Questions

1. `narrative` 块应该是 intent 的子块，还是独立块（`narrative ChangePlan { ... }`）？前者紧耦合，后者便于多语言维护（中英 narrative 并存）。
2. `refines` 关键字是否需要支持"多对一"（多个 L2 intent 共同精化一个 L1）？业务上常见，但形式化语义复杂。
3. `intent explain --as prd` 应该输出一个大文件还是按能力切分多个文件？大文件便于全局阅读，多文件便于增量 diff。
4. 派生的 `docs/PRD.md` 是否纳入 git？纳入则 review 友好但易冲突；不纳入则需要在 PR Bot 里实时渲染。
5. `source` 元数据中的 URL 类引用是否需要 CI 检查链接活性（可达性）？过严会 flaky，过松易腐烂。
6. 当 `goal.realized_by` 与 intent 上的 `narrative.user_story` 都存在时，PRD 生成时哪个优先？需要明确合并策略。
7. 与既有 RFC 的 D1 (artifact schema) 关系：本 RFC 提出的 `intent.coverage_report` 是否应包含 E5 提议的 exempt 信息？

---

## 11. 历史背景

本 RFC 来自一次关于"AI 时代产品文档应如何组织"的设计讨论。讨论顺序：

1. **AI 时代 PRD 的组织原则**（分层 / 原子化 / 结构化元数据 / SSOT）
2. **传统叙事 PRD vs 结构化 PRD 的对比**（以 SaaS billing 模块为例）
3. **结构化 PRD 仍有的"真相缺口"**（机器无法验证业务规则间一致性）
4. **intent-lang 嵌入结构化 PRD 作为真相层的具体方案**（目录骨架 + 工作流）
5. **由整合实践反推出的 5 条语言/工具扩展建议**（E1–E5）

姊妹 RFC：

- `docs/rfc-idd-positioning-and-improvements.md`（定位收敛、A1–A4 语言层、B1–B5 工具链）

本 RFC 的语言扩展建议（E1–E5）与既有 RFC 的 A1–A4 关系：

| 本 RFC | 关系 | 既有 RFC |
|---|---|---|
| E1 narrative | 下沉于 | A1 `goal` 一等公民 |
| E2 refines | 全新 | — |
| E3 source 元数据 | 全新 | — |
| E4 goal 关系查询 | 衔接 | A1 + B4 影响分析 |
| E5 coverage 升级 | 演进 | A3 完备性声明 + B2 完备性检查 |
