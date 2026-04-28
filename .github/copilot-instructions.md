# Copilot Instructions for Popsicle

Popsicle is a spec-driven development orchestration engine. It provides a CLI (consumed by AI agents like Claude Code and Cursor) and an optional Tauri desktop UI (read-only). The core domain revolves around **Skills** (reusable workflow units with state machines), **Pipelines** (ordered sequences of Skills), and **Specs** (versioned feature containers).

## Build, Test, and Lint

```bash
# Full quality check (format + clippy + tests)
make check

# Individual checks
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" cargo clippy --all-targets --all-features
RUSTFLAGS="-Dwarnings" cargo test --all-targets --all-features

# Run a single test
cargo test <test_name> --all-features

# Build CLI only
cargo build --release

# Build with desktop UI (requires frontend build first)
cd ui && npm install && npm run build && cd ..
cargo build --release --features ui

# Frontend dev server (port 1420)
cd ui && npm run dev

# Frontend lint
cd ui && npx eslint .
```

CI enforces zero warnings via `RUSTFLAGS="-Dwarnings"`. The pre-commit hook (`make install-hooks`) runs the same checks locally.

## Architecture

### Workspace layout

- **`crates/popsicle-core`** — Domain models, storage, engine logic, registries. No CLI or UI concerns.
- **`crates/popsicle-cli`** — clap v4 CLI (`popsicle` binary) with ~17 top-level subcommands (low-frequency operations grouped under `popsicle admin`). Also contains the Tauri backend behind `--features ui`.
- **`ui/`** — React 19 + TypeScript + Tailwind CSS frontend, bundled by Vite into `ui/dist/` and embedded into the Tauri binary.

### Entity hierarchy

```
Namespace → Spec (locked_by_run_id) → Issue → PipelineRun → Document
                                                          └─ WorkItem (kind: bug | story | testcase)
```

- **Spec lock**: Only one PipelineRun can operate on a Spec at a time. `issue start` acquires the lock; it auto-releases when all stages complete.
- **Documents have no independent state machine** — they are "active" while a stage runs and "final" when the stage completes.
- **WorkItem unified**: Bugs / user stories / test cases share a single `work_items` table, distinguished by a `kind` discriminator and a JSON `fields` blob for kind-specific data (severity, acceptance criteria, steps). The legacy `Bug` / `UserStory` / `TestCase` model files no longer exist.
- **Discussions are Documents**: Multi-role debates produced by `arch-debate` / `product-debate` are stored as `Document` rows with `kind = "discussion"`. There is no separate Discussion entity, table, or CLI command.

### Skill anatomy (three files, three audiences)

- **`skill.yaml`** — Orchestration config for the engine: workflow state machine, inputs, artifacts, guards, hooks.
- **`guide.md`** — Writing guidance for the AI agent producing documents.
- **`templates/*.md`** — Document skeleton for the AI to fill in. H2 headings must match `has_sections` guard parameters.

### Three-layer loading priority

Skills, Pipelines, and Tools resolve in this order (later wins):

1. `.popsicle/modules/<active>/` — Module defaults (lowest)
2. `.popsicle/skills/` / `.popsicle/pipelines/` — Project-local overrides
3. `skills/` / `pipelines/` at workspace root — Active development (highest)

### Storage

- **Markdown + YAML frontmatter** for documents (Git-friendly)
- **TOML** for configuration (`.popsicle/config.toml`)
- **SQLite** for indexing (`.popsicle/popsicle.db`)
- **Flat files** for artifacts (`.popsicle/artifacts/<run-id>/`)

### Context assembly for LLM prompts

Upstream documents are assembled by `engine::context` and sorted by `Relevance` (Low → Medium → High). Low-relevance inputs inject summary-only; High injects full text. This optimizes for LLM attention distribution (most relevant content sits closest to the user prompt).

The higher-level `engine::context_layer::ContextLayer` trait pluggably composes prompt sections. Built-in layers: `ProjectContextLayer`, `MemoriesLayer`, `HistoricalRefsLayer`, `UpstreamDocsLayer`. The `commands::prompt` command builds a `Vec<Box<dyn ContextLayer>>` and calls `assemble_layers(layers, &base_prompt)` instead of hand-rolling section ordering.

### CLI ↔ UI boundary

The CLI is the **only write path**. The desktop UI is strictly read-only and communicates with the Rust backend via Tauri IPC (`invoke` commands defined in `crates/popsicle-cli/src/ui/commands.rs`, TypeScript bindings in `ui/src/hooks/useTauri.ts`).

## Conventions

### Rust

- **Edition 2024** with workspace-level dependency management.
- **Error handling**: `popsicle-core` uses `thiserror` with a `PopsicleError` enum and a `Result<T>` alias. CLI commands use `anyhow::Result<()>`.
- **CLI command pattern**: Each command module defines a clap `Subcommand` enum and an `execute(cmd, &OutputFormat)` function. All commands support `--format json` for structured output.
- **IDs**: UUIDs (`uuid::Uuid::new_v4`) for entities; human-readable keys (`BUG-PRJ-1`, `TC-PRJ-1`) for work items.
- **Serialization**: `serde` with `serde_yaml_ng` for skill/pipeline definitions, `serde_json` for API output, `toml` for config.

### UI (React/TypeScript)

- State-based routing via a `Page` union type in `App.tsx` — not URL-based React Router.
- All backend data types are mirrored as TypeScript interfaces in `ui/src/hooks/useTauri.ts`.
- Styling uses Tailwind CSS utility classes with CSS custom properties for theming (dark theme, defined in `index.css`).
- Icons from `lucide-react`.

### Skill design rules

- Guards only on forward transitions (`submit`, `approve`), never on backward ones (`revise`).
- `artifact_type` is a global contract — must be unique across all skills and match exactly between upstream `artifacts` and downstream `inputs`.
- Template H2 section names must match `has_sections` guard parameters.
- Hooks receive context via env vars: `$POPSICLE_DOC_ID`, `$POPSICLE_SKILL`, `$POPSICLE_RUN_ID`.

### Build-time embedding

`crates/popsicle-core/build.rs` collects builtin skill and pipeline YAML files from workspace `skills/` and `pipelines/` directories and embeds them as `BUILTIN_FILES` const data for zero-dependency distribution.
