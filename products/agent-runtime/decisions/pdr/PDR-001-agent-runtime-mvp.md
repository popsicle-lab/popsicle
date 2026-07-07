# PDR-001: agent-runtime MVP — 手机派活与本机 Daemon

> **Status**: Accepted
> **Date**: 2026-07-06
> **Target Product**: `agent-runtime`
> **Decision Type**: Product Decision Record (PDR)
> **Related ADRs**: ADR-001 [Accepted 2026-07-06]
> **Related CADR**: CADR-001 [Accepted 2026-07-06]
> **Related Journey**: ——

---

## Decision Context

### 触发因素

PROJ-81：用户需要从手机远程派活，开发机自动执行 popsicle pipeline + Agent CLI（C 档），反转现有「Agent 手动调 popsicle」模型。

### 多角色辩论摘要

**来源**: `doc-181.product-debate.md`（PROJ-81 debate stage）

**参与角色**: PM, UXR, GROWTH, ENGLD, BIZ

**用户置信度**: 4/5

**关键分歧**: charter「非通用工作流平台」→ 收敛为 **IDD-only 派活** + **CADR-001 候选**

**核心事实引用**:
- F-1: 无现有 remote/daemon acceptance block
- F-2: legacy sync-collab 已砍（PDR-001 skill-runtime）
- F-3: `popsicle prompt` deferred（ADR-019）

### 备选方案

| 方案 | 提案者 | 否决理由 |
|------|--------|---------|
| Multica 桥接 | GROWTH | 双 Issue 模型，IDD 门禁难对齐 |
| 仅 Daemon 调 popsicle | ENGLD | 无法完成 implement stage |

---

## Decision

新增 **agent-runtime** product：Server 协调 Task Queue + 本机 Daemon subprocess 调用 `popsicle` 与 Agent CLI（首期 Cursor Agent）；Mobile 提供派活、进度与 `requires_approval` 远程确认。

---

## Consequences

### Task File Updates

#### 新增 Tasks

- [x] `products/agent-runtime/tasks/onboarding/T-AR-0001-install-daemon.md`
- [x] `products/agent-runtime/tasks/onboarding/T-AR-0002-first-mobile-dispatch.md`
- [x] `products/agent-runtime/tasks/daily-ops/T-AR-0003-view-run-progress.md`
- [x] `products/agent-runtime/tasks/daily-ops/T-AR-0004-mobile-stage-approval.md`
- [x] `products/agent-runtime/tasks/troubleshooting/T-AR-0005-dispatch-failure-diagnosis.md`
- [x] `products/agent-runtime/tasks/admin/T-AR-0006-self-host-server.md`

### PRODUCT.md Top-Level Updates

- [x] `products/agent-runtime/PRODUCT.md` — 一行用途、Catalog、Problem、Metrics

### Tasks Index Updates

- [x] `products/agent-runtime/tasks/README.md`

### Intent Updates

- [x] `products/agent-runtime/intents/acceptance.intent` — 4 block 种子
- [x] `products/agent-runtime/intents/invariants.intent` — SecretsStayOnRuntimeMachine
- [x] `products/agent-runtime/intents/contracts.intent` — ADR-001 Accepted 2026-07-06（DaemonSubprocessInvokesPopsicle / ServerNeverExecutesAgent）

### Risk Side-Effects

| Risk | 触发条件 | 缓解 |
|------|---------|------|
| charter 冲突 | 产品被用作通用看板 | CADR-001 Accepted：IDD-only [CADR-001] |
| Agent CLI 协议变更 | cursor-agent 升级 | adapter 插件 |

---

## Intent Impact

| Intent 层 | 修改类型 | 涉及 block | 关联 Task |
|-----------|---------|----------|----------|
| acceptance.intent | 新增 | RuntimeRegistersWhenDaemonStarts | T-AR-0001 |
| acceptance.intent | 新增 | DispatchQueuedWhenRuntimeOnline | T-AR-0002 |
| acceptance.intent | 新增 | DispatchRejectedWhenRuntimeOffline | T-AR-0005 |
| acceptance.intent | 新增 | ApprovalCreatesConfirmTask | T-AR-0004 |
| invariants.intent | 新增 | SecretsStayOnRuntimeMachine | 跨 task |
| contracts.intent | 已新增 | DaemonSubprocessInvokesPopsicle | T-AR-0001 | ADR-001 Accepted |

---

## Validation Plan

- `popsicle tool run intent-validate path=products/agent-runtime` exit 0（收紧后）
- P0 dogfood：T-AR-0001 + T-AR-0002 端到端一次

---

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-87 pipeline（delegate-dangerous）；与 CADR-001 同日 Accepted
- **Approval date**: 2026-07-06

---

## References

- **Source Debate**: `.popsicle/artifacts/.../doc-181.product-debate.md`
- **PRD Overview**: `.popsicle/artifacts/.../doc-184.prd-writer.md`
- **Issue**: PROJ-81
