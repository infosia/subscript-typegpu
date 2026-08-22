# P2 — compute kernels

Status: **in progress**. Opened 2026-08-22. Plan §8 P2. Contracts:
`specs/blocks/kernel.md` (K), `specs/blocks/pipeline.md` (PI).

## Slice 1 — the RC-2 probe, the wrappers, the emitter, `b02-vecadd`

Handoff issued 2026-08-22. Result: pending.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b02-vecadd`, `b03-saxpy-uniform`, `b04-particles` gate-green on both tiers with WGSL goldens | — |
| 2 | `x01`–`x03` print `PASS` on a real adapter | — |
| 3 | Every rejection rule has a red fixture | — |
| 4 | Every generator diagnostic names its rule id and its owner | — |
| 5 | Build-time budgets hold | — |
