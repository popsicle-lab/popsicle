# ADR-019 · Per-project agent config (yaml + AGENTS.md + Settings UI)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux / slice-4-ui
> **Generated-by**: cutover-author（PROJ-37）
> **Extends**: ADR-016, ADR-017

## Context

Global multi-project registry (`~/.popsicle/global.json`, ADR-016) tracks
**which** repos are open, not **how agents should behave inside each repo**.
Legacy had `.popsicle/config.toml` and context scan; self-host MVP deferred
`context` / `prompt`. PROJ-37 adds a per-workspace preferences file and two
sync channels: durable `AGENTS.md` markers and runtime injection on
`issue start` / `doc create`.

## Decision

1. **Source of truth**: `.popsicle/project.yaml` (`project_config.rs`).
   Fields: `agent.language` (`zh-CN` | `en`), `paths.products_dir`,
   optional `paths.default_spec`, `workflow.sync_agents_md`,
   `workflow.inject_on_run`.
2. **Init**: `popsicle init` calls `ensure_project_config` (default language
   from `LANG`, default `products_dir: products`).
3. **AGENTS.md sync**: idempotent marker block
   `<!-- popsicle:project-config:start/end -->`; refreshed on init (when
   enabled), `admin sync-project-config`, and UI save.
4. **Runtime injection**: `issue start` JSON field `agent_context`;
   `doc create` frontmatter `agent_context` (does not affect `doc check`).
5. **UI**: sidebar **Settings** page; Tauri `get_project_config` /
   `save_project_config_cmd`.
6. **Readers**: `list_products` respects configured `products_dir`.
7. **Golden**: `docs/baseline/2026-06-11/cli-ux-project-config/` (5 scripts).

## Divergences

- **D-801**: No full `ContextLayer` / `popsicle prompt` assembly yet;
  injection is a short preferences block, not upstream doc harvesting.

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| `make check` | fmt + clippy + test | pass |
| Project config golden 5/5 | `cli-ux-project-config/run-all.sh` | pass |
| `npm run build` (ui/) | golden-005 | pass |
| `doc check` unchanged for stubs | `local_workspace` test | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-37 slice-delivery (user-approved pipeline advance)
- **Approval date**: 2026-06-11
