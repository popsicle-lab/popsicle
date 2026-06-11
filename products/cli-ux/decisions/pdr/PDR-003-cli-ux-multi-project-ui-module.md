# PDR-003: cli-ux multi-project registry, UI shell, embedded intent-coder

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Source**: PROJ-35（retro `slice-spec` for PROJ-29 / PROJ-30 / PROJ-34 deliveries）

## Decision

Retroactively formalize user-visible behavior already shipped without a full
spec chain:

- **Global multi-project** (`~/.popsicle/global.json`, `popsicle project *`,
  `--project` override) — PROJ-29
- **Tauri UI project shell** (recents, switcher, `.app` auto-launch) — PROJ-30 /
  ADR-016
- **Embedded intent-coder** (`init` + `admin sync-intent-coder`, DMG-safe) —
  PROJ-34 / ADR-017
- **macOS DMG install path** (`Install CLI.command`, PATH to `~/.local/bin`) —
  PROJ-29 packaging

## Rationale

Features were delivered via `slice-delivery` (PROJ-29/30/34) without new
`acceptance.intent` blocks or task files. Agents and UI explorer need L2 task
anchors and Z3-checkable acceptance fragments for recall and regression.

## Intent Impact

| Intent | Task | File | Meaning |
|---|---|---|---|
| `ProjectRegistryOverridesWorkspace` | T-CU-0009 | `acceptance.intent` | register projects, set default, `--project` resolves |
| `UiProjectOpenPersistsRecents` | T-CU-0010 | `acceptance.intent` | UI open updates `global.json` MRU |
| `InitInstallsEmbeddedIntentCoder` | T-CU-0011 | `acceptance.intent` | fresh `init` extracts module without repo checkout |
| `MacosDmgInstallExposesCli` | T-CU-0012 | `acceptance.intent` | DMG install script places CLI on user PATH |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-35 retro spec
- **Approval date**: 2026-06-11
