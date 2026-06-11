# ADR-002: SkillLoadResult 契约 + 命令树重组

> **Status**: Accepted
> **Date**: 2026-06-08
> **Product**: skill-runtime
> **Generated-by**: rfc-writer（骨架）→ 待 adr-writer 固化
> **Source-RFC**: rfc-002-skill-runtime-skillloadresult-契约--命令树重组.rfc.md
> **Source-Debate**: skill-runtime-arch-debate-skillloadresult-契约--命令树重组-adr-002-候选.arch-debate.md
> **CADR**: 否（命令重组属 internal API，ADR-001 上游已拍板但尚未落盘——其「intent-coder 是内部消费者」原则已被 product-debate 引用；不触 charter 四条铁律 / Layer Map）

## Context

CON-SR-01「skill load 必须返回 SkillLoadResult 含 name/version/state_machine」当前仅「部分验证」
（只做 schema 校验、无版本语义）。命令树现状混在一棵 popsicle CLI 树、无 product 边界（F-3）。
arch-debate 经 4 角色评审（SEC/PERF/OPS/DATA/DEV）+ 用户拍板，收敛到方案 B + 双字段过渡。

## Decision

1. `popsicle skill load` 返回 `SkillLoadResult{name, pkg_version, schema_version, state_machine}`。
2. `schema_version` 独立于 `pkg_version` 版本化（registry-backed）；向后兼容变更时 `schema_version` 不变。
3. `state_machine` 仅表达 `{pending → in_progress → completed/blocked}` 转移（HC-2 不变量）。
4. 命令树按 **noun-first** 分组（`skill <load|run>`、`pipeline <start|stage|status|recover>`）；
   skill-runtime product 暴露 ≤ 7 个公开命令。

## Alternatives

- **方案 A（内嵌薄契约，version=包版本）**：可演进性弱、无法表达「schema 兼容但包升级」。退路。
- **方案 C（能力清单动态投影）**：要求先把 hardcode 的 guard（F-4）抽成可配置，scope 冲突 + HC-3 风险最高。未来演进。

## Consequences

> 与 RFC-002 § File Manifest 镜像一致（落地物三项；ADR-002 文件自身不自列）。

- [ ] `products/skill-runtime/ARCHITECTURE.md` § 加载契约与命令树 —— 记录四字段 + 双版本语义 + noun-first 命令树。
- [ ] `products/skill-runtime/intents/contracts.intent` 2 个 goal（`skill load 暴露稳定的加载结果契约`、`state_machine schema 版本独立于包版本`）解锁，由 intent-spec-writer 收紧。
- [ ] `products/skill-runtime/intents/invariants.intent` 落地 state_machine 转移不变量（HC-2）。

## Migration

- 命令迁移期 alias 窗口 + 6 个 task chunk 命令字面量对齐（HC-3）；与 RFC-002 § Migration / Rollout 一致。

## Compliance

- ADR-001（intent-coder 是内部消费者，CLI 变动算 internal API）：上游已拍板但尚未落盘；本 ADR 依赖其原则，命令重组 0 外部破坏（PDR-001 §Phase3 ENGLD-Q1 已验证）。
- 能力边界 D2：skill load 时延不进 intent，由 RFC-002 § NFR 的 benchmark 守护。

## Approval

- **Approved by**: @curtiseng（arch-debate Phase 4 暂停点 #3/#4 拍板「方案 B + 双字段」；adr 阶段确认固化）
- **Approval date**: 2026-06-08
- ⚠️ 本 ADR 已 Accepted，依 docs/CHARTER.md 第 2 条铁律**永不修改**。纠错请新建 ADR 标 `Supersedes: ADR-002`。
