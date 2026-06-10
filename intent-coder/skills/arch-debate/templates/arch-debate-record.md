---
artifact: arch-debate-record
slug: {slug}
topic: "{一句话技术议题}"
participants: [ARCH, SEC, PERF, OPS, DEV]
confidence: 3
input_mode: legacy-fact-baseline / greenfield-architecture-brief
date: {YYYY-MM-DD}
query_anchors:
  - "这个架构问题当时为什么这么定？"
  - "哪些方案被否了，理由是什么？"
  - "这个决策要走 ADR 还是 CADR？"
---

# 架构辩论纪要 — {slug}

> 由 `arch-debate` skill 生成。本纪要是技术决策的**审计轨迹**，供 rfc-writer /
> adr-writer 追溯论据，也供后人理解「当时为什么这么选」。

## Topic

{一句话技术议题。注明来源：prd-overview § Intent Mapping 第 N 行 [ADR 候选]。}

## Participants

| 角色 | 立场速写 |
|---|---|
| ARCH | …… |
| SEC | …… |
| PERF | …… |
| OPS | …… |
| DEV | …… |

用户置信度：{N}/5

## Phase 1 — 技术问题 + 质量属性（NFR）

- 要解决的问题：……
- 硬约束：（兼容性 / 合规 / charter Layer Map）……
- 质量属性优先级：性能 / 可用性 / 安全 / 可演进性 ……（排序）
- 事实基引用：F-1 ……（fact-extraction-report § … / PRD Overview / Product Brief）

## Phase 2 — 方案发散

- **方案 A**（提案者 {角色}）：模块边界 + 数据流 ……
- **方案 B**（提案者 {角色}）：……
- **方案 C**（提案者 {角色}）：……

## Phase 3 — 多角色评审

| 方案 | SEC | PERF | OPS | DATA | DEV |
|---|---|---|---|---|---|
| A | 威胁… | 容量… | 可观测… | 一致性… | 成本… |
| B | … | … | … | … | … |

## Phase 4 — 收敛与决策

- ARCH 综合：……
- 角色投票：……
- **用户最终决策**：……（若覆盖多数角色意见，在「用户决策点」标明）

## Decision

> 一到三句话陈述「选了什么」，现在时，不解释理由（理由在上文）。

{决策陈述}

## 关键分歧

- {分歧 1}：{角色 A 立场} vs {角色 B 立场} → {如何收敛 / 仍悬置}

## 用户决策点

- [ ] 用户决策是否覆盖了多数角色意见？覆盖了 → 记录理由与冷静期建议。

## 下游接驳建议

- rfc-writer：把本纪要 + rfc-draft 打磨成正式 RFC + contracts 种子 + ADR 骨架。
- 需要 CADR 的条目（如有）：先走 charter 修订。

## Output Checklist

- [ ] Phase 1-4 小结齐全
- [ ] 关键分歧与各方立场已记录
- [ ] 用户决策点已显式记录（含覆盖情况）
- [ ] 每个数字/模块名引用可追溯到事实基（legacy：fact report/api-contracts；greenfield：PRD/Product Brief）
- [ ] Topic 与另两份 artifact 一致
