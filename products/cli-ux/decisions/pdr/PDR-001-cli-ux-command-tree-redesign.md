# PDR-001 · cli-ux command tree redesign

> **Status**: Proposed
> **Date**: 2026-06-09
> **Product**: cli-ux
> **Source**: PROJ-6 product-debate + fact-extractor

## Context

legacy `popsicle` exposes 22 subcommands. The cli-ux migration is not a full compatibility rewrite; it exposes the already-migrated `skill-runtime` and `artifact-system` capabilities through a smaller, agent-friendly command shell.

## Decision

| legacy command | disposition | new surface |
|---|---|---|
| `init` | preserve | `popsicle init` |
| `module` | preserve | `popsicle module` |
| `tool` | preserve | `popsicle tool` |
| `skill` | preserve | `popsicle skill` |
| `pipeline` | preserve | `popsicle pipeline` |
| `spec` | preserve | `popsicle spec` |
| `issue` | preserve | `popsicle issue` |
| `namespace` | preserve-minimal | `popsicle namespace create/list/use` |
| `doc` | preserve-redesign | `popsicle doc create/list/show/check` |
| `extract` | redesign | under `doc extract` or artifact command, not separate MVP top-level |
| `prompt` | redesign | `popsicle prompt` as agent context shell |
| `admin` | preserve | `popsicle admin` |
| `migrate` | move | `popsicle admin migrate` |
| `reinit` | move | `popsicle admin reinit` |
| `git` | preserve | `popsicle git` |
| `memory` | preserve | `popsicle memory` |
| `context` | preserve | `popsicle context` |
| `registry` | preserve | `popsicle registry` |
| `completions` | preserve | `popsicle completions` |
| `checklist` | drop | use `doc check` |
| `item` | drop | task_chunk/doc path |
| `sync` | defer/drop | out of IDD MVP |

## Consequences

- `products/cli-ux/tasks/**` contains 7 task chunks.
- `products/cli-ux/intents/acceptance.intent` captures command effects, not byte parity.
- ADR-007 must define `crates/cli-ux` as IO shell only.

## Approval

- **Status**: Proposed
- **Approved by**: pending prd review
- **Approval date**:
