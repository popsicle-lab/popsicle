# intent-lang grammar & GitHub Linguist support

This directory holds the TextMate grammar for intent-lang and the
roadmap for getting `.intent` files highlighted on github.com.

## Three-level rollout plan

| Level | Coverage | Effort | Status |
|-------|----------|--------|--------|
| 1. `.gitattributes` borrow | github.com (this repo only) | 1 file | ✅ shipped |
| 2. VSCode/Cursor extension | local editors everywhere | this dir + extension wrapper | ✅ grammar ready |
| 3. github-linguist upstream | github.com (every repo) | linguist PR + 200+ usage repos | ⏳ pending adoption |

---

## Level 1 — `.gitattributes` borrow (already live)

`/.gitattributes` declares:

```
*.intent linguist-language=Scala
*.intent linguist-detectable=true
```

Effect: github.com renders `.intent` files using Scala's highlighter,
which is the closest visual match (curly-brace blocks, `name: Type`
ascriptions, infix operators, `forall`/`exists`).

**Limitations**: Scala highlights `intent`/`safety`/`goal`/`@asis` as
plain identifiers — they won't pop visually. But blocks, types,
strings, numbers and comments all render correctly. This is the
zero-effort 80% solution.

---

## Level 2 — VSCode / Cursor extension (grammar ready)

`intent.tmLanguage.json` is a complete TextMate grammar covering:

- Comments (`//` line, `/* */` block)
- Annotations (`@asis`, `@tobe`, `@deprecated`, generic `@foo`)
- Declarations (`intent`, `safety`, `theorem`, `axiom`, `function`,
  `type`, `enum`, `goal`, `coverage`, `import`)
- Control keywords (`require`, `ensure`, `invariant`, `forall`,
  `exists`, `if`/`then`/`else`, `after`, `as`)
- Goal sub-keywords (`rationale`, `stakeholder`, `measure`,
  `realized_by`, `dimensions`)
- Operators (`==>`, `<=>`, `&&`, `||`, `!`, comparisons, arithmetic,
  primed `x'`)
- Built-in types (`Int`, `Bool`, `String`, `Real`, `Seq`, `Set`, `Map`)
- String/numeric literals
- PascalCase → type, lowercase → variable

### Wrapping it as a VSCode extension

Minimal `package.json` (place outside this repo or in a sibling
`vscode-intent-lang/` folder):

```json
{
  "name": "vscode-intent-lang",
  "displayName": "intent-lang",
  "description": "Syntax highlighting for intent-lang (.intent) files",
  "version": "0.1.0",
  "publisher": "intent-lang",
  "engines": { "vscode": "^1.80.0" },
  "categories": ["Programming Languages"],
  "contributes": {
    "languages": [{
      "id": "intent",
      "aliases": ["intent-lang", "Intent"],
      "extensions": [".intent"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "intent",
      "scopeName": "source.intent",
      "path": "./syntaxes/intent.tmLanguage.json"
    }]
  }
}
```

Add `language-configuration.json`:

```json
{
  "comments": { "lineComment": "//", "blockComment": ["/*", "*/"] },
  "brackets": [["{","}"], ["[","]"], ["(",")"]],
  "autoClosingPairs": [
    {"open":"{","close":"}"},
    {"open":"[","close":"]"},
    {"open":"(","close":")"},
    {"open":"\"","close":"\""}
  ]
}
```

Then `vsce package && vsce publish` (or just sideload locally with
`code --install-extension intent-lang-0.1.0.vsix`).

---

## Level 3 — github-linguist upstream (when ready)

GitHub's syntax highlighting is governed by
<https://github.com/github-linguist/linguist>. To add intent-lang:

### Step 1 — Vendor the grammar

Linguist consumes TextMate grammars via git submodule under
`vendor/grammars/`. Two paths:

- **Recommended**: publish this grammar in a standalone repo
  `intent-lang/intent-lang.tmbundle` (or `intent-lang/vscode-intent-lang`
  with the grammar at `syntaxes/intent.tmLanguage.json`), then add it
  as a submodule in linguist.
- **Alternative**: contribute the grammar to an existing community
  bundle.

### Step 2 — Languages.yml entry

Add to `lib/linguist/languages.yml`, alphabetically:

```yaml
Intent:
  type: programming
  color: "#7B61FF"
  extensions:
    - ".intent"
  tm_scope: source.intent
  ace_mode: text
  language_id: 9000XXX   # assigned by linguist maintainers
```

(Pick a `language_id` not in use; maintainers will adjust.)

### Step 3 — Sample files

Add 1–2 representative `.intent` files under `samples/Intent/`.
Linguist's classifier uses these for ambiguity resolution.

### Step 4 — Submit PR

linguist's [CONTRIBUTING.md](https://github.com/github-linguist/linguist/blob/main/CONTRIBUTING.md)
states:

> The language has been in use for **at least 6 months**.
> The language has hundreds of repositories and unique users using it.

This is the hard gate. Strategy:

1. Ship Level 1 + 2 first.
2. Encourage adopters to publish their `.intent` files publicly.
3. Once GitHub code search shows ≥200 public repos with `.intent`
   files, open the PR with that evidence.

Until then, keep Level 1's `.gitattributes` borrow as the production
solution for this repo and any consumer who wants real highlighting
on github.com.

---

## Quick local test

```bash
# Validate JSON
python3 -c "import json; json.load(open('tools/grammar/intent.tmLanguage.json'))"

# Convert to .plist if a tool needs it (some Linguist tooling prefers plist):
npx js-yaml tools/grammar/intent.tmLanguage.json   # sanity-roundtrip
```
