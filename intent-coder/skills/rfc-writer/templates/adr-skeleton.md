# ADR-{id}: {title}

> **Status**: Proposed
> **Date**: {YYYY-MM-DD}
> **Target Product**: `{target_product}`
> **Decision Type**: Architecture Decision Record (ADR)
> **Supersedes**: —— / `ADR-XXXX`
> **Related PDRs**: —— / `PDR-XXXX`（本 ADR 服务的产品决策）
> **Related RFC**: `{slug}.rfc.md`
> **Related Journey**: —— / `J-XXXX`

---

## Decision Context

### 触发因素

{是什么技术问题 / 约束 / 风险让我们必须现在做这个架构决策？}

### 多角色辩论摘要

> 经 `arch-debate` 时从 `{slug}.arch-debate.md` 继承；否则标「未经架构辩论」。

**参与角色**: {ARCH, SEC, PERF, OPS, DATA, DEV ...}

**关键分歧**:
- {分歧 1}: {角色 A 立场 vs 角色 B 立场}

**核心事实引用**（fact-extraction-report / api-contracts）:
- F-1: {引用}

### 备选方案

> 详细打分见 `{slug}.tech-decision-matrix.md`。

| 方案 | 提案者 | 否决理由 |
|------|--------|---------|
| {方案 B} | {角色} | {理由} |

---

## Decision

> 一到三句话陈述「选了什么」。现在时，**不解释理由**（理由在 Decision Context）。

{决策陈述，例：「auth 对外只通过 `verify_session(token) -> Option<UserContext>`
暴露会话校验；下游产品不得直接读 session 存储。」}

---

## Consequences

> 本 ADR Accepted 后，被强制更新的文档/intent/代码必须在同一 PR 同步（charter 铁律）。
> Consequences 精确到文件级。

### ARCHITECTURE.md Updates
- [ ] `products/{target_product}/ARCHITECTURE.md` § {章节} — {增量}

### Intent Updates
- [ ] `products/{target_product}/intents/contracts.intent` 解锁并收紧 goal：`{goal 名}`
      （`[Awaiting ADR-{id}]` → 由 intent-spec-writer 收紧为可验证形态）
- [ ] `products/{target_product}/intents/invariants.intent` 新增 safety：`{block}`（如适用）

### Code Updates (informational, not enforced)
- 模块 `crates/{mod}/`：实现 {接口}

### Risk Side-Effects
| Risk | 触发条件 | 缓解 |
|------|---------|------|
| {风险} | {何时} | {措施} |

---

## Intent Impact

> charter 强制：每份 ADR 显式声明它修改哪一层 intent。CI 拒绝缺这一项的决策。

| Intent 层 | 修改类型 | 涉及 block | 备注 |
|-----------|---------|----------|------|
| `intents/contracts.intent` | 解锁+收紧 | `{goal/intent 名}` | 本 ADR 落地后由 intent-spec-writer 收紧 |
| `intents/invariants.intent` | 新增 | `{safety 名}` | 如适用 |
| `docs/invariants/*.intent` (全局) | 无影响 | —— | 涉及则需升级 CADR |

> ⚠️ 若本决策触及 charter「四条铁律」/「Layer Map」，它本身需升级为 **CADR**。

---

## Validation Plan

### Contracts 验证（ADR Accepted 后）
- 跑 `intent check products/{target_product}/intents/contracts.intent`：goal 块解析通过；
  收紧后的 contracts intent/safety 与既有 acceptance/invariants 无矛盾。

### 质量属性验证（NFR）
- {压测 / SLO 监控项}：目标 {阈值}（D2：性能由测试守护，不进 intent）。

### 回滚条件
如果 {度量} 在 {窗口} 内劣化超 {阈值}，回滚到 {前一状态}。回滚通过新建 ADR 标注
`Supersedes: ADR-{id}` 实现，**不修改本 ADR**。

---

## Approval

- **Status**: Proposed → Accepted（adr-writer 固化、用户审批后改）
- **Approved by**: {审批人}
- **Approval date**: {YYYY-MM-DD}

---

## References

- **Source RFC**: `{slug}.rfc.md`
- **Source Debate**: `{slug}.arch-debate.md`
- **Decision Matrix**: `{slug}.tech-decision-matrix.md`
- **Contracts Seed**: `{slug}.contracts.intent`

---

*本 ADR 由 rfc-writer 起草为 Proposed，adr-writer 固化为 Accepted。Charter 第 2 条
铁律：Accepted 之后永不修改；纠正错误请新建 ADR 并标注 Supersedes。*
