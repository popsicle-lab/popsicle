# PDR-001 · saas-billing-module task graph

> **Status**: Proposed
> **Date**: 2026-06-10
> **Product**: saas-billing-module
> **Source**: PROJ-8 product-debate + Product Brief

## Context

The project is a greenfield SaaS billing module. There is no legacy fact baseline, so the Product Brief is the ground truth for product scope: plan catalog, subscriptions, invoices, payment retries, credits, tax-ready audit trail, and intent-verifiable billing invariants.

## Decision

Adopt a subscription-first user task graph with an audit-first invariant baseline. The PRD creates 7 task chunks across the 5 fixed journey stages and seeds acceptance/invariant intent files. Module boundary contracts remain an ADR candidate.

## Consequences

| File | Action |
|---|---|
| `products/saas-billing-module/PRODUCT.md` | Create |
| `products/saas-billing-module/tasks/README.md` | Create |
| `products/saas-billing-module/tasks/onboarding/T-BILL-0001-create-sellable-plan.md` | Create |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0002-open-or-change-subscription.md` | Create |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0003-confirm-invoice-amount-source.md` | Create |
| `products/saas-billing-module/tasks/troubleshooting/T-BILL-0004-handle-payment-failure-retry.md` | Create |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0005-issue-and-apply-credit.md` | Create |
| `products/saas-billing-module/tasks/lifecycle/T-BILL-0006-export-billing-audit-trail.md` | Create |
| `products/saas-billing-module/tasks/admin/T-BILL-0007-configure-payment-retry-policy.md` | Create |
| `products/saas-billing-module/intents/acceptance.intent` | Create |
| `products/saas-billing-module/intents/invariants.intent` | Create |

## Intent Impact

- `acceptance.intent`: plan creation, subscription audit, payment retry visibility, credit audit, audit export.
- `invariants.intent`: invoice total balance, credit remaining balance, paid invoice immutability.
- `contracts.intent`: deferred until architecture branch defines module boundaries.

## Approval

- **Status**: Proposed
- **Approved by**: pending PRD review
- **Approval date**:
