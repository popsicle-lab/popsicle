---
id: 6b9c8c40-8774-403b-8d1e-eae069a6e8d5
doc_type: rfc
title: RFC-009 cli-ux self-hosting workspace backend
status: final
skill_name: rfc-writer
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags:
- cli-ux
- self-hosting
- RFC-009
metadata: null
created_at: 2026-06-10T11:08:00Z
updated_at: 2026-06-10T11:52:09.073317Z
---

# RFC-009: cli-ux self-hosting workspace backend

> **Status**: Accepted for ADR finalization
> **Date**: 2026-06-10
> **Product**: cli-ux
> **Decision Candidate**: ADR-009

## Summary

Add a small file-backed backend inside `crates/cli-ux` so the local binary can run a minimal IDD workflow smoke without delegating to another popsicle binary.

## Command Surface

| Command | MVP behavior |
|---|---|
| `doctor --format json` | Print executable path, workspace root, expected workspace binary, and match flag |
| `issue create` | Append issue record to `.popsicle/self-host/state.tsv` |
| `issue list/show` | Read issue records |
| `issue start` | Create a pipeline run with one ready stage from the issue pipeline |
| `pipeline status` | Read run state and documents |
| `pipeline next` | Recommend doc create or stage complete |
| `pipeline stage complete` | Mark current stage completed and issue done when all completed |
| `doc create` | Write a Markdown artifact under `.popsicle/artifacts/<run>/` and record it in state |
| `doc list/show` | Read recorded docs and artifact content |
| `tool run intent-validate` | Execute installed tool command for file or directory targets |

## File Store

Path: `.popsicle/self-host/state.tsv`

Line types:

```text
issue<TAB>key<TAB>status<TAB>title<TAB>pipeline
run<TAB>run_id<TAB>issue_key<TAB>pipeline<TAB>stage<TAB>stage_state
doc<TAB>doc_id<TAB>run_id<TAB>skill<TAB>title<TAB>status<TAB>file_path
```

The format is intentionally simple and inspectable. It is not a replacement for future storage work.

## Provenance

`doctor --format json` returns:

- `executable`
- `workspace_root`
- `expected_workspace_binary`
- `current_workspace_binary_match`

## Architecture Manifest

| Path | Responsibility | Status |
|---|---|---|
| `crates/cli-ux/src/main.rs` | binary entrypoint; calls file-backed self-host backend | ADR-009 |
| `crates/cli-ux/src/self_host.rs` | MVP workspace store and command handlers | ADR-009 |
| `.popsicle/self-host/state.tsv` | local self-host smoke state | ADR-009 |

## Out of Scope

- Legacy SQLite compatibility.
- Full legacy command parity.
- Cloud sync and Tauri bridge.

## Checklist

- [x] Command surface defined
- [x] File format defined
- [x] Provenance fields defined
- [x] Architecture manifest ready
