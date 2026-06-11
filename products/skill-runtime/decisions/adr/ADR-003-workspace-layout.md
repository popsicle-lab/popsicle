# ADR-003: workspace 布局 — 根级 crates/<slice>/

> **Status**: Accepted
> **Date**: 2026-06-08
> **Product**: popsicle（skill-runtime / artifact-system / cli-ux 同属一个产品）
> **Generated-by**: rfc-writer（骨架）→ adr-writer 固化
> **Source-RFC**: rfc-workspace-布局--products-product-crates--与-members-glob-adr-workspace-layout.rfc.md
> **Source-Debate**: workspace-布局决策-crate-切分与-products--归位-adr-workspace-layout-候选.arch-debate.md
> **CADR**: 否（纯目录/构建布局约定，不暴露跨模块 API，不触 charter 四条铁律 / Layer Map）

## Context

root `Cargo.toml` 占位为 `members = []`、注释倾向 `products/<product>/crates/*`，而
ARCHITECTURE.md § File Manifest 写根级 `crates/<name>/`——两处自相矛盾。skill-runtime
即将进入 in-shadow 实现期，落第一行 Rust 前必须收敛 member crate 的物理落点。**用户澄清**：
`skill-runtime / artifact-system / cli-ux` 三个 slice 同属**一个产品 popsicle**，非独立产品。
arch-debate 经 6 角色（ARCH/SEC/PERF/OPS/DATA/DEV）评审 3 方案 + 用户拍板，收敛到方案 C。

## Decision

1. popsicle 的 member crate 一律落在**根级** `crates/<slice>/`（如 `crates/skill-runtime`），
   每个迁移 slice 对应一个 crate。
2. root `Cargo.toml` 声明 `members = ["crates/*"]`，沿用 legacy 扁平布局。
3. `products/<slice>/` 仅承载该 slice 的 IDD 文档（intents/tasks/decisions/proposals），
   与 `crates/<slice>/` 按 slice 名平行对齐。
4. 跨 crate 依赖经 `[workspace.dependencies]` 显式 `path` 声明。`cli-ux` crate 承载
   popsicle CLI 二进制，`skill-runtime` / `artifact-system` 为 lib（lib/bin 内部细分留实现期）。

## Alternatives

- **方案 A（每产品 `products/<product>/crates/*`）**：前提是「多产品」；用户澄清三 slice 同属
  一个产品后，前提不成立，否决。
- **方案 B（根级 `crates/` 按技术层 `popsicle-core`/`-cli`）**：技术层 crate 横跨多个 slice，
  与 migration 的逐 slice shadow/cutover 边界不对齐，切换需跨 crate 协调，否决。

## Consequences

> 与 RFC § File Manifest 镜像一致（落地物三项；ADR-003 文件自身不自列）。

- [x] `products/skill-runtime/ARCHITECTURE.md` § File Manifest —— 路径明确为根级 `crates/<slice>/`，crate = slice。
- [x] `products/skill-runtime/ARCHITECTURE.md` § Open Decisions —— 移除 `[TBD] ADR-Workspace-Layout`，指向本 ADR。
- [x] root `Cargo.toml` —— `members = ["crates/*"]`，并改写占位注释。
- [x] `crates/.gitkeep` —— 建空 `crates/` 目录使 glob 可解析（首个真实 crate 落地前 cargo metadata 通过）。

## Migration

- 当前 `members = []`、无既有 crate ⟹ 零迁移成本。首个 crate `crates/skill-runtime` 随实现
  落地、独立进入 in-shadow；与 RFC § Migration / Rollout 一致。回滚 = 改 `members` glob。

## Compliance

- 不暴露跨模块 API ⟹ **无 contracts 种子**、无 intent 解锁；纯布局约定。
- 能力边界 D2：迁移局部性/编译期边界不进 intent，由 `migration/progress.md` 看板 + CI
  `cargo metadata` 依赖图断言守护（RFC § NFR）。

## Approval

- **Approved by**: @curtiseng（arch-debate 两轮 ask_user 拍板：先澄清「三 slice 同属一个产品」，
  再选定方案 C「每 slice 一 crate、根级扁平」；adr 阶段确认固化）
- **Approval date**: 2026-06-08
- ⚠️ 本 ADR 已 Accepted，依 docs/CHARTER.md 第 2 条铁律**永不修改**。纠错请新建 ADR 标 `Supersedes: ADR-003`。
