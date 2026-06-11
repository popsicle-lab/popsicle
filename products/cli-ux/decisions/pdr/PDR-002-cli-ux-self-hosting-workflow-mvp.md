# PDR-002: cli-ux self-hosting workflow MVP

> **Status**: Accepted
> **Date**: 2026-06-10
> **Product**: cli-ux
> **Source**: PROJ-9

## Decision

Add a self-hosting MVP to the `popsicle-new` binary. The MVP covers the IDD dogfood loop:

- `issue create/list/show/start`
- `pipeline status/next/stage complete`
- `doc create/list/show`
- `doctor` provenance output
- `tool run intent-validate path=products`

## Rationale

ADR-008 accepted a semantic shell and explicitly deferred storage-backed real workspace mutation. SaaS billing dogfood exposed that the deferral blocks credible self-migration: the workflow completed only because the agent used `../target/debug/popsicle`.

## Consequences

- `crates/cli-ux` must stop using only `MemoryDocumentStore` in the binary path.
- The MVP may use a compact file-backed state store under `.popsicle/self-host/`; full legacy database compatibility is not required.
- `doctor` must expose binary and workspace provenance.
- Full legacy byte parity remains out of scope.

## Intent Impact

| Intent | File | Meaning |
|---|---|---|
| `SelfHostedWorkflowSmokePasses` | `acceptance.intent` | local binary can complete the workflow smoke |
| `BinaryProvenanceVisible` | `acceptance.intent` | source binary and workspace are visible |
| `WorkflowSmokeDoesNotDependOnParentBinary` | `self-hosting-invariants.intent` | smoke does not silently use parent/system binary |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-9 workflow
