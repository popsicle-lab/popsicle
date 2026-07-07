# CADR-001: agent-runtime 为 IDD 专用远程派活（非通用工作流平台）

> **Status**: Accepted
> **Date**: 2026-07-06
> **Decision Type**: Charter Amendment Decision Record (CADR)
> **Target**: `docs/CHARTER.md` — L0 Product Scope Boundaries
> **Related PDR**: PDR-001（Accepted 同日）
> **Related ADR**: ADR-001（agent-runtime 执行面分层）
> **Source**: PROJ-81 product-debate、PROJ-87 arch-debate（doc-208）

---

## Context

PDR-001 引入 **agent-runtime**：手机远程派活 + 本机 Daemon 执行 popsicle pipeline 与 Agent CLI。product-debate（PROJ-81）识别风险：若无 charter 级边界，产品易被扩展为**通用任务看板 / 非 IDD 工作流引擎**，与 popsicle 的 spec-driven IDD 定位冲突。

ADR-001 将本能力落为 `agent-server` + `agent-daemon`，并标注 **CADR-001 候选**。实现 P0–P4（PROJ-82–86）已交付，现正式修订 charter。

### 备选方案

| 方案 | 否决理由 |
|------|---------|
| 仅在 `PRODUCT.md` 声明 IDD-only | 不约束 L0；CI/贡献者无 charter 级依据 |
| Multica / 通用看板桥接 | 双 Issue 模型，IDD 门禁难对齐（PDR-001 已否决）|
| 写入普通 ADR-002 | 触及 charter Layer Map / 产品存在理由，须 CADR |

---

## Decision

1. **charter 增补**「Product Scope Boundaries」节（见 Consequences），明确 `agent-runtime` 产品边界。
2. **PDR-001** 同日 **Accepted**，作为 agent-runtime MVP 的产品决策档案。
3. agent-runtime **不得**承载：任意第三方 workflow 定义、非 popsicle Issue/Pipeline 任务类型、与 IDD 无关的通用自动化编排。

---

## Consequences

### Charter Updates（活文档 — 引用本 CADR）

| 路径 | 变更 |
|------|------|
| `docs/CHARTER.md` | 新增 § Product Scope Boundaries [CADR-001] |

### Product Updates

| 路径 | 变更 |
|------|------|
| `products/agent-runtime/PRODUCT.md` | `Last-Decision-Ref: CADR-001`；Roadmap 移除 CADR 候选 |
| `products/agent-runtime/ARCHITECTURE.md` | 关闭 Open Question「CADR-001 是否一并 Accepted」|

### PDR Updates

| 路径 | 变更 |
|------|------|
| `products/agent-runtime/decisions/pdr/PDR-001-agent-runtime-mvp.md` | `Status: Accepted`；Approval 块；contracts 后果勾选 |

### Intent Impact

| 层 | 修改 | 说明 |
|---|---|---|
| `products/agent-runtime/intents/invariants.intent` | 无新增 block | `SecretsStayOnRuntimeMachine` 已覆盖密钥边界；IDD-only 由 charter + PDR 陈述 |
| `docs/invariants/*.intent`（全局）| 无影响 | 不触及全局 invariant |

---

## Compliance

| 门禁 | 证据 |
|---|---|
| 触及 charter | 是 → 本文件为 CADR，非普通 ADR |
| product-debate | PROJ-81 doc-181 |
| arch-debate | PROJ-87 doc-208 |
| P0–P4 实现 | PROJ-82–86 closed |

---

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-87 pipeline（delegate-dangerous）
- **Approval date**: 2026-07-06
