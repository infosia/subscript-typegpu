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

## Slice 2 — the kernel forms (2026-10-30)

Kernels accept `do…while` and statement-position `&&=` / `||=` (K7
Rev 20), the `as f64` / `as f32` cast form around K11 calls with the
§201 fold to nested `max`/`min` (K11 Rev 1), enums (K32), and generic
instances, value-class method helpers, local functions, and calls
through `T extends I` under emitter-made names (K33): `id_Vec2f`,
`first_f32_u32`, `P_scale(self_: P)`, `outer_id_u32`, `apply_P`.
`Math.max()` / `Math.min()` with no argument, `isNaN`, `isFinite`,
`Number.isNaN`, and `Number.isFinite` are K11 diagnostics.

Correction: K32 Rev 0 required a diagnostic for a member value that
does not fit the emitted type. The checker limits member values to
`i32` (S100 for an explicit value outside it, S008 for an implicit
overflow), so no such value reaches the emitter. The clause is
deleted.

Fixtures: 16 new reject fixtures (`k7-logical-assignment-value`,
four `k11-` NaN and infinity tests, two `k11-` empty calls,
`k33-this-write`, and eight `k33-name-collision` forms), each with
exit 1 and its diagnostic recorded. The accept tests failed 26 of 69
against the old emitter. `k17-method.ts` and `rn16-texture-sample.ts`
tested that a value-class method in a kernel is K10. K33 accepts that
form, so both became accept tests, and `k10-sampler-host-method.ts`
keeps the K10 rejection of a host-only method.

Programs: `b25-kernel-forms` (structure and the host
`simulateCompute` result `42,41`, with its `.wgsl` and `.expected`
goldens) and `x25-live-kernel-forms` (zeroed output, dispatch,
readback, comparison with an independent host run). Under Noop, `x25`
prints `FAIL 0 expected=42 got=0`, which shows the comparison is
real. A first `b25` draft wrote the expected values into the GPU
buffer before the dispatch, so its Noop readback compared the prefill
with itself. The review caught it before the commit.

Evidence: `tools/live.sh` on Metal (yawgpu):
`live::every_x_program_passes_on_a_real_adapter ... ok`, 151.6 s.
`tools/gate.sh --require-backend` with the yawgpu library, `gate:
green`, 332 passed, 0 failed, 1 ignored, 356.2 s wall. No existing
WGSL or `.expected` golden moved.

## Slice 3 — the device constraint (2026-10-30)

`lib/webgpu.ts` emits `export interface GPUDeviceCreation` with the
`queue` read accessor, `hasFeature`, and the 15 creation methods that
both device classes emit with the same signature. Both classes
declare `implements GPUDeviceCreation`. `GPUHostOwnedDevice.create`
caches one queue reference, `queue` is a read accessor, and
`dispose()` releases only that reference. `createBuffer`,
`createComputePipeline`, `createRenderPipeline`, `createBindGroup`,
`createTimestampPair`, `UiRenderer.create`, and the generated
`create<Decl>BindGroup<N>` factories take `D extends
GPUDeviceCreation`. The `*Host` variants, `guardQueueOwned`, and
`ownsQueue` are removed. Each windowed example disposes its
host-owned device wrapper in `shutdown` and on each failed `init`
path.

Correction: api-layer.md Rev 1 first named the creation methods and
`queue` as the members. `createTimestampPair` calls `hasFeature`, so
the checker rejected the generic body with S018. The members are now
every method that both classes emit with the same signature.

Probes at `c560df1` (the coding agent): an implementation's `= null`
default conforms to an interface member with no default, and an
object literal argument gets its type through a constraint call. A
call through the constraint cannot omit the argument, so a generic
body writes `device.createSampler(null)`. A class without `implements
GPUDeviceCreation` passed as `D` fails with S100 "`Device` does not
declare `implements GPUDeviceCreation`".

Open: `createBuffer<T, D>` has `T` only in its result. subscript
rejects a partial type argument list, and §149 rule 7 leaves the
contextual result type out of inference, so 81 call sites write
`createBuffer<Particle, GPUDevice>(…)`. Measured through the harness
dev tier: `using b: Buffer<Particle> = createBuffer(device, …)` fails
with S100 "cannot infer type parameter `T` of `createBuffer`: no
candidate". The owner chose a subscript change request (2026-10-30):
a generic call infers a type parameter from its contextual result
type, as §197 rule 3 does for a constructor. The call sites shorten
at the re-pin that lands it.

Open: `subscript-typegpu-gen gen` exited 0 on the unannotated
`using b = createBuffer(device, …)` that the dev tier rejects. To
investigate.

Evidence: `tools/gate.sh --require-backend` with the yawgpu library,
`gate: green`, 340 passed, 0 failed, 1 ignored, 347.5 s wall. No WGSL
or `.expected` golden moved. `tools/live.sh` on Metal:
`live::every_x_program_passes_on_a_real_adapter ... ok`, 143.6 s.
`tools/window.sh <example> --frames 30` on Metal: the 27 window
examples exit 0 with no `FAIL`. The five examples with no `frame`
entry exit 1 as before.

## Phase review (2026-10-30)

A fresh Opus review of `5a0ff03..016151a` found 1 MAJOR and 8 MINOR.

- MAJOR M1, fixed: a repeated `switch` selector (two literals, or two
  enum members with one value) exited 0 with a WGSL `case` repeated.
  It is K18 now (K18 Rev 1). Fixtures `k18-duplicate-literal-case.ts`
  and `k18-duplicate-enum-case.ts`, red then green.
- MINOR, fixed: an empty last case emits an empty body (K18 Rev 1). A
  one-argument `Math.max` / `Math.min` in the cast form emits its
  argument: subscript lowers the call to its argument, so the emitter
  accepts the round trip `x as f64 as f32` on an `f32` value and no
  other `f64` (K12). A non-finite float literal gets a diagnostic that
  names `Infinity`, `-Infinity`, or `NaN` (`k11-non-finite-literal.ts`).
- MINOR, fixed: `subscript-typegpu-gen gen` ran only the discovery
  check, which accepts a missing generated module
  (`crates/typegpu-gen/src/lib.rs` about 400), and never the full
  check that the harness runs after it adds the generated modules. The
  CLI now runs the full check before it writes output
  (`crates/typegpu-gen/src/main.rs` about 86). A CLI test expects exit
  1 for the unannotated `createBuffer` call; it exited 0 before.
- MINOR, fixed in specs: K33 names the receiver `self_`, K11 Rev 1
  states the WGSL domain of the builtins (`pow` with a negative base
  can differ from `simulateCompute`), UI15 Rev 6 names one factory,
  and this record counts 15 creation methods.
- MINOR, open: a `_g_constructor_` capture evaluates its arguments
  before earlier operands of the enclosing expression, so
  `a.add(1) + new P(a.add(2), a.add(3)).x` with atomics runs in a
  different order on the GPU. The conditional lowering has the same
  pre-existing behavior. Not fixed in P17.

Evidence: `tools/gate.sh --require-backend` with the yawgpu library,
`gate: green`, 349 passed, 0 failed, 1 ignored, 518.7 s wall. The
harness `tests/main.rs` took 429.7 s and the generator tests 57.3 s,
against 347.5 s, about 290 s, and 35 s at slice 3: 1.49x, below the
2x red line, cause not measured. No WGSL or `.expected` golden moved.

P17 is COMPLETE: no CRITICAL or MAJOR finding is open.
