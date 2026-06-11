# cli-ux command surface alignment — equivalence baseline (PROJ-17)

> **Run**: `00000011-0000-4011-8001-11000000000011` (slice-delivery)
> **Stage**: equivalence
> **Date**: 2026-06-11

Golden baselines for the PROJ-17 command surface alignment. Unlike slice
cutover baselines, the reference here is not legacy byte parity but the
**re-adjudicated surface contract**:

1. help advertises only implemented commands (7 families)
2. deferred commands fail with actionable `deferred` errors
3. `--format json` works on every command, including errors
4. `tool run intent-validate` resolves strictly in-workspace
5. the pre-existing self-host goldens (2026-06-11/cli-ux-self-host) still pass

Run everything:

```bash
bash docs/baseline/2026-06-11/cli-ux-command-alignment/run-all.sh
```
