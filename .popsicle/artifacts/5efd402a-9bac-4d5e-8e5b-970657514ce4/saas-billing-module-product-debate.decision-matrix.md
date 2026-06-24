# 决策矩阵 — SaaS billing module product debate

> **Topic**: SaaS billing module product debate
> **Target Product**: `saas-billing-module`
> **Source Debate**: `saas-billing-module-product-debate.product-debate.md`
> **生成条件**: 候选方案 ≥ 2

---

## 评分标准

- 3 = 优秀
- 2 = 一般
- 1 = 较差

## 评估矩阵

| 评估维度 | 权重 | 维度提出者 | 权重理由 | A: Subscription-first | B: Ledger-and-audit-first | C: Modular contract-first |
|---|---:|---|---|---:|---:|---:|
| 用户可理解性 | 3 | UXR | Billing admin 首先需要能完成日常任务 | 3 | 2 | 1 |
| 财务/审计风险 | 3 | FINOPS/RISK | Billing 的错误成本高 | 2 | 3 | 2 |
| Intent 可验证性 | 3 | PM/ENGLD | 本 dogfood 目标是验证 intent-coder greenfield 链 | 2 | 3 | 3 |
| 首版交付复杂度 | 2 | PM | 需要尽快跑通 workflow | 3 | 1 | 1 |
| 架构演进空间 | 2 | ENGLD | 后续 contracts.intent 需要清晰边界 | 2 | 2 | 3 |

---

## 加权总分

| 方案 | 加权计算 | 总分 |
|---|---|---:|
| A | 3×3 + 2×3 + 2×3 + 3×2 + 2×2 | 31 |
| B | 2×3 + 3×3 + 3×3 + 1×2 + 2×2 | 30 |
| C | 1×3 + 2×3 + 3×3 + 1×2 + 3×2 | 26 |

---

## 综合推荐

**推荐方案**: A + B 修正，即 Subscription-first user journey + audit-first invariant baseline。

**与最高分一致？**: 部分一致。A 得分最高，但 FINOPS/RISK 的 audit-first 要求被纳入推荐方案，避免把 billing 风险后置。

---

## 维度来源说明

- **用户可理解性**（UXR）：订阅计费必须能被 billing admin 和 support 按任务完成。
- **财务/审计风险**（FINOPS/RISK）：invoice、payment、credit 的错误会直接造成收入和信任损失。
- **Intent 可验证性**（PM/ENGLD）：本 run 的目标是验证 greenfield workflow 能产出可形式化规则。
- **首版交付复杂度**（PM）：先完成 PRD / intent 链路，再进入架构支线。
- **架构演进空间**（ENGLD）：contracts 候选必须保留，但不要在产品辩论里写死技术方案。

---

## 评分依据归档

- A 的用户可理解性最高：从创建 plan 到处理 payment failure 是自然的 billing admin 任务链。
- B 的财务/审计风险最低：audit event 和 immutable money movement 前置。
- C 的 intent/contracts 可验证性强，但首版进入 architecture 支线的负担最大。
- 推荐方案把 A 的任务图与 B 的 invariant baseline 合并，并把 C 的 contracts 留给 arch-debate。
