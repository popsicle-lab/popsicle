---
id: a85692f9-b106-4774-9b58-cdb37757085c
doc_type: adr-finalization-report
title: ADR-009 cli-ux self-hosting backend finalization
status: final
skill_name: adr-writer
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags:
- cli-ux
- ADR-009
- self-hosting
metadata: null
created_at: 2026-06-10T11:12:00Z
updated_at: 2026-06-10T14:11:01.490581Z
---

# ADR-009 Finalization — cli-ux self-hosting backend

> **Status**: Final
> **Date**: 2026-06-10
> **Source RFC**: `rfc-009-cli-ux-self-hosting-workspace-backend.rfc.md`

## Final Decision

ADR-009 is accepted. `popsicle-new` may implement a file-backed self-hosting workflow MVP in `crates/cli-ux`, with state under `.popsicle/self-host/state.tsv`, artifact files under `.popsicle/artifacts/<run>/`, and binary provenance exposed through `doctor`.

## Scope Lock

- Include: workflow smoke commands, provenance, and `intent-validate` tool bridge.
- Exclude: full legacy DB compatibility, cloud sync, Tauri bridge, and byte-for-byte CLI parity.

## Contracts Unlocked

- `SelfHostedWorkflowSmokePasses`
- `BinaryProvenanceVisible`
- `WorkflowSmokeDoesNotDependOnParentBinary`

## Checklist

- [x] ADR status accepted
- [x] Scope lock recorded
- [x] Implementation handoff clear
