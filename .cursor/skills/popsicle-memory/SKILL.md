---
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
