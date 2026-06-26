# cli-ux — Tasks 索引

> **Status**: 19/19 task 已实现（含 T-CU-0018 feature-arch-spec / ADR-030）；Roadmap P1–P6 + issue_tasks 已交付（PROJ-42/43 / ADR-022/023）
> **Last-Updated**: 2026-06-26

5 个固定旅程阶段。**缺一不可，也不允许第 6 个**（intent-coder/skills/prd-writer/references/task-organization.md）。

| 旅程阶段 | 任务数 | 已实施 | 健康度 |
|---|---|---|---|
| `onboarding/` | 6 | 6 | 🟢 implemented |
| `daily-ops/` | 7 | 7 | 🟢 implemented |
| `troubleshooting/` | 1 | 1 | 🟢 implemented |
| `admin/` | 1 | 1 | 🟢 implemented |
| `lifecycle/` | 3 | 3 | 🟢 implemented |

## Task 清单（PDR-001）

| Task | 旅程 | 标题 | acceptance/invariant |
|---|---|---|---|
| T-CU-0001 | onboarding | 第一次初始化 popsicle-new 并看到下一步 | InitShowsNextStep |
| T-CU-0002 | daily-ops | 创建 issue 并启动 pipeline run | IssueStartCreatesRun |
| T-CU-0003 | daily-ops | 创建/查看/校验 stage 文档 | DocCommandWritesArtifact |
| T-CU-0004 | daily-ops | 查询 pipeline 状态并推进 stage | StageAdvanceReflectsState |
| T-CU-0005 | troubleshooting | guard/lock/not-found 错误可诊断 | ErrorsAreActionable |
| T-CU-0006 | admin | 执行低频 admin migrate/reinit | AdminCommandsAreExplicit |
| T-CU-0007 | lifecycle | 确认旧命令 disposition | RemovedCommandsStayRemoved |
| T-CU-0008 | lifecycle | 自举运行 workflow 并确认 binary provenance | SelfHostedWorkflowSmokePasses / BinaryProvenanceVisible |
| T-CU-0009 | daily-ops | 在多个已注册项目之间切换 | ProjectRegistryOverridesWorkspace |
| T-CU-0010 | onboarding | 用桌面 UI 打开项目并看最近列表 | UiProjectOpenPersistsRecents |
| T-CU-0011 | onboarding | 全新目录 init 后获得 intent-coder 模块 | InitInstallsEmbeddedIntentCoder |
| T-CU-0012 | onboarding | macOS DMG 安装 CLI 并加入 PATH | MacosDmgInstallExposesCli |
| T-CU-0013 | onboarding | 配置本项目 Agent 偏好 | ADR-019 / project.yaml |
| T-CU-0014 | daily-ops | 导出当前筛选 Issue 列表 Markdown 简报 | IssueExportMarkdownBrief |
| T-CU-0015 | daily-ops | 维护 Issue 与 Task 关联（issue link） | IssueTaskLinksMutable |
| T-CU-0016 | daily-ops | 工程画像（PROJECT_CONTEXT）与 weekly 健康巡检 | ProjectContextEditableInSettings / WeeklyHealthCheckPipeline / AgentContextIncludesProjectContext |
| T-CU-0017 | onboarding | 浏览 intent-coder 工作流帮助（Pipeline / Skill） | WorkflowHelpCenterBrowsable |
| T-CU-0018 | lifecycle | 为大增量能力选用 feature-arch-spec pipeline 并完成 bundled 安装 | ADR-030 |

## 命名约定

task 文件命名：`<旅程阶段>/<动词-名词-短语>.md`（小写连字符）。
每个 task 文件**必须**带 YAML frontmatter（id / acceptance refs / intent refs）——见 prd-writer 模板。

## Roadmap 增强（PROJ-42，retro — 无新 task 文件）

| 优先级 | 能力 | 主要落点 | 关联 task |
|:---:|---|---|:---:|
| P1 | `workflow_profile` | `project_config.rs` · Settings / Issue 向导 | 扩展 T-CU-0013 |
| P2 | Product 健康仪表盘 | `scan_product_health` · `ProductHealthPanel` | — |
| P3 | Issue 分组 | `issueGroup.ts` · `IssuesView` | 扩展 T-CU-0002 |
| P4 | Retro doc checklist | `retro-doc-checklist.md` · `RetroDocBanner` | — |
| P5 | Mermaid in task MD | `MarkdownWithMermaid` | — |
| P6 | `issue_tasks` 多对多 | `issue_tasks` 表 + `--tasks` + `issue-author` | 扩展 T-CU-0002 |
| P7 | `issue-author` skill | 独立 skill（不进 pipeline）| 扩展 T-CU-0002 |

`epic_task_id` 已废弃（只读兼容）。详见 ADR-022、ADR-023。

## 何时新增 task

- 由 prd-writer 在产出 PRD 五件套时铺；不在 bootstrap 期间手工添加
- 已存在的 task 进入 in-progress / done / blocked 由 `migration/progress.md` 同步追踪
- retro 增量（如 PROJ-42）可先记在 README Roadmap + 本表，正式 task 化走 prd-writer
