# Popsicle — Agent Instructions

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
