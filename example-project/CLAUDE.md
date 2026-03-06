# Popsicle — Claude Code Instructions

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
