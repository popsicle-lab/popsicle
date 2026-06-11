# cli-ux UI golden baseline (PROJ-27 / ADR-015)

Validates the optional Tauri UI slice without pulling it into the default
`make check` path.

| Script | Checks |
|---|---|
| golden-001 | `popsicle help` advertises `ui` |
| golden-002 | `npm run build` + `cargo build --features ui -p cli-ux` |
| golden-003 | `workspace_readers` unit tests |
| golden-004 | Task scan finds `T-CU-*` nodes |

Run: `bash docs/baseline/2026-06-11/cli-ux-ui/run-all.sh`
