# Popsicle Desktop UI

Read-only Tauri desktop application for visualizing Popsicle pipeline state, documents, and git status.

## Prerequisites

- Node.js 22+
- Rust stable toolchain
- Platform-specific dependencies (see below)

### macOS

No additional dependencies required.

### Linux

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

## Development

```bash
# Install frontend dependencies
npm ci

# Start dev server (hot reload)
npm run dev
```

The dev server runs the standalone Tauri application at `ui/src-tauri/`.

## Build

```bash
npm run build
```

## Architecture

- **Frontend**: React + TypeScript + Vite
- **Backend**: Tauri 2 (Rust) using `popsicle-core` for all business logic
- **Communication**: Tauri IPC commands defined in `src-tauri/src/commands.rs`

The UI is read-only — all mutations go through the `popsicle` CLI.
