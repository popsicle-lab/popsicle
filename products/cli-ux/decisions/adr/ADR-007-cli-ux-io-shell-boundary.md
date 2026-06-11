# ADR-007 · cli-ux IO shell boundary

> **Status**: Accepted
> **Date**: 2026-06-09
> **Product**: cli-ux
> **Source-RFC**: RFC-007 cli-ux IO shell boundary

## Context

cli-ux exposes `skill-runtime` and `artifact-system` through the `popsicle` binary. The old CLI mixed command parsing with domain behavior. The new design keeps CLI behavior observable without reintroducing legacy coupling.

## Decision

`crates/cli-ux` is an IO shell. Command handlers parse argv, call domain crates, format output, and produce actionable errors. They do not own pipeline state-machine logic, document parsing, guard evaluation, context assembly, extraction logic, or storage schema logic.

## Consequences

- `crates/cli-ux/` may depend on `skill-runtime`, `artifact-system`, and `storage`.
- `products/cli-ux/intents/contracts.intent` unlocks `CliShellDelegatesToDomainCrates`.
- `products/cli-ux/ARCHITECTURE.md` records the dependency boundary.

## Intent Impact

| Goal | Status |
|---|---|
| `CliShellDelegatesToDomainCrates` | unlocked by ADR-007 |

## Approval

- **Status**: Accepted
- **Approved by**: @curtiseng（经 `pipeline stage complete adr --confirm`）
- **Approval date**: 2026-06-09
