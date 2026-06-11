# Popsicle — Agent Instructions

This project uses Popsicle for spec-driven development orchestration.

## Binary Resolution

When running `popsicle` commands, prefer the binary in the **project root directory** first,
then fall back to the one on the system PATH:

- **Linux / macOS**: `./popsicle` if it exists, otherwise `popsicle`
- **Windows**: `.\popsicle.exe` if it exists, otherwise `popsicle`

Before running any popsicle command, resolve the binary:

```bash
# Linux / macOS
if [ -x "./popsicle" ]; then POPSICLE=./popsicle; else POPSICLE=popsicle; fi

# Windows (PowerShell)
# if (Test-Path ".\popsicle.exe") { $POPSICLE = ".\popsicle.exe" } else { $POPSICLE = "popsicle" }
```

Then use `$POPSICLE` in place of `popsicle` for all commands.

## ⛔ MANDATORY: Before Starting ANY Development Task

You MUST follow this checklist before writing ANY code or making ANY changes.
No exceptions — not for "small" fixes, not for low-level modules, not for "just one line".

### Step 0: Verify namespace and specs exist

`issue start` requires at least one namespace and one spec. These are created during bootstrap.

If `issue start` fails with "No namespace found", the project needs bootstrapping:

```bash
popsicle context bootstrap --generate-prompt --format json
```

Bootstrap analyzes the codebase, proposes namespaces (product domains) and specs (specification documents), then asks the user to confirm before creating them. Do NOT skip this step.

Note: `popsicle init` is a manual step that creates the directory structure — it is NOT your concern. Bootstrap is the automated entry point.

### Step 1: Check for an active pipeline run

```bash
popsicle pipeline status --format json
```

If an active pipeline run exists → skip to **Step 4**.

### Step 2: If NO active pipeline run — find or create an Issue first

An Issue is REQUIRED before any pipeline run can start. `issue start` is the ONLY way to create a pipeline run.

**2a. Check for an existing issue:**

```bash
popsicle issue list --format json
```

If the user's task matches an existing issue → skip to **Step 3**.

**2b. If no matching issue exists — create one:**

Determine the issue type from the user's request:
- New feature / user-facing change → `product` (maps to `full-sdlc` pipeline)
- Refactoring / migration / internal improvement → `technical` (maps to `tech-sdlc` pipeline)
- Bug fix → `bug` (maps to `test-only` pipeline)
- Exploration / research → `idea` (maps to `design-only` pipeline)

```bash
popsicle issue create --type <product|technical|bug|idea> --title "<concise title>" --description "<what and why>" [--pipeline <name>]
```

Use `--pipeline <name>` to explicitly bind a pipeline template. When specified, `issue start` will use this pipeline directly, bypassing the recommender. Available pipelines can be listed with `popsicle pipeline list`. If the user explicitly asks for a specific workflow (e.g. "run full process" → `--pipeline full-sdlc`), always use `--pipeline`.

Show the created issue key to the user before proceeding.

### Step 3: Start the Issue (creates a pipeline run automatically)

```bash
popsicle issue start <ISSUE-KEY>
```

This automatically creates the appropriate pipeline run linked to the issue. `issue start` is the ONLY way to create a pipeline run — there are no shortcuts or alternatives.

### Step 4: Follow the pipeline

```bash
popsicle pipeline next --format json
```

Execute the suggested action. NEVER skip pipeline steps or write code outside of a pipeline run.

**When `context_command` is present** (action = `create`):

1. Run the `context_command` first — it returns an enriched prompt with project context, memories, historical references, and upstream documents.
2. Use the `full_prompt` from the JSON output as the writing instruction for the new document.
3. Then run the `cli_command` to create the document.

This ensures every new document benefits from cross-run historical context and accumulated project memories.

## Key Commands

### Issue (start here)

- `popsicle issue create --type <t> --title "<title>" --description "<desc>" [--pipeline <name>]` — create a new issue (use `--pipeline` to bind a specific pipeline)
- `popsicle issue list --format json` — list all issues
- `popsicle issue show <key> --format json` — show issue details
- `popsicle issue start <key>` — start workflow (creates pipeline run linked to issue)

### Pipeline

- `popsicle pipeline next --format json` — what to do next (with CLI command + guide)
- `popsicle pipeline status` — current pipeline state
- `popsicle pipeline recommend --task "<desc>"` — recommend pipeline for task
- `popsicle pipeline verify` — verify pipeline completion

### Document & Git

- `popsicle context --format json` — all documents for current run
- `popsicle doc create <skill> --title "<t>" --run <id>` — create document (must hold Spec lock)
- `popsicle doc summarize <id> --generate-prompt` — get LLM prompt for summarization
- `popsicle doc summarize <id> --summary "..." --tags "a,b,c"` — write LLM-generated summary/tags
- `popsicle context search <query>` — search documents across all runs (FTS5)
- `popsicle git link --doc <id> --stage <s>` — link commit to document

### Pipeline Stage Progression

- `popsicle pipeline stage start <stage>` — start a pipeline stage (marks it in-progress)
- `popsicle pipeline stage complete <stage> [--confirm]` — complete a stage; all documents become "final". Use `--confirm` for stages with `requires_approval`
- `popsicle pipeline unlock` — force-release the Spec lock (for stuck/abandoned runs)

## Document Summarization (MANDATORY after stage complete)

When `popsicle pipeline stage complete` outputs `[ACTION REQUIRED]` (text mode) or a `llm_summarize` field (JSON mode), you **MUST** immediately execute the LLM summarize workflow for each document in the completed stage. Without this step, documents will NOT be indexed for cross-run search.

1. Run `step1_generate_prompt` — this outputs a structured prompt with the document content
2. Send the `prompt` field to your LLM, parse the JSON response: `{"summary": "...", "tags": [...]}`
3. Run `step2_write_result` with the LLM-generated summary and tags

```bash
# Step 1: Get the summarize prompt
popsicle doc summarize <doc-id> --generate-prompt --format json

# Step 2: Send the "prompt" field to your LLM, then write results back
popsicle doc summarize <doc-id> --summary "LLM-generated summary" --tags "tag1,tag2,tag3"
```

Do NOT skip this step. Documents without LLM-generated summaries will not appear in cross-run search results.

## Workflow Rules

1. **NEVER write code without an active pipeline run** — no exceptions
2. **Namespace → Spec → Issue → Pipeline → Skill** — always follow this hierarchy; `issue start` is the ONLY way to create pipeline runs and acquires an exclusive Spec lock
3. Always check `popsicle pipeline next` before starting work on a step
4. Guards enforce upstream **stage completion** before downstream work proceeds
5. Fill document sections with real content — template placeholders are rejected
6. Link commits to documents with `popsicle git link`
7. **STOP after each stage** — after creating all documents for a stage, you MUST STOP, present a summary of what was done, show the `pipeline stage complete <stage>` command, and **wait for the user to confirm before proceeding**. Do NOT auto-execute `pipeline stage complete`. The user decides when a stage is done.
8. Stages marked `requires_approval`: require `--confirm` flag. The user MUST run the command themselves after review. No exception.
9. **Namespace and Spec names require human confirmation** — when bootstrap proposes namespaces and specs, you MUST present the proposed names and descriptions to the user and ask for confirmation BEFORE creating them. Do NOT auto-create namespaces or specs with names the user hasn't approved. Let the user rename, merge, or reject proposals.
9. **Spec lock**: do not attempt to operate on a Spec that is locked by another run. Use `popsicle pipeline unlock` only when explicitly told to force-release.
10. **Documents are "active" when created** and become "final" when their stage is completed via `pipeline stage complete`. There is no `doc transition` command.
11. **NEVER report a task as "complete" unless `popsicle pipeline verify` passes.** If stages remain incomplete, say which stages are remaining and what the next step is. Reporting completion prematurely is a critical error.

## Review Checklist Protocol

When a pipeline run is complete or when running `popsicle pipeline review --checklist`:

1. Run `popsicle pipeline review --checklist` to get all unchecked items as structured JSON (includes doc IDs and line numbers)
2. For **each unchecked item** (`- [ ]`), scan the codebase to determine if the work was actually done
3. If the item IS implemented in code → check it off: `popsicle checklist check --doc <doc_id> --lines <line1>,<line2>,...`
4. If the item is NOT implemented → keep it unchecked and add to the remaining-work summary
5. After updating documents, report:
   - How many items were auto-checked (with brief evidence for each)
   - How many items remain unchecked (with next-step suggestions)
6. Run `popsicle pipeline verify` to confirm final status

### Checklist CLI Commands

- `popsicle checklist status --run <run_id>` — view checkbox status for all docs in a run
- `popsicle checklist status --doc <doc_id>` — view checkbox status for one document
- `popsicle checklist check --doc <doc_id> --lines 5,12,23` — check off items by line number
- `popsicle checklist check --doc <doc_id> --match "search text"` — check items matching text
- `popsicle checklist check --doc <doc_id> --all` — check all unchecked items
- `popsicle checklist uncheck --doc <doc_id> --lines 5` — uncheck items (for corrections)

**IMPORTANT**: Acceptance criteria in PRD documents represent features that should be checked off as they are implemented throughout the pipeline. Before completing the final stage, review the PRD and update all checkboxes that correspond to implemented functionality.

## Memory Management

Project memories persist bugs, decisions, patterns, and gotchas across sessions.

- `popsicle memory save --type <bug|decision|pattern|gotcha> --summary "..." --detail "..." --tags "t1,t2" --files "path1,path2"`
- `popsicle memory list` — list all memories
- `popsicle memory stats` — usage statistics (line count / 200 limit)
- `popsicle memory promote <id>` — promote short-term → long-term
- `popsicle memory gc` — remove stale memories
- `popsicle memory check-stale` — detect outdated memories via git diff

### When to Save Memories

1. **Bug fix**: after fixing a non-trivial bug, save the root cause and fix approach (`--type bug`)
2. **Technical decision**: when choosing between alternatives with trade-offs (`--type decision`)
3. **Repeated pattern**: when the same issue appears 2+ times, consolidate into a pattern (`--type pattern`)
4. **Gotcha / pitfall**: when discovering a non-obvious constraint or trap (`--type gotcha`)

### Memory Writing Principles

1. **One sentence summary** — the `--summary` must be self-contained and actionable
2. **Detail is optional but precise** — root cause, not symptoms; solution, not narrative
3. **Tag related files** — use `--files` so the memory can be matched to future prompts and auto-detected as stale


---

## Module: intent-coder

The following skills and pipelines are provided by the `intent-coder` module.

### Skill Catalog

| Skill | Artifact | Inputs | States |
|-------|----------|--------|--------|
| `adr-writer` | adr-finalization-report | rfc-writer, rfc-writer, rfc-writer, arch-debate | review → completed → reviewing → finalizing |
| `arch-debate` | arch-debate-record | prd-writer, fact-extractor, fact-extractor, product-debate | debating → setup → completed → concluding |
| `cutover-author` | cutover-adr | equivalence-baseline, intent-consistency-check, shadow-implementer | drafting → completed → review → gating |
| `equivalence-baseline` | equivalence-report | shadow-implementer, fact-extractor | completed → inventory → running → reporting → review |
| `fact-extractor` | fact-extraction-report | none | completed → drafting → scanning → review |
| `intent-consistency-check` | intent-consistency-report | intent-spec-writer | review → completed → checking → reporting |
| `intent-spec-writer` | formal-acceptance-intent | prd-writer | completed → ingesting → tightening → verifying → review |
| `living-doc-author` | living-doc-sync-report | intent-consistency-check, prd-writer, prd-writer, shadow-implementer, equivalence-baseline, cutover-author | syncing → review → reporting → completed → scanning |
| `prd-writer` | prd-overview | product-debate, product-debate, fact-extractor, project-init | scoring → ingesting → drafting → completed → review |
| `product-debate` | product-debate-record | fact-extractor, fact-extractor, project-init | setup → debating → completed → concluding |
| `project-init` | project-init-plan | fact-extractor | planning → surveying → scaffolding → completed |
| `rfc-writer` | rfc | arch-debate, arch-debate, prd-writer, fact-extractor | review → completed → ingesting → scoring → drafting |
| `shadow-implementer` | implementation-coverage | adr-writer, rfc-writer, intent-consistency-check | review → completed → verifying → implementing → scoping |

