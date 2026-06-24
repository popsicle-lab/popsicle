---
agent_context: [Project preferences]
- 界面 / Agent 语言：简体中文
- 产品目录：`products/`
- ADR：`products/<product>/decisions/adr/`
- PDR：`products/<product>/decisions/pdr/`
- Pipeline 审批：delegate-dangerous（危险操作需审批（其余代批））
- 非危险 `requires_approval` 阶段可由 agent 代批完成；危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。
doc_type: shadow-implementer
id: doc-94
pipeline_run_id: 0000002a-0000-402a-8002-2a00000000002a
status: active
title: PROJ-42 Roadmap P1-P6 implementation coverage
version: 1
artifact: implementation-coverage
slug: proj-42-roadmap-p1-p6
generated_by: shadow-implementer
slice: cli-ux
last_updated: 2026-06-11
crate: crates/cli-ux/
cargo_test_exit: 0
intent_blocks_total: 0
intent_blocks_covered: 0
query_anchors:
  - "Roadmap P1-P6 哪些已有代码？"
  - "workflow_profile / product health / epic_task_id 落在哪里？"
---

# 实现覆盖报告 — PROJ-42 Roadmap P1–P6

> 增量 UI/工作流增强，挂在既有 `cli-ux` slice 上。未新增 `.intent` acceptance block（retro 路径）；验证靠单元/集成测试 + `make check` + `intent-validate` 无回归。

## Scope Checklist

- [x] target slice 已确认：`cli-ux`
- [x] Roadmap P1–P6 六项已枚举
- [x] File Manifest 路径清单已列出
- [x] 每项已标 已实现
- [x] 端口归属：`cli-ux`（CLI + Tauri bridge）+ `storage`（epic 列）

## Summary

| 指标 | 值 |
|---|---|
| Slice | `cli-ux` |
| Crate | `crates/cli-ux/` + `ui/` + `crates/storage/` |
| Roadmap 项 | 6（P1–P6）|
| 已覆盖 | 6 |
| `make check` | PASS |
| `intent-validate path=products/cli-ux` | PASS（无新增失败）|

一句话结论：README Roadmap P1–P6 六项均已落地；`workflow_profile` 与 `epic_task_id` 贯通 CLI/SQLite/UI；产品健康扫描与 retro checklist 为只读增强，不改动既有 intent 语义。

## Roadmap → 实现覆盖表

| 项 | 实现位置 | 测试 / 验证 | 状态 |
|---|---|---|---|
| **P1** `workflow_profile` | `project_config.rs::WorkflowProfile`；`ui/commands.rs` DTO；`SettingsView.tsx` / `IssuesView.tsx` 默认 pipeline | `project_config::tests::workflow_profile_default_pipelines` | ✅ |
| **P2** Product 健康仪表盘 | `workspace_readers.rs::scan_product_health`；`commands.rs::get_product_health`；`ProductHealthPanel.tsx` | `tests/product_health.rs::scan_cli_ux_product_health_ok` | ✅ |
| **P3** Issue 分组 / Epic 视图 | `ui/src/lib/issueGroup.ts`；`IssuesView.tsx` 按 product/pipeline 聚合 | `npm run build`（UI 编译）| ✅ |
| **P4** Retro doc 闭环 | `intent-coder/guides/retro-doc-checklist.md`；`RetroDocBanner.tsx`（无 pipeline Issue）| 人工 checklist + UI 横幅逻辑 | ✅ |
| **P5** Mermaid 内嵌 | `MarkdownWithMermaid.tsx`；`TaskDetailPanel.tsx` 接入 | `npm run build` | ✅ |
| **P6** `epic_task_id` | `storage/sqlite.rs` 迁移列；`workspace.rs::IssueRow`；`self_host.rs` TSV 第 10 列 + `--epic-task`；UI 创建/列表/详情 | `local_workspace.rs` epic 持久化断言 | ✅ |

状态：✅ 已覆盖 / ⚠️ 部分 / ❌ 缺失

## File Manifest 对账

| 路径 | 预期责任 | 磁盘存在 | 备注 |
|---|---|---|---|
| `crates/cli-ux/src/project_config.rs` | P1 WorkflowProfile | ✅ | |
| `crates/cli-ux/src/workspace_readers.rs` | P2 scan_product_health | ✅ | |
| `crates/cli-ux/src/ui/commands.rs` | Tauri bridge | ✅ | |
| `crates/cli-ux/src/self_host.rs` | CLI `--epic-task`、TSV | ✅ | |
| `crates/storage/src/sqlite.rs` | epic_task_id 列迁移 | ✅ | |
| `crates/storage/src/workspace.rs` | IssueRow 字段 | ✅ | |
| `crates/cli-ux/tests/product_health.rs` | P2 集成测 | ✅ | |
| `intent-coder/guides/retro-doc-checklist.md` | P4 指南 | ✅ | |
| `ui/src/components/ProductHealthPanel.tsx` | P2 UI | ✅ | |
| `ui/src/components/RetroDocBanner.tsx` | P4 UI | ✅ | |
| `ui/src/components/MarkdownWithMermaid.tsx` | P5 UI | ✅ | |
| `ui/src/lib/issueGroup.ts` | P3 分组 | ✅ | |
| `ui/src/pages/IssuesView.tsx` | P1/P3/P6 | ✅ | |
| `ui/src/pages/SettingsView.tsx` | P1 | ✅ | |
| `ui/src/pages/ProductExplorerView.tsx` | P2 | ✅ | |
| `README.md` | Roadmap 表 P1–P6 ☑ | ✅ | |

## cargo test / make check

```
make check — PASS（fmt + clippy + test -Dwarnings）
cargo test -p cli-ux — 全绿（含 workflow_profile、product_health、local_workspace epic）
npm run build — PASS（Tauri UI）
intent-validate path=products/cli-ux — exit 0
```

## 待办（留给 living-docs / 后续 spec）

| 项 | 类型 | 跟进 |
|---|---|---|
| 为 P1–P6 正式铺 task 文件（T-CU-0014+）| retro living-doc | `living-docs` stage |
| 新 acceptance block（如 `WorkflowProfilePersists`）| intent-spec | 可选 slice-spec；当前 retro 路径已够用 |
| DMG / release tag | 发布 | 用户确认后 `v0.6.2` |

## 检查清单

- [x] 每个 Roadmap 项在表中有行
- [x] 每个 ✅ 行有具体路径与 test 名
- [x] File Manifest 与改动文件一致
- [x] `make check` 已实跑
- [x] 待办项未冒充 ✅
