# Skill Design Guide

This guide covers how to create Skills, connect them to Pipelines, and avoid common pitfalls.

## Skill Anatomy

A Skill is a directory with three files, each serving a distinct purpose:

```
skills/my-skill/
├── skill.yaml              # Orchestration: workflow, inputs, guards, hooks
├── guide.md                # Writing guide: how to write this document well
└── templates/
    └── my-artifact.md      # Template: document skeleton with sections
```

| File | Who reads it | What it does |
|------|-------------|-------------|
| `skill.yaml` | Popsicle CLI | Defines the workflow state machine, input dependencies, guards, and hooks |
| `guide.md` | AI agents (via generated commands) | Your writing standards, good/bad examples, thinking frameworks, common mistakes |
| `templates/*.md` | `popsicle doc create` | Document skeleton — the H2 sections an agent fills in |

These three files have different audiences and should not be mixed:

- **skill.yaml** is for the orchestration engine — keep it pure configuration, no prose
- **guide.md** is for the AI agent — write it as if teaching a junior developer how to produce this document
- **template** is the starting point — structure the H2 sections to match your `has_sections` guard

## How `popsicle init` Assembles Agent Files

On `popsicle init`, each Skill's three files are assembled into Agent-native command files:

```
skill.yaml  ──→  ## Workflow (auto-generated: states, transitions, guards)
                 ## Commands (auto-generated: CLI command sequence)
guide.md    ──→  ## Writing Guide (your content, injected as-is)
```

For example, `popsicle init --agent claude` generates `.claude/skills/popsicle-domain-analysis/SKILL.md`:

```markdown
Perform the "domain-analysis" step in the Popsicle pipeline.

## Workflow                            ← from skill.yaml (auto)
- Initial state: draft
- Final state: approved
- Transitions: draft→review via submit, review→approved via approve

## Commands                            ← from skill.yaml (auto)
popsicle doc create domain-analysis --title "<title>" --run <run-id>
popsicle doc transition <doc-id> submit
popsicle doc transition <doc-id> approve

## Writing Guide                       ← from guide.md (your content)
A domain model document defines the system's core boundaries...
Each bounded context must answer: ...
```

The agent reads this single file and knows both **what commands to run** and **how to write the document well**.

## skill.yaml Reference

```yaml
name: my-skill                          # Unique, lowercase with hyphens
description: What this skill does       # One-line summary
version: "0.1.0"

inputs:                                 # Dependencies on upstream Skills
  - from_skill: upstream-skill
    artifact_type: upstream-artifact
    required: true                      # false = optional dependency

artifacts:                              # What this Skill produces
  - type: my-artifact                   # Globally unique type name
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

hooks:                                  # Lifecycle event handlers (shell commands)
  on_enter: null
  on_artifact_created: null
  on_complete: null
```

Note: **no `prompts` field**. Writing guidance belongs in `guide.md`, not in YAML.

## guide.md Reference

`guide.md` is a Markdown file that teaches the AI agent how to produce a high-quality document of this type. Structure it with these sections:

```markdown
# [Skill Name] Writing Guide

## Purpose
What this document is for and who reads it.

## Section Standards
For each H2 section in the template:
- What it should contain
- Good examples (quote blocks)
- Bad examples (quote blocks)
- How much detail is expected

## Thinking Framework
Questions the writer should ask themselves.

## Common Mistakes
Patterns to avoid, with explanations of why.
```

Tips for writing effective guides:
- Write as if teaching — the agent has no prior context about your project
- Use **Good/Bad** example pairs — agents learn better from contrast
- Keep it practical — focus on "what to write" not "what the theory says"
- Reference the template sections — the guide and template should correspond

## How Skills Connect to Pipelines

```
Pipeline defines execution order (stages + depends_on)
Skill defines capability (inputs + artifacts + workflow)
```

Pipeline stages reference Skills by name. `depends_on` in the Pipeline must align with `inputs` in the Skill:

```yaml
# pipeline.yaml
stages:
  - name: domain
    skill: domain-analysis
  - name: product
    skill: product-prd
    depends_on: [domain]             # must align: product-prd has inputs from domain-analysis

# product-prd/skill.yaml
inputs:
  - from_skill: domain-analysis     # matches the upstream stage's skill
    artifact_type: domain-model
    required: true
```

A single stage can run multiple Skills in parallel:

```yaml
  - name: tech-design
    skills:
      - tech-rfc
      - tech-adr
    depends_on: [product]
```

## Seven Rules for Skill Design

### 1. inputs must align with Pipeline depends_on

If Skill B declares `inputs: [{from_skill: A}]`, then B's stage must `depends_on` A's stage. Otherwise B might start before A finishes.

### 2. artifact_type is a global contract

The upstream Skill's `artifacts[].type` must exactly match the downstream Skill's `inputs[].artifact_type`. Naming convention: lowercase nouns with hyphens (`domain-model`, `test-spec`, `impl-plan`).

### 3. Place guards at forward-moving transitions only

Put guards on "advance" transitions (`submit`, `approve`), not on "revert" transitions (`revise`):

```yaml
states:
  draft:
    transitions:
      - to: review
        action: submit
        guard: "upstream_approved"         # guard the forward step
  review:
    transitions:
      - to: approved
        action: approve
        guard: "has_sections:Background,Goals"  # guard the approval
      - to: draft
        action: revise                     # no guard on revert
```

Available guards:

| Guard | What it checks |
|-------|---------------|
| `upstream_approved` | All required input documents exist and are in a final state |
| `has_sections:A,B,C` | Document body has these H2 headings with non-placeholder content |

### 4. Write guide.md, not YAML prompts

Put writing guidance in `guide.md`, not in `skill.yaml`'s `prompts` field. The guide is injected into Agent command files during `popsicle init`. This gives you full Markdown formatting — examples, tables, code blocks — instead of YAML block scalars.

### 5. Design templates for guard detection

The `has_sections` guard checks H2 headings. Structure templates so guards can verify completion:

```markdown
## Background                     ← guard checks this H2 exists
Describe the business context.    ← placeholder: guard will reject
```

Placeholder patterns detected (will be rejected by guard):
`...`, `[Name]`, `[Title]`, `Describe `, `TODO`, `TBD`, `Add detailed content here`

### 6. Use hooks for side effects

| Hook | Triggers when | Typical use |
|------|--------------|-------------|
| `on_artifact_created` | After `doc create` | Logging, notifications |
| `on_enter` | After `doc transition` to non-final state | Start automation |
| `on_complete` | After `doc transition` to final state | Trigger downstream |

Hooks receive context via environment variables: `$POPSICLE_DOC_ID`, `$POPSICLE_SKILL`, `$POPSICLE_DOC_STATUS`, `$POPSICLE_RUN_ID`, etc.

### 7. file_pattern determines output naming

`{slug}` is derived from `--title`: lowercased, non-alphanumeric replaced with hyphens. Files are stored at `.popsicle/artifacts/<run-id>/{slug}.{type}.md`.

## Checklist

Before adding a Skill to a Pipeline:

```
□ skill.yaml
  □ name          Unique, lowercase with hyphens
  □ inputs        Each from_skill has a corresponding upstream Stage in the Pipeline
  □ artifacts     type is globally unique; template file exists
  □ workflow      Has initial state with transitions; at least one final state
  □ guards        Forward transitions have appropriate guards
  □ hooks         Defined where side effects needed; null otherwise

□ guide.md
  □ Purpose       One paragraph: what this document is for
  □ Sections      Standards for each H2 section in the template
  □ Examples      Good/Bad pairs for key sections
  □ Mistakes      Common pitfalls to avoid

□ template
  □ H2 sections   Match the has_sections guard parameters
  □ Placeholders  Use recognizable placeholder text the guard can detect

□ pipeline
  □ Stage added   With correct depends_on matching inputs
```

## Example: Creating a "security-review" Skill

### Step 1: Scaffold

```bash
popsicle skill create security-review \
  --description "Security threat modeling and review" \
  --artifact-type security-review
```

This creates:

```
.popsicle/skills/security-review/
├── skill.yaml
├── guide.md
└── templates/
    └── security-review.md
```

### Step 2: Edit skill.yaml (orchestration)

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
```

### Step 3: Edit guide.md (writing standards)

```markdown
# Security Review Writing Guide

## Purpose

A security review identifies threats, assesses risks, and defines mitigations
before implementation begins. It should be specific enough that a developer
knows exactly what to protect against.

## Section Standards

### Threat Model

Use the STRIDE model. For each threat:
- Describe the attack vector specifically
- Classify by category (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege)
- Assign risk level with justification

**Good:**
> **JWT Token Theft** (Spoofing, High Risk): An attacker intercepts the JWT
> from an insecure WebSocket connection and replays it. Mitigated by
> using WSS and short token expiry (15min).

**Bad:**
> Security might be a problem.

### Mitigations

Each mitigation must be actionable and linked to a specific threat.

## Common Mistakes

- Listing generic threats not specific to this system
- Missing mitigations for identified threats
- Not considering the supply chain (dependencies, third-party APIs)
```

### Step 4: Edit template (document skeleton)

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

### Step 5: Add to Pipeline

```yaml
# In your pipeline YAML
stages:
  # ... existing stages ...
  - name: security
    skill: security-review
    description: Security threat modeling
    depends_on: [tech-design]

  - name: implementation
    skill: implementation
    depends_on: [tech-design, test-design, security]
```

### Step 6: Regenerate Agent instructions

```bash
popsicle init --agent claude,cursor
```

This regenerates all Agent skill files, now including the new `security-review` skill with its workflow, CLI commands, and your writing guide.

## Design Principle

**Separate concerns by file.** `skill.yaml` is pure orchestration config — the engine reads it. `guide.md` is pure writing guidance — the agent reads it. `template` is the document skeleton — the agent fills it in. If you find yourself putting prose in YAML or workflow logic in Markdown, you're mixing concerns.
