# cli-ux SQLite Phase 2 — golden baseline (PROJ-25 / tracks PROJ-11)

> **Run**: `00000019-0000-4019-8001-19000000000019` (slice-delivery)
> **Date**: 2026-06-11

Golden baselines for the ADR-009 Phase 2 storage backend: SQLite at
`.popsicle/self-host/state.db` (NOT `.popsicle/popsicle.db` — that path is the
legacy binary's database, see ADR-013), TSV legacy read/write compatibility,
`admin migrate` TSV → SQLite migration, and the full e2e workflow running on
the SQLite backend.

Run everything (chains all earlier cli-ux baselines first):

```bash
bash docs/baseline/2026-06-11/cli-ux-sqlite-phase2/run-all.sh
```
