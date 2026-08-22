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

Live run at `010e846` (Metal): `x08-live-reduction` failed with
`FAIL expected=4608 got=0`, `x09` passed. Bisection by the planner
through the ship tier on Metal: atomics alone pass (A); a workgroup
array written and read by the same thread passes (D); a workgroup
scalar written by thread 0 and read after a barrier by others
reads zero (F); hand-written modules with the same shapes — f32,
256 threads, `local_invocation_index`, the workgroup variable
before or after the bindings — all pass (w1–w3); the `x08` module
verbatim through a hand-written pipeline fails (w4); the same
module without its early `return` passes (w5, w7) and with the
`return` fails regardless of the loop (w6). Cause: `if (global >=
1024u) { return; }` before `workgroupBarrier()` is a uniformity
violation. naga did not report it, yawgpu's Tint compiled it, and
Metal read the workgroup memory as zero. Resolution: K22 Rev 1
moves uniform placement into the generator (a taint rule over
builtins, bindings, and helper results, and no `return` before a
barrier), K24 gains three rejections, and `x08` loads under a
condition and barriers unconditionally. Observation for the
backend's owner: Dawn rejects this module at pipeline creation
(docs), yawgpu accepted it.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b09-kernel-depth` and `b10-workgroup` gate-green with WGSL goldens | — |
| 2 | `x08-live-reduction` `PASS` | — |
| 3 | `x09-live-switch` `PASS` | — |
| 4 | Every K24 rejection has a red fixture, a non-uniform barrier fails naga in the harness | — |
| 5 | Budgets hold | — |
