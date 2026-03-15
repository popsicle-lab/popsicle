# Popsicle

Popsicle is a spec-driven development orchestration engine — a border collie that oversees AI coding.

It organizes the full software development lifecycle through composable **Skills** and **Pipelines**, provides a CLI for AI agents to call, tracks Git commits with document associations, and offers a desktop UI for read-only visualization. It also auto-scans project context, maintains cross-session memory, tracks work items (bugs, stories, test cases), and recommends the right pipeline for every task.

## Core Concepts

- **Module** — A self-contained distribution unit packaging a set of Skills and Pipelines together; one active module per project, upgradeable via CLI or binary update
- **Skill** — A reusable development capability unit with its own sub-workflow, document templates, AI prompts, and lifecycle hooks (e.g., `domain-analysis`, `arch-debate`, `rfc-writer`)
- **Pipeline** — Orchestrates Skills into a full development lifecycle as a DAG with dependency management between stages
- **Document** — Artifacts produced by Skills, stored as YAML frontmatter + Markdown files for Git-friendliness
- **Discussion** — Persistent multi-role review conversations captured during debate skills (e.g., `arch-debate`, `product-debate`), stored in SQLite with conversational UI rendering
- **Git Tracking** — Links Git commits to pipeline stages, skills, and documents; tracks review status per commit
- **Guard** — Conditions on workflow transitions that enforce upstream approval and document completeness
- **Advisor** — Recommends the next step (CLI command + AI prompt) based on current pipeline and document state
- **Project Context** — Auto-scanned technical profile (tech stack, structure, dev practices, dependencies) injected into all AI prompts as background context
- **Memory** — Two-layer (short-term / long-term) cross-session memory for bugs, decisions, patterns, and gotchas, stored in a single Markdown file with event-driven staleness
- **Work Items** — First-class Bug, UserStory, and TestCase entities extracted from documents or created manually, linked to pipeline runs, issues, and commits
- **Pipeline Recommender** — Suggests the best pipeline based on task description keywords and project scale
- **Extractor** — Parses structured entities (stories, test cases, bugs) from Markdown documents
- **Desktop UI** — Read-only Tauri app that visualizes pipelines, documents, discussions, git status, work items, memories, and commit-document associations

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

# Install the spec-development module (skills + pipelines)
popsicle module install github:curtiseng/popsclice-spec-development

# Initialize for multiple agents
popsicle init --agent claude,cursor

# Install git post-commit hook for automatic tracking
popsicle git init

# Scan project to generate technical profile
popsicle context scan

# Get pipeline recommendation for your task
popsicle pipeline recommend --task "Add user authentication"

# Start a development pipeline
popsicle pipeline run full-sdlc --title "My Feature"

# See what to do next
popsicle pipeline next

# Quick change (skip full pipeline ceremony)
popsicle pipeline quick --title "Fix login button"

# Extract structured entities from documents
popsicle extract user-stories --from-doc <doc-id>
popsicle extract test-cases --from-doc <doc-id> --type unit

# Save a memory for future sessions
popsicle memory save --type bug --summary "SQLite WAL mode required for concurrent access"
```

## Agent Support

`popsicle init` auto-generates agent instruction files with the full skill catalog.
Use `--agent` to select targets (default: `claude`).

| Agent | Flag | Generated Files |
|-------|------|-----------------|
| Claude Code | `--agent claude` | `.claude/CLAUDE.md`, `.claude/skills/popsicle-*/SKILL.md` |
| Cursor | `--agent cursor` | `.cursor/rules/popsicle.mdc`, `.cursor/skills/popsicle-*/SKILL.md` |

Examples:

```bash
# Claude Code only (default)
popsicle init

# Cursor only
popsicle init --agent cursor

# Both agents
popsicle init --agent claude,cursor

# Skip agent files entirely
popsicle init --no-agent-files
```

Generated files include the complete skill registry — agent names, artifact types, input dependencies, workflow states, transitions, and guard conditions — so each agent knows the full development workflow without calling the CLI first.

## CLI Commands

### Project & Pipeline

| Command | Description |
|---------|-------------|
| `popsicle init [--agent <targets>]` | Initialize project (idempotent); install a module separately |
| `popsicle pipeline list` | List available pipeline templates |
| `popsicle pipeline create <name>` | Create a custom pipeline template |
| `popsicle pipeline run <pipeline> --title <t>` | Start a pipeline run |
| `popsicle pipeline quick --title <t> [--skill <s>]` | Quick single-stage run (skip full pipeline) |
| `popsicle pipeline status [--run <id>]` | Show pipeline run status with documents |
| `popsicle pipeline next [--run <id>]` | Advisor: recommended next steps |
| `popsicle pipeline verify [--run <id>]` | Verify all stages complete and documents approved |
| `popsicle pipeline archive [--run <id>]` | Archive a completed pipeline run |
| `popsicle pipeline recommend --task <desc>` | Recommend pipeline based on task description |

### Module Management

| Command | Description |
|---------|-------------|
| `popsicle module list` | List installed modules (marks the active one) |
| `popsicle module show [<name>]` | Show module details: skills, pipelines, version, source |
| `popsicle module install <source>` | Install a module from local path or `github:user/repo[#ref][//subdir]` |
| `popsicle module upgrade [--force]` | Upgrade active module (builtin: from binary; remote: re-fetch from source) |

Examples:

```bash
# Show the current module
popsicle module show

# Install from a local directory
popsicle module install /path/to/my-module

# Install from GitHub
popsicle module install github:myorg/custom-skills

# Install a specific tag from a repo subdirectory
popsicle module install github:myorg/mono-repo#v2.0//modules/security

# Upgrade after updating the popsicle binary
popsicle module upgrade

# Force reinstall even if version matches
popsicle module upgrade --force
```

Skill loading priority (later overrides earlier):

1. `.popsicle/modules/<active>/skills/` — from the active module (lowest)
2. `.popsicle/skills/` — project-local overrides
3. `skills/` — workspace-level (highest, for development)

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

### Bug Tracking

| Command | Description |
|---------|-------------|
| `popsicle bug create` | Create a bug report |
| `popsicle bug list [--severity/--status/--issue/--run]` | List bugs with filters |
| `popsicle bug show <key>` | Show bug details |
| `popsicle bug update <key>` | Update bug fields |
| `popsicle bug link <key> --commit <sha>` | Link bug to a fix commit |
| `popsicle bug record --from-test --error <msg>` | Create bug from test failure (auto-dedup) |

### User Stories

| Command | Description |
|---------|-------------|
| `popsicle story create` | Create a user story |
| `popsicle story list [--status/--issue/--run]` | List user stories |
| `popsicle story show <key>` | Show story details with acceptance criteria |
| `popsicle story update <key>` | Update story fields |
| `popsicle story extract --from-doc <doc-id>` | Extract stories from a PRD document |
| `popsicle story link --ac <ac-id> --test-case <tc-key>` | Link acceptance criterion to test case |

### Test Cases

| Command | Description |
|---------|-------------|
| `popsicle test list [--type/--priority/--status]` | List test cases |
| `popsicle test show <key>` | Show test case details |
| `popsicle test extract --from-doc <doc-id> --type <t>` | Extract test cases from test spec document |
| `popsicle test run-result` | Record a test execution result |
| `popsicle test coverage` | Show test coverage summary |

### Entity Extraction

| Command | Description |
|---------|-------------|
| `popsicle extract user-stories --from-doc <doc-id>` | Parse user stories from document |
| `popsicle extract test-cases --from-doc <doc-id> --type <t>` | Parse test cases from document |
| `popsicle extract bugs --from-doc <doc-id>` | Parse bugs from document |

### Git Tracking

| Command | Description |
|---------|-------------|
| `popsicle git init` | Install post-commit hook for automatic tracking |
| `popsicle git link [--sha] [--doc] [--stage] [--skill]` | Link commit to document/stage/skill |
| `popsicle git status` | Git status + review statistics |
| `popsicle git log [-n]` | Commit history with review status and associations |
| `popsicle git review <sha> <passed/failed/skipped>` | Update commit review status |

### Discussion Persistence

| Command | Description |
|---------|-------------|
| `popsicle discussion create --skill <s> --topic <t> --run <id>` | Create a new discussion session |
| `popsicle discussion message <id> --role <r> --phase <p> --content <c>` | Add a message to a discussion |
| `popsicle discussion role <id> --role-id <r> --name <n>` | Register a participant role |
| `popsicle discussion list [--run/--skill/--status]` | Query discussions |
| `popsicle discussion show <id>` | Show discussion with full conversation |
| `popsicle discussion conclude <id> [--confidence <1-5>]` | Conclude a discussion |
| `popsicle discussion export <id> [--output <path>]` | Export discussion as Markdown |

### Memory

| Command | Description |
|---------|-------------|
| `popsicle memory save --type <t> --summary <s>` | Save a memory (bug/decision/pattern/gotcha) |
| `popsicle memory list [--layer/--type]` | List memories with filters |
| `popsicle memory show <id>` | Show memory details |
| `popsicle memory delete <id>` | Delete a memory |
| `popsicle memory promote <id>` | Promote short-term → long-term |
| `popsicle memory stale <id>` | Mark a memory as stale |
| `popsicle memory gc` | Garbage collect all stale memories |
| `popsicle memory check-stale` | Detect stale memories via git diff |
| `popsicle memory stats` | Show memory statistics |

### Project Context & AI Integration

| Command | Description |
|---------|-------------|
| `popsicle context scan` | Auto-scan project and generate technical profile |
| `popsicle context show [--run <id>] [--stage <s>]` | Full pipeline context with document bodies (JSON) |
| `popsicle context update --section <name>` | Update a section in project-context.md |
| `popsicle prompt <skill> [--state <s>] [--run <id>]` | AI prompt with upstream context + memory injected |
| `popsicle migrate --skill <s> <paths...>` | Import existing Markdown docs into a pipeline run |
| `popsicle completions <zsh/bash/fish>` | Generate shell completions |

All commands support `--format json` for machine consumption.

## Official Module: `spec-development`

Popsicle ships as a bare orchestration engine. Install the official module to get skills and pipelines:

```bash
popsicle module install github:curtiseng/popsclice-spec-development
```

### Skills (17)

| Skill | Artifact Type | Description |
|-------|---------------|-------------|
| `domain-analysis` | domain-model | Domain boundary analysis and model definition |
| `product-debate` | product-debate-record | Multi-persona product debate to explore options |
| `prd-writer` | prd | Product requirements document with quality scoring |
| `arch-debate` | arch-debate-record | Multi-persona architecture debate for technical decisions |
| `rfc-writer` | rfc | Technical RFC for design decisions and consensus building |
| `adr-writer` | adr | Architecture Decision Record |
| `priority-test-spec` | test-gate-report | Test priority classification (P0/P1/P2) |
| `api-test-spec` | api-test-spec | API integration test specification (gRPC/HTTP) |
| `e2e-test-spec` | e2e-test-spec | Functional end-to-end test specification |
| `ui-test` | ui-test-spec | UI test specification (Playwright) |
| `implementation` | impl-record | Code implementation guided by design and test specs |
| `unit-test-codegen` | unit-test-report | Unit test code generation from priority specs |
| `api-test-codegen` | api-test-report | API test code generation from test specs |
| `e2e-test-codegen` | e2e-test-report | E2E test code generation from test specs |
| `ui-test-codegen` | ui-test-report | UI test code generation from test specs |
| `bug-tracker` | bug-report | Bug tracking and issue management |
| `test-report` | test-summary | Test report analysis and aggregation |

### Pipelines (5)

#### `full-sdlc` — Full software development lifecycle (scale: full)

```
product-debate → prd-writer → arch-debate → rfc-writer + adr-writer
                     ↓                            ↓
                ui-test-planning      test-planning (priority-test-spec + api-test-spec + e2e-test-spec)
                     ↓                            ↓
                     │                   implementation
                     ↓                         ↓
                     └──→ test-codegen ←───────┘
                     (unit-test-codegen + api-test-codegen + e2e-test-codegen + ui-test-codegen)
                               ↓
                      quality (bug-tracker + test-report)
```

#### `tech-sdlc` — Technical refactoring & migration (scale: standard)

```
arch-debate → rfc-writer + adr-writer → test-planning → implementation → test-codegen → quality
```

#### `design-only` — Design & planning only (scale: planning)

```
product-debate → prd-writer → arch-debate → rfc-writer + adr-writer
```

#### `impl-test` — Implementation & testing (scale: light)

```
implementation → test-codegen → quality (bug-tracker + test-report)
```

#### `test-only` — Testing only (scale: minimal)

```
test-planning → test-codegen → quality
```

Use `popsicle pipeline recommend --task "<description>"` to get a recommendation based on your task.

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
                                    ├── Discussion viewer (conversational UI)
                                    ├── Bug / Story / TestCase tracking
                                    ├── Memory browser
                                    ├── Git tracking + commit-document links
                                    └── Next Step Advisor
```

### Design Principles

- **CLI executes, UI observes** — AI agents and developers operate through CLI; UI only visualizes and suggests
- **Skills are first-class** — Each skill carries its own sub-workflow, templates, prompts, and hooks
- **Pipeline orchestrates** — DAG-based stage dependencies with automatic state propagation
- **Guards enforce discipline** — Upstream approval and content completeness checked before transitions
- **Git-aware** — Post-commit hooks auto-track commits; link commits to documents, stages, and skills
- **Multi-agent** — Native support for Claude Code and Cursor with auto-generated skills following the Agent Skills open standard
- **Hybrid storage** — Documents as Markdown files (Git-friendly), metadata and state indexed in SQLite
- **Context-aware** — Project tech profile auto-scanned and injected with attention-optimized ordering (low → medium → high relevance)
- **Memory-driven** — Cross-session memory with event-driven staleness, two-layer promotion, and 200-line budget
- **Work item traceability** — Bugs, stories, and test cases linked across documents, pipeline runs, issues, and commits
- **Scale-adaptive** — Pipeline recommender matches task complexity to the right workflow depth
- **Modular distribution** — One active module per project; install from Git; `popsicle init` is idempotent
- **Extensible** — Custom skills (`skill create`), pipelines (`pipeline create`), hooks for lifecycle events

### Project Layout (after `popsicle init`)

```
your-project/
├── .popsicle/                    # Popsicle data (CLI reads from here)
│   ├── modules/                  # Installed modules (via `popsicle module install`)
│   │   └── <module-name>/       # e.g. spec-development
│   │       ├── module.yaml       # Module metadata (name, version)
│   │       ├── skills/           # Module-provided skills
│   │       └── pipelines/        # Module-provided pipelines
│   ├── skills/                   # Project-local skill overrides
│   ├── pipelines/                # Pipeline templates
│   ├── artifacts/                # Documents organized by pipeline run
│   ├── project-context.md        # Auto-scanned technical profile
│   ├── memories.md               # Cross-session memory store (≤200 lines)
│   ├── popsicle.db               # SQLite index (docs, bugs, stories, test cases, memories)
│   └── config.toml               # Project config (includes [module] section)
├── .claude/                      # Claude Code (--agent claude)
│   ├── CLAUDE.md                 # Instructions + skill catalog
│   └── skills/popsicle-*/        # Per-skill SKILL.md files
└── .cursor/                      # Cursor (--agent cursor)
    ├── rules/popsicle.mdc        # Always-apply rules + skill catalog
    └── skills/popsicle-*/        # Per-skill SKILL.md files
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
- **Discussions** — Conversational UI for multi-role review sessions with phase grouping, role color coding, participant sidebar, and message type differentiation (role statements, user input, pause points, phase summaries, decisions)
- **User Stories** — Story list with acceptance criteria verification progress, detail view with linked test cases
- **Test Cases** — Test case list with type/priority/status filters, coverage summary, execution results
- **Bugs** — Bug list with severity/status filters, statistics cards, detail view with reproduction steps and linked commits
- **Memories** — Memory browser with layer/type filters, capacity gauge, staleness indicators
- **Git Tracking** — Branch/HEAD status, tracked commits with review status, commit-document-stage associations
- **Skills Registry** — Browse all skills with workflow diagrams and input dependencies

For frontend hot-reload development, use the standalone Tauri app:

```bash
cd ui && npm install && npm run tauri dev
```

## License

Apache-2.0
