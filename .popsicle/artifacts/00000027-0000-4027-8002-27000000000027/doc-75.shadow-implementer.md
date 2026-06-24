---
doc_type: shadow-implementer
id: doc-75
pipeline_run_id: 00000027-0000-4027-8002-27000000000027
status: active
title: Workflow approval_mode and i18n implementation
version: 1
---

# Implementation coverage — cli-ux-approval-i18n

Issue **PROJ-39**. ADR: `products/cli-ux/decisions/adr/ADR-020-workflow-approval-and-i18n.md`.

## Checklist

- [x] Code on `main`; `make check` green
- [x] File Manifest paths implemented
- [x] Tests cover CLI/storage/UI paths

## Mapping

| Behavior | Evidence |
|---|---|
| Regression | `cargo test -p cli-ux` |
| Config | `project_config` / `local_workspace` tests |
