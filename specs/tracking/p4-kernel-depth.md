# P4 — kernel depth

Status: **COMPLETE 2026-08-23.** Opened 2026-08-23. Closed at
`ea25b01`. Plan §8 P4. Contract:
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
violation. naga did not report it. Resolution: K22 Rev 1
moves uniform placement into the generator (a taint rule over
builtins, bindings, and helper results, and no `return` before a
barrier), K24 gains three rejections, and `x08` loads under a
condition and barriers unconditionally.

Corrected 2026-08-23 (plan §10 C4): the same probe with a
`validation` error scope around `createShaderModule` and
`createComputePipeline` shows that yawgpu (Metal) and Dawn both
reject the module — "'workgroupBarrier' must only be called from
uniform control flow" at the shader module, then "shader module
must not be an error module" / "Invalid ShaderModule" at the
pipeline. The zeros were read from an invalid pipeline. No backend
defect exists, and no observation goes to yawgpu's owner. The gap
was the program: no error scope, so a rejection looked like a
numeric result. PI14 closes it.

Round 2 landed at `9670180`: the taint rule, the conditional load in
`x08`, three K24 fixtures. Live run by the planner outside the
sandbox at `9670180` (Metal): ok, 18.47 seconds, `x01`–`x09`
`PASS` — the four-workgroup tree reduction with barriers and an
atomic add equals the host sum, and the `switch` program decides
every output as the host does.

## Phase review (2026-08-23)

A fresh reviewer ran the gate (green, 46.66 s) and found CRITICAL 1,
MAJOR 8, MINOR 12. CRITICAL: the K22 taint ignored `break` and
`continue`, so a local a loop writes stayed uniform after a
non-uniform early exit, and a barrier guarded by that local passed
(a counterexample kernel is in the review). MAJOR: the `for` step
ran under the outer control, not the loop condition; an empty
`default` case lost its shared body; the K7 and the
workgroup-initializer fixtures proved other rules; plan exit
criterion 4 still named naga (now §10 C1); the round 2 measurement
row was missing; K18 forbade the `continue` two goldens use; an
atomic receiver's `?:` index dropped its lowering. Resolutions in
the specs: K22 Rev 3 states the break/continue and `for`-step rules
and admits uniform-buffer reads and `length()`; K18 admits
`continue` and `default` grouping; K19 folds with checked
arithmetic; K21 rejects atomics in uniform and read-only storage;
K14 names the constants' position; plan §10 C1. The code findings
landed at `ea25b01`: the counterexample is a K22 diagnostic, the
`for` step and the loop-exit taint exist, `default` groups, the K7
fixture is back, K19 folds with checked arithmetic, atomics behind
`Uniform` and `Storage` are rejected, `x08` runs 1000 values so 24
lanes take the conditional load's false branch. Planner
verification at the close: gate green, 211 tests in six executables
(facade 4, typegpu-gen 7 and 32, harness 18 in 38.37 seconds plus
the ignored live test, webgpu-gen 3 and 147). Live run outside the
sandbox at `ea25b01` (Metal): ok, 18.47 seconds, `x01`–`x09`
`PASS`. Measurement: 44 s / 0.2 s / 49 s / 46 s / 6.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b09-kernel-depth` and `b10-workgroup` gate-green with WGSL goldens | Slice 1 and round 2, `--require-backend` green |
| 2 | `x08-live-reduction` `PASS` | Metal at `9670180` and `ea25b01` (1000 values, the false branch taken) |
| 3 | `x09-live-switch` `PASS` | Metal at `010e846`, `9670180`, `ea25b01` |
| 4 | Every K24 rejection has a red fixture, a non-uniform barrier is a generator diagnostic (§10 C1) | The close corpus, one diagnostic each, the reviewer's counterexample rejected |
| 5 | Budgets hold | 44 s / 0.2 s / 49 s / 46 s / 6 at the close |
