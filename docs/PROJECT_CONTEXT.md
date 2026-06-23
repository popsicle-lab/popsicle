# Project Context

> **Last-Updated**: 2026-06-23
> **Last-Decision-Ref**: ADR-026
> **Owner**: 仓库维护者 + Settings UI；§现在状态 由 weekly-health-check 刷新

`docs/PROJECT_CONTEXT.md` 是本仓库**唯一的工程画像权威源**（git 追踪）。Agent 在 `inject_on_run: true` 时通过 `issue start` / `doc create` 的 `agent_context` 注入 §工程画像（不含 §现在状态）。

## 工程画像

### Tech Stack

- **语言**：Rust（workspace，`rust-toolchain.toml` pin）
- **前端**：Tauri 2 + React + TypeScript（`ui/`，`cargo build --features ui -p cli-ux`）
- **形式化**：IntentLang + Z3（`intent-validate` / `make intent`）
- **存储**：SQLite @ `.popsicle/self-host/state.db`（ADR-013）

### 仓库布局（目标态）

| 路径 | 职责 |
|---|---|
| `crates/skill-runtime/` | Skill 加载、Pipeline 会话、Issue 实体 |
| `crates/artifact-system/` | Document、Guard、Context 装配 |
| `crates/cli-ux/` | `popsicle` CLI + Tauri IPC |
| `crates/storage/` | DocumentRow / WorkspaceStore |
| `products/<product>/` | IDD 四件套（PRODUCT / ARCHITECTURE / intents / decisions） |
| `legacy/popsicle/` | Legacy submodule（equivalence baseline 对照） |
| `migration/` | 切流 traceability + progress 看板 |
| `docs/` | CHARTER、PROJECT_CONTEXT、baseline |

Workspace members：`crates/*`（ADR-003）。

### DevOps 标准

- **本地门禁**：`make check`（fmt + clippy + test，`-Dwarnings`）
- **Golden**：`make golden`（legacy vs new 对账链）
- **Intent**：`make intent` / `popsicle tool run intent-validate path=products`
- **Hooks**：`make install-hooks`（pre-commit fmt/clippy/test）
- **CI/Release**：GitHub Actions 纯 Rust 矩阵 + 可选 UI job（ADR-014）
- **安装**：`scripts/install.sh`（无 UI/completions defer）

### 关键约束

- Pipeline 审批：`workflow.approval_mode`（默认 `delegate-dangerous`）；危险阶段 `cutover`、`living-docs`
- Spec 门禁：新能力须 `slice-spec` → `slice-delivery`；`bugfix` 不得滥用（PROJ-53）
- 活文档：只用现在时；历史进 ADR/PDR / `migration/traceability.md`

## 现在状态

> 由 `weekly-health-check` pipeline + `living-doc-author --target product-context` 机械刷新。请勿在 Settings 中手工编辑本节。

| 指标 | 值 |
|---|---|
| Products | skill-runtime, artifact-system, cli-ux |
| 迁移 slice | skill-runtime / artifact-system / cli-ux → cutover-done（见 `migration/progress.md`） |
| 最近决策 | ADR-026（工程画像单一源 + weekly 巡检） |

## 相关链接

- 迁移对照：[`migration/traceability.md`](../migration/traceability.md)
- 切流进度：[`migration/progress.md`](../migration/progress.md)
- 人类迁移指南：[`docs/MIGRATION.md`](MIGRATION.md)
- Legacy 事实基：[`docs/baseline/2026-06-08/dependency-graph.md`](baseline/2026-06-08/dependency-graph.md)
- 各 product 结构：`products/*/ARCHITECTURE.md`

## 未来 collab 触发条件

`popsicle-sync` / 实时协作已整砍（PDR-001）。**重启条件**（须同时满足）：

1. ≥2 个 product 的 task 明确要求跨仓库实时协同编辑同一 artifact
2. 现有 git + Issue pipeline 无法满足 SLA（定量指标在 PDR 中定义）
3. 新的 collab product 有独立 PDR + ADR，不污染 skill-runtime 边界

在此之前不恢复 sync crate 或 WebSocket 依赖。
