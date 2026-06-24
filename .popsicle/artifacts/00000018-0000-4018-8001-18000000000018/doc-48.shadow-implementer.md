---
doc_type: shadow-implementer
id: doc-48
pipeline_run_id: 00000018-0000-4018-8001-18000000000018
status: active
title: PROJ-24 usability completion coverage
version: 1
---

# PROJ-24 usability completion coverage

> **Issue**: PROJ-24 · self-host usability completion
> **Stage**: implement (slice-delivery)
> **Date**: 2026-06-11

## Scope

Close the ADR-011 follow-up gaps that block daily usability of the self-host
binary: `doc check`, `issue close`, bundled default pipelines (D-101), smoke
isolation + residue cleanup (O-102).

## Changes Delivered

### 1. `doc check <doc_id>` (PDR-001 checklist replacement)

- `storage::DocCheckRow` + `WorkspaceStore::check_doc`; TSV impl validates:
  file exists, frontmatter carries id/doc_type/title, body has prose beyond
  the heading, placeholder scan (TBD-markers and double-brace template tokens), checkbox totals.
- CLI: `doc check` returns all fields; `status: failed` + exit 1 until the
  document holds real content (main.rs now exits 1 on any failed response).

### 2. `issue close <key>`

- `WorkspaceStore::close_issue`: blocked with an actionable `active-run:`
  error while a run is incomplete; sets issue status `done` and persists.
- Closes the lifecycle loop: create → start → stages → run completed → close.

### 3. D-101: issue-type default pipelines now bundled (ADR-012)

- `IssueType::default_pipeline` remapped: product→greenfield-product-spec,
  technical→tech-decision, bug→bugfix (new minimal 2-stage template),
  idea→tech-decision. Legacy names (full-sdlc/tech-sdlc/test-only/design-only)
  never shipped as templates.
- New bundled `bugfix.pipeline.yaml`: implement → verify, no approvals.
- `load_pipeline_def` self-heals: bundled templates install on demand into
  `.popsicle/pipelines/` for workspaces bootstrapped by older binaries.

### 4. O-102: smoke isolation + residue cleanup

- `smoke_workflow.rs` rewritten: read-only doctor check against the repo;
  the entire mutating loop runs in a temp workspace (init → bug issue with
  default pipeline → start → doc create/check fail → fill → check pass →
  both stages → run completed → issue close). Temp dir removed afterwards.
- Real workspace cleaned: removed 19 smoke issues + 19 runs + 19 session
  files + 18 doc rows/artifacts (backup kept at
  `.popsicle/self-host/state.tsv.bak-proj24`). 6 real issues + PROJ-24 remain.

## Problems Hit & Optimizations (记录并优化)

| # | 问题 | 处置 |
|---|---|---|
| P1 | `doc create` 输出字段是 `id` 而非 `doc_id`,smoke 取错字段 | 修正测试;字段命名不一致记入 deferred 清单(Phase 2 统一) |
| P2 | `issue close` 响应字段 `status` 与响应状态行撞名,文本模式出现两行 `status:` | 字段更名 `issue_status`;教训:fields 命名避开 response 保留键 |
| P3 | 清理脚本用 "smoke" 子串匹配标题,误删了标题含 "smoke isolation" 的活跃 issue PROJ-24(连带 run/session) | 从备份恢复 issue/run 行并按模板重建 session JSON;教训:状态外科手术前必须备份(已做,因此可恢复),匹配条件用精确标题集而非子串 |
| P4 | 旧工作区缺新打包模板会让默认管线失效 | `load_pipeline_def` 增加 bundled 兜底自愈安装 |
| P5 | doc check 的占位符扫描命中本文档中"谈论占位符语法"的字面量,首次 dogfood 即误报 | 改写文档措辞规避字面量;"反引号内豁免"记入 deferred 优化清单 |

## Verification

- `cargo test -p cli-ux`: 全绿(golden 11 · intent 7 · smoke 1 · tsv 7 · unit 9)
- `cargo test -p skill-runtime -p storage -p artifact-system`: 全绿
- 全链 golden:self-host 8 + alignment 5 + usability 5 = 18/18 pass
- golden-004 证明 smoke 不再改动真实工作区(issue 数保持 7)
- `tool run intent-validate path=products` exit 0

## Out of Scope

- PROJ-11 SQLite Phase 2(TSV 满足当前可用性)
- 10 个 deferred 命令的逐个永久裁决
- 字段命名规范化(`id` vs `doc_id`)→ Phase 2

- [x] doc check 实现并有失败/通过双路径测试
- [x] issue close 实现并有 active-run 阻断测试
- [x] D-101 默认映射全部指向 bundled 模板
- [x] O-102 smoke 隔离 + 残留清理
- [x] 端到端 dogfood 在隔离工作区跑通
