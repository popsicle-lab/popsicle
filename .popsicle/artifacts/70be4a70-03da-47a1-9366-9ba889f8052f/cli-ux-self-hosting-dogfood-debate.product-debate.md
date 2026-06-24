---
id: d29f987b-e248-4d19-9a79-9697ff186dab
doc_type: product-debate-record
title: cli-ux self-hosting dogfood debate
status: final
skill_name: product-debate
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags:
- cli-ux
- self-hosting
- dogfood
- binary-provenance
metadata: null
created_at: 2026-06-10T10:50:33.335219Z
updated_at: 2026-06-10T10:53:25.136080Z
---

# 产品辩论纪要 — cli-ux self-hosting dogfood

> **Status**: Final
> **Date**: 2026-06-10
> **Target Product**: `cli-ux`
> **Input Mode**: `legacy-fact-baseline` + dogfood failure report
> **User Confidence**: 5/5
> **Participants**: PM, UXR, ENGLD, OPS, BIZ
> **Fact Basis**: imported cli-ux fact report + ADR-008 divergence + observed `./target/debug/popsicle` failures

## Topic

`popsicle-new/target/debug/popsicle` must self-host the IDD workflow MVP instead of relying on `../target/debug/popsicle` or a system-installed binary.

## Boundary

- **In Scope**: workflow MVP commands needed for dogfood: issue create/start/list/show, pipeline status/next/stage complete, doc create/list/show, basic tool run for `intent-validate`, and binary provenance output.
- **Out of Scope**: full legacy byte parity, cloud sync, Tauri UI bridge, all admin subcommands, and complete SQLite schema migration.
- **Touches charter?**: No. This is cli-ux delivery scope and provenance safety, not a document-system rule change.

## Phase 1: Problem Definition

### Pain

The SaaS billing dogfood run appeared to validate `popsicle-new`, but it actually used the outer repo binary at `../target/debug/popsicle`. The `popsicle-new` binary is distinct and currently rejects concrete workflow commands such as `issue list`, `pipeline`, and `doc`.

### Users

- **Primary**: AI coding agent running intent-coder workflows inside `popsicle-new`.
- **Secondary**: human maintainer checking whether a dogfood run truly used the new binary.

### Must Satisfy

- `./target/debug/popsicle` can run a minimal workflow smoke without invoking `../target/debug/popsicle`.
- The CLI exposes binary provenance so a run can tell which executable and workspace it is using.
- The implementation remains an IDD MVP, not a full resurrection of legacy 22-command parity.

### Success Metrics

- Current: `./target/debug/popsicle issue list --format json` returns `[invalid-args]`.
- Target: `./target/debug/popsicle issue create/start/list/show`, `pipeline status/next/stage complete`, `doc create/list/show` all return structured, actionable output for a local workspace smoke.
- Current: `path=products` was documented for intent validation but failed on directories.
- Target: `intent-validate path=products format=text` remains passing and is covered by the smoke or delivery evidence.

## Phase 2: Candidate Options

| Option | Core Idea | Strength | Weakness |
|---|---|---|---|
| A. Keep semantic shell only | Keep ADR-008 as-is; document that self-hosting waits | No code churn | Dogfood remains misleading |
| B. Shell-out compatibility bridge | `popsicle-new` delegates workflow commands to the outer/legacy CLI | Fastest path | Hides provenance and keeps old binary dependency |
| C. Workspace-backed MVP | Implement a small local state/artifact backend in `cli-ux` for IDD workflow commands | Self-hosting becomes real and testable | Requires focused CLI/storage work |

## Phase 3: Review

| Role | Preference | Reason | Concern | Cite |
|---|---|---|---|---|
| PM | C | Directly fixes dogfood trust | Must stay MVP-sized | PROJ-9 observed failure |
| UXR | C | Agent can trust `./target/debug/popsicle` output | Provenance must be visible | ADR-008 D-002 |
| ENGLD | C | Avoids shelling to old binary; preserves architecture boundary | Need simple file format, not giant DB rewrite | `crates/cli-ux/src/main.rs` uses `MemoryDocumentStore` |
| OPS | C | Binary path/workspace reporting prevents recurrence | Smoke should fail loudly when outside workspace | Dogfood failure |
| BIZ | C | Self-hosting is prerequisite for credible external reuse | Full parity is too expensive now | cli-ux fact report |

## Phase 4: Decision

**Selected**: Option C — Workspace-backed MVP plus binary provenance guard.

**Rationale**: ADR-008 deliberately accepted semantic shell and deferred real workspace mutation. PROJ-9 closes that deferred item only for the IDD workflow minimum needed by intent-coder dogfood. It does not reopen full legacy CLI parity.

## Intent Layer Mapping

| # | Core Statement | Intent Layer | Follow-up |
|---|---|---|---|
| 1 | CLI workflow commands mutate/read local workspace state under `.popsicle` | `acceptance.intent` | PDR-002 |
| 2 | A run reports executable path and workspace root for provenance | `acceptance.intent` | PDR-002 |
| 3 | The shell must not silently depend on parent/system binaries for workflow smoke | `invariants.intent` | PDR-002 |
| 4 | Workspace-backed storage is an implementation boundary decision | `contracts.intent` / ADR | ADR-009 |

## Downstream Handoff

- prd-writer: add a focused self-hosting task to `products/cli-ux`.
- arch-debate/rfc/adr: decide the MVP workspace state file format and provenance command surface.
- intent-spec-writer: add minimal intent coverage for workflow smoke and provenance.
- slice-delivery: implement CLI wiring and smoke using `./target/debug/popsicle`.

## Checklist

- [x] Input mode recorded
- [x] Topic is one sentence
- [x] Target product bound to `cli-ux`
- [x] Fact basis recorded
- [x] Roles selected
- [x] Final decision made
