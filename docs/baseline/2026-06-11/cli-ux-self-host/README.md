# cli-ux self-host golden baseline (2026-06-11)

Equivalence baselines for PROJ-10 / ADR-009 Phase 1 (`equivalence-baseline` skill).

| ID | Script | Assertion |
|---|---|---|
| G-001 | `golden-001-semantic-help.sh` | IDD help surface without removed commands |
| G-002 | `golden-002-issue-start-mock.sh` | Mock `start_issue_run` contract |
| G-003 | `golden-003-doc-artifact.sh` | Document artifact + row |
| G-004 | `golden-004-stage-complete.sh` | Stage complete requires `--confirm` |
| G-005 | `golden-005-admin-nested.sh` | Admin subcommands explicit |
| G-006 | `golden-006-removed-commands.sh` | Removed commands actionable errors |
| G-007 | `golden-007-smoke-workflow.sh` | E2E self-host workflow via `./target/debug/popsicle` |
| G-008 | `golden-008-doctor-provenance.sh` | Doctor shows binary + workspace match |

## Run

```bash
./docs/baseline/2026-06-11/cli-ux-self-host/run-all.sh
```

From repo root. Requires `cargo build -p cli-ux` first.

## Divergence

| ID | Legacy | New | ADR |
|---|---|---|---|
| D-001 | Legacy `popsicle.db` SQLite | TSV `.popsicle/self-host/` | ADR-009 / ADR-010 |
| D-002 | Parent repo `../target/debug/popsicle` | `./target/debug/popsicle` only for dogfood | ADR-010 § doctor |
| D-003 | Full 22-command CLI | IDD MVP command subset | ADR-008 D-001 |

Phase 2 SQLite: PROJ-11.
