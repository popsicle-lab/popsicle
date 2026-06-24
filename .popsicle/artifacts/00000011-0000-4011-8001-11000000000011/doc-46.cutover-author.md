---
doc_type: cutover-author
id: doc-46
pipeline_run_id: 00000011-0000-4011-8001-11000000000011
status: active
title: PROJ-17 command surface alignment cutover ADR
version: 1
---

# PROJ-17 command surface alignment cutover ADR

> **Promoted to**: `products/cli-ux/decisions/adr/ADR-011-command-surface-realignment.md`
> **Stage**: cutover (slice-delivery)
> **Date**: 2026-06-11

This artifact is the working copy of ADR-011. The promoted decision record in
`products/cli-ux/decisions/adr/` is authoritative; see it for the full text:
context, re-adjudication table (7 implemented / 10 deferred / 3 removed),
gate compliance (intent Z3 pass · 13/13 golden · cargo test green), and
divergences D-101 (unbundled default pipelines) / O-102 (smoke workspace
pollution).

## Cutover Gate Checklist

- [x] intent gate: `tool run intent-validate path=products` exit 0 (all verified)
- [x] equivalence gate: 13/13 golden pass (8 regression + 5 surface-contract)
- [x] cargo test -p cli-ux exit 0
- [x] doctor `current_workspace_binary_match=true`
- [x] ADR-011 drafted and promoted to products/cli-ux/decisions/adr/

## Waiver Checklist

- [x] 无豁免

## Approval

Completion of this stage requires the user to run:
`popsicle pipeline stage complete cutover --run 00000011-0000-4011-8001-11000000000011 --confirm`
