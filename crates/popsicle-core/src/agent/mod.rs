use std::path::Path;

use crate::error::Result;

/// Generate agent instruction files during `popsicle init`.
pub struct AgentInstaller;

impl AgentInstaller {
    /// Install all agent instruction files into the project.
    pub fn install(project_root: &Path) -> Result<Vec<String>> {
        let mut installed = Vec::new();

        Self::write_agents_md(project_root)?;
        installed.push("AGENTS.md".to_string());

        Self::write_claude_md(project_root)?;
        installed.push("CLAUDE.md".to_string());

        let cursor_dir = project_root.join(".cursor").join("rules");
        std::fs::create_dir_all(&cursor_dir)?;
        Self::write_cursor_rules(project_root)?;
        installed.push(".cursor/rules/popsicle.mdc".to_string());

        Ok(installed)
    }

    fn write_agents_md(root: &Path) -> Result<()> {
        let content = r#"# Popsicle — Agent Instructions

This project uses **Popsicle** for spec-driven development orchestration.
All development follows a structured pipeline: Domain → PRD → RFC/ADR → TestSpec → Implementation.

## Available Commands

```bash
# See current pipeline status and what to do next
popsicle pipeline status
popsicle pipeline next --format json

# Get AI prompt with upstream document context
popsicle prompt <skill> --run <run-id> --format json

# Get full pipeline context (all documents)
popsicle context --format json

# Create a document from a skill template
popsicle doc create <skill> --title "<title>" --run <run-id>

# Advance a document through its workflow
popsicle doc transition <doc-id> <action>

# Track a commit
popsicle git link --doc <doc-id> --stage <stage> --skill <skill>
```

## Workflow

1. Run `popsicle pipeline next --format json` to see recommended next steps
2. Each step includes a CLI command and an AI prompt
3. Create documents using `popsicle doc create`, fill in real content
4. Advance documents with `popsicle doc transition` (guards may block if prerequisites aren't met)
5. After completing a stage, downstream stages automatically unlock

## Rules

- Always check `popsicle pipeline next` before starting work
- Never skip stages — guard conditions enforce upstream document approval
- Use `popsicle prompt <skill> --run <run-id>` to get context-aware prompts
- After implementation changes, link commits with `popsicle git link`
- All output supports `--format json` for structured consumption
"#;
        std::fs::write(root.join("AGENTS.md"), content)?;
        Ok(())
    }

    fn write_claude_md(root: &Path) -> Result<()> {
        let content = r#"# Popsicle — Claude Code Instructions

This project uses Popsicle for spec-driven development. Follow the pipeline workflow.

## Before Starting Any Task

```bash
popsicle pipeline next --format json
```

This shows what to do next, including the CLI command and AI prompt.

## Key Commands

- `popsicle pipeline status` — current pipeline state
- `popsicle pipeline next --format json` — recommended next steps with prompts
- `popsicle context --format json` — all documents for current pipeline run
- `popsicle prompt <skill> --run <run-id>` — get prompt with upstream context
- `popsicle doc create <skill> --title "<t>" --run <run-id>` — create document
- `popsicle doc transition <doc-id> <action>` — advance document state
- `popsicle git link --doc <doc-id> --stage <stage>` — link commit to document

## Workflow Discipline

1. Check `popsicle pipeline next` for the current recommended step
2. Follow the suggested skill and action
3. Guard conditions enforce that upstream documents must be approved before downstream work
4. Fill in document sections with real content — template placeholders will be rejected by guards
5. Link your commits to the relevant documents and stages
"#;
        std::fs::write(root.join("CLAUDE.md"), content)?;
        Ok(())
    }

    fn write_cursor_rules(root: &Path) -> Result<()> {
        let content = r#"---
description: Popsicle spec-driven development workflow
globs:
alwaysApply: true
---

# Popsicle Workflow

This project uses Popsicle for spec-driven development orchestration.

## Before Any Task

Run `popsicle pipeline next --format json` to see what to do next.

## Commands

- `popsicle pipeline next --format json` — next steps with CLI commands and AI prompts
- `popsicle context --format json` — full pipeline context with all documents
- `popsicle prompt <skill> --run <run-id>` — context-aware AI prompt
- `popsicle doc create <skill> --title "<t>" --run <run-id>` — create document
- `popsicle doc transition <doc-id> <action>` — advance workflow state
- `popsicle git link --doc <doc-id> --stage <stage>` — link commit to document

## Rules

- Always check pipeline next before starting work
- Guards enforce upstream approval before downstream work proceeds
- Fill document sections with real content, not template placeholders
- Link commits to documents with `popsicle git link`
"#;
        let rules_dir = root.join(".cursor").join("rules");
        std::fs::write(rules_dir.join("popsicle.mdc"), content)?;
        Ok(())
    }
}
