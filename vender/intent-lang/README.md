# intent-lang

A **requirements modeling DSL** with formal verification.
Translate business intents into machine-readable, machine-verifiable logic — and let Z3 prove they're free of contradictions.

> **Positioning** — intent-lang is **not** a program specification language (unlike Dafny), **not** a contract language for API calls, and **not** a system modeling language (unlike TLA+). It models **requirements & invariants**, then verifies they don't contradict each other.
> Read [POSITIONING.md](docs/lang/POSITIONING.md) for the full boundary declaration.

```intent
// A business safety rule (requirement-first style)
safety NeverNegativeBalance {
  forall a: Account, a.balance >= 0
}

// An intent that must respect the safety rule
intent TransferSafe(sender: Account, receiver: Account, amount: Int) {
  require amount > 0
  require sender.balance >= amount
  invariant sender.balance' >= 0
  ensure sender.balance' == sender.balance - amount
  ensure receiver.balance' == receiver.balance + amount
}

// A theorem about the requirement set as a whole
theorem TransferPreservesTotal {
  forall s: Account, r: Account, a: Int,
    TransferSafe(s, r, a) ==>
      s.balance' + r.balance' == s.balance + r.balance
}
```

```bash
$ intent check transfer.intent
  ✅ safety  NeverNegativeBalance         — verified
  ✅ intent  TransferSafe                  — verified
  ✅ theorem TransferPreservesTotal        — verified
```

The output above is what matters: **your stated requirements are mutually consistent**. Implementation is somebody else's job (LLM, human engineer, popsicle Skill).

---

## What intent-lang is for

- **Capture requirements as logic** — translate PRDs / domain rules / safety policies into formal intents
- **Detect contradictions early** — Z3 finds paradoxes before they become production bugs
- **Anchor the IDD pipeline** — downstream tools (test-spec generators, contract validators, code synthesizers) all consume intents as the source of truth
- **Survive LLM hallucination** — every LLM-generated intent must pass SMT verification; nothing escapes the gate

## What intent-lang is **not** for

- ❌ Generating implementation code (that's the job of code-gen Skills, not intent-lang)
- ❌ Verifying that a specific algorithm is correct (use Dafny / Why3)
- ❌ Modeling distributed protocols / state machines (use TLA+ / Alloy)
- ❌ Replacing API schemas, type systems, or unit tests

See [POSITIONING.md](docs/lang/POSITIONING.md) for the design discipline that flows from this positioning.

---

## Key Features

- **Declarative & verifiable** — write `safety`/`invariant`/`require`/`ensure`/`theorem`, get Z3 verdicts
- **LLM-assisted, never LLM-trusted** — natural language drafts intents; SMT decides whether they hold
- **Consistency first, completeness next** — the language guarantees no contradictions today; coverage tooling is on the roadmap
- **Domain plugins** — core language stays fixed; domains (smarthome, billing, network…) extend via plugins
- **Multi-level refinement** — business goals → domain invariants → API-shaped intents, each level verified separately and against each other

---

## Documentation

### 📘 Language

| Document | What you'll learn |
|----------|-------------------|
| [定位声明](docs/lang/POSITIONING.md) | intent-lang **是什么 / 不是什么** —— 所有设计决策的最高准绳 |
| [5 分钟概览](docs/lang/README.md) | intent-lang 核心概念速查 |
| [语法规范](docs/lang/SPEC.md) | 完整语法 EBNF、表达式优先级、SMT 编码 |
| [设计决策](docs/lang/DECISIONS.md) | 为什么选混合方式、SMT 验证、Rust |
| [与大模型的关系](docs/lang/LLM.md) | 为什么是最 LLM-friendly 的形式化语言 |

### 🏗️ Architecture

| Document | What you'll learn |
|----------|-------------------|
| [插件系统](docs/architecture/PLUGINS.md) | 4 层插件结构、5 个领域示例 |
| [执行架构](docs/architecture/EXECUTION.md) | 意图→规划→执行→验证的 4 层桥接 |

### 🎯 Use Cases

| Document | What you'll learn |
|----------|-------------------|
| [软件开发](docs/software/README.md) | PRD→意图→验证→生成测试/断言/API 契约 |
| [智能家居](docs/smarthome/README.md) | 安全验证、冲突检测、可解释性、平台对比 |

### 📂 Examples

| File | Description |
|------|-------------|
| **[requirements/billing.intent](examples/requirements/billing.intent)** | **Pure requirements style — goals & domain invariants, no procedural detail** |
| **[requirements/access-control.intent](examples/requirements/access-control.intent)** | **Authorization rules as business invariants** |
| [transfer.intent](examples/basics/transfer.intent) | Bank transfer with bug detection (mixed style) |
| [auth.intent](examples/basics/auth.intent) | Login lockout & access control |
| [sorting.intent](examples/basics/sorting.intent) | Sort specification |
| [smarthome.intent](examples/smarthome/smarthome.intent) | Voice control with safety rules |
| [comparison/](examples/comparison/) | Side-by-side with Lean 4 & TLA+ |
| [CLI usage](examples/USAGE.md) | Full command-line walkthrough |

---

## Implementation

| | |
|---|---|
| **Language** | Rust |
| **Parser** | `logos` + recursive descent |
| **Verification** | Z3 via SMT-LIB2 |
| **LLM** | OpenAI / Anthropic API |
| **Roadmap** | [PLAN.md](PLAN.md) |

## License

TBD
