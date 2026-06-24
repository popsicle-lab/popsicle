# Product: cli-ux

> **Layer**: L2（用户可见行为）
> **Audience**: PM、销售、客户成功、AI Copilot
> **Status**: cutover-done + 多项目/UI/DMG + 工作流画像/健康仪表盘/Epic 关联 + **工作流帮助中心**（ADR-014–022、ADR-027 Accepted 2026-06-23）
> **Last-Updated**: 2026-06-23
> **Last-Decision-Ref**: ADR-027（PROJ-57 工作流帮助）

## 一行用途

把 IDD runtime/artifact 暴露成 agent-friendly 命令行。

## 用户视角的入口（实现面 = 宣传面，ADR-011）

- `popsicle init` / `doctor`：准备 workspace 并校验二进制/工作区来源。
- `popsicle issue` / `pipeline`：完整生命周期（create/list/show/start/**close** · status/next/stage complete）；issue 类型默认管线全部 bundled（含最小 `fix-regression` 模板，ADR-012）；可选 `--epic-task` 绑定 task（ADR-022）。
- **工作流画像**（`workflow.profile` in `project.yaml`）：`daily-dev` / `migration` / `pm-spec-only` / `opc-full` 切换默认 pipeline 与审批模式（ADR-022）。
- **桌面 UI**：Products 健康信号、Issue 按 product/pipeline 分组、task 正文 Mermaid、无 pipeline Issue retro 横幅（ADR-022）；Issues 列表可按当前筛选**导出 Markdown 简报**至剪贴板（PDR-004 / T-CU-0014）；**工作流帮助**侧栏页浏览 Pipeline/Skill 目录，并从 Issue 上下文高亮当前 stage（PDR-007 / T-CU-0017）。
- `popsicle doc`：生产、召回与**校验** stage artifact（create/list/show/**check**——frontmatter/实文/占位符/checkbox）。
- `popsicle tool run intent-validate`：Z3 intent 校验（仓库内严格解析）。
- `popsicle admin`：低频维护（migrate/reinit），不污染主路径。
- 全命令支持 `--format json`；错误同样 JSON 化并携带 actionable next-step；`failed` 响应统一非零退出。

**Deferred**（不在 help 宣传，调用返回结构化 `deferred` 错误）：`module` / `skill` /
`spec` / `namespace` / `prompt` / `git` / `memory` / `context` / `registry` /
`completions` — 永久去留待逐个 PDR 修订。**Removed**：`checklist` / `item` / `sync`。

## Tasks Catalog

> 5 个旅程阶段的入口（v0.2 任务图）。具体 task 文件由 prd-writer 写到 `tasks/<stage>/` 下。

- [Onboarding](tasks/onboarding/) — 初始化、DMG 安装、UI 选项目、嵌入 intent-coder、**工作流帮助**（6 task）
- [Daily-Ops](tasks/daily-ops/) — 创建 issue/run、doc、stage 推进、多项目切换（4 task）
- [Troubleshooting](tasks/troubleshooting/) — guard/lock/not-found 错误诊断（1 task）
- [Admin](tasks/admin/) — 低频维护命令（1 task）
- [Lifecycle](tasks/lifecycle/) — 旧命令裁剪与迁出确认（1 task）

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/acceptance.intent`](intents/acceptance.intent) — 命令树 + 多项目/UI/模块验收（12 block，PDR-001/002/003）
- [`intents/invariants.intent`](intents/invariants.intent) — CLI shell 自然律（`RenderTopLevelHelp` verified）
- [`intents/contracts.intent`](intents/contracts.intent) — CLI shell 对 runtime/artifact/storage 的端口契约（ADR-007 Accepted）

## Committed Roadmap

- PDR-001：命令树重组，legacy 22 命令分为 preserve/redesign/drop/defer。**其 preserve 清单已被 ADR-011 二次裁决修订**（17 preserve → 7 implemented + 10 deferred）。
- ADR-007：`crates/cli-ux` 只做 IO shell，依赖 `skill-runtime` / `artifact-system` / `storage`。
- ADR-008：`crates/cli-ux` binary `popsicle` 切为 semantic shell 主路径，不追求 legacy byte parity。
- ADR-010：self-host Phase 1（TSV workspace + IDD workflow + doctor provenance）。
- ADR-011：命令面对齐——help 收敛到实现面、`--format json` 全局化、工具解析仓库内严格化、根 AGENTS.md 与实现面绑定。
- ADR-012：可用性闭环——`doc check` / `issue close` 落地、默认管线 bundled 化（D-101）+ 模板自愈、smoke 隔离与残留清理（O-102）。
- ADR-013 / ADR-032：SQLite 存储——`.popsicle/state.db` + `.popsicle/runs/`（自动从 legacy `self-host/` 上提）、TSV 兼容、`admin migrate` / `admin relocate-workspace`、doctor 动态报告。PROJ-11 / PROJ-62 关闭。
- ADR-014：DevOps 工具链迁移——Makefile（check/golden/intent）、install.sh（裁剪 UI/completions）、pre-commit hook、CI/Release workflows（纯 Rust 矩阵）；fmt/clippy 欠账清零。
- ADR-015：Tauri 2 桌面 UI（MVP+）— `popsicle ui`，Cargo feature `ui`，直连 `WorkspaceDomain`。
- ADR-016：UI 项目切换器 + MRU，桥接 `global.json`；`.app` 零参数启动 UI。
- ADR-018：UI modern layout — collapsible sidebar、breadcrumb、Issues/Pipeline/Products master-detail split（≥1100px）；D-701 美学 divergence。
- ADR-017：intent-coder 编译期嵌入，`init` 解压到 `.popsicle/modules/intent-coder/`（DMG 无独立 module 目录）。
- PDR-003：多项目注册表、UI shell、嵌入模块、DMG 安装路径的 retro task/intent（PROJ-35，pipeline closed）。
- ADR-020：`workflow.approval_mode`、默认产品、`agent.language` 驱动 UI/CLI i18n（PROJ-39）。
- ADR-021：Issue `product_id` 用户面字段，`--product` CLI/UI（PROJ-38）。

## Open Questions
- `sync` 不进 IDD 主路径；是否 hibernate 或独立 product 另开 PDR。

---

> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
