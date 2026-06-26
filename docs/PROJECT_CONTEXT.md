# Project Context

> **Last-Updated**: 2026-06-26
> **Last-Decision-Ref**: ADR-001
> **Owner**: 仓库维护者 + Settings UI；§现在状态 由 doc-sync-weekly 刷新

`docs/PROJECT_CONTEXT.md` 是本仓库**唯一的工程画像权威源**（git 追踪）。Agent 在 `inject_on_run: true` 时通过 `issue start` / `doc create` 的 `agent_context` 注入 §工程画像（不含 §现在状态）。

## 工程画像

### Tech Stack

- **语言**：Rust（workspace，`rust-toolchain.toml` pin）
- **前端**：Tauri 2 + React + TypeScript（`ui/`，`cargo build --features ui -p cli-ux`）
- **形式化**：IntentLang + Z3（`intent-validate` / `make intent`）
- **存储**：SQLite @ `.popsicle/state.db`（ADR-013 / ADR-032）；本仓库 `git.track_workspace: true`，`.popsicle/`（含 state、artifacts、runs）纳入 git

### 仓库布局（目标态）

| 路径 | 职责 |
|---|---|
| `crates/skill-runtime/` | Skill 加载、Pipeline 会话、Issue 实体 |
| `crates/artifact-system/` | Document、Guard、Context 装配 |
| `crates/cli-ux/` | `popsicle` CLI + Tauri IPC + telemetry inject |
| `crates/telemetry/` | Agent 观测旁路：WAL + OTLP export（fail-open） |
| `crates/storage/` | DocumentRow / WorkspaceStore |
| `products/<product>/` | IDD 四件套（PRODUCT / ARCHITECTURE / intents / decisions） |
| `migration/` | 切流 traceability + progress 看板（历史归档） |
| `docs/` | CHARTER、PROJECT_CONTEXT、baseline（含 legacy 冻结快照） |

Workspace members：`crates/*`（ADR-003）。

### DevOps 标准

- **本地门禁**：`make check`（fmt + clippy + test，`-Dwarnings`）
- **Golden**：`make golden`（self-host 回归链，见 `docs/baseline/`）
- **Intent**：`make intent` / `popsicle tool run intent-validate path=products`
- **Hooks**：`make install-hooks`（pre-commit fmt/clippy/test）
- **CI/Release**：GitHub Actions 纯 Rust 矩阵 + 可选 UI job（ADR-014）
- **安装**：`scripts/install.sh`（无 UI/completions defer）

### 关键约束

- Pipeline 审批：`workflow.approval_mode`（默认 `delegate-dangerous`）；危险阶段 `cutover`、`living-docs`
- Spec 门禁：新能力须 `feature-spec` / `migration-slice-spec` → `feature-delivery` / `migration-slice-delivery`；`fix-regression` 不得滥用（PROJ-53）
- 活文档：只用现在时；历史进 ADR/PDR / `migration/traceability.md`

### intent-coder 分发物 vs 本仓库 products（dogfood 边界）

| 层 | 路径 | 谁读 | 内容 |
|---|---|---|---|
| **Module tool guide** | `intent-coder/tools/<tool>/guide.md` → `init` 后 `.popsicle/modules/intent-coder/...` | 任意已 init 项目的 Agent | **自包含**操作约定；`action=guide` 打印全文 |
| **本仓库 product spec** | `products/<product>/`（如 `SPAN_SCHEMA.md`） | 仅 popsicle monorepo dogfood | IDD 字段表、ARCHITECTURE manifest、intent 追溯 |

**反例（PROJ-75）**：在 `intent-coder/tools/telemetry/guide.md` 里写「monorepo 另有 `products/telemetry/SPAN_SCHEMA.md`」——会把**本仓库路径**塞进**公共分发物**，init 后的外部项目无法解析，且与「span schema 留在 products/」的定案矛盾。

**正确分工**：

- Module guide：内嵌 span 速查与命令模板（Agent 唯一 portable 入口）。
- `products/telemetry/SPAN_SCHEMA.md` / `AGENT_TELEMETRY.md`：monorepo 产品文档；可**指向** module guide，**不得**被 module guide **反向引用**。
- 本仓库 `AGENTS.md` Workflow Rule 12 仅指向 `action=guide`；`products/telemetry/SPAN_SCHEMA.md` 供 spec/实现对账，Agent 运行时不必读。

**Intent goal 追溯（PROJ-76）**：`intent-validate` 对存在 `contracts.intent` 的 product 要求 ≥1 个 `goal` + 非空 `realized_by`；否则 `E_PRODUCT_MISSING_GOALS` / `E_GOAL_*` → exit 1。

## 现在状态

> 由 `doc-sync-weekly` pipeline + `living-doc-author --target product-context` 机械刷新。请勿在 Settings 中手工编辑本节。

| 指标 | 值 |
|---|---|
| Products | skill-runtime, artifact-system, cli-ux, telemetry（MVP 已交付） |
| 迁移 slice | skill-runtime / artifact-system / cli-ux → cutover-done（见 `migration/progress.md`） |
| 最近决策 | ADR-002（telemetry report）；PROJ-67–77 telemetry MVP + 观测增强已交付 |
| Telemetry 健康 | 最近 10 个 run；doc_check 失败 0 次；3 个 run 含 gen_ai、2 个含 score；6 个 run 共 19 处 stage-doc 缺 Agent span（weekly 2026-06-26；`action=report` 含 `agent_coverage.gaps`） |

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
