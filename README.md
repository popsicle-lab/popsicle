# Popsicle

Popsicle is a spec-driven development orchestration engine — a border collie that oversees AI coding.

It organizes the full software development lifecycle through composable **Skills** and **Pipelines**, provides a CLI for AI agents to call, tracks Git commits with document associations, and offers a desktop UI for read-only visualization. It also auto-scans project context, maintains cross-session memory, tracks work items (bugs, stories, test cases), and recommends the right pipeline for every task.

## Core Concepts

- **Module** — A self-contained distribution unit packaging Skills, Pipelines, and a Bootstrap spec; one active module per project, distributed via a git-based [registry](https://popsicle-lab.com/registry)
- **Skill** — A reusable development capability unit with its own sub-workflow, document templates, AI prompts, and lifecycle hooks (e.g., `arch-debate`, `rfc-writer`, `implementation`). Skills declare `doc_lifecycle: singleton | cumulative` (see below).
- **Pipeline** — Orchestrates Skills into a full development lifecycle as a DAG with dependency management between stages; runs are scoped to a Topic for continuity and revision support. **The pipeline stage is the single source of truth for document state** — documents are created as "active" and become "final" when their stage is completed
- **Namespace** — Named scope for organizing Topics. A codebase can have multiple namespaces for different product domains (e.g., "backend-v2", "mobile-app"). Must be created before any Topics or Issues
- **Topic** — Document collection with tags, reusable across Issues; belongs to a Namespace (required). Tags enable automatic Issue→Topic matching. Groups pipeline runs and documents under a development theme (e.g., "jwt-migration"). Topics have an **exclusive lock** (`locked_by_run_id`) — only one pipeline run can operate on a Topic at a time
- **Issue** — Requirement entry point — must be created before any work. Auto-matched to a Topic by tags, or explicitly specified with `--topic`. `issue start` is the **only** way to create a PipelineRun and acquires the Topic lock — rejects if the Topic is already locked by another run
- **PipelineRun** — State machine on a Topic, triggered exclusively by `issue start`. Cannot be created directly — use `issue start` to begin a run
- **Document** — Belongs to a PipelineRun and Topic. Created as "active" and becomes "final" when the owning pipeline stage is completed. Documents no longer have their own state machine — the pipeline stage is the source of truth. Singleton skills reuse/update the same document across runs; cumulative skills create a new document each time. Stored as YAML frontmatter + Markdown files for Git-friendliness
- **Bootstrap** — LLM-driven project initialization that reads a module's `bootstrap.md` (natural language spec), scans the project, and generates a structured plan covering architecture, bounded contexts, conventions, and team decisions — all before the first pipeline run
- **Discussion** — Persistent multi-role review conversations captured during debate skills (e.g., `arch-debate`, `product-debate`), stored in SQLite with conversational UI rendering
- **Git Tracking** — Links Git commits to pipeline stages, skills, and documents; tracks review status per commit
- **Guard** — Conditions on stage transitions that enforce upstream stage completion and document completeness
- **Advisor** — Recommends the next step (CLI command + AI prompt) based on current pipeline and document state
- **Project Context** — Auto-scanned technical profile (tech stack, structure, dev practices, dependencies) injected into all AI prompts as background context
- **Memory** — Two-layer (short-term / long-term) cross-session memory for bugs, decisions, patterns, and gotchas, stored in a single Markdown file with event-driven staleness
- **Work Items** — First-class Bug, UserStory, and TestCase entities extracted from documents or created manually, linked to pipeline runs, issues, and commits
- **Pipeline Recommender** — Suggests the best pipeline based on task description keywords and project scale
- **Extractor** — Parses structured entities (stories, test cases, bugs) from Markdown documents
- **Desktop UI** — Read-only Tauri app that visualizes pipelines, documents, discussions, git status, work items, memories, and commit-document associations

### Document Lifecycle (`doc_lifecycle`)

Skills declare how documents behave across pipeline runs:

- **`singleton`** (default) — One document per topic per skill. Reused and updated across runs (e.g., PRD, RFC)
- **`cumulative`** — New document created each run. Each run produces a separate artifact (e.g., ADRs, test reports)

When `pipeline next` runs, it shows cross-run document reuse: singleton docs are flagged for skip/update, cumulative docs always create new.

## Installation

### Download from GitHub Releases

Pre-built binaries (CLI + Desktop UI) are available for macOS, Linux, and Windows:

> **[Latest Release](https://github.com/popsicle-lab/popsicle/releases/latest)**

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
# One-command setup: init + install module + scan context + generate agent instructions
popsicle init --agent claude,cursor --module github:popsicle-lab/popsicle-spec-development

# Install git post-commit hook for automatic tracking
popsicle git init

# Bootstrap: generate a project plan from the module's bootstrap spec
popsicle context bootstrap --generate-prompt   # Outputs an LLM prompt to stdout
# Feed the prompt to your LLM, save the output as bootstrap-plan.json, then:
popsicle context bootstrap --apply @bootstrap-plan.json

# 1. Create a namespace (or let bootstrap do it)
popsicle namespace create "backend-v2" -d "Backend rewrite"

# 2. Create a topic with tags (required before issues)
popsicle topic create "jwt-migration" -t auth,security --namespace "backend-v2"

# 3. Create an issue — auto-matched to topic by tags, or use --topic
popsicle issue create --type product --title "Add user authentication" --pipeline full-sdlc
# Or explicitly: --topic "jwt-migration"

# 4. Start the issue — the ONLY way to create a PipelineRun
popsicle issue start PROJ-1

# 5. See what to do next (shows cross-run doc reuse: skip/update/create)
popsicle pipeline next

# Revise specific stages of a completed run
popsicle pipeline revise <run-id> --stages design,implementation

# View topic details with all runs and documents
popsicle topic show "jwt-migration"

# Extract structured entities from documents
popsicle extract user-stories --from-doc <doc-id>
popsicle extract test-cases --from-doc <doc-id> --type unit

# Save a memory for future sessions
popsicle memory save --type bug --summary "SQLite WAL mode required for concurrent access"
```

`popsicle init --module` combines project initialization, module installation, project context scanning, and agent instruction generation into a single command. You can also run them separately:

```bash
popsicle init                     # Initialize project structure
popsicle module install <source>  # Install a module (auto-regenerates agent instructions)
popsicle context scan --force     # Re-scan project context
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

# Both agents + module in one command
popsicle init --agent claude,cursor --module github:popsicle-lab/popsicle-spec-development

# Skip agent files entirely
popsicle init --no-agent-files
```

Agent instruction files are automatically regenerated whenever modules change — `popsicle module install` and `popsicle module upgrade` both trigger regeneration, so there is no need to re-run `popsicle init` after installing a module.

Generated files include the complete skill registry — agent names, artifact types, input dependencies, workflow states, transitions, and guard conditions — so each agent knows the full development workflow without calling the CLI first.

## CLI Commands

### Namespace & Pipeline

| Command | Description |
|---------|-------------|
| `popsicle init [--agent <targets>] [--module <source>]` | Initialize project (idempotent); optionally install a module in the same step |
| `popsicle pipeline list` | List available pipeline templates |
| `popsicle pipeline create <name>` | Create a custom pipeline template |
| `popsicle pipeline status [--run <id>]` | Show pipeline run status with documents |
| `popsicle pipeline next [--run <id>]` | Advisor: recommended next steps (shows cross-run doc reuse) |
| `popsicle pipeline verify [--run <id>]` | Verify all stages complete and documents approved |
| `popsicle pipeline archive [--run <id>]` | Archive a completed pipeline run |
| `popsicle pipeline recommend --task <desc>` | Recommend pipeline based on task description |
| `popsicle pipeline revise <run-id> --stages <list>` | Revise specific stages of a completed pipeline run |
| `popsicle pipeline stage start <stage> [--run <id>]` | Start a pipeline stage (marks it in-progress) |
| `popsicle pipeline stage complete <stage> [--run <id>] [--confirm]` | Complete a pipeline stage; `--confirm` required for stages with `requires_approval` |
| `popsicle pipeline unlock [--run <id>]` | Force-release the Topic lock held by a pipeline run |

### Topic Management

| Command | Description |
|---------|-------------|
| `popsicle topic create <name> [-d <desc>] [-t <tags>] --namespace <name>` | Create a new topic under a namespace (required); tags enable Issue auto-matching |
| `popsicle topic list [--namespace <name>]` | List all topics; filter by namespace |
| `popsicle topic show <name>` | Show topic details with issues, runs, and documents |
| `popsicle topic delete <name> [--force]` | Delete a topic (--force to delete with existing runs) |

### Namespace Management

| Command | Description |
|---------|-------------|
| `popsicle namespace create <name> [-d <desc>] [-t <tags>]` | Create a new namespace |
| `popsicle namespace list [--status <s>]` | List namespaces; filter by status (active/completed/archived) |
| `popsicle namespace show <name>` | Show namespace details with associated topics |
| `popsicle namespace update <name> [--status/--description/--tags]` | Update namespace fields |
| `popsicle namespace delete <name> [--force]` | Delete a namespace (--force to delete with existing topics) |

### Module Management

| Command | Description |
|---------|-------------|
| `popsicle module list` | List installed modules (marks the active one) |
| `popsicle module show [<name>]` | Show module details: skills, pipelines, version, source |
| `popsicle module install <source>` | Install a module from local path or `github:user/repo[#ref][//subdir]`; auto-regenerates agent instructions |
| `popsicle module upgrade [--force]` | Upgrade active module (re-fetch from recorded source); auto-regenerates agent instructions |
| `popsicle module publish` | Publish the current module to the registry (requires `POPSICLE_REGISTRY_TOKEN`) |
| `popsicle registry search <query>` | Search the registry for modules |

Examples:

```bash
# Show the current module
popsicle module show

# Install the official spec-development module
popsicle module install github:popsicle-lab/popsicle-spec-development

# Install from a local directory
popsicle module install /path/to/my-module

# Install a specific tag from a repo subdirectory
popsicle module install github:myorg/mono-repo#v2.0//modules/security

# Upgrade module to latest from source
popsicle module upgrade

# Force reinstall even if version matches
popsicle module upgrade --force

# Publish a module to the registry
cd my-module && popsicle module publish
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

### Issue Tracking

| Command | Description |
|---------|-------------|
| `popsicle issue create --type <t> --title "<title>" [--topic <name>] [--pipeline <name>]` | Create an issue; auto-matched to a Topic by tags, or use `--topic` to specify explicitly |
| `popsicle issue list [--type/--status/--label/--topic]` | List issues with filters |
| `popsicle issue show <key>` | Show issue details with topic and pipeline runs |
| `popsicle issue start <key>` | Start workflow — the **only** way to create a PipelineRun (supports multiple runs per issue) |
| `popsicle issue update <key> [--status/--priority/--title]` | Update issue fields |

When `--topic` is not specified, `issue create` auto-matches the issue to a Topic by comparing tags. When `--pipeline` is specified at creation, `issue start` uses that pipeline directly. Otherwise, the pipeline is chosen by the recommender (keyword match on title/description) or falls back to the issue type default:

| Issue Type | Default Pipeline |
|------------|-----------------|
| `product` | `full-sdlc` |
| `technical` | `tech-sdlc` |
| `bug` | `test-only` |
| `idea` | `design-only` |

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
| `popsicle context bootstrap --generate-prompt` | Generate a bootstrap LLM prompt from the module's `bootstrap.md` |
| `popsicle context bootstrap --apply <file>` | Apply a bootstrap plan (LLM output) to create namespaces and topics |
| `popsicle prompt <skill> [--state <s>] [--run <id>]` | AI prompt with upstream context + memory injected |
| `popsicle migrate --skill <s> <paths...>` | Import existing Markdown docs into a pipeline run |
| `popsicle completions <zsh/bash/fish>` | Generate shell completions |

All commands support `--format json` for machine consumption.

## Official Module: `spec-development`

Popsicle ships as a bare orchestration engine. Install the official module to get skills, pipelines, and a bootstrap spec:

```bash
popsicle module install github:popsicle-lab/popsicle-spec-development
```

Browse the full module catalog on the [Popsicle Registry](https://popsicle-lab.com/registry).

### Bootstrap

The module includes a `bootstrap.md` that guides LLM-driven project initialization:

```bash
# Generate a prompt for your LLM (includes project scan + bootstrap spec)
popsicle context bootstrap --generate-prompt

# Apply the LLM-generated plan
popsicle context bootstrap --apply @bootstrap-plan.json
```

The bootstrap process analyzes the project, proposes namespaces (product domains) and topics (document collections with tags), and imports existing documentation as references. It does NOT create pipelines — those are created when you start an issue.

### Skills (16)

| Skill | Artifact Type | Description |
|-------|---------------|-------------|
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

### Pipelines (8)

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

#### `quick-feature` — Fast feature with clear requirements (scale: light)

```
prd-writer → rfc-writer + adr-writer → implementation → test-codegen → quality
```

Keywords: `quick`, `fast`, `clear-requirement`

#### `single-doc` — Single document workflow (scale: minimal)

One-shot document generation — pick any skill as a standalone task.

Keywords: `single`, `doc`, `one-off`, `write-rfc`, `write-adr`, `write-prd`

#### `hotfix` — Emergency fix workflow (scale: minimal)

```
implementation → unit-test-codegen → bug-tracker
```

Keywords: `hotfix`, `bug`, `fix`, `urgent`, `emergency`

Use `popsicle pipeline recommend --task "<description>"` to get a recommendation based on your task.

## Enforced Workflow

Popsicle enforces a strict entity creation order and an exclusive Topic lock:

```
Namespace → Topic (with tags) → Issue → PipelineRun → Document
```

1. **Create a Namespace** — `popsicle namespace create "my-project"`
2. **Create Topics with tags** — `popsicle topic create "auth" -t security,backend --namespace "my-project"`
3. **Create an Issue** — `popsicle issue create --type product --title "Add JWT auth"` (auto-matched to topic by tags, or use `--topic`)
4. **Start the Issue** — `popsicle issue start PROJ-1` — this is the **only** way to create a PipelineRun. This also **acquires an exclusive lock** on the Topic — if the Topic is already locked by another run, the command is rejected. Direct `pipeline run` no longer exists.
5. **Follow the pipeline** — `popsicle pipeline next` shows the next step, including cross-run document reuse (singleton docs: skip/update; cumulative docs: new)
6. **Start stages** — `popsicle pipeline stage start <stage>` begins work on a stage
7. **Create documents** — `popsicle doc create` succeeds only when the run holds the Topic lock and the stage is unblocked; blocked stages are rejected. Documents are created as "active"
8. **Complete stages** — `popsicle pipeline stage complete <stage>` marks a stage as done; all documents in the stage become "final". Stages with `requires_approval` need `--confirm`
9. **Lock auto-releases** when all stages complete. Use `popsicle pipeline unlock` to force-release the lock if needed

## Guard Conditions

Pipeline stages can define guards that enforce discipline before stage completion:

```yaml
# In pipeline stage definition
stages:
  - name: implementation
    skills: [implementation]
    depends_on: [design]
    guard: "upstream_approved"        # upstream stages must be completed
    requires_approval: true           # stage completion requires --confirm
```

| Guard | Description |
|-------|-------------|
| `upstream_approved` | All required upstream pipeline stages must be completed |
| `has_sections:A,B,C` | Document must contain the specified H2 sections with non-template content |

## Architecture

```
AI Agents ──→ CLI (only write path) ──→ Core Engine ──→ Files + SQLite
Developer ──→                               ↑
                                  Desktop UI (read-only)
                                    ├── Dashboard with topic overview
                                    ├── Topic browser & detail view
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
- **Pipeline orchestrates** — DAG-based stage dependencies with automatic state propagation; pipeline stage is the single source of truth for document state
- **Bootstrap before build** — LLM-driven project planning from natural language specs before the first pipeline run
- **Guards enforce discipline** — Upstream stage completion and content completeness checked before stage transitions
- **Git-aware** — Post-commit hooks auto-track commits; link commits to documents, stages, and skills
- **Multi-agent** — Native support for Claude Code and Cursor with auto-generated skills following the Agent Skills open standard
- **Hybrid storage** — Documents as Markdown files (Git-friendly), metadata and state indexed in SQLite
- **Topic-driven** — Pipeline runs and documents grouped by topic for cross-run document sharing, version tracking, and iterative revision
- **Issue-gated** — `issue start` is the only way to create PipelineRuns; acquires an exclusive Topic lock; no direct `pipeline run` command
- **Doc lifecycle-aware** — Singleton skills reuse/update docs across runs; cumulative skills create new docs each time
- **Context-aware** — Project tech profile auto-scanned and injected with attention-optimized ordering (low → medium → high relevance)
- **Memory-driven** — Cross-session memory with event-driven staleness, two-layer promotion, and 200-line budget
- **Work item traceability** — Bugs, stories, and test cases linked across documents, pipeline runs, issues, and commits
- **Scale-adaptive** — Pipeline recommender matches task complexity to the right workflow depth
- **Modular distribution** — Git-based registry; one active module per project; install from Git; `popsicle init` is idempotent
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
│   ├── popsicle.db               # SQLite index (topics, docs, pipeline runs, bugs, stories, test cases, memories)
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

- **Dashboard** — Topic overview, pipeline run summary, Git status bar, document statistics, quick actions with copyable commands
- **Topics** — Topic list with run/document counts, detail view with related pipeline runs, documents (latest versions), and revision history
- **Issues** — Issue list with type/status filters, pipeline binding indicator, create form with pipeline selector, detail view with progress tracking
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
