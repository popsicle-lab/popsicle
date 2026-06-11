# cli-ux self-host usability — golden baseline (PROJ-24)

> **Run**: `00000018-0000-4018-8001-18000000000018` (slice-delivery)
> **Date**: 2026-06-11

Golden baselines for the usability completion slice: `doc check`,
`issue close`, bundled default pipelines (D-101), and smoke isolation (O-102).
The end-to-end dogfood loop (init → issue → run → doc → check → stages →
close) runs entirely in an isolated temp workspace.

Run everything (chains the ADR-010 and ADR-011 baselines first):

```bash
bash docs/baseline/2026-06-11/cli-ux-usability/run-all.sh
```
