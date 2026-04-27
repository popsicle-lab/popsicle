# Product Roadmap & Open Optimisation Backlog

> **Audience**: maintainers + commercial sponsors of Popsicle / popsicle-cloud.
> **Purpose**: a single place to record the product-shaped opportunities we
> have *not* shipped yet, prioritised so we can pick the next quarter's bets
> without re-deriving the rationale every time.

This file is a living backlog. Items live here until a maintainer picks one
up and turns it into an RFC under `docs/rfcs/` (or a private design doc
under `popsicle-cloud/docs/`).

---

## Status legend

| Tag | Meaning |
|-----|---------|
| ✅ Shipped | The item is implemented in `main`. |
| 🚧 In progress | An RFC or PR exists. |
| 🟢 Picked | Next quarter; owner assigned. |
| 🟡 Queued | Agreed valuable, not yet scheduled. |
| 🔵 Idea | Captured for debate; may be dropped. |

---

## Recently shipped (delta vs. the audit on 2026-04-27)

- ✅ **Web Issue Kanban** — `/issues` route on popsicle-cloud groups issues by
  `IssueStatus` columns with live updates from the WS feed.
- ✅ **Pipeline run timeline** — `/runs` and `/runs/:id` render stage states
  with colour-coded status pills and the documents attached to each run.
- ✅ **README hero rewrite** — clarifies the user/agent/CLI/UI roles in
  ~150 words and gives a 30-second demo block.

The remaining items below are still open.

---

## P0 — Value-proposition gaps

### CLI: MCP server mode

**Problem.** Today AI agents discover Popsicle by reading `CLAUDE.md` /
`AGENTS.md` and inferring how to call the CLI. This is brittle: every model
upgrade or prompt revision risks regressions.

**Proposal.** Ship `popsicle mcp serve` that exposes Popsicle as a
[Model Context Protocol](https://modelcontextprotocol.io/) server. Tools
include:

- `skill.list`, `skill.show`, `skill.invoke`
- `pipeline.next`, `pipeline.advance`
- `spec.search`, `spec.read`
- `memory.recall`, `memory.write`
- `context.for_stage`

Models then call structured tools instead of guessing CLI syntax. This is
the single highest-leverage change for the agent-integration story.

**Acceptance.** Claude Code, Cursor, and OpenCode can all complete
`issue start → pipeline next → doc submit` without any natural-language
preamble in their system prompt — only the MCP tool descriptions.

---

### Cloud: choose one v1 paid wedge

popsicle-cloud sync works, but the "why pay?" story is unfocused. Pick **one**
of the three below for v1 and write the spec for it.

1. **Team workspace.** Convert `entities.user_id` to `owner_id` (user *or*
   team), add `teams`, `team_memberships`, `invites` tables, role-based
   ACL on `/v1/changes`. Web gets `/team` settings page.
2. **Pipeline-as-a-service.** Run pipelines server-side; agents stream
   progress over WS. Hooks into existing CRDT doc channel for
   collaborative editing of the pipeline output.
3. **Centralised AI key + usage dashboard.** Cloud holds the LLM API key,
   client-side agent calls a Popsicle proxy, server records token usage and
   cost per spec/skill/run. Surfaces in `/usage` page with weekly limits.

Recommendation: **Team workspace** first (#1) — straightforward to build,
clearest payment trigger, foundation for #2 and #3.

---

### Cloud: web is currently a viewer

Even after the Issue Kanban + Run Timeline work, the web app still does not
let you *do* anything beyond browsing. The next set of write features:

- Conflict resolver UI (reads `.sync/conflicts.log`, surfaces via a new
  `/v1/conflicts` endpoint, lets the user choose local vs. server per
  field).
- Issue triage: drag-drop between Kanban columns calls a new
  `/v1/issues/:id/status` write endpoint and produces a regular sync change.
- Discussion viewer: render multi-agent debates as chat threads (this is
  one of Popsicle's actual differentiators and isn't visible anywhere in
  the cloud UI yet).

---

## P1 — Experience cliffs

### Onboarding wizard

`popsicle init` succeeds silently and the user has no idea what to do next.
Replace with an interactive wizard (default on TTY, `--no-interactive` for
CI). Bundle a `hello-world` module so a fresh install can run a full
`issue start → pipeline complete` round-trip in under 60 seconds.

Add `popsicle doctor`: checks config, login, module integrity, model
availability, common Git mistakes.

### Skill / pipeline discoverability

CLI entry points for the registry beyond raw `module install`:

- `popsicle skill search <query>`
- `popsicle skill show <name>` (renders guide.md + skill.yaml inline)
- `popsicle pipeline graph <name>` (ASCII DAG)

Web side: `/marketplace` page mirroring the registry index with stars,
descriptions, and screenshots.

### Conflict resolution UX (active, not passive)

M8 logs conflicts but does not surface them actively. We need:

- WS event `type=conflict` pushed to all subscribed clients.
- Web `/conflicts` page with diff viewer.
- CLI `popsicle sync resolve <id> --use=local|server`.

Without an active flow, the audit trail is functionally invisible.

---

## P2 — Functional depth

### Document time-travel

The CRDT log already retains every update; `popsicle doc history <id>` and
`popsicle doc revert <id> --to=<v>` are essentially free to implement and
become an immediate Notion/Obsidian differentiator. Web should render diffs
between any two versions side-by-side.

### Multi-device presence

Use the existing `client_id` per push to maintain a presence channel over
WS. Web shows "🟢 2 clients online (mac-laptop, server-prod)". Helps
explain why a doc just changed and reinforces the "yes, sync is real-time"
brand promise.

### Pipeline observability

Agents should self-report stage start/end + token usage. Server stores it,
web renders a flame-graph view at `/runs/:id/trace`. Free tier: last 7 days;
paid: full history. Doubles as the pricing differentiator and the answer to
"my agent ran for 20 minutes — what was it doing?".

### Mobile / PWA

Read-only mobile dashboard so users can check pipeline status from a phone.
Push notifications via FCM/APNs when a pipeline completes or a conflict
fires. Bundle should target <50KB gzip on mobile (consider islands).

### Team / sharing primitives

Once Team workspace lands (P0), add invite links with expiring tokens,
role matrix (owner / editor / viewer), audit-log filtering by user,
spec-level ACLs.

---

## P3 — Engineering & brand polish

### Web hygiene

- Delete the duplicated `*.js` files alongside `*.tsx` under
  `popsicle-cloud/web/src/pages/` (probably stale build output).
- Bundle is 247KB (78KB gzip) for what is presently five pages — move the
  router lazy, split the docviewer / settings out of the initial chunk.
- Add favicon, OG tags, system dark-mode follow, branded 404/500 pages.

### Docs split

`popsicle/docs/` mixes external API spec (`sync-api.md`) with internal RFCs.
Restructure into `docs/api/` (versioned, OpenAPI-style, externally stable)
and `docs/rfcs/` (internal evolution). Publish `sync-api.md` as a versioned
OpenAPI 3.1 document so third parties can generate clients.

### Billing & quotas (commercial readiness)

popsicle-cloud has no billing infra. Before flipping a paywall on:

- Stripe integration (`/v1/billing/portal`, `/v1/billing/webhook`).
- `subscriptions` + `quotas` tables (entities, doc updates, AI tokens
  per month).
- Soft enforcement: warn → throttle → block, never silent failure.

### Privacy & E2E option

- One-time signed-URL responses for `/v1/me/export` instead of an inline
  JSON dump (defeats opportunistic exfil).
- Optional client-side encryption: encrypt CRDT updates with a key the
  server never sees. Trade off: server-side document search and Web doc
  rendering become impossible — make it an explicit account toggle.

### Brand consistency

The "popsicle" + "border collie" metaphors are not reconciled in any
written copy. Pick one (suggest collie — it actually fits the
"watches the AI flock" concept) and use it in landing copy, error
messages, and product names; or commit to popsicle and explain it.

---

## Operational note

When picking up an item:

1. Move it to **🟢 Picked** with a date and an owner handle.
2. Open an RFC in `docs/rfcs/rfc-<slug>.md`.
3. When the RFC merges, link it from the item.
4. When the implementation ships, move the item to the
   **Recently shipped** section at the top with a one-line delta note.

The list itself is intentionally opinionated. Disagreements should land as
PRs editing this file, not as Slack rants that vanish.
