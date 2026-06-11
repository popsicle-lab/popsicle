# ADR-017 · intent-coder embedded bundle + install path

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: PROJ-34
> **Supersedes**: informal “copy from repo root only” behavior (pre-0.4.1)

## Context

Legacy popsicle exposed `popsicle module add <path>` to install modules such as
intent-coder into `.popsicle/modules/`. Self-host MVP **deferred** `module` per
ADR-011; only **pipeline YAMLs** were `include_str!`’d into the binary.

PROJ-34 added `install_intent_coder_module()` copying from **workspace-root**
`intent-coder/` only. That works for the popsicle **dogfood monorepo** but fails
for:

- DMG / `cargo install` users starting a **new project** elsewhere (no
  `intent-coder/` sibling directory)
- `popsicle tool run intent-validate` (needs
  `.popsicle/modules/intent-coder/tools/intent-validate/tool.yaml`)

The macOS DMG ships **only** `Popsicle.app`, `popsicle` CLI, and
`Install CLI.command` — **not** a separate `intent-coder/` folder.

## Decision

1. **Compile-time bundle**: `include_dir!` embeds repo `intent-coder/` into the
   `popsicle` binary at build time (`crates/cli-ux/src/intent_coder_bundle.rs`).
2. **`popsicle init`**: installs pipelines (unchanged) **and** extracts intent-coder
   into `.popsicle/modules/intent-coder/` from embedded bundle when workspace root
   has no `intent-coder/`.
3. **Dogfood override**: if `intent-coder/module.yaml` exists at workspace root,
   `init` / `admin sync-intent-coder` copy from there instead (live module dev).
4. **`popsicle module add`**: remains **deferred**; replacement is
   `admin sync-intent-coder` + embedded fallback. Document in README/AGENTS.md.
5. **Doctor** reports `intent_coder_module` (version) and `intent_coder_bundle`
   (`embedded` | `workspace_root_override`).

## Consequences

| Scenario | Behavior |
|---|---|
| popsicle repo dogfood | Sync from live `intent-coder/` at root |
| DMG CLI → `mkdir p && cd p && popsicle init` | Embedded module extracted; pipelines OK; intent-validate OK |
| DMG contents | Still no separate intent-coder folder; module lives **inside binary** |
| Legacy `module add` | Deferred error with next-step (unchanged ADR-011) |

## Divergences

- **D-701**: Module install is no longer path-based for MVP; embedded snapshot
  may lag repo `intent-coder/` until rebuild. Dogfood uses workspace override.
- **D-702**: DMG does not ship intent-coder as files on disk (by design — smaller
  DMG, single CLI artifact).

## Compliance

- `make check` green
- `intent_coder_install` test: repo root + isolated temp dir (embedded path)

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-34 / user review
- **Approval date**: 2026-06-11
