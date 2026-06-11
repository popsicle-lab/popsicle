---
name: popsicle-bootstrap
description: Bootstrap a project by analyzing its structure. Creates Namespaces and Specs — no pipelines or skills involved.
---

Bootstrap a project by analyzing its structure and organizing it into namespaces and specs.

Prefer the project-root binary (`./popsicle` or `.\popsicle.exe`) over the system PATH one.

## When to Use

- After `popsicle init --module <source>` when the project needs namespace/spec setup
- When starting a new popsicle workflow on a project
- Any time the project lacks namespaces or specs

## What Bootstrap Does (and Does NOT Do)

Bootstrap analyzes the project and creates:
- **Namespaces** — product domains (e.g. "backend-api", "mobile-app")
- **Specs** — specification document collections with tags (e.g. "auth-system", "payment-integration")
- **Reference documents** — imports existing docs into specs

Bootstrap does **NOT**:
- Choose or create pipelines (that happens at `issue start`)
- Map documents to skills (that happens at `issue start`)
- Create pipeline runs

## Module's bootstrap.md

The active module may provide a `bootstrap.md` file at `.popsicle/modules/<module>/bootstrap.md`.
This file contains **domain-specific instructions** that guide how the LLM organizes namespaces and specs
(e.g. recommended naming conventions, required specs, tagging strategies).

The bootstrap prompt generator (`--generate-prompt`) automatically loads and injects this file
into the LLM prompt under the "Module Bootstrap Instructions" section. If the file does not exist,
the section reads "No specific bootstrap instructions provided."

You do NOT need to load or read `bootstrap.md` yourself — it is handled automatically.

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

### Step 3: Send prompt to LLM, present to user, and apply

Send the `prompt` field to your LLM. It will return a JSON bootstrap plan like:

```json
{
  "namespaces": [
    {
      "name": "backend-api",
      "description": "Backend REST API service",
      "specs": [
        {
          "name": "auth-system",
          "description": "Authentication and authorization",
          "tags": ["auth", "login", "jwt", "session"],
          "documents": [
            {"path": "docs/auth.md", "doc_type": "reference", "title": "Auth Design Notes"}
          ]
        }
      ]
    }
  ],
  "summary": "Mono-repo with backend API and frontend SPA"
}
```

**⚠️ IMPORTANT: Before applying the plan, you MUST present the proposed namespace and spec names to the user and ask for confirmation.** The user may want to rename, merge, split, or reject proposals. Do NOT auto-apply.

Once the user approves (or modifies) the plan, apply it:

```bash
popsicle context bootstrap --apply '<JSON plan>'
```

If the JSON is large, save it to a file and use:

```bash
popsicle context bootstrap --apply @bootstrap-plan.json
```

## After Bootstrap

The project now has namespaces and specs. To start working:
1. Create an issue: `popsicle issue create --spec <spec-id> --title "..."`
2. Start the issue (creates pipeline run): `popsicle issue start <issue-id>`
3. Follow the pipeline: `popsicle pipeline next`
