# ADR-009: cli-ux self-hosting workspace backend

> **Status**: Accepted
> **Date**: 2026-06-10
> **Product**: cli-ux
> **Source RFC**: RFC-009

## Context

ADR-008 cut over cli-ux as a semantic shell and explicitly deferred storage-backed real workspace mutation. PROJ-9 exposed the cost of that deferral: SaaS billing dogfood ran through `../target/debug/popsicle`, not `popsicle-new/target/debug/popsicle`.

## Decision

Implement a self-hosting MVP backend in `crates/cli-ux`:

- keep command parsing/output in cli-ux,
- use a compact `.popsicle/self-host/state.tsv` file for local smoke state,
- write doc artifacts under `.popsicle/artifacts/<run>/`,
- expose `doctor` provenance fields,
- keep full legacy DB compatibility out of scope.

## Consequences

- `popsicle-new/target/debug/popsicle` becomes capable of running the minimal IDD workflow smoke.
- Future storage work may replace the TSV backend without changing the smoke contract.
- Provenance output makes wrong-binary dogfood visible.

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-9 ADR stage
