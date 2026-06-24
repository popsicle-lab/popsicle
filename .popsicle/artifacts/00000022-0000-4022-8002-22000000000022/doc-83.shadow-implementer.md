---
doc_type: shadow-implementer
id: doc-83
pipeline_run_id: 00000022-0000-4022-8002-22000000000022
status: active
title: intent-coder embedded bundle implementation
version: 1
---

# Implementation coverage — cli-ux-intent-coder-bundle

Issue **PROJ-34**. ADR: `products/cli-ux/decisions/adr/ADR-017-intent-coder-embedded-bundle.md`.

## Checklist

- [x] Code on `main`; `make check` green
- [x] File Manifest paths implemented
- [x] Tests cover CLI/storage/UI paths

## Mapping

| Behavior | Evidence |
|---|---|
| Regression | `cargo test -p cli-ux` |
| Config | `project_config` / `local_workspace` tests |
