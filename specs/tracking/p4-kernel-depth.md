# P4 — kernel depth

Status: **in progress**. Opened 2026-08-23. Plan §8 P4. Contract:
`specs/blocks/kernel.md` Rev 2 (K18–K24).

## Slice 1 — statements, constants, variables, atomics, barriers

Delivered 2026-08-23. Planner verification: `tools/gate.sh
--require-backend` green, 205 tests in six executables (facade 4,
typegpu-gen 7 and 26, harness 18 in 36.82 seconds plus the ignored
live test, webgpu-gen 3 and 147). `b09-kernel-depth` and
`b10-workgroup` equal on both tiers with their goldens — the
workgroup module declares `var<private>`, `var<workgroup>
array<u32, 4>`, `var<workgroup> atomic<u32>`, a storage schema with
`atomic<u32>`, two barriers, and `atomicAdd`/`atomicStore`/
`atomicLoad` on places. Eight K24 fixtures and the naga uniformity
test. Measurement: 45 s / 0.2 s / 48 s / 47 s / 6.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b09-kernel-depth` and `b10-workgroup` gate-green with WGSL goldens | — |
| 2 | `x08-live-reduction` `PASS` | — |
| 3 | `x09-live-switch` `PASS` | — |
| 4 | Every K24 rejection has a red fixture, a non-uniform barrier fails naga in the harness | — |
| 5 | Budgets hold | — |
