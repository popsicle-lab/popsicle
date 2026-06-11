# skill-runtime golden baseline (2026-06-09)

Equivalence baselines for `slice-delivery` / `equivalence-baseline` skill.

Each `golden-*.sh` runs one `cargo test` filter; `run-all.sh` runs the full set.

| ID | Script | Assertion |
|---|---|---|
| G-001 | `golden-001-load-project-init.sh` | Load `project-init` → ADR-002 `SkillLoadResult` |
| G-002 | `golden-002-migration-bootstrap.sh` | `migration-bootstrap` 10 stages, valid DAG |
| G-003 | `golden-003-slice-delivery.sh` | `slice-delivery` 4 stages |
| G-004 | `golden-004-skill-registry.sh` | 13 intent-coder skills registered |
| G-005 | `golden-005-state-machine.sh` | Canonical 3 transitions, no bypass |
| G-006 | `golden-006-pipeline-session.sh` | Session bootstrap → complete advances index |

## Run

```bash
./docs/baseline/2026-06-09/skill-runtime/run-all.sh
```

From repo root. Requires `intent-coder` v0.4 module at `.popsicle/modules/intent-coder/`.

## Divergence (documented)

| ID | Legacy | New | ADR |
|---|---|---|---|
| — | Full SQLite `IndexDb` | In-memory `MemoryDocumentStore` only | Pending cutover ADR |

Lib-level golden only; CLI byte-parity deferred to `cli-ux` slice.
