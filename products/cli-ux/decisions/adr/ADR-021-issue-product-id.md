# ADR-021 · Issue `product_id` (user-facing product, internal spec lock)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux / slice-4-ui
> **Generated-by**: PROJ-38

## Context

Issues stored a user-visible `spec_id` (e.g. `slice-4-ui`) while products live
under `products/cli-ux/`. Agents and UI had to mentally map slice specs to
product directories. Spec lock still needs a stable key but users should pick
**products**, not migration slice names.

## Decision

1. **`IssueRow.product_id`**: canonical user-facing field; persisted in SQLite
   `issues.product_id` with backfill from legacy `spec_id` via
   `product_for_spec()`.
2. **CLI**: `--product` flag (primary); `--spec` deprecated alias. Issue create
   validates `products/<name>/` exists.
3. **Internal lock**: `spec_id` column retained; normalized to `product_id` for
   lock key after backfill.
4. **UI**: create-issue product dropdown; list/detail show `product_id`.
5. **Guidance**: `guidance_for_issue` uses `product_id` directly.

## File Manifest

| Path | Change |
|---|---|
| `crates/storage/src/sqlite.rs` | `product_id` column migration |
| `crates/cli-ux/src/self_host.rs` | create/list/show/start with product |
| `crates/cli-ux/src/workspace_readers.rs` | `resolve_product_id`, backfill |
| `crates/cli-ux/src/ui/commands.rs` | product form options |
| `ui/src/pages/IssuesView.tsx` | product selector |

## Compliance

| Gate | Evidence | Result |
|---|---|---|
| `make check` | full workspace tests | pass |
| Product resolution | `workspace_readers` tests | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-38 cutover stage
- **Approval date**: 2026-06-11
