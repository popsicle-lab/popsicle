# ADR-020 · Workflow approval mode, default product, UI/CLI i18n

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: PROJ-39
> **Extends**: ADR-019

## Context

ADR-019 introduced per-project `project.yaml` with agent language and paths.
Agents were hard-coded to wait for human `--confirm` on every `requires_approval`
stage. Settings still exposed legacy `default_spec` instead of product ids.
CLI help and desktop UI strings were English-only regardless of `agent.language`.

## Decision

1. **`workflow.approval_mode`**: `manual` | `auto` | `delegate-dangerous` (default
   `manual`). Dangerous stages remain `cutover` and `living-docs`; others may
   delegate per project policy (`project_config::stage_needs_explicit_confirm`).
2. **`paths.default_product`**: replaces user-facing `default_spec` (serde alias
   retained). Resolved via `resolve_default_product()`; validated on save.
3. **i18n**: `agent.language` drives CLI `help` usage lines (`i18n.rs`), Settings
   / sidebar / breadcrumbs (`ui/src/i18n/`), and bilingual `AGENTS.md` marker sync.
4. **Doctor** reports `approval_mode` and `default_product`.

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/project_config.rs` | `ApprovalMode`, `default_product`, localized marker |
| `crates/cli-ux/src/i18n.rs` | CLI usage strings |
| `crates/cli-ux/src/self_host.rs` | approval gate + `project_language()` |
| `crates/cli-ux/src/ui/commands.rs` | Settings DTO, product dropdown |
| `ui/src/i18n/*` | Locale provider + messages |
| `README.md` | legacy vs new design differences |

## Compliance

| Gate | Evidence | Result |
|---|---|---|
| `make check` | fmt + clippy + test | pass |
| Approval policy tests | `project_config`, `local_workspace` | pass |
| UI build | `npm run build` | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-39 cutover stage
- **Approval date**: 2026-06-11
