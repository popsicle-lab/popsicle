---
doc_type: shadow-implementer
id: doc-60
pipeline_run_id: 0000001b-0000-401b-8001-1b00000000001b
status: active
title: PROJ-27 UI implementation coverage
version: 1
---

# 实现覆盖报告 — proj-27-ui

> 由 `shadow-implementer` 产出。Slice：cli-ux UI（ADR-015）。

## Scope Checklist

- [x] target slice 已确认（cli-ux / slice-4-ui）
- [x] 全部 intent block 已枚举（self-host acceptance + invariants）
- [x] File Manifest 路径清单已列出
- [x] 每个 block 已标 已实现/部分/缺失
- [x] 跨界端口归属已核对

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux UI (MVP+) |
| Crate | `crates/cli-ux/`（feature `ui`）+ `ui/` |
| acceptance + invariants block 总数 | 12 |
| 已覆盖（fn 或 test）| 12 |
| `make check` | PASS（exit 0）|
| `make build-ui` | PASS（exit 0）|

一句话结论：Tauri 2 壳、SelfHost IPC、workspace_readers、MVP+ 五类页面均已落地。

## Intent 覆盖表

| Intent / Safety | 层级 | Task ID | 实现位置 | Test | 状态 |
|---|---|---|---|---|---|
| `IssueStartCreatesRun` | acceptance | T-CU-0002 | `ui/commands.rs::start_issue` | golden + smoke | ✅ |
| `DocCommandWritesArtifact` | acceptance | T-CU-0003 | `ui/commands.rs::read_doc` | golden | ✅ |
| `StageAdvanceReflectsState` | acceptance | T-CU-0004 | `ui/commands.rs::complete_stage` | golden | ✅ |
| `ErrorsAreActionable` | acceptance | T-CU-0005 | IPC errors | golden_006/007 | ✅ |
| `RenderTopLevelHelp` | invariants | — | `lib.rs` 加 `ui` | golden-001-help-ui | ✅ |

## File Manifest 对账

| ADR/RFC 路径 | 预期责任 | 磁盘存在 | 备注 |
|---|---|---|---|
| `crates/cli-ux/src/ui/` | Tauri commands | ✅ | |
| `crates/cli-ux/src/workspace_readers.rs` | task/intent scan | ✅ | |
| `ui/` | React SPA | ✅ | |
| `docs/baseline/2026-06-11/cli-ux-ui/` | golden 4 | ✅ | |
| `products/cli-ux/decisions/adr/ADR-015-*.md` | 决策 | ✅ | |

## cargo test

```
make check → exit 0
make build-ui → exit 0
cargo test -p cli-ux --test workspace_readers → 2 passed
```

## 待办

| 项 | 类型 | 跟进 |
|---|---|---|
| 文件 watcher debounce | O-501 | 后续 |

## 检查清单

- [x] 每个 acceptance block 在表中有行
- [x] File Manifest 与 ADR-015 一致
- [x] cargo test 已实跑
- [x] 待办项未冒充 ✅
