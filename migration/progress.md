# Migration Progress

> **Status**: 活文档 —— 每个 slice 状态变化时同步刷新
> **Last-Updated**: 2026-06-11
> **Last-Decision-Ref**: ADR-014（devops tooling migration，PROJ-26）

## Slices 看板

| # | Slice | 范围（legacy popsicle 子模块） | 状态 | Spec ID | 当前阶段 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `skill-runtime` | `crates/popsicle-core/src/{model,engine/hooks,registry,memory}/` 大部分 + `issue` 实体 + `model/pipeline.rs::PipelineRun` | **cutover-done** | `slice-1-skill-runtime` | PROJ-4 slice-delivery ✓ | ADR-005 Accepted · lib golden 6/6 · CLI 仍 legacy（cli-ux）|
| 2 | `artifact-system` | `crates/popsicle-core/src/{model/document, engine/markdown,engine/guard,engine/context,engine/context_layer,engine/extractor}.rs` + `namespace` 实体 + `task_chunk_entity`（旧 `work_item` 重命名）+ doc/extract/summarize 命令族 | **cutover-done** | `slice-2-artifact-system` | PROJ-5 slice-delivery ✓ | ADR-006 Accepted · lib golden 6/6 · CLI 仍 legacy（cli-ux）|
| 3 | `cli-ux` | `crates/popsicle-cli/` + `crates/popsicle-core/src/commands/` + `prompt/migrate/admin` 命令族 | **cutover-done** | `slice-3-cli-ux` | PROJ-26 devops ✓ | ADR-008/010/011/012/013 + **ADR-014**（DevOps 五件套迁移：Makefile / install.sh / hooks / CI / Release）· 27/27 golden · **dogfood-usable** |
| 4 | ~~`sync-collab`~~ | ~~`crates/popsicle-sync/`~~ | **decided: dropped** | — | — | PDR-001 整砍 · 触发条件文档 = `docs/PROJECT_CONTEXT.md §未来 collab 触发条件`（待 living-doc）|

## 状态语义

| 状态 | 含义 |
|---|---|
| `not-started` | bootstrap 后未启动；spec 未建 |
| `in-progress` | spec 建立、issue 启动、pipeline run 跑中 |
| `in-shadow` | 已实现，等价性 baseline 跑通，legacy 仍是主路径 |
| `cutover-done` | 切流通过，popsicle-new 该 slice 为主路径 |
| `legacy-sunset` | legacy 该范围被删除/冻结，不再维护 |
| `pending-decision` | 需要 fact-extractor / product-debate 判定是否要这个 slice |
| `decided: dropped` | product-debate 已决定不进 product inventory（如 PDR 记录）|

## 切流硬门禁（提醒）

任何 slice 从 `in-progress` 推进到 `cutover-done` 前**必须**满足：

1. 全部 `.intent` `intent check` Z3 PASS（observe 模式连续 ≥3 次零失败）
2. ≥5 条 golden-output 对账脚本通过（legacy vs new diff 为空）
3. 切流本身的 ADR 进入 `Status: Accepted`

详细见 [`docs/CHARTER.md`](../docs/CHARTER.md) + [`CONTRIBUTING.md`](../CONTRIBUTING.md) §4。
