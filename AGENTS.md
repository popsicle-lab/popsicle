# Popsicle — Agent Instructions

This project uses Popsicle for spec-driven development orchestration.

> **Scope note (PROJ-17)**: this document describes the **self-host MVP command
> surface** actually implemented by `crates/cli-ux`. Legacy commands that are
> not listed here do not exist in the binary — see §Deferred & Removed Commands.

## Binary Resolution

When running `popsicle` commands, resolve the binary in this order:

1. `./popsicle` in the project root, if it exists
2. `./target/debug/popsicle` (the self-host build), if it exists
3. `popsicle` on the system PATH

```bash
# Linux / macOS
if [ -x "./popsicle" ]; then POPSICLE=./popsicle;
elif [ -x "./target/debug/popsicle" ]; then POPSICLE=./target/debug/popsicle;
else POPSICLE=popsicle; fi
```

Then use `$POPSICLE` in place of `popsicle` for all commands. If no binary is
found, build it first: `cargo build -p cli-ux`.

Run `popsicle doctor --format json` before starting work. It must report
`current_workspace_binary_match: true`; otherwise rebuild with
`cargo build -p cli-ux` and use `./target/debug/popsicle`.

## DevOps Entry Points (ADR-014)

- `make check` — fmt + clippy + test, all `-Dwarnings` (CI runs the same trio)
- `make golden` — full golden-baseline chain; `make intent` — Z3 intent validation
- `make install-hooks` — install the pre-commit hook (fmt/clippy/test)
- `scripts/install.sh [--prefix <dir>] [--uninstall]` — install the CLI (no UI, no completions — deferred)
- Releases: push a `v*` tag → `.github/workflows/release.yml` builds 4 targets

## Global Flags

Every command accepts:

- `--format json` — machine-readable output; errors also JSON with actionable `next`
- `--project <path>` — target a specific `.popsicle/` workspace (overrides default)
- `POPSICLE_PROJECT` env var — same as `--project`
- `POPSICLE_HOME` — override `~/.popsicle/` global config directory

Workspace resolution: `--project` → `POPSICLE_PROJECT` → `global.json` default → cwd walk.

## ⛔ MANDATORY: Before Starting ANY Development Task

You MUST follow this checklist before writing ANY code or making ANY changes.
No exceptions — not for "small" fixes, not for low-level modules, not for "just one line".

### Step 1: Check for an active pipeline run

```bash
popsicle issue list --format json
```

Issues with an active run show it via `popsicle issue show <key> --format json`
(`active_run_id` field). If your task matches an issue with an active run →
skip to **Step 4**.

### Step 2: If NO active run — find or create an Issue first

An Issue is REQUIRED before any pipeline run can start. `issue start` is the
ONLY way to create a pipeline run.

**2a.** Check `popsicle issue list --format json` for an existing matching issue.

**2b.** If none exists, create one. `--product` is required (maps to
`products/<name>/`; check recent issues for product ids in use).

**2b-i. MANDATORY — run `issue-author` before `issue create`**

Read `intent-coder/skills/issue-author/guide.md`（Issue 创建唯一入口，含 pipeline
决策树与门禁）。按 guide 填 `issue-create-report` 或同等检查项：

1. Scan `products/<product>/intents/acceptance.intent` — does this capability
   already have a block?
2. Apply the pipeline decision tree (below) — **do not** default to
   `slice-delivery` for new UI/CLI features.
3. Set `--tasks` for **existing** tasks this run implements; use
   `--proposed-task "title|journey"` for **new** user journeys (then
   `slice-spec`, not `slice-delivery`).
4. Put **every** linked `task_id` in `--description` (CLI enforces this on
   `issue start` for `slice-delivery`).

```bash
popsicle issue create --type <product|technical|bug|idea> \
  --title "<concise title>" --product <product-id> --pipeline <name> \
  --tasks T-XXXX [--proposed-task "新旅程|daily-ops"] \
  --description "<what and why; must cite each --tasks id for slice-delivery>" \
  --format json
```

The bundled pipeline templates are:

| Pipeline | Use for | Default for `--type` |
|---|---|---|
| `migration-bootstrap` | first-time migration bootstrap | — |
| `migration-slice-spec` | migration slice spec chain | — |
| `migration-slice-delivery` | implement → equivalence → cutover → living-docs | — |
| `product-greenfield-spec` | new product/module with no legacy code | `product` |
| `feature-spec` | incremental capability spec (no legacy facts) | — |
| `feature-arch-spec` | large incremental spec on existing product (PDR+ADR+task+intent) | — |
| `feature-delivery` | spec-ready feature implement + verify | — |
| `doc-retro-spec` | backfill PDR/task/intent after code merged | — |
| `doc-sync-weekly` | periodic tasks-index + PROJECT_CONTEXT §现在状态 | — |
| `arch-decision` | architecture decision (arch-debate → rfc → adr) | `technical`, `idea` |
| `fix-regression` | minimal fix loop (implement → verify) | `bug` |
| `platform-refactor` | internal refactor / infra | — |

Deprecated aliases (`slice-delivery`, `bugfix`, etc.) still resolve — see ADR-029.

If `--pipeline` is omitted, the issue type's default (last column) is used
(ADR-012 / ADR-029). Pass `--pipeline` explicitly when the default doesn't fit. Missing
templates self-heal: bundled definitions are installed on demand, and a
"pipeline not found" error lists all available templates.

**Pipeline routing（`issue-author` 内嵌，下表仅速查）：**

| Situation | Pipeline |
|---|---|
| New product module, no spec | `product-greenfield-spec` (`--type product` default) |
| Migration slice, capability not in intent yet | `migration-slice-spec` |
| Daily feature, capability not in intent yet | `feature-spec` |
| Large incremental on existing product (needs ADR) | `feature-arch-spec` |
| Spec decided, migration cutover | `migration-slice-delivery` |
| Spec decided, daily feature | `feature-delivery` |
| Architecture decision only | `arch-decision` (`--type technical` default) |
| Regression fix | `fix-regression` |
| Retro doc backfill | `doc-retro-spec` |

`migration-slice-delivery` / `feature-delivery` are **not** substitutes for spec work.

**CLI hard gate (`issue create`):** rejects `migration-slice-delivery` + `--proposed-task` together.
Rejects **`fix-regression` misuse** (`fix-regression-gate:*`) when `--type product` + `fix-regression`, or when
title/description indicates intent file edits, intent-coder skill-chain work, or new UI
capabilities (not single-point fixes). See `intent-coder/skills/issue-author/guide.md`.

**CLI hard gate (`issue start`):** `migration-slice-delivery` is rejected when the issue has
`proposed` task links, no `linked` tasks, `description` omits a linked `task_id`, or
linked tasks lack resolvable `related_intents` in `products/<product>/intents/`.

**intent-coder module (ADR-017):** compiled into the `popsicle` binary (`include_dir!`).
`popsicle init` extracts it to `.popsicle/modules/intent-coder/`. In the popsicle
monorepo, workspace-root `intent-coder/` overrides the embedded snapshot. Refresh:
`popsicle admin sync-intent-coder`. Legacy `popsicle module add` remains **deferred**.
`doctor --format json` reports `intent_coder_module` + `intent_coder_bundle`
(`embedded` | `workspace_root_override`). DMG ships only the CLI — module is inside the binary.

Show the created issue key to the user before proceeding.

### Step 3: Start the Issue (creates a pipeline run)

```bash
popsicle issue start <ISSUE-KEY> --format json
```

This creates the pipeline run, acquires the Spec lock, and returns `run_id`.
One active run per issue; complete or cancel before starting another.

### Step 4: Follow the pipeline

```bash
popsicle pipeline next --run <run_id> --format json
```

Execute the suggested action. NEVER skip pipeline steps or write code outside
of a pipeline run.

## Command Reference (complete)

This is the full implemented surface. Anything not listed here is deferred or
removed (see below).

### Setup & Diagnostics

- `popsicle init` — bootstrap `.popsicle/` workspace (installs bundled pipelines)
- `popsicle doctor [--format json]` — binary/workspace provenance check
- `popsicle help` — top-level commands + full usage lines
- `popsicle ui [--project <path>]` — Tauri 2 desktop UI（需 `cargo build --features ui -p cli-ux`；见 ADR-015）

### Project（全局多项目，macOS DMG / `~/.local/bin` 安装）

- `popsicle project list` — 已注册项目 + 默认项
- `popsicle project add <path> [--name <n>]` — 注册已有 `.popsicle/` 工作区
- `popsicle project use <name|path>` — 设置默认项目
- `popsicle project remove <name>` — 从注册表移除
- `popsicle project current` — 当前解析到的工作区与来源

全局标志（任意命令）：`--project <path>`；环境变量 `POPSICLE_PROJECT`。注册表 `~/.popsicle/global.json`。

### Issue

- `popsicle issue create --type <t> --title "<t>" --spec <spec-id> [--pipeline <name>] [--priority <p>] [--description "<d>"] [--tasks T1,T2] [--proposed-task "title\|journey"]`（`--epic-task` 已废弃）
- `popsicle issue list`
- `popsicle issue show <key>`
- `popsicle issue start <key> [--spec <spec-id>] [--pipeline <name>]`
- `popsicle issue close <key>` — close after the run completes (fails actionably while a run is active)
- `popsicle issue link <key> --tasks T1,T2 [--replace] [--drop-proposed]` — add/replace linked tasks after create (proposed→linked promotion)

### Pipeline

- `popsicle pipeline status --run <run_id>` — stage list with statuses
- `popsicle pipeline next --run <run_id>` — what to do next
- `popsicle pipeline stage complete <stage> --run <run_id> [--confirm]` — complete a stage; `--confirm` required for stages with `requires_approval`

### Document

- `popsicle doc create <skill> --title "<t>" --run <run_id>` — create a stage artifact under `.popsicle/artifacts/<run_id>/`; `<skill>` is an intent-coder skill name (see Skill Catalog)
- `popsicle doc list [--run <run_id>]`
- `popsicle doc show <doc_id>`
- `popsicle doc check <doc_id>` — validate frontmatter, filled body, placeholders (`[TBD`, `{{`), checkbox counts; exits 1 with `status: failed` until the document has real content. Run it after filling every stage document.

### Tool & Admin

- `popsicle tool run intent-validate path=<dir> [format=<text|json>]` — Z3 intent check
- `popsicle tool run telemetry action=<guide|record|flush|status|report> run=<run_id> [span=…] [format=<text|json>]` — Agent 观测旁路（fail-open）；`action=guide` 打印 `intent-coder/tools/telemetry/guide.md`
- `popsicle tool run mermaid-diagram action=<guide|scaffold|validate> [type=…] [path=…] [title=…] [format=<text|json>]` — Mermaid 画图技能（PRD/task/RFC/ADR）；`action=guide` 打印 `intent-coder/tools/mermaid-diagram/guide.md`
- `popsicle admin migrate [--workspace <path>]` — migrate legacy TSV state to the SQLite backend (`.popsicle/state.db`); idempotent, keeps `state.tsv.migrated` for rollback
- `popsicle admin relocate-workspace [--dry-run] [--workspace <path>]` — lift `.popsicle/self-host/` to flat layout (ADR-032)
- `popsicle admin reinit [--workspace <path>]`
- `popsicle admin backfill-pipeline-names [--dry-run] [--workspace <path>]` — ADR-029：canonicalize issues/runs pipeline 名；删除废弃 pipeline YAML
- `popsicle admin purge-legacy-workspace [--dry-run] [--workspace <path>]` — ADR-031：删除 `.popsicle/popsicle.db` 等 legacy workspace 文件

Storage (ADR-013, ADR-032): fresh workspaces use SQLite at `.popsicle/state.db`;
legacy TSV is import-only (`admin migrate` or open-time auto-import). `doctor` reports the
active backend in `storage_backend`. Do NOT use `.popsicle/popsicle.db` — legacy binary only.

## Deferred & Removed Commands

**Deferred from the self-host MVP** (fail with a `deferred` error; do not use):
`module`, `skill`, `spec`, `namespace`, `prompt`, `git`, `memory`, `context`,
`registry`, `completions`.

**Removed permanently** (PDR-001): `checklist`, `item`, `sync`.

Replacement practices until these are re-adjudicated:

| Missing capability | Do this instead |
|---|---|
| `memory save/list` | record decisions as PDR/ADR files under `products/<product>/decisions/`, gotchas in `docs/PROJECT_CONTEXT.md` |
| `context search` | use `rg` over `products/`, `docs/`, `.popsicle/artifacts/` |
| `doc summarize` | write a `## Summary` section directly in the artifact document |
| `git link` | reference the run id and doc ids in the commit message body |
| `checklist check` | `popsicle doc check <doc_id>` validates content/checkboxes; check items off by editing `- [ ]` → `- [x]` in the document |
| `pipeline verify` | `pipeline status --run <run_id> --format json` must show every stage `completed` and `run_status: completed`; then `issue close <key>` |
| spec/namespace creation | specs are plain identifiers recorded on issues; reuse existing spec ids or introduce a new one in the issue and document it in `migration/traceability.md` |

## Workflow Rules

1. **NEVER write code without an active pipeline run** — no exceptions
2. **Issue → `issue start` → pipeline run** — `issue start` is the ONLY way to create pipeline runs and acquires an exclusive Spec lock
3. Always check `popsicle pipeline next --run <run_id>` before starting work on a step
4. Fill document sections with real content — template placeholders are rejected
5. **Stage completion** — follow `workflow.approval_mode` in `.popsicle/project.yaml` (also in the project-config marker below): `manual` (default) — STOP after each stage and wait for the user before `pipeline stage complete`; `auto` — after `doc check` passes you may complete stages without waiting (`--confirm` implied for `requires_approval`); `delegate-dangerous` — auto-complete non-dangerous `requires_approval` stages, but dangerous stages (`cutover`, `living-docs`) still need explicit human `--confirm`.
6. Stages marked `requires_approval` — apply the approval mode above; in `manual` mode the user MUST run `--confirm` themselves after review.
7. **Spec lock**: one active run per issue; do not operate on a spec locked by another run
8. Documents live under `.popsicle/artifacts/<run_id>/`; decision records are promoted into `products/<product>/decisions/` at their stage's completion
9. **NEVER report a task as "complete" unless `pipeline status` shows all stages completed.** If stages remain, say which stages are remaining and what the next step is. Reporting completion prematurely is a critical error.
10. Run `popsicle tool run intent-validate path=products` before completing implementation/cutover stages when intents changed
11. Run `popsicle doc check <doc_id>` on every stage document after filling it; complete the run, then `popsicle issue close <key>` to close the loop
12. **Agent 观测（telemetry，fail-open）**：不确定用法时先 `popsicle tool run telemetry action=guide`；每个 pipeline stage 内至少上报一次 `gen_ai.chat`；`doc check` 通过后可上报 `popsicle.run.score`。编排 span 已自动写入，无需重复 `issue start` / `stage complete` 类事件。

---

## Module: intent-coder

The following skills are provided by the `intent-coder` module. Use these
names with `popsicle doc create <skill>`.

### Skill Catalog

| Skill | Artifact | Inputs | States |
|-------|----------|--------|--------|
| `issue-author` | issue-create-report | none (standalone; **not** in pipeline yaml) | analyzing → drafting → completed |
| `adr-writer` | adr-finalization-report | rfc-writer, arch-debate | review → completed → reviewing → finalizing |
| `arch-debate` | arch-debate-record | prd-writer, fact-extractor, product-debate | debating → setup → completed → concluding |
| `cutover-author` | cutover-adr | equivalence-baseline, intent-consistency-check, shadow-implementer | drafting → completed → review → gating |
| `equivalence-baseline` | equivalence-report | shadow-implementer, fact-extractor | completed → inventory → running → reporting → review |
| `fact-extractor` | fact-extraction-report | none | completed → drafting → scanning → review |
| `intent-consistency-check` | intent-consistency-report | intent-spec-writer | review → completed → checking → reporting |
| `intent-spec-writer` | formal-acceptance-intent | prd-writer | completed → ingesting → tightening → verifying → review |
| `living-doc-author` | living-doc-sync-report | intent-consistency-check, prd-writer, shadow-implementer, equivalence-baseline, cutover-author | syncing → review → reporting → completed → scanning |
| `prd-writer` | prd-overview | product-debate, fact-extractor, project-init | scoring → ingesting → drafting → completed → review |
| `product-debate` | product-debate-record | fact-extractor, project-init | setup → debating → completed → concluding |
| `project-init` | project-init-plan | fact-extractor | planning → surveying → scaffolding → completed |
| `rfc-writer` | rfc | arch-debate, prd-writer, fact-extractor | review → completed → ingesting → scoring → drafting |
| `shadow-implementer` | implementation-coverage | adr-writer, rfc-writer, intent-consistency-check | review → completed → verifying → implementing → scoping |
<!-- popsicle:project-config:start -->
## 本项目偏好

- **界面 / Agent 语言**：简体中文
- **产品文档目录**：`products/`
- **决策记录**：`products/<product>/decisions/{adr,pdr}/`
- **Pipeline 审批模式**：`delegate-dangerous`（危险操作需审批（其余代批））
- **Issue / 文档文案**：创建或更新 Issue / 文档时，`--title` 与 `--description` 使用简体中文（除非用户明确要求英文）。

### 阶段完成策略

非危险 `requires_approval` 阶段可由 agent 代批完成；危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。
<!-- popsicle:project-config:end -->
