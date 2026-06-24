---
id: ff0e2f98-16da-4d05-a330-e10fa2350abd
doc_type: prd-overview
title: cli-ux self-hosting workflow MVP PRD
status: final
skill_name: prd-writer
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags:
- cli-ux
- self-hosting
- PDR-002
metadata: null
created_at: 2026-06-10T10:53:52.737629Z
updated_at: 2026-06-10T10:59:31.659886Z
---

# PRD Overview — cli-ux self-hosting workflow MVP

> **Status**: Approved
> **Target Product**: `cli-ux`
> **Source Debate**: `cli-ux-self-hosting-dogfood-debate.product-debate.md`
> **Input Mode**: `legacy-fact-baseline` + dogfood failure
> **Fact Basis**: imported cli-ux fact report; ADR-008 D-002; observed `./target/debug/popsicle` failures
> **PDR**: `PDR-002-cli-ux-self-hosting-workflow-mvp.md`
> **Quality Score**: 92/100
> **Last-Updated**: 2026-06-10

## 1. Core Intent

Users can run a minimal IDD workflow with `popsicle-new/target/debug/popsicle` itself and can verify which binary/workspace handled the run.

## 2. Problem Statement

**Current Situation**: `./target/debug/popsicle` reports the semantic top-level command list, but concrete workflow commands such as `issue list`, `pipeline`, and `doc` fail with `[invalid-args]`. The SaaS billing dogfood run therefore used `../target/debug/popsicle`, which invalidates self-hosting confidence.

**Proposed Solution**: Add a workspace-backed workflow MVP to cli-ux and a provenance command/output path that reports executable path and workspace root.

**Business Impact**: External users of intent-coder get a reusable self-hosting CLI instead of a repo-specific bootstrap shortcut.

`Decision-Ref: PDR-002`

## 3. Success Metrics

| Metric | Baseline | Target | Measurement |
|---|---|---|---|
| Self-host workflow smoke | `issue list` / `pipeline` / `doc` fail | create issue -> start run -> next -> create doc -> complete stage -> status all via `./target/debug/popsicle` | scripted local smoke |
| Provenance visibility | no binary path/workspace output | `popsicle doctor` reports binary path, workspace root, current-workspace binary match | CLI smoke |
| Directory intent validation | fixed during SaaS billing run | `intent-validate path=products format=text` remains exit 0 | tool smoke |

## 4. File List

### New Task

| Task ID | Title | Journey Stage | Path |
|---|---|---|---|
| T-CU-0008 | 自举运行 workflow 并确认 binary provenance | lifecycle | `products/cli-ux/tasks/lifecycle/T-CU-0008-self-host-workflow-provenance.md` |

### Modified Files

| File | Change |
|---|---|
| `products/cli-ux/tasks/README.md` | Add T-CU-0008 and update lifecycle count |
| `products/cli-ux/intents/acceptance.intent` | Add `SelfHostedWorkflowSmokePasses` and `BinaryProvenanceVisible` |
| `products/cli-ux/intents/invariants.intent` | Add `WorkflowSmokeDoesNotDependOnParentBinary` |
| `products/cli-ux/decisions/pdr/PDR-002-cli-ux-self-hosting-workflow-mvp.md` | Record accepted product decision |

## 5. Intent Mapping

| # | Core Statement | Layer | Task | Intent |
|---|---|---|---|---|
| 1 | Self-hosted binary can run the workflow smoke | `acceptance.intent` | T-CU-0008 | `SelfHostedWorkflowSmokePasses` |
| 2 | Binary provenance is visible | `acceptance.intent` | T-CU-0008 | `BinaryProvenanceVisible` |
| 3 | The smoke does not silently use parent/system binaries | `invariants.intent` | T-CU-0008 | `WorkflowSmokeDoesNotDependOnParentBinary` |

## 6. Out of Tasks

- Full legacy CLI byte parity.
- Cloud sync and Tauri bridge.
- Complete SQLite-backed legacy database compatibility.

## 7. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| MVP becomes a second full CLI rewrite | Medium | High | Restrict to workflow smoke commands only |
| Provenance reports but does not prevent wrong binary use | Medium | Medium | Smoke must call `./target/debug/popsicle`; doctor reports mismatch |
| Tool bridge hides intent-coder behavior | Low | Medium | Keep `intent-validate path=products` direct tool check in evidence |

## Checklist

- [x] One task added
- [x] Acceptance and invariant mapping complete
- [x] Out-of-scope excludes full legacy parity
- [x] Ready for arch-debate/rfc/adr
