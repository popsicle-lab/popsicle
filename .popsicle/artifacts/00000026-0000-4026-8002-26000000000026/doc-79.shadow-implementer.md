---
doc_type: shadow-implementer
id: doc-79
pipeline_run_id: 00000026-0000-4026-8002-26000000000026
status: active
title: Issue product_id field implementation
version: 1
---

# Implementation coverage — cli-ux-product-id

Issue **PROJ-38**. ADR: `products/cli-ux/decisions/adr/ADR-021-issue-product-id.md`.

## Checklist

- [x] Code on `main`; `make check` green
- [x] File Manifest paths implemented
- [x] Tests cover CLI/storage/UI paths

## Mapping

| Behavior | Evidence |
|---|---|
| Regression | `cargo test -p cli-ux` |
| Config | `project_config` / `local_workspace` tests |
