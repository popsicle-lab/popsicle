# ADR-001: SaaS billing core boundaries

> **Status**: Accepted
> **Date**: 2026-06-10
> **Target Product**: `saas-billing-module`
> **Decision Type**: Architecture Decision Record (ADR)
> **Supersedes**: ——
> **Related PDRs**: `PDR-001-saas-billing-module-task-graph.md`
> **Related RFC**: `saas-billing-core-boundaries-rfc.rfc.md`

## Decision Context

The PRD defines billing invariants that require stable module boundaries: invoice total balance, credit remaining balance, paid invoice adjustment, payment retry visibility, and tax-ready audit trail.

Source debate: `saas-billing-module-architecture-debate.arch-debate.md`.

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
- [x] `products/saas-billing-module/intents/contracts.intent` is unlocked by adr-writer after ADR acceptance.

## Intent Impact

| Intent Layer | Change Type | Blocks | Notes |
|---|---|---|---|
| `intents/contracts.intent` | add awaiting goals | five contract goals | unlock after ADR acceptance |
| `intents/invariants.intent` | supports existing seed | `InvoiceTotalBalances`, `CreditApplicationWithinBalance`, `PaidInvoiceAdjustmentOnly` | no direct change in this ADR |
| global intents | no impact | — | CADR not required |

## Approval

- **Status**: Accepted
- **Approved by**: dogfood workflow operator
- **Approval date**: 2026-06-10

## References

- **Source RFC**: `saas-billing-core-boundaries-rfc.rfc.md`
- **Source Debate**: `saas-billing-module-architecture-debate.arch-debate.md`
- **Decision Matrix**: `saas-billing-module-architecture-debate.tech-decision-matrix.md`
- **Contracts Seed**: `saas-billing-core-boundaries-rfc.contracts.intent`
