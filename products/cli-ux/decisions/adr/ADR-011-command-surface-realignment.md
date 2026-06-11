# ADR-011 · cli-ux command surface realignment

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-17）
> **Source-Baseline**: `docs/baseline/2026-06-11/cli-ux-command-alignment/`
> **Amends**: PDR-001（cli-ux command tree redesign）disposition table
> **Relates**: ADR-008（semantic shell cutover）· ADR-010（self-host Phase 1）

## Context

PDR-001 disposed the legacy 22-command tree as "preserve 17 / drop 3 /
redesign 2". PDR-002/ADR-010 then delivered a deliberately smaller self-host
MVP (7 command families), but the three facts-sources — `help` output, error
behavior, and root `AGENTS.md` — kept advertising the PDR-001 preserve list.
Agents discovered 17 top-level commands, 10 of which fell through to
"unknown or incomplete command", and only `doctor` honored `--format json`.

This ADR records the **re-adjudication of the PDR-001 preserve list against
the implemented surface**, aligning advertisement with reality instead of
reality with advertisement (D4 simplification: ≤7 public commands).

## Decision

1. **Implemented surface (advertised)** — 7 families, 15 subcommands:
   `init`, `doctor`, `issue` (create/list/show/start), `pipeline`
   (status/next/stage complete), `doc` (create/list/show), `tool` (run),
   `admin` (migrate/reinit).
2. **Deferred surface** — 10 PDR-001 "preserve" names are reclassified
   **deferred**: `module`, `skill`, `spec`, `namespace`, `prompt`, `git`,
   `memory`, `context`, `registry`, `completions`. They are not advertised in
   help and fail with a structured `deferred` error carrying a next-step.
   Permanent disposition (implement vs drop) requires a future PDR amendment
   per command; until then AGENTS.md documents replacement practices.
3. **Removed surface (unchanged)** — `checklist`, `item`, `sync` stay removed
   per PDR-001 / skill-runtime PDR-001 (D4).
4. **`--format json` is a global flag** — every command and every error emits
   machine-readable output on request; the actionable next-step contract
   (ErrorsAreActionable) extends to JSON mode.
5. **Tool resolution is workspace-strict** — `tool run intent-validate`
   resolves `tool.yaml` only inside the workspace
   (`intent-coder/` → `.popsicle/modules/intent-coder/`), removing the
   pre-promotion `root.parent()` lookup that could silently execute a sibling
   checkout's definition (same provenance class as ADR-010 D-003).
6. **Root `AGENTS.md` is bound to the implemented surface** — it documents
   only commands that parse, the five bundled pipeline templates, and
   replacement practices for deferred capabilities.

## Divergences

- **D-101**: issue-type default pipelines (`full-sdlc`, `tech-sdlc`,
  `test-only`, `design-only`) are not bundled. Mitigation: pipeline not-found
  errors list available templates; AGENTS.md mandates explicit `--pipeline`.
  Permanent fix (bundle or remap defaults) is a follow-up.
- **O-102**: `self_host_workflow_smoke_passes` mutates the real workspace,
  accreting smoke issues/runs per `cargo test`. Pre-existing; follow-up issue
  to isolate it into a temp workspace.

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `tool run intent-validate path=products` exit 0（含 RenderTopLevelHelp / RemovedCommandsStayRemoved）| pass |
| Golden ≥5 | `docs/baseline/2026-06-11/cli-ux-command-alignment/run-all.sh` 13 scripts（8 回归 + 5 新契约）| pass |
| cargo test | `cargo test -p cli-ux`（unit 6 · golden 9 · intent 7 · smoke 1 · tsv 5）| pass |
| Provenance | doctor `current_workspace_binary_match=true` | pass |

## Follow-ups

- PROJ-11: SQLite Phase 2 storage（已有 tracking）
- `doc check`（PDR-001 指定的 checklist 替代物）尚未实现 — 需要单独 issue
- D-101 永久处置：bundle 默认管线模板或改写 `IssueType::default_pipeline`
- O-102:smoke 测试隔离到临时工作区
- 10 个 deferred 命令的逐个永久裁决（PDR 修订）

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-17 slice-delivery cutover stage（user `--confirm`）
- **Approval date**: 2026-06-11
