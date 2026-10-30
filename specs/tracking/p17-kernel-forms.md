# P17 — the current language in kernels and the device constraint

Plan §8 P17. Pin `c560df1`.

## Base, measured 2026-10-30

Two read-only surveys ran at `c560df1`. The kernel survey probed each
form with `target/debug/subscript-typegpu-gen` on scratch fixtures.

Three silent miscompiles in accepted forms (Claude read the emitted
WGSL of A and B):

- A: `for (let i: u32 = 0; i < (c ? 4 : 8); i++) { if (i == 1) {
  continue; } … }` emits a `loop` with `i += 1u` as the last body
  statement. `continue` skips it, so the GPU loop does not end. The
  generator exits 0.
- B: `constructor(b: f32, a: f32) { this.a = a; this.b = b * 2.0; }`
  and `new P(1.0, 5.0)` emit `P(1.0f, 5.0f)`: the fields swap and the
  product is lost. A constructor with fewer parameters than fields
  emits an invalid struct constructor. The generator exits 0.
- C: a `#v` field of a bound schema emits `P_OFFSET_#v` into the
  support module (reported by the survey, not re-run by Claude).

Kernel forms the checker accepts and the kernel rejects: `do…while`
(K7), scalar `Math` with an `f32` argument (the `as f64` cast is
K12), enum members and enum types (K9, K5), a second instance of a
generic helper (a K14 collision), a method of a user value class (K10),
two local functions with one source name (K14), `&&=` and `||=` on a
local or field (K7 "an assignment is used as a value"), and
`isNaN`/`isFinite` (K9). Type aliases, `satisfies`, `debugger`, a
static method call, and parameter properties already emit.

API: `GPUDevice` and `GPUHostOwnedDevice` share nine creation
methods with identical text. `lib/typegpu.ts` and `lib/typegpu-ui.ts`
repeat about 230 lines for the `*Host` variants. The queue differs:
a cached read accessor on `GPUDevice`, a method that returns a new
owned wrapper on `GPUHostOwnedDevice`.

## Slice 1 — the miscompiles (2026-10-30)

- A: a loop-form `for` emits its step and the step prelude in the
  WGSL `continuing` block (K7 Rev 19). Claude re-ran the probe: the
  output holds `continuing { i += 1u; }`.
- B: `new P(args)` in a kernel needs a constructor of one
  `this.<field> = <parameter>` per field, in any order, or the
  parameter-property form. The emitter passes the arguments in field
  order and keeps their evaluation order. Any other body is K9
  "constructor of `P` must assign each field from one distinct
  parameter and contain no other statements" (K9 Rev 2). Claude
  re-ran the swap probe: exit 1 with that text.
- C: a `#name` field of a schema is SC11 (SC11 Rev 1).

Fixtures: `k9-constructor-body.ts`, `k9-constructor-missing-field.ts`,
`sc11-private-name.ts`, and accept tests for the `continuing` step
and a reordered constructor. The red run against the old emitter: 5
failed, 36 passed. Every committed program and example still
generates. No golden moved.

Evidence: `tools/gate.sh --require-backend` with the yawgpu library,
`gate: green`, 304 passed, 0 failed, 1 ignored, 345.4 s wall.
