---
doc_type: shadow-implementer
id: doc-41
pipeline_run_id: 00000011-0000-4011-8001-11000000000011
status: active
title: PROJ-17 command surface alignment coverage
version: 1
---

# PROJ-17 command surface alignment coverage

> **Issue**: PROJ-17 · cli-ux command surface alignment (help/json/AGENTS.md)
> **Stage**: implement (slice-delivery)
> **Spec**: slice-3-cli-ux
> **Date**: 2026-06-11

## Scope

Re-adjudicate the PDR-001 (cli-ux command tree redesign) "preserve" list
against the self-host MVP actually delivered by PDR-002/ADR-010, and align the
three facts-sources (help output, error behavior, root AGENTS.md) with the
implemented surface.

## Changes Delivered

### 1. Help surface shrunk to implemented commands (`crates/cli-ux/src/lib.rs`)

- `TOP_LEVEL_COMMANDS` reduced from 17 advertised names to the 7 implemented
  families: `init, doctor, issue, pipeline, doc, tool, admin`. This satisfies
  the D4 simplification target of <= 7 public commands per product surface.
- New `DEFERRED_TOP_LEVEL_COMMANDS` (10 names: `module, skill, spec, namespace,
  prompt, git, memory, context, registry, completions`): invoking any of them
  now returns an actionable `deferred` error pointing at `popsicle --help`,
  instead of a generic "unknown command".
- `help_response()` now emits `usage` (full syntax of all 15 subcommands),
  `global_flags`, and `deferred_commands` fields so agents can discover the
  real surface from one call.

### 2. `--format json` is now a global flag (`lib.rs`, `main.rs`)

- `parse_args` records and strips `--format <v>` before positional matching,
  so every command accepts it (previously only `doctor`).
- `main.rs` selects the output format from the raw args for all commands.
- Errors honor JSON mode: `{"status":"error","category","object","message","next"}`
  on stderr, preserving the actionable next-step contract.

### 3. Pipeline not-found errors list available templates (`self_host.rs`)

- `load_pipeline_def` failure now reports
  `pipeline <name> (available: greenfield-product-spec, migration-bootstrap, ...)`,
  closing the trap where issue-type default pipelines (`full-sdlc`, `tech-sdlc`,
  `test-only`, `design-only`) reference templates not bundled in this workspace.

### 4. Bugfix: `tool run intent-validate` resolved tool.yaml outside the repo (`self_host.rs`)

- `run_intent_validate` preferred `workspace.root.parent()/intent-coder/...`,
  a layout assumption from before the repo-root promotion (4d8b5c6). With a
  sibling checkout named `intent-coder` present, it silently used that stale
  copy and failed with exit 127 — the same provenance bug class ADR-010 D-003
  blocks for binaries. Resolution is now strictly in-workspace:
  `intent-coder/tools/...` then `.popsicle/modules/intent-coder/tools/...`.
- Verified: `tool run intent-validate path=products` now exits 0 with all
  intents verified.

### 5. Root `AGENTS.md` rewritten to the real surface

- Binary resolution now includes `./target/debug/popsicle`.
- Mandatory checklist uses only implemented commands; bundled pipeline table
  added; explicit warning against relying on issue-type default pipelines.
- New "Deferred & Removed Commands" section with replacement practices for
  `memory`, `context search`, `doc summarize`, `git link`, `checklist`,
  `pipeline verify`, and spec/namespace creation.
- Dropped instructions referencing nonexistent commands (`context bootstrap`,
  `doc summarize` protocol, checklist CLI, memory CLI, `pipeline verify`).

## Verification

- `cargo test -p cli-ux`: all suites pass (golden 6, intent_properties 7,
  smoke 1, tsv_workspace 5, unit 6).
- `tool run intent-validate path=products`: exit 0, all intents verified
  (incl. RenderTopLevelHelp, RemovedCommandsStayRemoved).
- `docs/baseline/2026-06-11/cli-ux-self-host/run-all.sh`: 8/8 golden pass.
- Manual: `help` shows 7 commands + usage; `issue show PROJ-17 --format json`
  returns JSON; `memory list --format json` returns structured deferred error.
- Intents untouched; `RemovedCommandsStayRemoved` / `RenderTopLevelHelp`
  invariants still hold (removed commands stay out of help and parse).

## Out of Scope (deferred to follow-ups)

- SQLite Phase 2 storage (PROJ-11).
- Implementing or permanently dropping the 10 deferred commands — the cutover
  stage ADR of this run records the re-adjudication; permanent disposition
  needs a PDR amendment.
- `doc check` (PDR-001 named it the checklist replacement; not yet implemented).
