# cli-ux project UI golden baseline (PROJ-30)

Validates global project registry integration in the Tauri UI: recents,
`open_project`, startup resolution, and UI build.

| Script | Checks |
|---|---|
| golden-001 | `open_project` records `last_opened_at` + default |
| golden-002 | `resolve_ui_startup_root` prefers default then MRU |
| golden-003 | UI `npm run build` + `cargo build --features ui -p cli-ux` |
| golden-004 | Tauri project commands compile (`ui` feature unit surface) |

Run: `bash docs/baseline/2026-06-11/cli-ux-project-ui/run-all.sh`
