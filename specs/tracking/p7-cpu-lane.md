# P7 — the CPU lane

Status: **in progress**. Opened 2026-08-23 by the owner's decision.
Plan §8 P7. Contract: `specs/blocks/cpu-lane.md` (CL1–CL4).

## Slice 1 — `simulateCompute`, the host-runnable constant, the oracles

Handoff issued 2026-08-23. Result: pending.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `x01`–`x04` and `x09` use `simulateCompute` as their oracle and print `PASS` on Metal | — |
| 2 | One `b` program's host golden is committed and compared on both tiers | — |
| 3 | `CL2` has a fixture | — |
| 4 | Budgets hold | — |
