# macOS DMG packaging (unsigned MVP)

## Contents

| Item | Purpose |
|---|---|
| `Popsicle.app` | Tauri desktop UI |
| `popsicle` | CLI binary (same build, `--features ui`); **embeds intent-coder** at compile time (ADR-017) |
| `intent` | [intent-lang](https://github.com/popsicle-lab/intent-lang) CLI from `packaging/intent-lang-pin.toml` (v0.1.1+ bundles Z3) |
| `Install CLI.command` | Optional fallback: copies `popsicle` + `intent` to `~/.local/bin` |
| `Applications` | Drag target symlink |

`intent` is also copied into `Popsicle.app/Contents/Resources/intent` for silent install on first app launch.

There is **no** separate `intent-coder/` folder on the DMG volume. On `popsicle init`
in a new project, the CLI extracts the bundled module into
`.popsicle/modules/intent-coder/` automatically.

## Pin file

Release assets are pinned in [`packaging/intent-lang-pin.toml`](../intent-lang-pin.toml).
DMG build downloads the matching macOS asset:

| `POPSICLE_DMG_ARCH` | GitHub asset |
|---|---|
| `aarch64` | `intent-macos-aarch64` |
| `x86_64` | `intent-macos-x86_64` |

`vender/intent-lang/` in the popsicle repo (if present) is for AI/docs only — runtime uses the installed `intent` binary.

## Build (local)

```bash
chmod +x packaging/macos/fetch-intent.sh
make build-dmg
```

Requires: macOS, Xcode CLI tools (`sips`, `iconutil`, `hdiutil`), Node 18+, Rust, `cargo-tauri`, network access to fetch intent-lang release.

Fetch only (smoke test):

```bash
POPSICLE_DMG_ARCH=aarch64 bash packaging/macos/fetch-intent.sh
```

## First open (unsigned)

The DMG is a **folder image**, not a wizard installer — mount it, then:

1. Drag **Popsicle.app** into **Applications** (do not run the app from inside the DMG).
2. Open **Popsicle** from Applications. If macOS blocks the unsigned app: **Right-click → Open** once, or System Settings → Privacy & Security → **Open Anyway**. On first launch the app **silently** copies `popsicle` and `intent` to `~/.local/bin` and ensures `~/.popsicle/` exists (PATH appended to `~/.zshrc` when needed).
3. *(Optional)* Double-click **Install CLI.command** on the mounted volume if you need the toolchain without opening the app first.

Terminal CLI remains `popsicle ui` (or any other subcommand); double-clicking the `.app` opens the desktop UI directly.

Verify intent after install:

```bash
intent --version
popsicle tool run intent-validate path=products format=text
```

## Multi-project

After CLI install:

```bash
cd ~/my-project && popsicle init
popsicle project add ~/my-project
popsicle project use my-project
popsicle issue list
```

Use `--project <path>` or `POPSICLE_PROJECT` to override the default workspace.
