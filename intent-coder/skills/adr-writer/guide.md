# adr-writer 使用指南

把 `rfc-writer` 的 ADR 骨架（Status: Proposed）**固化**为正式 ADR（Accepted，此后
不可变），并**解锁** `contracts.intent` 种子的 `[Awaiting ADR-XXXX]` 状态。是
contracts.intent 闭环里「解锁」这一棒。

它像 `intent-spec-writer` 一样**刻意保持薄**：不发明 ADR 内容（那是 rfc-writer），
只做三件事——固化门审查、Status 固化、解锁 contracts 交下游。

核对图示时用 **`mermaid-diagram` tool**（adr-writer 不新画图）：

```bash
popsicle tool run mermaid-diagram action=validate path=products/<product>/decisions/adr/ADR-….md
```

固化门须核对 ADR § Architecture Snapshot / RFC 内 mermaid 与 § Decision、§ Consequences
**一致**；缺失或矛盾则退回 rfc-writer。技能全文：`popsicle tool run mermaid-diagram action=guide`

## 在 Phase 3 链条里的位置

```
rfc-writer →（ADR 骨架 Proposed + contracts 种子 [Awaiting ADR]）→
adr-writer →（ADR Accepted + contracts 解锁 + 收紧工单）→
intent-spec-writer →（收紧进 acceptance/invariants）→ intent-consistency-check（Z3 闸）
```

## 固化门（五项，任一不过就退回 rfc-writer）

1. **决策无歧义** — § Decision 现在时、明确，无「将会/计划/视情况」。
2. **Consequences 落地** — 列的每个文件路径真实可落地，不指向虚空。
3. **Intent Impact 一致** — 与 RFC § Intent & Decision Mapping + contracts goal 对应。
4. **CADR 合规** — 没在普通 ADR 里偷改 charter 四铁律 / Layer Map（触及则走 CADR）。
5. **Decision Context 充分** — 触发因素 + 辩论摘要 + 备选否决理由齐全。
6. **图示与文字一致** — ADR § Architecture Snapshot（或 RFC § Proposed Design）中的
   Mermaid 节点/边与 § Decision、§ Consequences 无矛盾；语法符合 mermaid-diagrams 指南。

## 固化 + 解锁

- **固化**：编辑现有 ADR 骨架文件，Status Proposed→Accepted，填审批信息，落盘到
  `decisions/adr/`。**固化后永不修改**（charter 第 2 条铁律）——纠错请新建 ADR 标
  `Supersedes`。
- **解锁**：把 contracts 种子里被本 ADR 阻塞的 goal 注释从 `[Awaiting ADR-XXXX]` 改为
  `[ADR-XXXX Accepted]`，并列出「可收紧逻辑保证」工单（哪些契约前后置可进
  acceptance/invariants）。**不自己收紧**——那是 intent-spec-writer 的活。

## 产物

- `{slug}.adr-finalization-report.md` — 固化检查 + 解锁动作 + 移交工单 + Intent Impact 核对
- `{slug}.contracts-unlocked.intent` — 解锁后的 contracts 种子（能 `intent check`，0 VC），
  交 intent-spec-writer

## 为什么单独成一个 skill（不让 rfc-writer 直接产 Accepted ADR）

决策固化是**审批闸**，必须与起草分离：rfc-writer 起草（可能反复改），adr-writer 把关
（一旦 Accepted 即不可变 + 触发下游收紧）。这对应产品侧 PDR 的 Proposed→Accepted
也需要人审批落地——技术侧把这一步显式成 skill，并兼做 contracts 解锁的触发器。
