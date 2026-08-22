# P4 — kernel depth

Status: **in progress**. Opened 2026-08-23. Plan §8 P4. Contract:
`specs/blocks/kernel.md` Rev 2 (K18–K24).

## Slice 1 — statements, constants, variables, atomics, barriers

Handoff issued 2026-08-23. Result: pending.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b09-kernel-depth` and `b10-workgroup` gate-green with WGSL goldens | — |
| 2 | `x08-live-reduction` `PASS` | — |
| 3 | `x09-live-switch` `PASS` | — |
| 4 | Every K24 rejection has a red fixture, a non-uniform barrier fails naga in the harness | — |
| 5 | Budgets hold | — |
