# PDR-001 · cli-ux command tree redesign

> **Status**: Proposed — **disposition table amended by ADR-011 (2026-06-11)**
> **Date**: 2026-06-09
> **Product**: cli-ux
> **Source**: PROJ-6 product-debate + fact-extractor
> **Amended-by**: [`ADR-011`](../adr/ADR-011-command-surface-realignment.md)（PROJ-17）：
> 下表 "preserve" 命令中仅 `init/issue/pipeline/doc/tool/admin`（+ `doctor`）进入
> self-host 实现面；`module/skill/spec/namespace/prompt/git/memory/context/registry/completions`
> 重分类为 **deferred**（不宣传、结构化报错、逐个待 PDR 修订）；drop 清单不变。

## Context

legacy `popsicle` exposes 22 subcommands. The cli-ux migration is not a full compatibility rewrite; it exposes the already-migrated `skill-runtime` and `artifact-system` capabilities through a smaller, agent-friendly command shell.

## Decision

| legacy command | disposition | new surface |
|---|---|---|
| `init` | preserve | `popsicle init` |
| `module` | preserve | `popsicle module` |
| `tool` | preserve | `popsicle tool` |
| `skill` | preserve | `popsicle skill` |
| `pipeline` | preserve | `popsicle pipeline` |
| `spec` | preserve | `popsicle spec` |
| `issue` | preserve | `popsicle issue` |
| `namespace` | preserve-minimal | `popsicle namespace create/list/use` |
| `doc` | preserve-redesign | `popsicle doc create/list/show/check` |
| `extract` | redesign | under `doc extract` or artifact command, not separate MVP top-level |
| `prompt` | redesign | `popsicle prompt` as agent context shell |
| `admin` | preserve | `popsicle admin` |
| `migrate` | move | `popsicle admin migrate` |
| `reinit` | move | `popsicle admin reinit` |
| `git` | preserve | `popsicle git` |
| `memory` | preserve | `popsicle memory` |
| `context` | preserve | `popsicle context` |
| `registry` | preserve | `popsicle registry` |
| `completions` | preserve | `popsicle completions` |
| `checklist` | drop | use `doc check` |
| `item` | drop | task_chunk/doc path |
| `sync` | defer/drop | out of IDD MVP |

## Consequences

- `products/cli-ux/tasks/**` contains 7 task chunks.
- `products/cli-ux/intents/acceptance.intent` captures command effects, not byte parity.
- ADR-007 must define `crates/cli-ux` as IO shell only.

## Approval

- **Status**: Proposed
- **Approved by**: pending prd review
- **Approval date**:
