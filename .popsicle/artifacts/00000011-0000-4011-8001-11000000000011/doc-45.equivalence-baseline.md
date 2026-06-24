---
doc_type: equivalence-baseline
id: doc-45
pipeline_run_id: 00000011-0000-4011-8001-11000000000011
status: active
title: PROJ-17 command surface alignment equivalence report
version: 1
---

# PROJ-17 command surface alignment equivalence report

> **Issue**: PROJ-17 · cli-ux command surface alignment
> **Stage**: equivalence (slice-delivery)
> **Baseline dir**: `docs/baseline/2026-06-11/cli-ux-command-alignment/`
> **Date**: 2026-06-11

## Equivalence Reference

This change re-adjudicates the advertised command surface, so the reference is
not legacy byte parity. Equivalence is defined as:

1. **No regression** — the ADR-010 self-host baselines
   (`docs/baseline/2026-06-11/cli-ux-self-host/`, 8 scripts) must keep passing.
2. **Surface contract** — the re-adjudicated surface behaves as decided:
   help advertises only implemented commands; deferred commands fail
   actionably; `--format json` is global; tool resolution is in-workspace.

## Inventory

| # | Script | Asserts | Result |
|---|---|---|---|
| – | `../cli-ux-self-host/run-all.sh` | ADR-010 goldens (8 scripts: help, issue start, doc artifact, stage complete, admin nesting, removed commands, smoke workflow, doctor provenance) | 8/8 pass |
| 1 | `golden-001-deferred-commands.sh` | all 10 deferred commands return `deferred` category errors with next-step | pass |
| 2 | `golden-002-global-format-flag.sh` | `--format json` parses on issue/pipeline/help paths without arity breakage | pass |
| 3 | `golden-003-help-surface.sh` | help advertises only the 7 implemented families; `usage` + `deferred_commands` fields present; no deferred name in commands list (binary-level) | pass |
| 4 | `golden-004-json-errors.sh` | error output honors JSON mode: `status=error`, `category=deferred`, `next` present, exit 2 (binary-level) | pass |
| 5 | `golden-005-intent-validate-in-workspace.sh` | `tool run intent-validate path=products` exits 0 using in-workspace tool.yaml | pass |

Totals: **13/13 pass** (8 regression + 5 new surface-contract goldens).

## Test Suite

- `cargo test -p cli-ux`: unit 6 · golden 9 (6 existing + 3 new) ·
  intent_properties 7 · smoke 1 · tsv_workspace 5 — all pass.
- `tool run intent-validate path=products`: exit 0, all intents verified,
  including `RenderTopLevelHelp` and `RemovedCommandsStayRemoved`.

## Divergences / Observations

- **D-101**: issue-type default pipelines (`full-sdlc` etc.) remain unbundled;
  mitigated by the not-found error now listing available templates and by
  AGENTS.md mandating explicit `--pipeline`. Permanent fix (bundle or remap)
  deferred to the cutover ADR follow-ups.
- **O-102**: `self_host_workflow_smoke_passes` runs against the real workspace
  and accretes `PROJ-N smoke` issues/runs on every `cargo test` invocation
  (17 issues at baseline time, 23 after). Pre-existing behavior, not a
  regression; flagged for a follow-up issue (isolate smoke into a temp
  workspace like `tsv_workspace.rs` does).

## Verdict

Equivalence gate **green**: zero regressions, surface contract fully
asserted by repeatable scripts. Ready for cutover ADR.
