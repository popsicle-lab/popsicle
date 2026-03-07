# Popsicle

Popsicle is a spec-driven development orchestration engine — a border collie that oversees AI coding.

It organizes the full software development lifecycle through composable **Skills** and **Pipelines**, provides a CLI for AI agents to call, tracks Git commits with document associations, and offers a desktop UI for read-only visualization.

## Core Concepts

- **Skill** — A reusable development capability unit with its own sub-workflow, document templates, AI prompts, and lifecycle hooks (e.g., `domain-analysis`, `product-prd`, `tech-rfc`)
- **Pipeline** — Orchestrates Skills into a full development lifecycle as a DAG with dependency management between stages
- **Document** — Artifacts produced by Skills, stored as YAML frontmatter + Markdown files for Git-friendliness
- **Git Tracking** — Links Git commits to pipeline stages, skills, and documents; tracks review status per commit
- **Guard** — Conditions on workflow transitions that enforce upstream approval and document completeness
- **Advisor** — Recommends the next step (CLI command + AI prompt) based on current pipeline and document state
- **Desktop UI** — Read-only Tauri app that visualizes pipelines, documents, git status, and commit-document associations

## Installation

### Download from GitHub Releases

Pre-built binaries (CLI + Desktop UI) are available for macOS, Linux, and Windows:

> **[Latest Release](https://github.com/curtiseng/popsicle/releases/latest)**

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `popsicle-v*-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `popsicle-v*-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `popsicle-v*-x86_64-unknown-linux-gnu.tar.gz` |
| Windows (x86_64) | `popsicle-v*-x86_64-pc-windows-msvc.zip` |

### Add to PATH

**macOS / Linux:**

```bash
# Extract and install to /usr/local/bin (requires sudo)
tar xzf popsicle-v*-*.tar.gz
sudo mv popsicle /usr/local/bin/

# Or install to user-local directory (no sudo)
mkdir -p ~/.local/bin
mv popsicle ~/.local/bin/
# Add to your shell profile if not already in PATH:
# For zsh (~/.zshrc):
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
# For bash (~/.bashrc):
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
```

**Windows (PowerShell):**

```powershell
# Extract the zip, then add the directory to PATH
Expand-Archive popsicle-v*-x86_64-pc-windows-msvc.zip -DestinationPath C:\popsicle
# Add to user PATH permanently
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\popsicle", "User")
# Reload in current session
$env:Path += ";C:\popsicle"
```

### Shell Completions

```bash
# Zsh
mkdir -p ~/.zfunc
popsicle completions zsh > ~/.zfunc/_popsicle
# Add to ~/.zshrc: fpath=(~/.zfunc $fpath) && autoload -Uz compinit && compinit

# Bash
popsicle completions bash | sudo tee /etc/bash_completion.d/popsicle > /dev/null

# Fish
popsicle completions fish > ~/.config/fish/completions/popsicle.fish
```

### Build from Source

```bash
# CLI only
cargo build -p popsicle-cli --release

# CLI + Desktop UI (requires Node.js)
cd ui && npm install && npm run build && cd ..
cargo build -p popsicle-cli --features ui --release
```

## Quick Start

```bash
# Initialize a project (default: Claude Code agent)
popsicle init

# Initialize for multiple agents
popsicle init --agent claude,cursor,codex

# Install git post-commit hook for automatic tracking
popsicle git init

# Start a development pipeline
popsicle pipeline run full-sdlc --title "My Feature"

# See what to do next
popsicle pipeline next

# Quick change (skip full pipeline ceremony)
popsicle pipeline quick --title "Fix login button"
```

## Agent Support

`popsicle init` auto-generates agent instruction files with the full skill catalog.
Use `--agent` to select targets (default: `claude`).

| Agent | Flag | Generated Files |
|-------|------|-----------------|
| Claude Code | `--agent claude` | `.claude/CLAUDE.md`, `.claude/commands/popsicle-*.md` |
| Cursor | `--agent cursor` | `.cursor/rules/popsicle.mdc`, `.cursor/agents/popsicle.md` |
| Codex (OpenAI) | `--agent codex` | `AGENTS.md` |

Examples:

```bash
# Claude Code only (default)
popsicle init

# Cursor only
popsicle init --agent cursor

# All three agents
popsicle init --agent claude,cursor,codex

# Skip agent files entirely
popsicle init --no-agent-files
```

Generated files include the complete skill registry — agent names, artifact types, input dependencies, workflow states, transitions, and guard conditions — so each agent knows the full development workflow without calling the CLI first.

## CLI Commands

### Project & Pipeline

| Command | Description |
|---------|-------------|
| `popsicle init [--agent <targets>]` | Initialize project with built-in skills, pipelines, and agent instructions |
| `popsicle pipeline list` | List available pipeline templates |
| `popsicle pipeline create <name>` | Create a custom pipeline template |
| `popsicle pipeline run <pipeline> --title <t>` | Start a pipeline run |
| `popsicle pipeline quick --title <t> [--skill <s>]` | Quick single-stage run (skip full pipeline) |
| `popsicle pipeline status [--run <id>]` | Show pipeline run status with documents |
| `popsicle pipeline next [--run <id>]` | Advisor: recommended next steps |
| `popsicle pipeline verify [--run <id>]` | Verify all stages complete and documents approved |
| `popsicle pipeline archive [--run <id>]` | Archive a completed pipeline run |

### Skills & Documents

| Command | Description |
|---------|-------------|
| `popsicle skill list` | List all registered skills |
| `popsicle skill show <name>` | Show skill details (workflow, inputs, prompts) |
| `popsicle skill create <name>` | Scaffold a new custom skill |
| `popsicle doc create <skill> --title <t> --run <id>` | Create a document from skill template |
| `popsicle doc list [--skill/--status/--run]` | Query documents |
| `popsicle doc show <id>` | View document content and metadata |
| `popsicle doc transition <id> <action>` | Advance document through workflow (guards enforced) |

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
| `popsicle prompt <skill> [--state <s>] [--run <id>]` | AI prompt with upstream context injected |
| `popsicle migrate --skill <s> <paths...>` | Import existing Markdown docs into a pipeline run |
| `popsicle completions <zsh/bash/fish>` | Generate shell completions |

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

## Guard Conditions

Skills can define guards on workflow transitions to enforce discipline:

```yaml
# In skill.yaml
workflow:
  states:
    draft:
      transitions:
        - to: review
          action: submit
          guard: "upstream_approved"        # upstream docs must be in final state
    review:
      transitions:
        - to: approved
          action: approve
          guard: "has_sections:Background,Goals"  # document must have real content
```

| Guard | Description |
|-------|-------------|
| `upstream_approved` | All required upstream skill documents must exist and be in a final state |
| `has_sections:A,B,C` | Document must contain the specified H2 sections with non-template content |

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
- **Guards enforce discipline** — Upstream approval and content completeness checked before transitions
- **Git-aware** — Post-commit hooks auto-track commits; link commits to documents, stages, and skills
- **Multi-agent** — Native support for Claude Code, Cursor, and Codex with auto-generated instruction files
- **Hybrid storage** — Documents as Markdown files (Git-friendly), metadata and state indexed in SQLite
- **Extensible** — Custom skills (`skill create`), pipelines (`pipeline create`), hooks for lifecycle events

### Project Layout (after `popsicle init`)

```
your-project/
├── .popsicle/                    # Popsicle data (CLI reads from here)
│   ├── skills/                   # Built-in + custom skill definitions
│   ├── pipelines/                # Pipeline templates
│   ├── artifacts/                # Documents organized by pipeline run
│   ├── popsicle.db               # SQLite index
│   └── config.toml               # Project configuration
├── .claude/                      # Claude Code (--agent claude)
│   ├── CLAUDE.md                 # Instructions + full skill catalog
│   └── commands/                 # Slash commands
├── .cursor/                      # Cursor (--agent cursor)
│   ├── rules/popsicle.mdc        # Always-apply rules + skill catalog
│   └── agents/popsicle.md        # Popsicle sub-agent
└── AGENTS.md                     # Codex (--agent codex)
```

## Desktop UI

The CLI and Desktop UI are bundled into a single `popsicle` binary (when built with `--features ui`). Launch the graphical interface with:

```bash
popsicle ui
```

The Tauri desktop app provides read-only visualization:

- **Dashboard** — Pipeline run overview, Git status bar, document statistics, quick actions with copyable commands
- **Pipeline View** — Stage DAG with status highlighting, documents and commits per stage, verification status, archive hint, Next Step Advisor
- **Document Viewer** — Markdown rendering + metadata panel (type, status, skill, tags, timeline, linked commits)
- **Git Tracking** — Branch/HEAD status, tracked commits with review status, commit-document-stage associations
- **Skills Registry** — Browse all skills with workflow diagrams and input dependencies

For frontend hot-reload development, use the standalone Tauri app:

```bash
cd ui && npm install && npm run tauri dev
```

## License

Apache-2.0
