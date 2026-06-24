# arch-debate 使用指南

多角色**技术架构**辩论模拟器，是产品侧 `product-debate` 的技术侧对称体。在写 RFC
之前，用 4-6 个技术角色就一个架构问题充分辩论方案空间，暴露盲点，产出可审计的
辩论纪要 + RFC 草稿 + 技术决策矩阵。

`rfc-draft` § Proposed Design 建议附 Mermaid 草图：

```bash
popsicle tool run mermaid-diagram action=scaffold type=architecture title="<辩论主题>"
```

## 它在 Phase 3 链条里的位置

```
prd-writer →（PRD § Intent Mapping 标 [ADR 候选] 的行）→
arch-debate →（rfc-draft）→ rfc-writer →（ADR 骨架 + contracts 种子）→
adr-writer →（Accepted ADR + 解锁 contracts 种子）→ intent-spec-writer →（收紧合并）→ intent-consistency-check
```

arch-debate 只解决「产品侧留下的技术待决问题」——即 PRD 里标了 `contracts.intent` /
`[ADR 候选]` 的条目。它**不写 ARCHITECTURE.md**（那是 rfc-writer 的活），只产辩论摘要。

## 输入模式

| 模式 | 适用场景 | Ground truth |
|---|---|---|
| `legacy-fact-baseline` | 迁移 / slice spec | PRD + fact-extraction-report / api-contracts |
| `greenfield-architecture-brief` | 新产品 / 新模块架构支线 | PRD overview + Product Brief |

greenfield 模式没有 legacy API / risk hotspots 是正常状态。不要把它当成低质量输入；
只需把 PSP、存储、队列、税务服务、SLA、容量等超出 PRD/Product Brief 的判断标成
`[待验证]`，并在 RFC draft 的 Open Questions 或 ADR 候选里保留。

## 与 product-debate 的异同

| 维度 | product-debate | arch-debate |
|---|---|---|
| 角色 | PM/UXR/GROWTH/ENGLD/BIZ | ARCH/SEC/PERF/OPS/DATA/DEV |
| 主持 | PM | ARCH |
| 代言人 | UXR（代言用户）| SEC（代言攻击者）|
| 产出 intent 层 | acceptance / invariants | **contracts** / invariants |
| 决策载体 | PDR | **ADR**（触及 charter → CADR）|
| 下游 | prd-writer | rfc-writer |

机制完全复用 product-debate：4 Phase + 每 Phase 强制暂停 + 置信度调风格 + setup 需
人类 `start` 确认。角色定义见 `references/tech-roles.md`。

## 三条 IDD 硬纪律

1. **数字/LoC/模块名/churn 必须可追溯**：legacy 模式 cite fact-extraction-report
   或 api-contracts；greenfield 模式 cite PRD/Product Brief，超出 brief 的判断标
   `[待验证]`。
2. **每个核心技术声明在 Phase 4 由 ARCH 归类到 intent 层**——本链主产物是
   `contracts.intent`（模块间契约）；跨切面不变量进 `invariants.intent`。
3. **性能/时延/容量不进 `.intent`**（intent-lang 不验时间，D2）——写进 RFC「质量
   属性目标」，由压测/SLO 守护。触及 charter 铁律/Layer Map 的方案标「需要 CADR」。

## 产物

- `{slug}.arch-debate.md` — 辩论纪要（审计轨迹）
- `{slug}.rfc-draft.md` — RFC 草稿，含 § Intent & Decision Mapping（contracts 候选 +
  ADR/CADR 候选清单），喂给 rfc-writer
- `{slug}.tech-decision-matrix.md` — 方案 × NFR 打分矩阵（候选 ≥ 2 时）
