# Architecture: saas-billing-module

> **Status**: boundary accepted（ADR-001；implementation pending）
> **Last-Updated**: 2026-06-10
> **Last-Decision-Ref**: ADR-001

## Core Boundary

`saas-billing-module` uses modular monolith boundaries. PlanCatalog, Subscription, Invoice, Payment, Credit, AuditTrail, and BillingEvent Ledger are separate domain modules inside the product boundary.

| Module | Owns | Does Not Own |
|---|---|---|
| PlanCatalog | sellable plan metadata, billing interval, tax-ready plan fields | subscription state |
| Subscription | customer-plan lifecycle and status-change events | invoice totals |
| Invoice | line items, supplied tax amount, applied credits, adjustments, total projection | PSP calls |
| Payment | payment attempts, failure state, retry schedule, PSP adapter calls | invoice mutation bypass |
| Credit | issued credit, remaining balance, application commands | invoice line item authorship |
| AuditTrail | query/export over BillingEvent and source references | tax filing |
| BillingEvent Ledger | append-only amount/status events | accounting journal semantics |

Decision-Ref: ADR-001

## Billing Event Ledger

Amount-changing operations append BillingEvent records. Invoice, payment, and credit read models are projections over event-backed inputs. BillingEvent is an audit trace, not an accounting journal.

Decision-Ref: ADR-001

## Adapter Ports

PSP adapter reports payment outcomes and cannot mutate invoice totals directly. Tax adapter supplies tax amount, jurisdiction, and taxable basis fields; billing core does not calculate tax rates. Adapter failures become events or retryable outcomes.

Decision-Ref: ADR-001

## Open Questions

- Persistence model for BillingEvent.
- Consistency model between command write and read projection.
- PSP retry code taxonomy.
- Tax adapter response schema.
- Immutable external audit archive need.
