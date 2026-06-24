---
doc_type: fact-extractor
id: doc-70
pipeline_run_id: 00000023-0000-4023-8002-23000000000023
status: active
title: cli-ux retro spec fact baseline (PROJ-35)
version: 1
---

# Fact baseline — PROJ-35 retro spec

Shipped without full spec chain:

- PROJ-29: global `popsicle project *`, `--project`, DMG packaging
- PROJ-30: Tauri project switcher + recents (ADR-016)
- PROJ-34: embedded intent-coder on init (ADR-017)

## Facts

- [x] `products/cli-ux/intents/acceptance.intent` has T-CU-0009..0012 blocks
- [x] `PDR-003` Accepted; tasks T-CU-0009..0012 exist
- [x] `make check` and UI build pass on main
