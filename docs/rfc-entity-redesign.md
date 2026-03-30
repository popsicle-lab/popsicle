# RFC: Entity Relationship Redesign

## Summary

Redesign the domain entity hierarchy from a flat, disconnected model to a clear top-down ownership chain:

```
Project (optional)
  └── Topic
        └── Issue
              └── PipelineRun
                    └── Document
```

## Motivation

The original model had several structural problems:

1. **Topic ↔ Issue disconnection** — Topics and Issues had no direct relationship; both could independently own Documents, creating confusing dual-ownership
2. **Issue → PipelineRun one-to-one** — An Issue stored a single `pipeline_run_id`, making revision workflows awkward
3. **No Project concept** — No way to group related Topics under a higher-level initiative
4. **Topic documents lacked workflow state** — Documents attached to Topics didn't show pipeline stage/status context

## Design

### New Entity Hierarchy

| Entity | Parent | Relationship |
|--------|--------|-------------|
| **Project** | — | Top-level grouping (required) |
| **Topic** | Project (required) | `topic.project_id → project.id` |
| **Issue** | Topic | `issue.topic_id → topic.id` (required) |
| **PipelineRun** | Issue + Topic | `pipeline_run.issue_id → issue.id` (created exclusively via `issue start`) |
| **Document** | PipelineRun + Topic | Unchanged — linked via `run_id` and `topic_id` |

### Key Changes

#### Added Fields
- `Project` — New entity with `id`, `name`, `slug`, `description`, `status` (active/completed/archived), `tags`
- `Topic.project_id: Option<String>` — Optional link to a Project
- `Issue.topic_id: String` — Required link to a Topic (every Issue belongs to a Topic)
- `PipelineRun.issue_id: Option<String>` — Optional back-link to the Issue that spawned this run

#### Removed Fields
- `Issue.pipeline_run_id` — Removed; replaced by one-to-many via `PipelineRun.issue_id`

### Migration Strategy

SQLite doesn't support `DROP COLUMN` easily, so the migration:

1. Adds `topic_id TEXT DEFAULT ''` to `issues` table
2. Adds `issue_id TEXT DEFAULT ''` to `pipeline_runs` table
3. Adds `project_id TEXT DEFAULT ''` to `topics` table
4. Creates `projects` table
5. Backfills `issues.topic_id` from existing `pipeline_runs` where `pipeline_runs.id = issues.pipeline_run_id`
6. Backfills `pipeline_runs.issue_id` from the reverse lookup
7. Old `pipeline_run_id` column remains in the schema but is no longer read

## CLI Changes

### New Commands
- `popsicle project create|list|show|update|delete` — Full CRUD for Project entity

### Modified Commands
- `popsicle issue create` — Now requires `--topic <name>` to specify the parent Topic
- `popsicle issue list` — Accepts optional `--topic` filter
- `popsicle issue show` — Displays topic name and all associated pipeline runs (not just one)
- `popsicle issue start` — Creates a new PipelineRun with `issue_id` set; supports multiple runs per issue
- `popsicle topic create` — Accepts optional `--project <name>`
- `popsicle topic list` — Accepts optional `--project` filter
- `popsicle topic show` — Now displays associated Issues in addition to runs and documents

## Storage API Changes

### New Methods
- `create_project`, `get_project`, `list_projects`, `update_project`, `delete_project`
- `find_project_by_name` — Lookup by name
- `list_topics_by_project` — Filter topics by project
- `find_runs_by_issue` — Get all PipelineRuns for an Issue

### Modified Methods
- `create_issue` / `update_issue` / `get_issue` — Use `topic_id` instead of `pipeline_run_id`
- `query_issues` — Accepts optional `topic_id` filter parameter
- `find_issue_by_run_id` — Now queries via `pipeline_runs.issue_id` instead of `issues.pipeline_run_id`
- `create_topic` / `get_topic` / `list_topics` — Include `project_id` field

## DTO Changes

- `IssueInfo` / `IssueFull` — `pipeline_run_id` replaced with `topic_id`
- `IssueProgress` — Now contains `topic_id` and `pipeline_runs: Vec<PipelineRunInfo>` (supports multiple runs)
- `TopicInfo` / `TopicDetailInfo` — Added `project_id` field; detail includes `issues: Vec<IssueInfo>`
- `PipelineRunInfo` — Added `issue_id` field
- New: `ProjectEntityInfo`, `ProjectEntityDetail` DTOs

## Workflow Enforcement

### Strict Entity Creation Order

The redesign enforces a top-down creation order. Entities cannot be created out of sequence:

```
Project → Topic (with tags) → Issue → PipelineRun → Document
```

- **Project is required** — Topics cannot exist without a parent Project (`topic.project_id` is no longer optional)
- **Topic tags drive matching** — `issue create` auto-matches to a Topic by tag overlap; explicit `--topic` overrides
- **`issue start` is the only way to create PipelineRuns** — `pipeline run` and `pipeline quick` commands are removed. All runs originate from an Issue
- **`doc create` is stage-gated** — Documents can only be created when the target stage is unblocked (upstream dependencies satisfied)

### Document Lifecycle (`doc_lifecycle`)

Skills declare how their documents behave across pipeline runs:

| Lifecycle | Behavior | Example Skills |
|-----------|----------|---------------|
| `singleton` (default) | One document per topic per skill. Reused and updated across runs. | PRD, RFC |
| `cumulative` | New document created each run. Each run produces a separate artifact. | ADR, test reports |

`pipeline next` surfaces this information: singleton docs show skip/update options, cumulative docs always create new.

### Removed Commands

- `pipeline run <pipeline> --title <t>` — Replaced by `issue start <key>`
- `pipeline quick --title <t>` — Removed; use an appropriate pipeline via `issue start` instead
