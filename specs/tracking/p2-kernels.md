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
gate green, 179 tests, the harness executable 9.57 seconds, the
emitted module is 19 lines, naga-valid, and equal on both tiers.

Slice review (a fresh reviewer, 2026-08-22): CRITICAL 1, MAJOR 16,
MINOR 21. C1: the discovery stub hard-coded two generated names from
`b02-vecadd`, because the program imported a generated type and a
generated function. Resolution: PI8 Rev 1 — the support module
exports constants only, the stub types a name by its suffix, bind
groups are positional through the library (PI9). M1: a K15
structural check with no failing input (K15 Rev 1 moved validation to
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
harness 15 in 9.81 seconds, webgpu-gen 1 and 147), `b02-vecadd` equal on both tiers
with its `.vecAdd.wgsl` golden, 30 rejection fixtures each red with
one rule id and its owner, the keyword list equal to naga's, no
support module under `programs/`. Measurement: 44 s / 0.2 s / 40 s
/ 25 s / 6.

## Slice 2 — the backend request, the live lane, `b03`, `b04`, `x01`–`x03`

Delivered 2026-08-22. Planner verification: `tools/gate.sh
--require-backend` green, 191 tests in six executables (facade 4,
typegpu-gen 7 and 15, harness 15 in 16.00 seconds plus the ignored
live test, webgpu-gen 3 and 147). `b03-saxpy-uniform` (a `Uniform` binding)
and `b04-particles` (a `Vec3f` schema and a helper) equal on both
tiers with their `.wgsl` goldens. The facade reads
`SUBSCRIPT_TYPEGPU_BACKEND` and builds the yawgpu backend-select
chain (L13), tested in a child process. `tools/live.sh` and the
ignored `live` module exist. Live run by the owner (2026-08-22, Metal, yawgpu
`target/release/libyawgpu.dylib`, pin `bb9dadc`, the working tree
between `786aae7` and the P1 slice 2 landing):
`live::every_x_program_passes_on_a_real_adapter ... ok`, 9.71
seconds, `x01`–`x03` `PASS` on both tiers. Phase-close run by
the planner outside the sandbox (2026-08-22, Metal, the same
library, committed tree `b7e2533`, `x01`–`x03` on the R34 byte
path): `live::every_x_program_passes_on_a_real_adapter ... ok`,
6.70 seconds.
Measurement: 44 s / 0.2 s / 25 s / 22 s / 6.

Noted for a later round: the generated Rust of the backend request
carries no indentation (a template defect, `#[rustfmt::skip]` hides
it).

## Phase review (2026-08-22)

A fresh reviewer ran the gate (green, 26.5 s, 192 tests) and found
CRITICAL 3, MAJOR 9, MINOR 17. The CRITICALs are emitter defects
naga cannot see: a K10 method lowered to an operator is not
parenthesized as an operand, `Math.fround` substitutes its
argument's text bare, and the `?:` lowering of a loop condition
lands before the loop. The MAJORs: PI8 still described the typed
factory (a failed edit), PI2's type-argument check did not exist,
seven author diagnostics had no fixture, `build-time.md` had lost
its rows, a nested `?:` ran its inner branch unconditionally, the
uniform minimum binding size question (M6, a specification reading
with no Dawn run), the PI13 async and non-void cases relabelled
checker diagnostics through text matching, the differential test
had no non-empty assertion, and the gate citations quoted no wall
time. Spec resolutions in this commit: PI8 Rev 1 written as
intended, PI13 Rev 1 and K17 leave the checker-rejected cases to
the checker, K9 and K14 state where a lowering lands and how
precedence is judged, plan §5 matches LY11, T6 names the library
and binary equality. M6 is recorded as a Dawn measurement item for
P6: LY11 stays as measured in the proof of concept until a Dawn run
says otherwise. The code findings went to the coding agent in one round with the
P1 findings: the three emitter defects carry emitter tests that were
red before the fix (`a.add(b).scale(s)` emitted `a + b * s`), the
PI2 check exists with a fixture, the rejection corpus is 44 fixtures
each with exactly one diagnostic, no text matching over program
source or checker messages remains, the traps module carries the
T10 guard, naga offsets cover every `b` program, and
`x04-live-control-flow` exercises a `while` condition and a nested
`?:` against a host expectation. Planner verification: gate green,
194 tests in six executables (facade 4, typegpu-gen 7 and 17,
harness 16 in 19.17 seconds plus the ignored live test, webgpu-gen
3 and 147). Measurement: 42 s / 0.2 s / 29 s / 27 s / 6.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b02-vecadd`, `b03-saxpy-uniform`, `b04-particles` gate-green on both tiers with WGSL goldens | Slice 2 close: all three equal, goldens naga-valid |
| 2 | `x01`–`x03` print `PASS` on a real adapter | Metal, `b7e2533`: ok, 6.70 s (and an earlier run at 9.71 s) |
| 3 | Every rejection rule has a red fixture | Phase close: 44 fixtures, one diagnostic each, every author diagnostic site covered |
| 4 | Every generator diagnostic names its rule id and its owner | Slice 1: asserted per fixture |
| 5 | Build-time budgets hold | 44 s / 0.2 s / 25 s / 22 s / 6 at the slice 2 close |
