# P2 — compute kernels

Status: **in progress**. Opened 2026-08-22. Plan §8 P2. Contracts:
`specs/blocks/kernel.md` (K), `specs/blocks/pipeline.md` (PI).

## Slice 1 — the RC-2 probe, the wrappers, the emitter, `b02-vecadd`

Round 1 (2026-08-22): the RC-2 probe passed — the checker accepts
`computePipeline<L>(step, { workgroupSize: [64, 1, 1] })` with a
function-typed generic parameter, so the string-name fallback is
dead and RC-2 is closed. Delivered: the wrappers and runtime class,
the kernel emitter with the K10/K11 tables, the harness
`wgsl_goldens` module, `b02-vecadd` with its `.expected` and
`.vecAdd.wgsl` goldens, 21 rejection fixtures. Planner verification:
gate green, 179 tests, the emitted module is 19 lines, naga-valid,
and equal on both tiers.

Slice review (a fresh reviewer, 2026-08-22): CRITICAL 1, MAJOR 16,
MINOR 21. C1: the discovery stub hard-coded two generated names from
`b02-vecadd`, because the program imported a generated type and a
generated function. Resolution: PI8 Rev 1 — the support module
exports constants only, the stub types a name by its suffix, bind
groups are positional through the library (PI9). M1: a K15
structural check that could not fail (K15 Rev 1 moved validation to
the harness, the check is deleted). M2–M4: three identifier policies
and a 31-entry keyword list (K14 Rev: one mangler over naga's
lists). M5/M6: f16 vectors unspelled, schemas used only as locals
undeclared. M7: the snippet model carried unread fields (plan §5
defers the origin model to P4). M8/M9: swallowed diagnostics and
`Debug` dumps in messages. M10/M11: three fixtures shared one
diagnostic produced by matching the checker's message text (K17
Rev: one rule id per diagnostic, each fixture reaches its own
check). M12: a second size table. M13: `Capabilities::all()`. M14:
no emitter tests. M15: a stub `kernel` field. M16: no regeneration
path for the `.wgsl` golden. Round 2 handoff issued with every
finding.

Round 2 closed every item. Round 3 moved the regeneration of `.wgsl`
goldens through a scratch directory, because the first version left
support modules in `programs/`. Planner verification at the slice
close: `tools/gate.sh --require-backend` green, 188 tests in six
executables (facade 3, typegpu-gen unit 7 and integration 15,
harness 15, webgpu-gen 1 and 147), `b02-vecadd` equal on both tiers
with its `.vecAdd.wgsl` golden, 30 rejection fixtures each red with
one rule id and its owner, the keyword list equal to naga's, no
support module under `programs/`. Measurement: 44 s / 0.2 s / 40 s
/ 25 s / 6.

## Slice 2 — the backend request, the live lane, `b03`, `b04`, `x01`–`x03`

Delivered 2026-08-22. Planner verification: `tools/gate.sh
--require-backend` green, 191 tests in six executables (facade 4,
typegpu-gen 7 and 15, harness 15 plus the ignored live test,
webgpu-gen 3 and 147). `b03-saxpy-uniform` (a `Uniform` binding)
and `b04-particles` (a `Vec3f` schema and a helper) equal on both
tiers with their `.wgsl` goldens. The facade reads
`SUBSCRIPT_TYPEGPU_BACKEND` and builds the yawgpu backend-select
chain (L13), tested in a child process. `tools/live.sh` and the
ignored `live` module exist. The live run is pending the owner.
Measurement: 44 s / 0.2 s / 25 s / 22 s / 6.

Noted for a later round: the generated Rust of the backend request
carries no indentation (a template defect, `#[rustfmt::skip]` hides
it).

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b02-vecadd`, `b03-saxpy-uniform`, `b04-particles` gate-green on both tiers with WGSL goldens | Slice 2 close: all three equal, goldens naga-valid |
| 2 | `x01`–`x03` print `PASS` on a real adapter | — |
| 3 | Every rejection rule has a red fixture | Slice 1: 30 fixtures, one rule id each |
| 4 | Every generator diagnostic names its rule id and its owner | Slice 1: asserted per fixture |
| 5 | Build-time budgets hold | 44 s / 0.2 s / 25 s / 22 s / 6 at the slice 2 close |
