use std::path::Path;

use crate::error::Result;
use crate::model::SkillDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTarget {
    Claude,
    Cursor,
}

impl AgentTarget {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Cursor => "cursor",
        }
    }
}

pub struct AgentInstaller;

impl AgentInstaller {
    pub fn install(
        project_root: &Path,
        targets: &[AgentTarget],
        skills: &[&SkillDef],
    ) -> Result<Vec<String>> {
        let targets = if targets.is_empty() {
            vec![AgentTarget::Claude]
        } else {
            targets.to_vec()
        };

        let overview = build_overview(skills);
        let mut installed = Vec::new();

        for target in &targets {
            match target {
                AgentTarget::Claude => {
                    installed.extend(install_claude(project_root, skills, &overview)?);
                }
                AgentTarget::Cursor => {
                    installed.extend(install_cursor(project_root, skills, &overview)?);
                }
            }
        }

        Ok(installed)
    }
}

/// Build a per-skill command file: workflow info + CLI commands + guide content.
fn build_skill_command(skill: &SkillDef) -> String {
    let mut s = String::new();

    s.push_str(&format!(
        "Perform the \"{}\" step in the Popsicle pipeline.\n\n",
        skill.name
    ));

    // Workflow section (auto-generated)
    s.push_str("## Workflow\n\n");
    s.push_str(&format!(
        "- **Initial state**: `{}`\n",
        skill.workflow.initial
    ));
    let finals: Vec<&str> = skill
        .workflow
        .states
        .iter()
        .filter(|(_, sd)| sd.r#final)
        .map(|(n, _)| n.as_str())
        .collect();
    if !finals.is_empty() {
        s.push_str(&format!(
            "- **Final state(s)**: `{}`\n",
            finals.join("`, `")
        ));
    }
    s.push_str("- **Transitions**:\n");
    for (state, sd) in &skill.workflow.states {
        for t in &sd.transitions {
            let guard = t
                .guard
                .as_ref()
                .map(|g| format!(" (guard: `{}`)", g))
                .unwrap_or_default();
            let approval = if t.requires_approval {
                " **⚠ requires human approval**"
            } else {
                ""
            };
            s.push_str(&format!(
                "  - `{}` → `{}` via `{}`{}{}\n",
                state, t.to, t.action, guard, approval
            ));
        }
    }

    if !skill.inputs.is_empty() {
        s.push_str("\n## Inputs (upstream dependencies)\n\n");
        for input in &skill.inputs {
            s.push_str(&format!(
                "- `{}` from skill `{}` ({})\n",
                input.artifact_type,
                input.from_skill,
                if input.required {
                    "required"
                } else {
                    "optional"
                }
            ));
        }
    }

    // CLI commands section (auto-generated)
    s.push_str("\n## Prerequisites\n\n");
    s.push_str("An active pipeline run MUST exist before executing this skill. ");
    s.push_str("If `popsicle pipeline status` shows no active run, you MUST first ");
    s.push_str(
        "create an Issue (`popsicle issue create`) then start it (`popsicle issue start <key>`). ",
    );
    s.push_str("NEVER execute this skill outside of a pipeline run.\n");

    s.push_str("\n## Commands\n\n");
    s.push_str("```bash\n");
    s.push_str("# Verify an active pipeline run exists and this skill is the current step\n");
    s.push_str("popsicle pipeline next --format json\n\n");
    s.push_str("# Get enriched prompt with historical references and project context\n");
    s.push_str(&format!(
        "popsicle prompt {} --run <run-id> --related --format json\n\n",
        skill.name
    ));
    s.push_str("# Create the document\n");
    s.push_str(&format!(
        "popsicle doc create {} --title \"<title>\" --run <run-id>\n\n",
        skill.name
    ));
    s.push_str("# View the created document\n");
    s.push_str("popsicle doc show <doc-id>\n\n");

    s.push_str("# ⚠ STOP — Do NOT auto-complete stages\n");
    s.push_str("# After creating all documents for a stage, STOP and show the user:\n");
    s.push_str("#   1. What documents were created\n");
    s.push_str("#   2. The stage completion command below\n");
    s.push_str("# Let the user review and decide when to complete the stage.\n");
    s.push_str("popsicle pipeline stage complete <stage-name>\n\n");

    // Check if any transition in this skill requires approval
    let has_approval = skill
        .workflow
        .states
        .iter()
        .any(|(_, sd)| sd.transitions.iter().any(|t| t.requires_approval));
    if has_approval {
        s.push_str("# ⚠ This stage requires --confirm (approval gate):\n");
        s.push_str(
            "# 禁止代用户执行。必须由用户本人审阅后在终端执行：\n",
        );
        s.push_str("popsicle pipeline stage complete <stage-name> --confirm\n");
    }

    s.push_str("```\n");

    // Writing guide (from guide.md — your core asset)
    if let Some(ref guide) = skill.guide {
        s.push_str("\n## Writing Guide\n\n");
        s.push_str(guide.trim());
        s.push('\n');
    }

    s
}

/// Build the overview section: skill catalog + pipeline info.
fn build_overview(skills: &[&SkillDef]) -> String {
    let mut s = String::from(
        r#"This project uses Popsicle for spec-driven development orchestration.

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

### Step 0: Verify namespace and topics exist

`issue start` requires at least one namespace and one topic. These are created during bootstrap.

If `issue start` fails with "No namespace found", the project needs bootstrapping:

```bash
popsicle context bootstrap --generate-prompt --format json
```

Bootstrap analyzes the codebase, proposes namespaces (product domains) and topics (document collections), then asks the user to confirm before creating them. Do NOT skip this step.

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
- `popsicle doc create <skill> --title "<t>" --run <id>` — create document (must hold Topic lock)
- `popsicle doc summarize <id> --generate-prompt` — get LLM prompt for summarization
- `popsicle doc summarize <id> --summary "..." --tags "a,b,c"` — write LLM-generated summary/tags
- `popsicle context search <query>` — search documents across all runs (FTS5)
- `popsicle git link --doc <id> --stage <s>` — link commit to document

### Pipeline Stage Progression

- `popsicle pipeline stage start <stage>` — start a pipeline stage (marks it in-progress)
- `popsicle pipeline stage complete <stage> [--confirm]` — complete a stage; all documents become "final". Use `--confirm` for stages with `requires_approval`
- `popsicle pipeline unlock` — force-release the Topic lock (for stuck/abandoned runs)

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
2. **Namespace → Topic → Issue → Pipeline → Skill** — always follow this hierarchy; `issue start` is the ONLY way to create pipeline runs and acquires an exclusive Topic lock
3. Always check `popsicle pipeline next` before starting work on a step
4. Guards enforce upstream **stage completion** before downstream work proceeds
5. Fill document sections with real content — template placeholders are rejected
6. Link commits to documents with `popsicle git link`
7. **STOP after each stage** — after creating all documents for a stage, you MUST STOP, present a summary of what was done, show the `pipeline stage complete <stage>` command, and **wait for the user to confirm before proceeding**. Do NOT auto-execute `pipeline stage complete`. The user decides when a stage is done.
8. Stages marked `requires_approval`: require `--confirm` flag. The user MUST run the command themselves after review. No exception.
9. **Topic lock**: do not attempt to operate on a Topic that is locked by another run. Use `popsicle pipeline unlock` only when explicitly told to force-release.
10. **Documents are "active" when created** and become "final" when their stage is completed via `pipeline stage complete`. There is no `doc transition` command.
11. **NEVER report a task as "complete" unless `popsicle pipeline verify` passes.** If stages remain incomplete, say which stages are remaining and what the next step is. Reporting completion prematurely is a critical error.

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
"#,
    );

    if !skills.is_empty() {
        s.push_str("\n## Skill Catalog\n\n");
        s.push_str("| Skill | Artifact | Inputs | States |\n");
        s.push_str("|-------|----------|--------|--------|\n");
        for skill in skills {
            let artifact = skill
                .artifacts
                .first()
                .map(|a| a.artifact_type.as_str())
                .unwrap_or("-");
            let inputs = if skill.inputs.is_empty() {
                "none".to_string()
            } else {
                skill
                    .inputs
                    .iter()
                    .map(|i| i.from_skill.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let states: Vec<&str> = skill.workflow.states.keys().map(|k| k.as_str()).collect();
            s.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                skill.name,
                artifact,
                inputs,
                states.join(" → ")
            ));
        }
    }

    s
}

fn install_claude(root: &Path, skills: &[&SkillDef], overview: &str) -> Result<Vec<String>> {
    let claude_dir = root.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;

    let instructions = format!("# Popsicle — Claude Code Instructions\n\n{}\n", overview);
    std::fs::write(claude_dir.join("CLAUDE.md"), instructions)?;

    let mut installed = vec![".claude/CLAUDE.md".to_string()];

    let skills_dir = claude_dir.join("skills");
    for skill in skills {
        let skill_dir = skills_dir.join(format!("popsicle-{}", skill.name));
        std::fs::create_dir_all(&skill_dir)?;

        let content = build_agent_skill(skill);
        std::fs::write(skill_dir.join("SKILL.md"), &content)?;
        installed.push(format!(".claude/skills/popsicle-{}/SKILL.md", skill.name));
    }

    let next_dir = skills_dir.join("popsicle-next");
    std::fs::create_dir_all(&next_dir)?;
    std::fs::write(next_dir.join("SKILL.md"), SKILL_NEXT)?;
    installed.push(".claude/skills/popsicle-next/SKILL.md".to_string());

    let memory_dir = skills_dir.join("popsicle-memory");
    std::fs::create_dir_all(&memory_dir)?;
    std::fs::write(memory_dir.join("SKILL.md"), SKILL_MEMORY)?;
    installed.push(".claude/skills/popsicle-memory/SKILL.md".to_string());

    let ctx_scan_dir = skills_dir.join("popsicle-context-scan");
    std::fs::create_dir_all(&ctx_scan_dir)?;
    std::fs::write(ctx_scan_dir.join("SKILL.md"), SKILL_CONTEXT_SCAN)?;
    installed.push(".claude/skills/popsicle-context-scan/SKILL.md".to_string());

    let bootstrap_dir = skills_dir.join("popsicle-bootstrap");
    std::fs::create_dir_all(&bootstrap_dir)?;
    std::fs::write(bootstrap_dir.join("SKILL.md"), SKILL_BOOTSTRAP)?;
    installed.push(".claude/skills/popsicle-bootstrap/SKILL.md".to_string());

    Ok(installed)
}

fn install_cursor(root: &Path, skills: &[&SkillDef], overview: &str) -> Result<Vec<String>> {
    let rules_dir = root.join(".cursor").join("rules");
    std::fs::create_dir_all(&rules_dir)?;

    let rules = format!(
        "---\ndescription: Popsicle spec-driven development workflow\nglobs:\nalwaysApply: true\n---\n\n# Popsicle Workflow\n\n{}\n",
        overview
    );
    std::fs::write(rules_dir.join("popsicle.mdc"), rules)?;

    let mut installed = vec![".cursor/rules/popsicle.mdc".into()];

    let skills_dir = root.join(".cursor").join("skills");
    for skill in skills {
        let skill_dir = skills_dir.join(format!("popsicle-{}", skill.name));
        std::fs::create_dir_all(&skill_dir)?;

        let content = build_agent_skill(skill);
        std::fs::write(skill_dir.join("SKILL.md"), &content)?;
        installed.push(format!(".cursor/skills/popsicle-{}/SKILL.md", skill.name));
    }

    let memory_dir = skills_dir.join("popsicle-memory");
    std::fs::create_dir_all(&memory_dir)?;
    std::fs::write(memory_dir.join("SKILL.md"), SKILL_MEMORY)?;
    installed.push(".cursor/skills/popsicle-memory/SKILL.md".to_string());

    let ctx_scan_dir = skills_dir.join("popsicle-context-scan");
    std::fs::create_dir_all(&ctx_scan_dir)?;
    std::fs::write(ctx_scan_dir.join("SKILL.md"), SKILL_CONTEXT_SCAN)?;
    installed.push(".cursor/skills/popsicle-context-scan/SKILL.md".to_string());

    let bootstrap_dir = skills_dir.join("popsicle-bootstrap");
    std::fs::create_dir_all(&bootstrap_dir)?;
    std::fs::write(bootstrap_dir.join("SKILL.md"), SKILL_BOOTSTRAP)?;
    installed.push(".cursor/skills/popsicle-bootstrap/SKILL.md".to_string());

    Ok(installed)
}

/// Build a SKILL.md file following the Agent Skills open standard.
/// Used by both Claude Code (.claude/skills/) and Cursor (.cursor/skills/).
fn build_agent_skill(skill: &SkillDef) -> String {
    let mut s = String::new();

    s.push_str(&format!(
        "---\nname: popsicle-{}\ndescription: {}\n---\n\n",
        skill.name, skill.description
    ));

    s.push_str(&build_skill_command(skill));

    s
}

const SKILL_MEMORY: &str = r#"---
name: popsicle-memory
description: Save project memories (bug fixes, decisions, patterns, gotchas) for cross-session persistence. Use after fixing bugs, making technical decisions, discovering patterns, or hitting gotchas.
---

Save a project memory so it persists across sessions and gets injected into future prompts.

Prefer the project-root binary (`./popsicle` or `.\popsicle.exe`) over the system PATH one.

## IMPORTANT: Memory vs Bug Recording

- **Discovered a NEW bug?** → Use `popsicle bug create` or `popsicle bug record` (NOT memory). This stores the bug in the database and makes it visible in the Desktop UI's Bugs page.
- **FIXED a bug and want to remember the lesson?** → Use `popsicle memory save --type bug`. This saves the root cause and fix approach as a memory for future sessions.

Memory is for **cross-session experience**, not for **issue tracking**.

## When to Use

- **After fixing a bug**: save the root cause and fix so future sessions avoid re-introducing it
- **After a technical decision**: save the choice and rationale (e.g. "chose BTreeMap over HashMap for deterministic ordering")
- **After discovering a pattern**: when the same class of issue appears 2+ times, consolidate into a reusable pattern
- **After hitting a gotcha**: save non-obvious constraints (e.g. "serde requires #[serde(default)] for backward-compatible YAML fields")

## Commands

```bash
# Save a bug-fix memory (AFTER fixing — for the lesson learned, NOT for new bug reporting)
popsicle memory save --type bug \
  --summary "HashMap iteration causes non-deterministic context ordering" \
  --detail "assemble_input_context used HashMap, causing high-relevance docs to appear mid-prompt. Fix: use BTreeMap." \
  --tags "context-injection,ordering" \
  --files "engine/context.rs"

# Save a decision memory
popsicle memory save --type decision \
  --summary "Prompt ordering: context before instruction for attention optimization" \
  --detail "Based on LLM attention U-curve, high-relevance context placed adjacent to instruction at prompt end." \
  --tags "prompt,attention"

# Save a pattern memory (long-term)
popsicle memory save --type pattern --long-term \
  --summary "Always use #[serde(default)] for new YAML fields" \
  --detail "New fields without default break deserialization of existing files." \
  --tags "serde,yaml,backward-compat" \
  --files "model/skill.rs"

# Save a gotcha
popsicle memory save --type gotcha \
  --summary "Tauri invoke parameter names must be camelCase" \
  --tags "tauri,frontend"

# Check current memory usage
popsicle memory stats

# Review memories before saving to avoid duplicates
popsicle memory list
```

## Principles

1. **Be concise**: summary should be one self-contained sentence; detail is 1-3 lines max
2. **Tag files**: use `--files` so the memory auto-matches future prompts and stale detection works
3. **Avoid duplicates**: run `popsicle memory list` first; if a similar memory exists, consider promoting it or merging
4. **Default is short-term**: only use `--long-term` for validated, high-confidence memories
5. **Never use memory to report new bugs**: use `popsicle bug create` / `popsicle bug record` instead
"#;

const SKILL_CONTEXT_SCAN: &str = r#"---
name: popsicle-context-scan
description: Analyze the project codebase and generate a rich technical profile. Use when starting work on a new project or when the project context seems incomplete.
---

Analyze the project's codebase to build a comprehensive technical profile.

Prefer the project-root binary (`./popsicle` or `.\popsicle.exe`) over the system PATH one.

## When to Use

- After `popsicle init` (project-context.md has basic info but lacks depth)
- When `.popsicle/project-context.md` is missing or sparse
- When `popsicle pipeline next` suggests updating project context

## Analysis Steps

1. Read `.popsicle/project-context.md` to see what's already detected
2. Sample 3-5 representative source files to identify:
   - Coding conventions (naming, error handling, module organization)
   - Architecture patterns (layered, hexagonal, MVC, etc.)
   - Testing patterns (unit test structure, mocking approach)
3. Check for project-specific conventions:
   - README, CONTRIBUTING.md, or similar docs
   - Linter/formatter configurations for style rules
4. Write findings using the CLI:

```bash
popsicle context update --section "Architecture Patterns" --content "..."
popsicle context update --section "Coding Conventions" --content "..."
popsicle context update --section "Testing Patterns" --content "..."
```

## Guidelines

- Be concise: each section should be 3-10 bullet points
- Focus on patterns that affect AI code generation quality
- Don't repeat what's already in Tech Stack or Key Dependencies
- Use the project's actual terminology (e.g. "crate" for Rust, "package" for Node)
"#;

const SKILL_NEXT: &str = r#"---
name: popsicle-next
description: Check what to do next in the Popsicle pipeline. Use when starting work, after completing a step, or when unsure what to do next.
---

Check what to do next in the Popsicle pipeline and follow the recommended action.

Prefer the project-root binary (`./popsicle` or `.\popsicle.exe`) over the system PATH one.

## Step 1: Ensure project is bootstrapped and a pipeline run exists

```bash
popsicle pipeline status --format json
```

If this fails with "not bootstrapped" or "no topics", the project needs bootstrapping first:

```bash
popsicle context bootstrap --generate-prompt --format json
```

If no active pipeline run exists, do NOT proceed. First create an issue and start it:

```bash
popsicle issue create --type <product|technical|bug|idea> --title "<title>" --description "<desc>"
popsicle issue start <ISSUE-KEY>
```

## Step 2: Get next action

```bash
popsicle pipeline next --format json
```

Then execute the suggested CLI command to create the document.

**CRITICAL RULES:**
- After creating all documents for a stage, **STOP and present a summary** to the user. Show the `pipeline stage complete <stage>` command and wait for the user to decide when to proceed.
- Do NOT auto-execute `pipeline stage complete` — the user must confirm each stage transition.
- If a step has `requires_approval: true`, STOP and ask the user to review before proceeding.
- Do NOT report the task as "complete" unless `popsicle pipeline verify` passes. Always show remaining stages.

**Important**: When a step includes a `context_command` field, run it FIRST to get the enriched prompt (with historical references and project memories), then use that prompt to guide document creation.
"#;

const SKILL_BOOTSTRAP: &str = r#"---
name: popsicle-bootstrap
description: Bootstrap a project from an installed module. Discovers existing docs, generates an LLM-driven plan, and creates a Topic with imported documents and pipeline run.
---

Bootstrap a project by analyzing its existing documentation and mapping it to module skills.

Prefer the project-root binary (`./popsicle` or `.\popsicle.exe`) over the system PATH one.

## When to Use

- After `popsicle init --module <source>` when the project has existing documentation
- After `popsicle module install <source>` to set up the workflow for the project
- When starting a new popsicle workflow on a project with existing docs (README, specs, RFCs, etc.)

## 3-Step Bootstrap Flow

### Step 1: Ensure project context exists

```bash
popsicle context scan --force
```

### Step 2: Generate the bootstrap prompt

```bash
popsicle context bootstrap --generate-prompt --format json
```

This outputs a JSON object with a `prompt` field. The prompt includes:
- Project technical profile
- File tree
- Discovered documentation with content previews
- Module's bootstrap instructions (from `bootstrap.md`)
- Available skills and pipelines

### Step 3: Send prompt to LLM and apply the plan

Send the `prompt` field to your LLM. It will return a JSON bootstrap plan like:

```json
{
  "topic_name": "my-project",
  "pipeline": "full-sdlc",
  "documents": [
    {"path": "docs/prd.md", "skill": "prd-writer", "doc_type": "prd", "title": "Product Requirements"},
    {"path": "README.md", "skill": "domain-analysis", "doc_type": "overview", "title": "Project Overview"}
  ],
  "summary": "Rust CLI project with existing PRD and README"
}
```

Apply the plan:

```bash
popsicle context bootstrap --apply '<JSON plan>' --start
```

Use `--start` to also create a PipelineRun. Omit it to only create the Topic and import documents.

If the JSON is large, save it to a file and use:

```bash
popsicle context bootstrap --apply @bootstrap-plan.json --start
```

## After Bootstrap

Run `popsicle pipeline next --format json` to see the first recommended action in the new pipeline run.
"#;
