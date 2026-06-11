---
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
