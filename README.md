# Popsicle

Popsicle is a spec-driven development orchestration engine — a border collie that oversees AI coding.

It organizes the full software development lifecycle through composable **Skills** and **Pipelines**, provides a CLI for AI agents to call, tracks Git commits with document associations, and offers a desktop UI for read-only visualization.

## Core Concepts

- **Skill** — A reusable development capability unit with its own sub-workflow, document templates, AI prompts, and lifecycle hooks (e.g., `domain-analysis`, `product-prd`, `tech-rfc`)
- **Pipeline** — Orchestrates Skills into a full development lifecycle as a DAG with dependency management between stages
- **Document** — Artifacts produced by Skills, stored as YAML frontmatter + Markdown files for Git-friendliness
- **Git Tracking** — Links Git commits to pipeline stages, skills, and documents; tracks review status per commit
- **Advisor** — Recommends the next step (CLI command + AI prompt) based on current pipeline and document state
- **Desktop UI** — Read-only Tauri app that visualizes pipelines, documents, git status, and commit-document associations

## Quick Start

```bash
# Build
cargo build

# Initialize a project
popsicle init

# Install git post-commit hook for automatic tracking
popsicle git init

# Start a development pipeline
popsicle pipeline run full-sdlc --title "My Feature"

# See what to do next
popsicle pipeline next

# Create a document using a skill's template
popsicle doc create domain-analysis --title "Feature Domain Model" --run <run-id>

# Advance document through its workflow
popsicle doc transition <doc-id> submit
popsicle doc transition <doc-id> approve

# Link a commit to a document and stage
popsicle git link --doc <doc-id> --stage domain --skill domain-analysis

# Review a commit
popsicle git review <sha> passed --summary "LGTM"

# Check pipeline status
popsicle pipeline status

# Get full context for AI agents (JSON)
popsicle context --format json

# Get AI prompt with upstream document context injected
popsicle prompt product-prd --run <run-id>
```

## CLI Commands

### Project & Pipeline

| Command | Description |
|---------|-------------|
| `popsicle init` | Initialize `.popsicle/` project directory |
| `popsicle pipeline list` | List available pipeline templates |
| `popsicle pipeline create <name>` | Create a custom pipeline template |
| `popsicle pipeline run <pipeline> --title <t>` | Start a pipeline run |
| `popsicle pipeline status [--run <id>]` | Show pipeline run status with documents |
| `popsicle pipeline next [--run <id>]` | Advisor: recommended next steps |

### Skills & Documents

| Command | Description |
|---------|-------------|
| `popsicle skill list` | List all registered skills |
| `popsicle skill show <name>` | Show skill details (workflow, inputs, prompts) |
| `popsicle skill create <name>` | Scaffold a new custom skill |
| `popsicle doc create <skill> --title <t> --run <id>` | Create a document from skill template |
| `popsicle doc list [--skill/--status/--run]` | Query documents |
| `popsicle doc show <id>` | View document content and metadata |
| `popsicle doc transition <id> <action>` | Advance document through workflow |

### Git Tracking

| Command | Description |
|---------|-------------|
| `popsicle git init` | Install post-commit hook for automatic tracking |
| `popsicle git link [--sha] [--doc] [--stage] [--skill]` | Link commit to document/stage/skill |
| `popsicle git status` | Git status + review statistics |
| `popsicle git log [-n]` | Commit history with review status and associations |
| `popsicle git review <sha> <passed/failed/skipped>` | Update commit review status |

### AI Agent Integration

| Command | Description |
|---------|-------------|
| `popsicle context [--run <id>] [--stage <s>]` | Full pipeline context with document bodies (JSON) |
| `popsicle prompt <skill> [--state <s>] [--run <id>]` | AI prompt with upstream document context injected |

All commands support `--format json` for machine consumption.

## Built-in Skills

| Skill | Artifact Type | Description |
|-------|---------------|-------------|
| `domain-analysis` | domain-model | Domain boundary analysis and model definition |
| `product-prd` | prd | Product requirements document |
| `tech-rfc` | rfc | Technical RFC for design decisions |
| `tech-adr` | adr | Architecture Decision Record |
| `test-spec` | test-spec | Test specification and planning |
| `implementation` | impl-plan | Code implementation tracking |

## Built-in Pipeline

**`full-sdlc`** — Full software development lifecycle:

```
domain-analysis → product-prd → tech-rfc + tech-adr → test-spec → implementation
```

## Architecture

```
AI Agents ──→ CLI (only write path) ──→ Core Engine ──→ Files + SQLite
Developer ──→                               ↑
                                  Desktop UI (read-only)
                                    ├── Pipeline DAG visualization
                                    ├── Document viewer + metadata panel
                                    ├── Git tracking + commit-document links
                                    └── Next Step Advisor
```

### Design Principles

- **CLI executes, UI observes** — AI agents and developers operate through CLI; UI only visualizes and suggests
- **Skills are first-class** — Each skill carries its own sub-workflow, templates, prompts, and hooks
- **Pipeline orchestrates** — DAG-based stage dependencies with automatic state propagation
- **Git-aware** — Post-commit hooks auto-track commits; link commits to documents, stages, and skills
- **Hybrid storage** — Documents as Markdown files (Git-friendly), metadata and state indexed in SQLite
- **Extensible** — Create custom skills (`skill create`) and pipelines (`pipeline create`); hooks for lifecycle events

### Project Structure

```
popsicle/
├── crates/
│   ├── popsicle-core/        # Core library: models, engine, storage, git
│   └── popsicle-cli/         # CLI binary
├── ui/
│   ├── src/                  # React + TypeScript frontend
│   └── src-tauri/            # Tauri backend (read-only commands)
├── skills/                   # Built-in skill definitions
├── pipelines/                # Built-in pipeline templates
└── Cargo.toml                # Workspace root
```

## Desktop UI

The Tauri desktop app provides read-only visualization:

- **Dashboard** — Pipeline run overview, document statistics, activity
- **Pipeline View** — Stage DAG with status highlighting, documents per stage, linked commits, Next Step Advisor with copyable CLI commands
- **Document Viewer** — Markdown rendering + metadata panel (type, status, skill, tags, timeline, linked commits)
- **Git Tracking** — Branch/HEAD status, tracked commits with review status, commit-document-stage associations
- **Skills Registry** — Browse all skills with workflow diagrams and input dependencies

```bash
# Development
cd ui && npm install && npm run tauri dev

# Build
cd ui && npm run tauri build
```

## License

Apache-2.0
