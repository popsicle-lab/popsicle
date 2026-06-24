# ADR-001: SaaS billing core boundaries

> **Status**: Proposed
> **Date**: 2026-06-10
> **Target Product**: `saas-billing-module`
> **Decision Type**: Architecture Decision Record (ADR)
> **Supersedes**: ——
> **Related PDRs**: `PDR-001-saas-billing-module-task-graph.md`
> **Related RFC**: `saas-billing-core-boundaries-rfc.rfc.md`
> **Related Journey**: ——

## Decision Context

### Trigger

The PRD defines billing invariants that require stable module boundaries: invoice total balance, credit remaining balance, paid invoice adjustment, payment retry visibility, and tax-ready audit trail.

### Multi-role Debate Summary

Source: `saas-billing-module-architecture-debate.arch-debate.md`.

**Participants**: ARCH, SEC, PERF, OPS, DATA, DEV

**Key disagreements**:
- Modular monolith vs service-per-domain: resolved in favor of modular monolith for greenfield first spec.
- Event ledger vs invoice-centric CRUD: resolved in favor of append-only BillingEvent for auditability.
- Tax-ready vs tax-compliant: tax calculation remains outside billing core.

**Fact basis**:
- Product Brief
- `saas-billing-module-prd-task-graph.prd.md`
- `saas-billing-module-architecture-debate.arch-debate.md`

### Alternatives

| Option | Proposer | Rejection Reason |
|---|---|---|
| Service-per-domain + integration events | OPS | Too much operational overhead before contracts stabilize |
| Invoice-centric CRUD core | DEV | Weak auditability and paid invoice immutability |

## Decision

SaaS billing module uses modular monolith boundaries. Amount-changing operations append BillingEvent records. PSP and Tax integrations are adapter ports and do not mutate core invoice state directly.

## Consequences

### ARCHITECTURE.md Updates
- [x] `products/saas-billing-module/ARCHITECTURE.md` § Core Boundary
- [x] `products/saas-billing-module/ARCHITECTURE.md` § Billing Event Ledger
- [x] `products/saas-billing-module/ARCHITECTURE.md` § Adapter Ports

### Intent Updates
- [x] `products/saas-billing-module/intents/contracts.intent` adds awaiting goals:
  - `BillingCoreUsesAppendOnlyEvents`
  - `InvoiceProjectionUsesEventBackedAmounts`
  - `CreditApplicationUsesEventBackedBalance`
  - `PspAdapterCannotMutateInvoice`
  - `TaxAdapterSuppliesTaxDataOnly`
- [ ] `products/saas-billing-module/intents/contracts.intent` is unlocked by adr-writer after ADR acceptance.

### Code Updates (informational, not enforced)
- Future module boundary candidates: `PlanCatalog`, `Subscription`, `Invoice`, `Payment`, `Credit`, `AuditTrail`, `BillingEvent Ledger`, PSP adapter, Tax adapter.

### Risk Side-Effects

| Risk | Trigger | Mitigation |
|---|---|---|
| Event ledger becomes accounting ledger | revenue recognition enters scope | keep accounting non-goal |
| Adapter leaks vendor SDK into core | PSP SDK appears in domain interfaces | contracts forbid direct mutation/vendor coupling |
| Tax-ready overclaims compliance | tax filing expected | tax calculation remains adapter/out-of-scope |

## Intent Impact

| Intent Layer | Change Type | Blocks | Notes |
|---|---|---|---|
| `intents/contracts.intent` | add awaiting goals | five contract goals | unlock after ADR acceptance |
| `intents/invariants.intent` | supports existing seed | `InvoiceTotalBalances`, `CreditApplicationWithinBalance`, `PaidInvoiceAdjustmentOnly` | no direct change in this ADR |
| global intents | no impact | — | CADR not required |

## Validation Plan

### Contracts Validation

- Run parser/check over `products/saas-billing-module/intents/contracts.intent` after ADR acceptance.
- intent-spec-writer tightens goals into contracts that preserve existing acceptance/invariant seeds.

### Quality Attribute Validation

- Projection lag, retry observability, and adapter failure telemetry are `[待验证]` until implementation design.

### Rollback Condition

If event-backed boundaries make the first implementation unworkable, create a superseding ADR. Do not edit this ADR after acceptance.

## Approval

- **Status**: Proposed → Accepted by adr-writer
- **Approved by**:
- **Approval date**:

## References

- **Source RFC**: `saas-billing-core-boundaries-rfc.rfc.md`
- **Source Debate**: `saas-billing-module-architecture-debate.arch-debate.md`
- **Decision Matrix**: `saas-billing-module-architecture-debate.tech-decision-matrix.md`
- **Contracts Seed**: `saas-billing-core-boundaries-rfc.contracts.intent`
