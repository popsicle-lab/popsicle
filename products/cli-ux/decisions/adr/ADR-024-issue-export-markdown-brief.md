# ADR-024 · Issue 列表导出 Markdown 简报（PROJ-46）

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: rfc-writer / adr-writer（轻量 spec，无 legacy parity）
> **Source-PRD**: doc-103.prd-writer.md
> **Source-PDR**: PDR-004

## Context

cli-ux Issues UI 需把筛选结果导出为 Markdown 简报供人类/Agent 复用。能力在
`acceptance.intent#IssueExportMarkdownBrief`（T-CU-0014）中形式化。无 legacy
子模块对账；纯 greenfield UI 增量。

## Decision

1. **纯前端**：复用已有 `listIssues` DTO，在 `IssuesView` 已筛选的 `issues` 数组上格式化；不新增 Tauri command。
2. **输出形态**：摘要表 + `## 明细` 分段；元数据记录视图模式与筛选条件。
3. **剪贴板**：`navigator.clipboard.writeText`；失败时展示 i18n 错误提示。
4. **Divergence D-646**：PROJ-46 曾先开 `slice-delivery` 再补 spec；以本 ADR + PDR-004 为权威清单，implement 仅改 Manifest 内路径。

## Alternatives

| 方案 | 否决理由 |
|---|---|
| Rust 侧 `export_issues_md` | 无法获得 UI 搜索状态 |
| 服务端渲染 | 无服务端；桌面本地即可 |

## Consequences

- `products/cli-ux/tasks/README.md` — 新增 T-CU-0014 行（living-docs 标已实施）
- `products/cli-ux/PRODUCT.md` — Daily-Ops 能力一句
- `migration/traceability.md` — PROJ-46 行（cutover 时更新）

## File Manifest

| Path | Change |
|---|---|
| `ui/src/lib/issueExportMarkdown.ts` | 新增：筛选上下文 → Markdown |
| `ui/src/pages/IssuesView.tsx` | 工具栏「导出 Markdown」+ 剪贴板 |
| `ui/src/i18n/messages.ts` | `exportMarkdown` / `exportCopied` / `exportFailed` |
| `ui/src/index.css` | `.issues-export-notice`（可选） |
| `products/cli-ux/tasks/daily-ops/T-CU-0014-export-issue-markdown-brief.md` | 新增 task |
| `products/cli-ux/intents/acceptance.intent` | Block 13 + `IssueExportResult` |
| `products/cli-ux/decisions/pdr/PDR-004-issue-export-markdown-brief.md` | 本能力 PDR |

## Compliance（cutover 时填写）

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `intent-validate path=products/cli-ux/intents` | 待跑 |
| UI build | `npm run build` | 待跑 |
| Golden | UI-only；登记「无 legacy golden」| N/A |

## Approval

- **Status**: Proposed → Accepted（implement 前用户确认 spec）
- **Approved by**: —
