# RFC Writing Guide

## Purpose

An RFC (Request for Comments) lays out a problem and a proposed solution to build consensus among stakeholders before implementation. It is the primary tool for discussing significant technical changes that affect multiple teams, modules, or system guarantees.

## When to Write an RFC

An RFC is warranted when:
- The change significantly affects stakeholders or project contributors
- Major architectural changes or new features are proposed
- Changes could affect guarantees, support level, or security model
- The work is substantial and you want early feedback

An RFC is NOT needed for:
- Bug fixes and minor improvements
- Internal module refactoring that doesn't change interfaces
- Minor dependency version upgrades

## Section Standards

### Summary

One paragraph that answers: "What is this RFC proposing?"

**Good:**
> This RFC proposes migrating user authentication from session-based to JWT-based to support stateless auth in our microservice architecture, reducing Redis session storage operational cost.

**Bad:**
> This RFC discusses authentication improvements.

### Motivation

Must answer: Why? What use cases? What happens if we don't act?

- Clearly describe the problem in the current state
- Quantify the impact with data when possible (e.g., "3 incidents in last 6 months")
- Identify who is affected

**Good:**
> The current session-based auth shares a single Redis cluster. This creates three problems: (1) single point of failure — 3 auth outages in the past 6 months; (2) horizontal scaling difficulty — every new service needs Redis config; (3) cross-domain limitation — mobile apps cannot use cookie-based sessions.

**Bad:**
> Our auth system has some issues that need improvement.

### Proposal

The core design section. Include:
- Architecture overview or flow diagram
- API designs with code examples
- Data model changes (if applicable)
- Key interaction sequences

Focus on the "what" and "why" of the design, not implementation-level "how to code it" details — those belong in code PRs.

### Rationale and Alternatives

Must include at least 2 alternatives with honest trade-off analysis.

For each alternative:
- Describe the approach in 1-2 sentences
- List concrete pros and cons
- Explain why it was not chosen

**Good:**
> **Alternative A: Redis Cluster** — Upgrade to Redis Cluster HA.
> - Pros: Minimal change, team has Redis experience
> - Cons: Only fixes SPOF, doesn't address cross-domain or scaling; increases ops cost

**Bad:**
> We chose JWT because it's the best. No other options considered.

### Open Questions

Split into two categories:
1. Questions to resolve through the RFC process (before acceptance)
2. Questions to resolve during implementation (after acceptance)

If you have zero open questions, you probably haven't thought hard enough.

## Thinking Framework

Before writing, ask yourself:
1. Can I explain the problem in 2 sentences to someone unfamiliar?
2. Have I considered at least 2 alternatives?
3. Am I honest about the downsides of my proposal?
4. Are my open questions real uncertainties, not rhetorical questions?

## Common Mistakes

1. **Solution seeking a problem** — Starting with a technology you want to use, then inventing justification. RFCs should start from real pain points.

2. **Encyclopedia RFC** — Trying to solve every related issue in one RFC. Split into focused RFCs.

3. **Implementation doc disguised as RFC** — Writing step-by-step implementation instead of design rationale. RFC is about "what and why", not "how to code it line by line."

4. **Missing alternatives** — Only presenting the chosen approach without comparison. Reviewers cannot judge if you've been thorough.

5. **Empty open questions** — Writing "none" or "N/A". If there are no uncertainties, you probably don't need an RFC.

## Relationship with ADR

After an RFC is accepted, consider extracting an ADR to preserve the decision record long-term:
- ADR Context ← RFC Summary + Motivation (condensed)
- ADR Decision ← RFC Proposal (final version, key points only)
- ADR Consequences ← RFC review findings (positive/negative/risks)

Cross-reference: add `See RFC: {slug}` in the ADR, and `ADR: {slug}` in the RFC metadata.
