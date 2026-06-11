# Product: cli-ux

> **Layer**: L2（用户可见行为）
> **Audience**: PM、销售、客户成功、AI Copilot
> **Status**: cutover-done + self-host Phase 1（ADR-010 Accepted 2026-06-11）
> **Last-Updated**: 2026-06-11
> **Last-Decision-Ref**: ADR-010（self-hosting Phase 1）

## 一行用途

把 IDD runtime/artifact 暴露成 agent-friendly 命令行。

## 用户视角的入口

- `popsicle init` / `module` / `tool`：准备一个 IDD-ready workspace。
- `popsicle skill` / `pipeline` / `spec` / `issue`：启动并推进迁移工作流。
- `popsicle doc` / `prompt`：生产、校验和召回 stage artifact。
- `popsicle admin`：承载低频维护命令（如 migrate/reinit），不污染主路径。
- `popsicle context` / `memory` / `registry` / `git`：为 agent 提供项目背景、记忆、发布与审计辅助。

## Tasks Catalog

> 5 个旅程阶段的入口（v0.2 任务图）。具体 task 文件由 prd-writer 写到 `tasks/<stage>/` 下。

- [Onboarding](tasks/onboarding/) — 首次初始化并看到下一步（1 task）
- [Daily-Ops](tasks/daily-ops/) — 创建 issue/run、doc、stage 推进（3 task）
- [Troubleshooting](tasks/troubleshooting/) — guard/lock/not-found 错误诊断（1 task）
- [Admin](tasks/admin/) — 低频维护命令（1 task）
- [Lifecycle](tasks/lifecycle/) — 旧命令裁剪与迁出确认（1 task）

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/acceptance.intent`](intents/acceptance.intent) — 命令树验收契约（7 VC verified，ADR-008 gate_ready=true）
- [`intents/invariants.intent`](intents/invariants.intent) — CLI shell 自然律（`RenderTopLevelHelp` verified）
- [`intents/contracts.intent`](intents/contracts.intent) — CLI shell 对 runtime/artifact/storage 的端口契约（ADR-007 Accepted）

## Committed Roadmap

- PDR-001：命令树重组，legacy 22 命令分为 preserve/redesign/drop/defer。
- ADR-007：`crates/cli-ux` 只做 IO shell，依赖 `skill-runtime` / `artifact-system` / `storage`。
- ADR-008：`crates/cli-ux` binary `popsicle` 切为 semantic shell 主路径，不追求 legacy byte parity。

## Open Questions

- Tauri UI bridge 不进 MVP；是否迁入另开 slice。
- `sync` 不进 IDD 主路径；是否 hibernate 或独立 product 另开 PDR。

---

> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
