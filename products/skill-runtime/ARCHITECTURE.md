# Architecture: skill-runtime

> **Layer**: L4（实现视角）
> **Audience**: 工程师、AI agent
> **Status**: in-shadow 实现已落地（ADR-005 cutover-done 2026-06-09；CLI 仍 legacy）
> **Last-Updated**: 2026-06-09
> **Last-Decision-Ref**: ADR-005（skill-runtime cutover，Accepted 2026-06-09）

## 责任边界

[TBD: needs archaeology]

> 预期界定：什么属于 skill-runtime（属于本 product 的 crate / 模块），什么不是
> （由 artifact-system / cli-ux 承载）。

## 模块图

[TBD: 由 rfc-writer 在 RFC 文档里产，作为 ARCHITECTURE.md 增量提交]

## File Manifest（受 RFC 控制的目录与 crate）

> 布局由 ADR-003 固化：member crate 落根级 `crates/<slice>/`（crate = slice），
> `members = ["crates/*"]`；`products/<slice>/` 平行承载 IDD 文档。

| 路径 | 责任 | 状态 |
|---|---|---|
| `crates/skill-runtime/` | skill-runtime slice 实现（lib）| **已实现**（ADR-005 cutover-done）|
| `crates/storage/` | ADR-004 DocumentRow + MemoryDocumentStore | **已实现**（占位，无 SQLite）|
| `crates/artifact-system/` | artifact-system slice 实现（lib）| in-shadow（slice-2）|
| `crates/cli-ux/` | cli-ux slice 实现 + popsicle CLI 二进制（bin）| 未起（slice not-started）|

> 路径形态由 ADR-003 固化；各 crate 的内部模块图由 rfc-writer 在后续 RFC 增量补充。

## Contracts

`intents/contracts.intent` 持有跨模块 API 契约的形式化描述。任何 `crates/<name>/` 下
的 trait/struct 改动若影响 contracts，必须先走 ADR → 解锁 contracts → intent-spec-writer
收紧 → `intent check` 通过。

## 加载契约与命令树

> 来源：RFC-002 / ADR-002（Accepted 2026-06-08）。本段是 RFC「ARCHITECTURE.md 增量」的合并落点。

### SkillLoadResult（skill load 暴露的稳定加载结果契约）

`skill load` 返回 `SkillLoadResult`，四字段：

| 字段 | 含义 |
|---|---|
| `name` | skill 标识名 |
| `pkg_version` | skill 包的发布版本（semver；每次发布递增）|
| `schema_version` | 加载结果 / 状态机 schema 的版本，**独立于 `pkg_version`**——向后兼容的包升级不改它 |
| `state_machine` | 状态机定义，仅 `{pending → in_progress → completed / blocked}` 四态、单向前进 |

- **双版本解耦**（ADR-002）：消费方（intent-coder）按 `schema_version` 判断结果结构是否变化；`pkg_version` 变而 `schema_version` 不变 ⟹ 结构兼容，已 completed 的 run 不受影响（对应 `acceptance.intent#UpgradeDoesNotAffectCompletedRuns`）。
- **状态机审批闸**（HC-2）：任何使 stage 落到 `completed` 的转移，若该 stage `requires_approval`，必须先有 `approved_at`——形式化为 `invariants.intent` 的 `ApprovedBeforeCompleted` safety，经 intent-check Z3 verified。

### 命令树（noun-first，每 noun 组 = 一个 product 边界）

- 命令按**名词优先**分组（`issue` / `pipeline` / `doc` / `skill` / …），每个 product 暴露 **≤ 7** 个顶层 noun 组，避免命令面爆炸。
- 每个 noun 组的动词（`create` / `start` / `next` / `complete` …）在组内保持一致语义。

## Open Decisions

- ~~ADR-Workspace-Layout（products/* 下 crate 怎么放）~~ → **已决**：ADR-003（根级 `crates/<slice>/`，Accepted 2026-06-08）
- [TBD] ADR-skill-runtime-Boundary（与 artifact-system / cli-ux 的边界面）

---

> 本文件骨架；任何实质内容必须由 RFC（rfc-writer）+ ADR（adr-writer）固化后才能进。
> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
