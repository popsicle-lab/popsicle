# PDR-002: 手机需求 Chat Intake 与立项派活

> **Status**: Proposed
> **Date**: 2026-07-07
> **Target Product**: `agent-runtime`
> **Decision Type**: Product Decision Record (PDR)
> **Related ADRs**: ADR-001（执行面分层，扩展 chat/bootstrap API）
> **Related CADR**: CADR-001（IDD-only，Chat 仍收敛到 popsicle Issue + pipeline）
> **Related Journey**: ——

---

## Decision Context

### 触发因素

P8（PROJ-92）仅支持对 **已有 Issue Key** 远程派活。用户日常在 Cursor IDE 用自然语言开始需求（issue-author → issue create → pipeline），Mobile 无法覆盖 **Issue 之前的需求澄清**。对标 Multica Chat 的体验需求存在，但 PDR-001 已否决 Multica 桥接（双 Issue 模型）。

### 多角色辩论摘要

**未经多角色 product-debate**（feature-spec 路径；需求已在 PROJ-94 Issue 描述与用户对话中收敛）。

**参与角色（implicit）**: PM, ENGLD

**用户置信度**: 4/5

**关键分歧**:

- Multica 式 Server 真相源 vs popsicle 本机 `state.db` → 选后者，Chat 仅 Intake 会话
- 一句话立刻开跑 vs 草案确认 → 选 **实时 Chat + draft 确认后 bootstrap**

**核心事实引用**:

- F-1: P8 dispatch 要求已有 `issue_key`（T-AR-0002）
- F-2: Daemon orchestrator 已支持 issue start 后无人值守（PROJ-89）
- F-3: CADR-001 禁止通用 workflow 平台化

### 备选方案

| 方案 | 提案者 | 否决理由 |
|------|--------|---------|
| Multica 桥接 | GROWTH | 双 Issue 模型，IDD 门禁不对齐（PDR-001） |
| Mobile 仅收集 transcript，无实时 Agent | ENGLD | 不符合用户「Chat 里实时回话」倾向 |
| Server 直接 issue create | PM | 违反真相源在本机；须 Daemon subprocess |

---

## Decision

在 agent-server 增加 **Chat Intake 阶段**（`chat_sessions` / `chat_turn` / `bootstrap` 队列）；Mobile 多轮实时 Chat 由 Daemon 本机 `cursor-agent` 驱动；用户确认 draft 后 Daemon 执行 `issue create` + `issue start` + 现有 orchestrator。不引入第二套 Issue 模型。

---

## Consequences

### Task File Updates

#### 新增 Tasks

- [x] `products/agent-runtime/tasks/onboarding/T-AR-0007-mobile-intake-chat.md`
- [x] `products/agent-runtime/tasks/daily-ops/T-AR-0008-bootstrap-issue-from-chat.md`

### PRODUCT.md Top-Level Updates

- [x] User Intents Catalog — T-AR-0007 / T-AR-0008
- [x] Committed Roadmap — P9 Intake Chat
- [x] Deferred — 仍不做 Multica 桥接

### Tasks Index Updates

- [x] `products/agent-runtime/tasks/README.md`

### Intent Updates

- [ ] `acceptance.intent` 追加 3 block（种子，intent-spec 收紧）
- [ ] `contracts.intent` 追加 Chat/Bootstrap API goal（intent-spec 收紧）

### Code Updates (informational — feature-delivery PROJ-94 后续)

- `crates/agent-server/` — chat REST + schema + WS events
- `crates/agent-daemon/` — chat_turn + bootstrap handlers
- `apps/mobile/` — 需求 Tab Chat UI
- `deploy/agent-runtime/schema.sql` — chat_* tables
- `products/agent-runtime/ARCHITECTURE.md` — P9 API manifest

### Risk Side-Effects

| Risk | 触发条件 | 缓解 |
|------|---------|------|
| pipeline 选错 | Agent issue-author 推断错误 | Mobile 可改 draft_pipeline；CLI 门禁报错可观测 |
| headless Agent 弱于 IDE | cursor-agent -p 无 thinking | stream-json 日志 + T-AR-0005 诊断路径扩展 |
| Chat 与旧 dispatch 混用 | 同一 workspace 双入口 | UX 分离「需求 Tab」vs「派活 Tab」；文档说明 |

---

## Intent Impact

| 层 | 修改 | block | Task |
|---|---|---|---|
| acceptance.intent | 新增 | ChatTurnQueuedWhenRuntimeOnline | T-AR-0007 |
| acceptance.intent | 新增 | ChatTurnRejectedWhenRuntimeOffline | T-AR-0007 |
| acceptance.intent | 新增 | BootstrapCreatesIssueAndRun | T-AR-0008 |
| contracts.intent | 新增 | ChatIntakeApiSurface | T-AR-0007 / T-AR-0008 |

---

## Validation Plan

- `popsicle tool run intent-validate path=products/agent-runtime` 通过（intent-spec 后）
- Dogfood：Mobile Chat 3 轮 → bootstrap → 进度 Tab 见 run（feature-delivery）

---

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-95 feature-delivery
- **Approval date**: 2026-07-07

`Decision-Ref: PROJ-94 feature-spec`
