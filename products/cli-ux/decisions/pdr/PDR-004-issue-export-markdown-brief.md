# PDR-004: Issue 列表导出 Markdown 简报

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Source**: PROJ-46（PROJ-45 dogfood proposed task）
> **Supersedes**: —
> **Related ADRs**: ADR-024（待 Accepted）

## Decision Context

### 触发因素

Issues 马赛克视图已支持类型/状态/搜索筛选与 Task 关联展示，但用户无法把**当前子集**复制到 IM、周报或 Agent 上下文。PROJ-45 mock 验证提出「导出 Markdown 简报」旅程。

### 多角色辩论摘要

**未经多角色辩论**（cli-ux 增量 UI；`issue-author` guide § 已交付能力补 spec）。用户访谈要点：

- 导出必须尊重 UI 筛选，不是全量 `issue list`。
- 简报需含 `key`、`title`、`status`、`pipeline`、`linked task_ids`。
- 首选剪贴板，不新增 CLI 子命令。

### 备选方案

| 方案 | 否决理由 |
|---|---|
| `popsicle issue list --format md` | 超出 MVP；无法表达 UI 搜索语义 |
| 下载 `.md` 文件 | Tauri 需额外权限；剪贴板足够 |
| 仅表格无明细 | Agent 上下文常需 description/run 摘要 |

## Decision

在 Issues 工具栏增加 **导出 Markdown**：对当前筛选/排序后的列表生成简报，经 `navigator.clipboard.writeText` 写入剪贴板；不新增 Rust 命令。

## Consequences

### Task File Updates

#### 新增 Tasks

- [x] `products/cli-ux/tasks/daily-ops/T-CU-0014-export-issue-markdown-brief.md`

### Intent Updates

- [x] `products/cli-ux/intents/acceptance.intent` — Block 13 `IssueExportMarkdownBrief`

### Code Updates（informational，ADR-024 File Manifest）

- `ui/src/lib/issueExportMarkdown.ts` — 格式化
- `ui/src/pages/IssuesView.tsx` — 工具栏按钮
- `ui/src/i18n/messages.ts` — 文案

### 流程纠正（PROJ-46）

Issue 误用 `--pipeline slice-delivery` 在 spec 未就绪时进入 implement。本 PDR 在 implement 写码**之前**补齐五件套 + ADR File Manifest，登记 Divergence **D-646**（实现晚于 spec 门禁，spec 以本 PDR 为准）。

## Intent Impact

| Intent | Task | File | Meaning |
|---|---|---|---|
| `IssueExportMarkdownBrief` | T-CU-0014 | `acceptance.intent` | 导出尊重筛选、含 Markdown 正文、含 task 关联 |

## Validation Plan

- `popsicle tool run intent-validate path=products/cli-ux/intents` exit 0
- 手动：筛选后导出 → 粘贴核对条数与字段
- `cd ui && npm run build`

## Approval

- **Status**: Proposed
- **Approved by**: （待用户确认 PDR + ADR-024 后改 Accepted）
