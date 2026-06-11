# macOS DMG packaging (unsigned MVP)

## Contents

| Item | Purpose |
|---|---|
| `Popsicle.app` | Tauri desktop UI |
| `popsicle` | CLI binary (same build, `--features ui`) |
| `Install CLI.command` | Copies CLI to `~/.local/bin`, creates `~/.popsicle/` |
| `Applications` | Drag target symlink |

## Build (local)

```bash
make build-dmg
```

Requires: macOS, Xcode CLI tools (`sips`, `iconutil`, `hdiutil`), Node 18+, Rust, `cargo-tauri`.

## First open (unsigned)

1. Mount the DMG.
2. Drag **Popsicle.app** to **Applications**.
3. Double-click **Install CLI.command** (may need Right-click → Open once).
4. For the app: System Settings → Privacy & Security → Open Anyway, if Gatekeeper blocks.

## Multi-project

After CLI install:

```bash
cd ~/my-project && popsicle init
popsicle project add ~/my-project
popsicle project use my-project
popsicle issue list
```

Use `--project <path>` or `POPSICLE_PROJECT` to override the default workspace.
