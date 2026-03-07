# Skill Design Guide

This guide covers how to create Skills, connect them to Pipelines, and avoid common pitfalls.

## Skill Anatomy

A Skill is a directory containing `skill.yaml` and a `templates/` folder:

```
skills/my-skill/
├── skill.yaml              # Skill definition
└── templates/
    └── my-artifact.md      # Document template
```

### skill.yaml Structure

```yaml
name: my-skill                          # Unique, lowercase with hyphens
description: What this skill does       # One-line, shown to AI agents
version: "0.1.0"

inputs:                                 # Dependencies on other Skills' outputs
  - from_skill: upstream-skill
    artifact_type: upstream-artifact
    required: true

artifacts:                              # What this Skill produces
  - type: my-artifact                   # Globally unique artifact type name
    template: templates/my-artifact.md
    file_pattern: "{slug}.my-artifact.md"

workflow:                               # Internal state machine
  initial: draft
  states:
    draft:
      transitions:
        - to: review
          action: submit
          guard: "upstream_approved"
    review:
      transitions:
        - to: approved
          action: approve
          guard: "has_sections:Overview,Details"
        - to: draft
          action: revise
    approved:
      final: true

prompts:                                # AI prompts per workflow state
  draft: |
    Create a my-artifact document...
  review: |
    Review this document for...

hooks:                                  # Lifecycle event handlers (shell commands)
  on_enter: null
  on_artifact_created: null
  on_complete: null
```

## How Skills Connect to Pipelines

```
Pipeline defines order (stages + depends_on)
Skill defines capability (inputs + artifacts + workflow)
```

Pipeline stages reference skills by name. The `depends_on` field in the Pipeline must match the `inputs` field in the Skill:

```yaml
# pipeline.yaml
stages:
  - name: domain
    skill: domain-analysis           # Stage runs this skill
  - name: product
    skill: product-prd
    depends_on: [domain]             # Must match product-prd's inputs

# product-prd/skill.yaml
inputs:
  - from_skill: domain-analysis     # Matches the upstream stage's skill
    artifact_type: domain-model
    required: true
```

A single stage can run multiple skills in parallel:

```yaml
  - name: tech-design
    skills:                          # Parallel execution
      - tech-rfc
      - tech-adr
    depends_on: [product]
```

## Seven Rules for Skill Design

### 1. inputs must align with Pipeline depends_on

If Skill B declares `inputs: [{from_skill: A}]`, then B's stage in the Pipeline must `depends_on` A's stage. Otherwise B might start before A finishes.

**Correct:**
```yaml
# Skill: product-prd
inputs:
  - from_skill: domain-analysis     # depends on this skill

# Pipeline
stages:
  - name: domain
    skill: domain-analysis
  - name: product
    skill: product-prd
    depends_on: [domain]             # aligns with inputs
```

**Wrong:** Omitting `depends_on` when `inputs` exists — the stage may unlock before upstream documents are ready.

### 2. artifact_type is a global contract

`artifact_type` is how skills pass data to each other. The upstream skill's `artifacts[].type` must exactly match the downstream skill's `inputs[].artifact_type`.

```
domain-analysis produces  type: "domain-model"
                                   │
product-prd consumes  artifact_type: "domain-model"   ← must match
```

Naming convention: lowercase nouns with hyphens (`domain-model`, `test-spec`, `impl-plan`).

### 3. Place guards at forward-moving transitions

Guards check conditions before allowing a transition. Put them on "advance" transitions, not on "revert" transitions:

```yaml
states:
  draft:
    transitions:
      - to: review
        action: submit
        guard: "upstream_approved"       # ✓ guard the forward step
  review:
    transitions:
      - to: approved
        action: approve
        guard: "has_sections:Background,Goals"  # ✓ guard the approval
      - to: draft
        action: revise                   # ✗ no guard on revert — always allowed
```

Available guard types:

| Guard | What it checks |
|-------|---------------|
| `upstream_approved` | All required input documents exist and are in a final state |
| `has_sections:A,B,C` | Document body contains these H2 headings with non-placeholder content |

### 4. Write prompts for every non-final state

Agents get prompts via `popsicle prompt <skill> --run <id>`. Each state serves a different purpose:

```yaml
prompts:
  draft: |
    # Creation phase — tell the agent what to produce
    Based on the domain model, write a PRD covering:
    1. Background and motivation
    2. User stories with acceptance criteria

  discussion: |
    # Review phase — tell the agent how to review
    Review this PRD for completeness:
    - Are acceptance criteria measurable?
    - Any missing edge cases?
```

When `--run <id>` is provided, upstream documents are automatically appended to the prompt. The draft prompt only needs to say "based on the upstream document" without reproducing it.

Prompt template variables are expanded at runtime:

| Variable | Value |
|----------|-------|
| `{skill}` | Current skill name |
| `{state}` | Current workflow state |
| `{run_id}` | Pipeline run ID |
| `{date}` | Current date (YYYY-MM-DD) |
| `{branch}` | Current git branch |

### 5. Design templates for guard detection

The `has_sections` guard checks H2 headings and detects placeholder content. Structure templates so guards can verify completion:

```markdown
## Background                     ← guard checks this H2 exists
Describe the business context.    ← placeholder: guard will reject

## Goals                          ← guard checks this H2 exists
- Goal 1                          ← placeholder: guard will reject
```

Content patterns recognized as placeholders (unfilled):
- Empty content
- `...`
- `[Name]`, `[Title]`
- `Describe `, `Description...`
- `TODO`, `TBD`
- `Add detailed content here`

Agent must replace these with real content for the guard to pass.

### 6. Use hooks for side effects between Skills

| Hook | Triggers when | Typical use |
|------|--------------|-------------|
| `on_artifact_created` | After `popsicle doc create` | Logging, notifications, resource setup |
| `on_enter` | After `doc transition` to a non-final state | Start automation, notify reviewers |
| `on_complete` | After `doc transition` to a final state | Trigger downstream processes, notifications |

Hooks receive context via environment variables:

| Variable | Content |
|----------|---------|
| `$POPSICLE_EVENT` | Event name |
| `$POPSICLE_DOC_ID` | Document ID |
| `$POPSICLE_DOC_TYPE` | Artifact type |
| `$POPSICLE_DOC_TITLE` | Document title |
| `$POPSICLE_DOC_STATUS` | Current status |
| `$POPSICLE_SKILL` | Skill name |
| `$POPSICLE_RUN_ID` | Pipeline run ID |
| `$POPSICLE_FILE_PATH` | Document file path |

Example:

```yaml
hooks:
  on_complete: 'echo "✓ $POPSICLE_SKILL completed: $POPSICLE_DOC_TITLE"'
```

### 7. file_pattern determines output file naming

```yaml
artifacts:
  - type: prd
    template: templates/prd.md
    file_pattern: "{slug}.prd.md"
```

`{slug}` is derived from the `--title` argument: lowercased, non-alphanumeric characters replaced with hyphens. The suffix should match the `artifact_type` for easy identification.

Files are stored at: `.popsicle/artifacts/<pipeline-run-id>/{slug}.{type}.md`

## Checklist

Before adding a Skill to a Pipeline, verify:

```
□ name          Unique, lowercase with hyphens
□ description   One-line summary (agents display this directly)
□ inputs        Each from_skill has a corresponding upstream Stage in the Pipeline
□ artifacts     type name is globally unique; template file exists
□ workflow      Has an initial state with transitions; at least one final state
□ guards        Forward transitions (submit/approve) have appropriate guards
□ prompts       Every non-final state has a prompt
□ template      H2 sections match the has_sections guard parameters
□ hooks         Defined where side effects needed; null otherwise
□ pipeline      Target Pipeline has a Stage for this Skill with correct depends_on
```

## Example: Adding a "security-review" Skill

### 1. Create scaffold

```bash
popsicle skill create security-review \
  --description "Security threat modeling and review" \
  --artifact-type security-review
```

### 2. Edit skill.yaml

```yaml
name: security-review
description: Security threat modeling and review
version: "0.1.0"

inputs:
  - from_skill: tech-rfc
    artifact_type: rfc
    required: true

artifacts:
  - type: security-review
    template: templates/security-review.md
    file_pattern: "{slug}.security-review.md"

workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: review
          action: submit
          guard: "upstream_approved"
    review:
      transitions:
        - to: approved
          action: approve
          guard: "has_sections:Threat Model,Mitigations"
        - to: draft
          action: revise
    approved:
      final: true

prompts:
  draft: |
    Based on the RFC provided, create a security review:
    1. Identify threat vectors (STRIDE model)
    2. Assess risk levels
    3. Define mitigation strategies
    Follow the template structure.
  review: |
    Review the security assessment:
    - Are all threat vectors identified?
    - Are mitigations actionable?
    - Any missing attack surfaces?
```

### 3. Edit template

```markdown
## Threat Model

Identify threats using the STRIDE model.

| Threat | Category | Risk | Mitigation |
|--------|----------|------|------------|
| ... | ... | ... | ... |

## Mitigations

Describe mitigation strategies for each identified threat.

## Residual Risks

List any risks that cannot be fully mitigated.
```

### 4. Add to Pipeline

Edit your pipeline YAML to include the new stage:

```yaml
stages:
  # ... existing stages ...
  - name: security
    skill: security-review
    description: Security threat modeling
    depends_on: [tech-design]

  - name: implementation
    skill: implementation
    depends_on: [tech-design, test-design, security]  # add dependency
```

### 5. Regenerate agent instructions

```bash
# Re-run init to update agent files with the new skill catalog
popsicle init --agent claude,cursor,codex
```

## Design Principle

**Each Skill does one thing.** Skills connect through `inputs`/`artifacts` (loose coupling) and are orchestrated by Pipelines (unified sequencing). If a Skill is doing two unrelated things, split it into two Skills.
