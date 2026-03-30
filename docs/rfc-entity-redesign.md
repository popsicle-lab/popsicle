# RFC: Entity Relationship Redesign

## Summary

Redesign the domain entity hierarchy from a flat, disconnected model to a clear top-down ownership chain:

```
Namespace (optional)
  └── Spec
        └── Issue
              └── PipelineRun
                    └── Document
```

## Motivation

The original model had several structural problems:

1. **Spec ↔ Issue disconnection** — Specs and Issues had no direct relationship; both could independently own Documents, creating confusing dual-ownership
2. **Issue → PipelineRun one-to-one** — An Issue stored a single `pipeline_run_id`, making revision workflows awkward
3. **No Namespace concept** — No way to group related Specs under a higher-level initiative
4. **Spec documents lacked workflow state** — Documents attached to Specs didn't show pipeline stage/status context

## Design

### New Entity Hierarchy

| Entity | Parent | Relationship |
|--------|--------|-------------|
| **Namespace** | — | Top-level grouping (required) |
| **Spec** | Namespace (required) | `spec.namespace_id → namespace.id` |
| **Issue** | Spec | `issue.spec_id → spec.id` (required) |
| **PipelineRun** | Issue + Spec | `pipeline_run.issue_id → issue.id` (created exclusively via `issue start`) |
| **Document** | PipelineRun + Spec | Unchanged — linked via `run_id` and `spec_id` |

### Key Changes

#### Added Fields
- `Namespace` — New entity with `id`, `name`, `slug`, `description`, `status` (active/completed/archived), `tags`
- `Spec.namespace_id: Option<String>` — Optional link to a Namespace
- `Issue.spec_id: String` — Required link to a Spec (every Issue belongs to a Spec)
- `PipelineRun.issue_id: Option<String>` — Optional back-link to the Issue that spawned this run

#### Removed Fields
- `Issue.pipeline_run_id` — Removed; replaced by one-to-many via `PipelineRun.issue_id`

### Migration Strategy

SQLite doesn't support `DROP COLUMN` easily, so the migration:

1. Adds `spec_id TEXT DEFAULT ''` to `issues` table
2. Adds `issue_id TEXT DEFAULT ''` to `pipeline_runs` table
3. Adds `project_id TEXT DEFAULT ''` to `specs` table
4. Creates `projects` table
5. Backfills `issues.spec_id` from existing `pipeline_runs` where `pipeline_runs.id = issues.pipeline_run_id`
6. Backfills `pipeline_runs.issue_id` from the reverse lookup
7. Old `pipeline_run_id` column remains in the schema but is no longer read

## CLI Changes

### New Commands
- `popsicle namespace create|list|show|update|delete` — Full CRUD for Project entity

### Modified Commands
- `popsicle issue create` — Now requires `--spec <name>` to specify the parent Spec
- `popsicle issue list` — Accepts optional `--spec` filter
- `popsicle issue show` — Displays spec name and all associated pipeline runs (not just one)
- `popsicle issue start` — Creates a new PipelineRun with `issue_id` set; supports multiple runs per issue
- `popsicle spec create` — Accepts optional `--project <name>`
- `popsicle spec list` — Accepts optional `--project` filter
- `popsicle spec show` — Now displays associated Issues in addition to runs and documents

## Storage API Changes

### New Methods
- `create_project`, `get_project`, `list_projects`, `update_project`, `delete_project`
- `find_project_by_name` — Lookup by name
- `list_specs_by_project` — Filter specs by project
- `find_runs_by_issue` — Get all PipelineRuns for an Issue

### Modified Methods
- `create_issue` / `update_issue` / `get_issue` — Use `spec_id` instead of `pipeline_run_id`
- `query_issues` — Accepts optional `spec_id` filter parameter
- `find_issue_by_run_id` — Now queries via `pipeline_runs.issue_id` instead of `issues.pipeline_run_id`
- `create_spec` / `get_spec` / `list_specs` — Include `project_id` field

## DTO Changes

- `IssueInfo` / `IssueFull` — `pipeline_run_id` replaced with `spec_id`
- `IssueProgress` — Now contains `spec_id` and `pipeline_runs: Vec<PipelineRunInfo>` (supports multiple runs)
- `SpecInfo` / `SpecDetailInfo` — Added `project_id` field; detail includes `issues: Vec<IssueInfo>`
- `PipelineRunInfo` — Added `issue_id` field
- New: `NamespaceEntityInfo`, `NamespaceEntityDetail` DTOs

## Workflow Enforcement

### Strict Entity Creation Order

The redesign enforces a top-down creation order. Entities cannot be created out of sequence:

```
Namespace → Spec (with tags) → Issue → PipelineRun → Document
```

- **Namespace is required** — Specs cannot exist without a parent Namespace (`spec.namespace_id` is no longer optional)
- **Spec tags drive matching** — `issue create` auto-matches to a Spec by tag overlap; explicit `--spec` overrides
- **`issue start` is the only way to create PipelineRuns** — `pipeline run` and `pipeline quick` commands are removed. All runs originate from an Issue
- **`doc create` is stage-gated** — Documents can only be created when the target stage is unblocked (upstream dependencies satisfied)

### Document Lifecycle (`doc_lifecycle`)

Skills declare how their documents behave across pipeline runs:

| Lifecycle | Behavior | Example Skills |
|-----------|----------|---------------|
| `singleton` (default) | One document per spec per skill. Reused and updated across runs. | PRD, RFC |
| `cumulative` | New document created each run. Each run produces a separate artifact. | ADR, test reports |

`pipeline next` surfaces this information: singleton docs show skip/update options, cumulative docs always create new.

### Removed Commands

- `pipeline run <pipeline> --title <t>` — Replaced by `issue start <key>`
- `pipeline quick --title <t>` — Removed; use an appropriate pipeline via `issue start` instead

---

## Pipeline Lock & Stage-as-State-Source Redesign

### Summary

This follow-up redesign eliminates the dual-state problem where both documents and pipeline stages tracked progression independently. The pipeline stage is now the **single source of truth** for document state, and Specs gain an **exclusive lock** to prevent concurrent pipeline runs from conflicting.

### Motivation

The original entity redesign left documents with their own state machine (`draft → review → approved`) separate from the pipeline stage status. This created:

1. **Dual-state inconsistency** — A stage could be "complete" while its document was still in "draft", or vice versa
2. **Redundant commands** — `doc transition` and `pipeline stage complete` were two paths to express the same intent
3. **Concurrent run conflicts** — Multiple pipeline runs on the same Spec could modify shared singleton documents simultaneously

### Design Changes

#### Stage as Single Source of Truth

- Documents are created as **"active"** when `doc create` is called
- Documents become **"final"** when their owning pipeline stage is completed via `pipeline stage complete`
- Documents no longer have their own state machine — there are no draft/review/approved transitions
- The `doc transition` command has been **removed**

#### Spec Lock

Specs now carry an exclusive lock (`locked_by_run_id`):

- `issue start` **acquires** the lock on the Spec — if the Spec is already locked by another run, the command is rejected
- The lock **auto-releases** when all pipeline stages complete
- `pipeline unlock` **force-releases** the lock (e.g., for stuck or abandoned runs)
- `doc create` **verifies** that the current run holds the Spec lock before allowing document creation

#### Stage-Level Approval

- `requires_approval` has moved from skill workflow transitions to the **pipeline stage definition**
- `pipeline stage complete <stage> --confirm` is now the approval point
- Stages with `requires_approval: true` reject completion without `--confirm`

#### Guard Changes

- The `upstream_approved` guard now checks upstream **stage completion** (not document status)
- This aligns with the single-state model: a stage being complete implies all its documents are final

### New / Changed CLI Commands

| Command | Description |
|---------|-------------|
| `pipeline stage start <stage>` | Start a pipeline stage (marks it in-progress) |
| `pipeline stage complete <stage> [--confirm]` | Complete a stage; all documents become "final"; `--confirm` for approval stages |
| `pipeline unlock` | Force-release the Spec lock held by a pipeline run |

### Removed Commands

| Command | Reason |
|---------|--------|
| `doc transition <id> <action>` | Documents no longer have independent state transitions; stage completion finalizes documents |
