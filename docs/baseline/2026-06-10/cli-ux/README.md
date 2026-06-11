# cli-ux golden baseline (2026-06-10)

Equivalence baselines for `slice-delivery` / `equivalence-baseline` skill.

Each `golden-*.sh` runs one `cargo test -p cli-ux` filter from `crates/cli-ux/tests/golden.rs`; `run-all.sh` runs the full set. Counts are sourced from `baseline.yaml`.

| ID | Script | Assertion |
|---|---|---|
| G-001 | `golden-001-help-surface.sh` | Help exposes IDD main path and omits removed top-level commands |
| G-002 | `golden-002-issue-start.sh` | Issue start creates a run id and spec lock signal |
| G-003 | `golden-003-doc-create.sh` | Doc create produces artifact content and a document row |
| G-004 | `golden-004-stage-complete.sh` | Stage complete requires `--confirm`, then advances state |
| G-005 | `golden-005-admin-tree.sh` | `migrate` / `reinit` live under `admin` |
| G-006 | `golden-006-removed-commands.sh` | Dropped/deferred commands return actionable errors |

## Run

```bash
./docs/baseline/2026-06-10/cli-ux/run-all.sh
```

From repo root.

## Divergence (documented)

| ID | Legacy | New | ADR |
|---|---|---|---|
| D-001 | Full legacy stdout/stderr byte parity and 22-command compatibility | Semantic IDD command shell; acceptance locks effects, not wording bytes | ADR-007 |

Semantic golden only; this follows PDR-001, which explicitly says `products/cli-ux/intents/acceptance.intent` captures command effects, not byte parity.
