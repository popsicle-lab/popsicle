---
artifact: tech-decision-matrix
slug: saas-billing-module-architecture-debate
topic: "Define SaaS billing module boundaries, event ledger, PSP adapter, and tax-ready audit contracts"
generated_by: arch-debate
date: 2026-06-10
---

# 技术决策矩阵 — saas-billing-module-architecture-debate

> 候选架构方案 × 质量属性维度（NFR）的打分。维度由参与角色提出，权重由 ARCH 在 Phase 4 决定。每格分值 1-5。

## 维度与权重

| 维度（NFR）| 提出角色 | 权重 |
|---|---|---:|
| Correctness / auditability | SEC + DATA | 0.30 |
| Evolvability / contract clarity | ARCH | 0.25 |
| Operability / retry observability | OPS | 0.20 |
| Implementation cost | DEV | 0.15 |
| Performance / projection scalability | PERF | 0.10 |

## 打分矩阵

| 维度 \ 方案 | A: Modular monolith + event ledger | B: Service-per-domain + integration events | C: Invoice-centric CRUD core |
|---|---:|---:|---:|
| Correctness / auditability (0.30) | 5 — append-only events make audit explicit | 4 — good audit potential but distributed consistency risk | 2 — direct updates weaken traceability |
| Evolvability / contract clarity (0.25) | 5 — ports and modules are explicit | 4 — service contracts clear but heavy governance | 2 — invoice table becomes implicit boundary |
| Operability / retry observability (0.20) | 4 — adapter failures become events | 3 — queue/service ops overhead appears early | 3 — few components but less structured retry trace |
| Implementation cost (0.15) | 4 — modular monolith is feasible first step | 2 — too heavy for greenfield first spec | 5 — fastest implementation |
| Performance / projection scalability (0.10) | 4 — read model projection path exists | 4 — services can scale independently later | 4 — direct reads are simple |
| **Weighted Score** | **4.60** | **3.50** | **2.85** |

## 结论

推荐方案：**A: Modular monolith + append-only billing event ledger**。

矩阵是决策输入，不是终判；本次没有用户覆盖多数角色意见。
