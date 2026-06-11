# ADR-015 · Tauri UI self-host bridge (MVP+)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-27）
> **Reverses**: ADR-014 D-402（UI 不迁移）— 范围限定为 MVP+ 桌面壳

## Context

ADR-014 明确 DevOps 迁移时不带 UI（D-402）。自托管 CLI 闭环（ADR-010~013）
已 dogfood-usable，但 Issues/Pipeline/文档工作流仅有终端面。Legacy popsicle
在 pin c76d729 上有 Tauri 2 桌面壳，但其 IPC 绑定 legacy `popsicle-core` 与
`.popsicle/popsicle.db`，无法直接复用到新 `SelfHostDomain` + SQLite/TSV 后端。

## Decision

1. **Cargo feature `ui`**：可选启用 Tauri 2 + `tauri-plugin-shell`；默认
   `cargo build` 仍为纯 Rust（CI 主 `check` job 不变）。
2. **入口**：`popsicle ui [--project <path>]` 启动桌面窗口（1280×800，对齐 legacy）。
3. **IPC**：`crates/cli-ux/src/ui/commands.rs` 直接调用 `LocalWorkspace` /
   `WorkspaceStore`，不 subprocess CLI、不读 legacy DB。
4. **读取层**：`workspace_readers.rs` 扫描 `products/*/tasks`、`.intent` 与
   pipeline YAML；Intent 图优先 `intent goals --diagram`，否则 fallback 解析。
5. **前端**：`ui/` Vite + React 19 + Tailwind + `@xyflow/react` +
   `react-markdown` + `mermaid`；裁剪 legacy 14 页为 MVP+（Issues、Pipeline DAG、
   文档、Task 图、Intent 图）。
6. **DevOps**：恢复 `make build-ui` / `make ui-dev`；CI 增可选 `ui` job；
   golden 链 `docs/baseline/2026-06-11/cli-ux-ui/`（4 项）。

## Divergences / Deferred

- **D-501**：Legacy 全量页面（Dashboard、Namespaces、Git、Memories、Search、Skills）
  不在 MVP+；completions 仍 deferred（ADR-014 D-401）。
- **D-502**：`pipeline stage complete` 文档 guard 未硬接线 UI（与 CLI 一致，先不依赖文档）。
- **O-501**：文件 watcher 仅骨架（`popsicle://refresh`）；完整 debounced watch 后续补。

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| `cargo build --features ui -p cli-ux` | 本地构建 | pass |
| `make check`（无 ui feature） | fmt + clippy + test | pass |
| UI golden 4/4 | `docs/baseline/2026-06-11/cli-ux-ui/run-all.sh` | pass |
| Intent Z3 | `tool run intent-validate path=products` | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-27 slice-delivery（agent 实现 + 用户 stage 确认）
- **Approval date**: 2026-06-11
