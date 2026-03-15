# Module Development Guide

This guide covers how to create, publish, install, and upgrade Modules — Popsicle's self-contained distribution unit for Skills and Pipelines.

For Skill-level design (skill.yaml, guide.md, templates), see [Skill Design Guide](skill-guide.md).

## What Is a Module

A Module is the atomic unit of distribution. It packages a set of related Skills and their Pipelines into a single directory that can be installed, replaced, or upgraded as a whole.

```
Module ≈ { module.yaml, skills/*, pipelines/* }
```

Key properties:

- **Self-contained** — every Pipeline in a Module only references Skills within the same Module
- **Single active** — one project has exactly one active Module at a time; installing a new one replaces the current one
- **Runtime-transparent** — after installation, the engine loads Skills and Pipelines from flat directories; it never sees the Module boundary

## Module Directory Structure

```
my-module/
├── module.yaml                  # Module metadata (required)
├── skills/                      # All Skills in this Module
│   ├── domain-analysis/
│   │   ├── skill.yaml
│   │   ├── guide.md
│   │   └── templates/
│   ├── prd-writer/
│   │   ├── skill.yaml
│   │   ├── guide.md
│   │   └── templates/
│   └── ...
└── pipelines/                   # Pipelines that compose above Skills
    ├── full-sdlc.pipeline.yaml
    ├── tech-sdlc.pipeline.yaml
    └── ...
```

## module.yaml Reference

```yaml
name: my-module                  # Unique, lowercase with hyphens
version: "1.0.0"                 # Semver string
description: "One-line summary"  # Optional
author: "Your Name"              # Optional
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Module identifier, used as the directory name under `.popsicle/modules/` |
| `version` | yes | Version string for upgrade comparison |
| `description` | no | Human-readable summary shown in `module list` and `module show` |
| `author` | no | Author or organization name |

## Creating a Module

### Step 1: Scaffold the directory

```bash
mkdir -p my-module/skills my-module/pipelines
```

### Step 2: Write module.yaml

```yaml
name: security-focused
version: "0.1.0"
description: "Security-first development workflow with threat modeling"
author: "SecTeam"
```

### Step 3: Add Skills

Each subdirectory under `skills/` is a Skill. See the [Skill Design Guide](skill-guide.md) for the full anatomy (skill.yaml + guide.md + templates/).

```bash
# Create skills following the standard structure
mkdir -p my-module/skills/threat-model/{templates}
# Edit skill.yaml, guide.md, and templates/
```

### Step 4: Add Pipelines

Pipelines reference Skills by name. Every Skill referenced in a Pipeline **must** exist under the same Module's `skills/` directory.

```yaml
# my-module/pipelines/secure-sdlc.pipeline.yaml
name: secure-sdlc
description: "SDLC with mandatory security review"
scale: standard
stages:
  - name: threat-analysis
    skill: threat-model
    description: Threat modeling
  - name: design
    skill: secure-rfc
    depends_on: [threat-analysis]
  - name: implementation
    skill: implementation
    depends_on: [design]
```

### Step 5: Validate

Install locally to verify:

```bash
popsicle module install /path/to/my-module
popsicle module show
popsicle skill list
popsicle pipeline list
```

## Installing Modules

### From a local directory

```bash
popsicle module install /path/to/my-module
```

### From GitHub

```bash
# Basic — clone the repo root as the module
popsicle module install github:myorg/custom-skills

# With a specific branch or tag
popsicle module install github:myorg/custom-skills#v2.0

# Subdirectory within a monorepo
popsicle module install github:myorg/mono-repo#main//modules/security
```

The `github:` source format:

```
github:user/repo[#ref][//subdir]
```

| Part | Required | Description |
|------|----------|-------------|
| `user/repo` | yes | GitHub repository (owner/repo) |
| `#ref` | no | Branch name or tag (defaults to default branch) |
| `//subdir` | no | Subdirectory within the repo containing the module |

### What happens during install

1. Source is resolved — local path used directly; `github:` cloned with `git clone --depth 1` to a temp directory
2. `module.yaml` is validated — must exist and be parseable
3. `skills/` directory is checked — must exist
4. Existing module with the same name is removed from `.popsicle/modules/<name>/`
5. Files are copied recursively to `.popsicle/modules/<name>/`
6. `config.toml` is updated:

```toml
[module]
name = "security-focused"
source = "github:myorg/custom-skills#v2.0"
version = "0.1.0"
```

## Upgrading Modules

For modules installed from `github:`, `upgrade` re-fetches from the recorded source:

```bash
popsicle module upgrade
```

This is equivalent to re-running `popsicle module install <original-source>` — it clones the latest commit from the same branch/tag and overwrites the installed module.

## Three-Layer Loading Priority

Skills and Pipelines are loaded from three directories. Later layers overwrite earlier ones (same-name wins):

| Priority | Directory | Purpose |
|----------|-----------|---------|
| 1 (lowest) | `.popsicle/modules/<active>/skills/` | Module-provided defaults |
| 2 | `.popsicle/skills/` | Project-local overrides |
| 3 (highest) | `skills/` (workspace root) | Development / workspace-level |

This means you can override any Module Skill by placing a same-named Skill directory in `.popsicle/skills/` without modifying the Module itself.

The same priority applies to Pipelines (`.popsicle/modules/<active>/pipelines/` → `.popsicle/pipelines/` → `pipelines/`).

## Overriding a Single Skill

To customize a Module's Skill without forking the entire Module:

```bash
# Copy the skill you want to override
cp -r .popsicle/modules/spec-development/skills/prd-writer .popsicle/skills/prd-writer

# Edit the copy — this version takes priority over the module's
vim .popsicle/skills/prd-writer/guide.md
```

The project-local copy in `.popsicle/skills/prd-writer/` will be loaded instead of the Module's version. All other Skills continue to come from the Module.

## The Official Module

The official module (`spec-development`) is distributed as a separate Git repository:

```bash
popsicle module install github:curtiseng/popsclice-spec-development
```

This installs 17 Skills and 5 Pipelines covering the full spec-driven development lifecycle. After installation, `config.toml` records the module source for future upgrades via `popsicle module upgrade`.

## Publishing a Module

A Module is just a directory — publishing means making it accessible via Git:

1. Create a repository with the Module structure at the root (or in a subdirectory)
2. Add `module.yaml` with name and version
3. Tag releases with semver versions
4. Users install with:

```bash
popsicle module install github:your-org/your-module#v1.0
```

### Repository layout options

**Root module** — the repo root is the module:

```
your-module/
├── module.yaml
├── skills/
└── pipelines/
```

```bash
popsicle module install github:your-org/your-module
```

**Monorepo with subdirectory** — multiple modules in one repo:

```
mono-repo/
├── modules/
│   ├── security/
│   │   ├── module.yaml
│   │   ├── skills/
│   │   └── pipelines/
│   └── data-eng/
│       ├── module.yaml
│       ├── skills/
│       └── pipelines/
```

```bash
popsicle module install github:your-org/mono-repo#main//modules/security
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `popsicle module list` | List installed modules (marks the active one with `*`) |
| `popsicle module show [<name>]` | Show module details: metadata, skills, pipelines, source |
| `popsicle module install <source>` | Install a module (local path or `github:user/repo[#ref][//subdir]`) |
| `popsicle module upgrade [--force]` | Upgrade the active module (builtin: from binary; remote: re-fetch) |

All commands support `--format json` for machine consumption.

## Design Rules

### 1. Every Pipeline must be self-contained

All Skills referenced by a Pipeline must exist within the same Module. Cross-module Pipeline composition is not supported by design — it avoids dependency resolution complexity.

### 2. Use overrides instead of forking

If you only need to change one Skill, use the `.popsicle/skills/` override mechanism instead of creating a new Module. Forking is for when you need a fundamentally different workflow.

### 3. Version your module

Use semver in `module.yaml`. The `upgrade` command uses version comparison to decide whether to overwrite. Consistent versioning enables predictable upgrades.

### 4. Keep module.yaml minimal

`module.yaml` is metadata only — name, version, description, author. Do not add configuration knobs here; project-level configuration belongs in `config.toml`.

### 5. Test with local install first

Before pushing to Git, always validate locally:

```bash
popsicle module install /path/to/my-module
popsicle skill list     # verify all skills loaded
popsicle pipeline list  # verify all pipelines loaded
popsicle module show    # verify metadata
```

## Checklist

Before publishing a Module:

```
□ module.yaml
  □ name          Unique, lowercase with hyphens
  □ version       Valid semver string
  □ description   One-line summary

□ skills/
  □ Each Skill    Has skill.yaml + guide.md + templates/
  □ Skill names   Unique within the module, lowercase with hyphens

□ pipelines/
  □ Each Pipeline Only references Skills in this module's skills/
  □ depends_on    Aligns with Skill inputs (see Skill Design Guide)

□ Validation
  □ Local install Works: popsicle module install /path/to/module
  □ skill list    Shows all expected Skills
  □ pipeline list Shows all expected Pipelines
  □ module show   Displays correct metadata
```
