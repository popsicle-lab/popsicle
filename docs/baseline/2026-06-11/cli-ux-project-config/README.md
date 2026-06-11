# cli-ux project config golden baseline (PROJ-37)

Per-workspace `.popsicle/project.yaml`, `AGENTS.md` sync, doctor fields,
`issue start` agent context injection, and Tauri Settings page.

```bash
./run-all.sh
```

| Script | Checks |
|---|---|
| golden-001 | `init` → `project.yaml` + AGENTS.md marker |
| golden-002 | `admin sync-project-config` |
| golden-003 | `doctor` project config fields |
| golden-004 | `issue start` → `agent_context` |
| golden-005 | Settings UI + `npm run build` |
