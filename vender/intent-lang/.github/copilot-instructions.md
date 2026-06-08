# Copilot Instructions for intent-lang

## What This Project Is

intent-lang is a **specification language**, not a programming language. Users declare `require`/`ensure`/`invariant` conditions — never implementations. Z3 SMT solver auto-verifies or produces counterexamples. The core philosophy: **if you're writing "how", you're doing it wrong**.

## Project Status

Pre-implementation. All docs/spec/examples exist; no Rust code yet. Implementation follows PLAN.md milestones: M1 (parser) → M2 (type check + SMT) → M3 (CLI) → M4 (LLM) → M5 (WASM playground).

## Planned Build Commands

Once M1 begins, the project will be a Cargo workspace:

```bash
cargo build                          # build all crates
cargo test                           # run all tests
cargo test -p intent-syntax          # test a single crate
cargo test -- test_parse_transfer    # run a single test
cargo run -p intent-cli -- check examples/basics/transfer.intent
```

Key crates: `logos` (lexer), `clap` (CLI), `ariadne` or `miette` (error reporting), `z3` or CLI shelling (SMT), `insta` (snapshot tests).

## Architecture

```
crates/
├── intent-syntax/    # Lexer (logos) + recursive-descent parser → AST
├── intent-core/      # Type checker, VC generator, SMT-LIB2 encoder, Z3 caller
├── intent-cli/       # clap CLI: check, parse, fmt, generate
└── intent-llm/       # NL → intent-lang via LLM API + auto-verify retry loop
```

**Verification pipeline**: Parse → Type check → Generate verification conditions → Encode to SMT-LIB2 → Z3 solves → Report result/counterexample.

**VC encoding** (refutation): `(∧ require) ∧ (∧ invariant) → (∧ ensure) ∧ (∧ invariant')` — assert negation, check-sat; `unsat` = verified.

**Plugin system**: Domain plugins (smarthome, finance, healthcare, etc.) add types, safety rules, axioms, and functions. Safety rules from plugins auto-merge into all intent VCs in scope.

## Language Syntax Conventions

7 core keywords: `type`, `intent`, `require`, `ensure`, `invariant`, `theorem`, `safety`. Plus `axiom`, `function`, `enum`, `import`.

Key syntax rules:
- **Primed variables**: `x'` and `after(x)` are equivalent — both mean "value after execution". `after()` is the LLM-friendly alias; it desugars to prime form in the AST.
- **Quantifiers**: `forall x: T, P(x)` — comma separator, **not** `::`.
- **Implication**: `==>` (right-associative, lowest precedence).
- **Only in ensure/invariant**: primed variables cannot appear in `require`.

When writing or editing `.intent` files, follow the patterns in `examples/basics/` and `examples/smarthome/`.

## Documentation Map

| What you need | Where to look |
|---|---|
| Language syntax & EBNF | `docs/lang/SPEC.md` |
| Design rationale | `docs/lang/DECISIONS.md` |
| LLM integration design | `docs/lang/LLM.md` |
| Plugin architecture | `docs/architecture/PLUGINS.md` |
| Execution architecture | `docs/architecture/EXECUTION.md` |
| Implementation roadmap | `PLAN.md` |

## Commit Conventions

Format: `<category>: <imperative message>`

Categories: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `spec` (for language spec changes).

All documentation is in Chinese (zh-CN). Code comments and commit messages are in English.
