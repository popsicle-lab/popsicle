# macOS DMG packaging (unsigned MVP)

## Contents

| Item | Purpose |
|---|---|
| `Popsicle.app` | Tauri desktop UI |
| `popsicle` | CLI binary (same build, `--features ui`); **embeds intent-coder** at compile time (ADR-017) |
| `Install CLI.command` | Optional fallback: copies CLI to `~/.local/bin` (same as first app launch) |
| `Applications` | Drag target symlink |

There is **no** separate `intent-coder/` folder on the DMG volume. On `popsicle init`
in a new project, the CLI extracts the bundled module into
`.popsicle/modules/intent-coder/` automatically.

## Build (local)

```bash
make build-dmg
```

Requires: macOS, Xcode CLI tools (`sips`, `iconutil`, `hdiutil`), Node 18+, Rust, `cargo-tauri`.

## First open (unsigned)

The DMG is a **folder image**, not a wizard installer — mount it, then:

1. Drag **Popsicle.app** into **Applications** (do not run the app from inside the DMG).
2. Open **Popsicle** from Applications. If macOS blocks the unsigned app: **Right-click → Open** once, or System Settings → Privacy & Security → **Open Anyway**. On first launch the app **silently** copies `popsicle` to `~/.local/bin` and ensures `~/.popsicle/` exists (PATH appended to `~/.zshrc` when needed).
3. *(Optional)* Double-click **Install CLI.command** on the mounted volume if you need the CLI without opening the app first.

Terminal CLI remains `popsicle ui` (or any other subcommand); double-clicking the `.app` opens the desktop UI directly.

## Multi-project

After CLI install:

```bash
cd ~/my-project && popsicle init
popsicle project add ~/my-project
popsicle project use my-project
popsicle issue list
```

Use `--project <path>` or `POPSICLE_PROJECT` to override the default workspace.
