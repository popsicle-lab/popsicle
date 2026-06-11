# cli-ux devops tooling migration — golden baseline (PROJ-26)

> **Run**: `0000001a-0000-401a-8001-1a00000000001a` (slice-delivery)
> **Date**: 2026-06-11
> **Legacy source**: `legacy/popsicle` @ c76d729 (Makefile, scripts/install.sh,
> hooks/pre-commit, .github/workflows/{ci,release}.yml)

Asserts the migrated DevOps entry points work in the new workspace: make
targets, install script, git hook installation, and workflow file validity.
Adaptations vs legacy (ADR-014): no UI bundle/toolchain, no shell completions
(deferred command), golden/intent make targets added.

Run everything (chains all earlier cli-ux baselines first):

```bash
bash docs/baseline/2026-06-11/cli-ux-devops/run-all.sh
```
