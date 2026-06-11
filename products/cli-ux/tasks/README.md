# cli-ux — Tasks 索引

> **Status**: 13/13 task 已实现；retro T-CU-0009..0012 formalized（PROJ-35 / PDR-003）
> **Last-Updated**: 2026-06-11

5 个固定旅程阶段。**缺一不可，也不允许第 6 个**（intent-coder/skills/prd-writer/references/task-organization.md）。

| 旅程阶段 | 任务数 | 已实施 | 健康度 |
|---|---|---|---|
| `onboarding/` | 5 | 5 | 🟢 implemented |
| `daily-ops/` | 4 | 4 | 🟢 implemented |
| `troubleshooting/` | 1 | 1 | 🟢 implemented |
| `admin/` | 1 | 1 | 🟢 implemented |
| `lifecycle/` | 2 | 2 | 🟢 implemented |

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

## 命名约定

task 文件命名：`<旅程阶段>/<动词-名词-短语>.md`（小写连字符）。
每个 task 文件**必须**带 YAML frontmatter（id / acceptance refs / intent refs）——见 prd-writer 模板。

## 何时新增 task

- 由 prd-writer 在产出 PRD 五件套时铺；不在 bootstrap 期间手工添加
- 已存在的 task 进入 in-progress / done / blocked 由 `migration/progress.md` 同步追踪
