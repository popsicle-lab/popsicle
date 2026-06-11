# artifact-system — Tasks 索引

> **Status**: 全 6 task 已实现（ADR-006 cutover-done）；intent 锚点经 living-doc-author 对账，6/6 related_intents Z3 verified
> **Last-Updated**: 2026-06-09

5 个固定旅程阶段。**缺一不可，也不允许第 6 个**（intent-coder/skills/prd-writer/references/task-organization.md）。

| 旅程阶段 | 任务数 | 已实施 | 健康度 |
|---|---|---|---|
| `onboarding/` | 1 | 1 | ✅ 已实现 + 锚点 verified（last_verified 2026-06-09）|
| `daily-ops/` | 3 | 3 | ✅ 已实现 + 锚点 verified（last_verified 2026-06-09）|
| `troubleshooting/` | 1 | 1 | ✅ 已实现 + 锚点 verified（last_verified 2026-06-09）|
| `admin/` | 0 | 0 | n/a |
| `lifecycle/` | 1 | 1 | ✅ 已实现 + 锚点 verified（last_verified 2026-06-09）|

## Task 清单（PDR-001）

| Task | 旅程 | 标题 | acceptance/invariant |
|---|---|---|---|
| T-AS-0001 | onboarding | 读懂文档制品怎么生产/读回 | DocumentRoundTrips（承接）|
| T-AS-0002 | daily-ops | doc create 存盘一字还原 | DocumentRoundTrips |
| T-AS-0003 | daily-ops | doc check 章节 + checklist | GuardChecklistCompleteIffNoUnchecked |
| T-AS-0004 | daily-ops | prompt 装配相关度排序 | ContextAssemblyOrdersByRelevance |
| T-AS-0005 | troubleshooting | extract/guard 未知类型排查 | ExtractPreservesKind / EvaluateGuard |
| T-AS-0006 | lifecycle | work_item→task_chunk 重命名 | RenameWorkItemToTaskChunk |

## 命名约定

task 文件命名：`<旅程阶段>/<动词-名词-短语>.md`（小写连字符）。
每个 task 文件**必须**带 YAML frontmatter（id / acceptance refs / intent refs）——见 prd-writer 模板。

## 何时新增 task

- 由 prd-writer 在产出 PRD 五件套时铺；不在 bootstrap 期间手工添加
- 已存在的 task 进入 in-progress / done / blocked 由 `migration/progress.md` 同步追踪
