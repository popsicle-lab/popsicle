# PDR-005: issue link 命令（Issue↔Task 生命周期回写）

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Source**: PROJ-48（PROJ-46 工作流缺口）

## Decision

新增 `popsicle issue link <key> --tasks T1,T2 [--replace] [--drop-proposed]`：

- 在 **issue create 之后** 增删 `linked` task 关联（晋升 proposed、纠错错链）。
- `--replace` 先移除既有 `linked` 再写入；`--drop-proposed` 移除 `proposed` 行（spec 落地后晋升）。
- 校验 task 文件存在于 `products/<product>/tasks/`。

## Intent Impact

| Intent | Task | Meaning |
|---|---|---|
| `IssueTaskLinksMutable` | T-CU-0015 | link 更新关联且校验 task 文件 |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-48
