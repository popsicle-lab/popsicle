# ADR-025 · issue link 命令（PROJ-48）

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Source-PDR**: PDR-005

## Decision

补齐 Issue↔Task **create 之后** 的回写 API；living-doc-author 晋升 proposed→linked 时调用同一命令。

## File Manifest

| Path | Change |
|---|---|
| `crates/storage/src/workspace.rs` | `link_issue_tasks` trait |
| `crates/cli-ux/src/self_host.rs` | 存储实现 + domain |
| `crates/cli-ux/src/lib.rs` | parse / run / help |
| `crates/cli-ux/tests/local_workspace.rs` | 集成测试 |
| `intent-coder/skills/issue-author/guide.md` | 晋升路径文档 |
| `intent-coder/skills/living-doc-author/guide.md` | 调用 issue link |
| `products/cli-ux/tasks/daily-ops/T-CU-0015-issue-task-link.md` | task |
| `products/cli-ux/intents/acceptance.intent` | Block 14 |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-48 cutover
