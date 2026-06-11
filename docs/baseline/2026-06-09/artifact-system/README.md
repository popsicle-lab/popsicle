# artifact-system golden baseline (2026-06-09)

Equivalence baselines for `slice-delivery` / `equivalence-baseline` skill.

Each `golden-*.sh` runs one `cargo test` filter; `run-all.sh` runs the full set.

| ID | Script | Assertion |
|---|---|---|
| G-001 | `golden-001-document-roundtrip.sh` | Document file-content round-trip preserves id/version/body |
| G-002 | `golden-002-guard-checks.sh` | `has_sections` + `checklist_complete` outcomes |
| G-003 | `golden-003-context-assembly.sh` | Relevance-aware deterministic context ordering |
| G-004 | `golden-004-extractors.sh` | Extractors preserve kind; no-match returns empty |
| G-005 | `golden-005-task-chunk-rename.sh` | `work_item` -> `task_chunk` preserves kind/fields |
| G-006 | `golden-006-upstream-port.sh` | `upstream_approved` requires injected checker port |

## Run

```bash
./docs/baseline/2026-06-09/artifact-system/run-all.sh
```

From repo root.

## Divergence (documented)

| ID | Legacy | New | ADR |
|---|---|---|---|
| D-001 | Full serde YAML `Document` wire format | Minimal deterministic frontmatter model | ADR-006 |
| D-002 | CLI `doc` / `prompt` / `extract` commands | Lib-level API only | ADR-006 |

Lib-level golden only; CLI byte-parity remains in `cli-ux` slice.
