---
id: e38a1288-606a-4bce-a4ff-69c599b67ef7
doc_type: arch-debate-record
title: cli-ux self-hosting workspace backend debate
status: final
skill_name: arch-debate
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags:
- cli-ux
- self-hosting
- ADR-009
metadata: null
created_at: 2026-06-10T11:05:00Z
updated_at: 2026-06-10T11:01:26.280612Z
---

# Architecture Debate — cli-ux self-hosting workspace backend

> **Status**: Final
> **Date**: 2026-06-10
> **Target Product**: `cli-ux`
> **Source PRD**: `cli-ux-self-hosting-workflow-mvp-prd.prd.md`

## Topic

Choose the smallest backend that lets `popsicle-new/target/debug/popsicle` run a local workflow smoke without delegating to another binary.

## Candidate Designs

| Option | Summary | Pros | Cons |
|---|---|---|---|
| A. Legacy DB compatibility | Reuse `.popsicle/popsicle.db` schema | Max compatibility | Too much surface for self-hosting MVP |
| B. Shell-out bridge | Call `../target/debug/popsicle` for workflow commands | Fast | Recreates the dogfood failure |
| C. File-backed MVP | Store smoke issues/runs/docs under `.popsicle/self-host/state.tsv` and artifact files under `.popsicle/artifacts/<run>` | Small, inspectable, self-contained | Not full legacy compatibility |

## Decision

Select **C. File-backed MVP**.

## Boundary

- `cli-ux` owns parsing, output formatting, provenance, and MVP file-backed command effects.
- `artifact-system` still owns `Document` serialization.
- `storage` can remain memory-only for prior tests; this MVP may use a cli-ux internal file store until a broader storage slice exists.

## Required Commands

- `doctor --format json`
- `issue create/list/show/start`
- `pipeline status/next/stage complete`
- `doc create/list/show`
- `tool run intent-validate path=... format=...`

## Risks

| Risk | Mitigation |
|---|---|
| Temporary file store becomes permanent by accident | Mark as self-host MVP and keep path under `.popsicle/self-host/` |
| JSON output diverges from old CLI | Smoke only requires stable fields, not byte parity |
| Binary provenance is informational only | Smoke must invoke `./target/debug/popsicle` and check `current_workspace_binary_match` |

## Handoff

- rfc-writer: create RFC-009 with file format and command surface.
- adr-writer: accept ADR-009 and unlock implementation.

## Checklist

- [x] Options considered
- [x] Decision selected
- [x] Boundaries named
- [x] RFC handoff ready
