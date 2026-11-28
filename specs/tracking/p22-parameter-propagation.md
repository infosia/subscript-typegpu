# P22 — parameter propagation

## The audit (2026-11-28, revision `e758f5d`)

An audit traced values from each public entry to their use. It
scanned 474 tracked source files for defaults, discarded inputs,
repeated components, and forwarding sites. It compared 49 public
descriptor declarations (221 fields) with their converters. All 47
generated `convert_*` functions read every source field. All 157
generated C exports with a body reference their named parameters.
These two results are structural: a field reference does not prove
its effect on the backend.

The audit ran `cargo test --offline -p subscript-typegpu-webgpu-gen
--test main` (157 passed, 1.66 s) and `cargo test --offline -p
subscript-typegpu-gen --test main` (141 passed, 81.11 s). It did not
run the full gate, the ship tier, or a real device.

## Candidates and dispositions

| ID | Finding at `e758f5d` | Disposition |
| --- | --- | --- |
| 1 | `pair(tick())` against `pair(a: u32, b: u32 = 7)` emits one argument. `naga`: `ArgumentCount { required: 2, seen: 1 }`. | Defect. K34. |
| 2 | A `FixedArray<Item, 2>` field emits `array<Item, 2u>` with no declaration of `Item`, in compute and in render. `naga`: `no definition in scope for identifier: Item`. | Defect. K14 Rev 10. |
| 3 | `computePipeline2` over two layouts with a `pixels` field emits two globals named `pixels`. `naga`: `redefinition of pixels`. | Defect. PI6 Rev 1. |
| 4 | With 16 bytes of a `Buffer<u32>` mapped, the free functions `readOne(b, 1)` and `readBuffer(b, 1, 1)` return zero bytes. The byte offset 4 is not a multiple of 8. | Defect. BF3 Rev 1. |
| 5 | `nextVec().step(edgeVec())` emits `step(edgeVec(), nextVec())`. `edgeVec()` runs first. | Defect. K35 rule 1. |
| 6 | `pair(tick(), k == 0 ? tick() : 0)` emits the second `tick()` before the call. At `k == 0` the arguments are `(2, 1)`, not `(1, 2)`. | Defect. K35 rule 1. |
| 7 | `k != 0 && (k == 0 ? tick() > 0 : false)` emits the conditional before `&&`. At `k == 0` the counter moves. | Defect. K35 rule 2. |
| 8 | A `UiRenderer` owns one viewport buffer and one vertex buffer. Two `render` calls before one submit write both buffers twice, and both draws read the second write. | Contract (owner, 2026-11-28). UI15 Rev 10: one `render` per renderer per submit. No code change. |
| 9 | The host sampler conversion reads no LOD clamp. A host probe with `lodMinClamp: 1, lodMaxClamp: 1` on a one-level image returned the base texel. CL5 requires a trap for a setting that changes a level-zero nearest sample. | Not a defect. CL5 Rev 2: the host texture holds one level, and the clamp changes no sample there. `x29-live-sampler-lod` is the control. |
| 10 | The window host passes a physical wheel delta. `examples/ui-demo` applies it to a logical scroll offset. UI17 leaves the wheel unit open. | Defect (owner, 2026-11-28). W3 Rev 4, UI17 Rev 3: surface pixels from the host, divided by the scale in the script. |

The code at the cited sites matches findings 1 to 7 (read
2026-11-28).

## Evidence

### Slice 1 (rows 1 to 4)

`hir::Param.default` holds an `Option<Expr>` at pin `9b8e2d15`, and
the call holds only the explicit arguments. The generator emits the
default in the position of the parameter.

Red, at the unchanged sources: `cargo test --offline -p
subscript-typegpu-gen --test main call_shapes` gave 0 passed and 11
failed in 1.18 s. The failures were the `naga` argument count (row
1), `no definition in scope for identifier: Item` in compute, in
render, and in a nested array (row 2), the second declaration of
`pixels` (row 3), and a default that reads a parameter with no
diagnostic (K34). `b12-readback` on both tiers returned zero bytes
at indices 1 and 3 and for the middle pair (row 4).

Green: the same command gave 11 passed in 1.08 s. The full generator
suite gave 152 passed in 82.86 s. The harness tests
`differential::every_program_matches_both_tiers_and_golden`,
`wgsl_goldens::every_pipeline_matches_its_golden_and_validates`, and
`rejections::every_fixture_is_red_with_rule_and_owner` passed in
205.88 s: 38 program goldens on both tiers, 61 WGSL modules, 139
rejection fixtures.

No existing WGSL golden moved. `b12-readback.expected` gained the
lines `word0=11` to `word3=44` and `middle=22,33`. The new program
is `b31-call-shapes` with three modules.

The render API takes zero or one bind group, so PI6 Rev 1 has no
render case. The subscript checker rejects
`Context.fromBytes<u32>` with S100, so `b12` decodes one word as
`FixedArray<u32, 1>`.

### Slice 2 (rows 5 to 7)

The emitter computes one effect per part (pure, read, write) and
orders the parts through one routine. The sites: helper and shell
calls with their defaults, mapped free functions, vector and matrix
constructors, value-class methods, library methods, binary and
short-circuit expressions, K11 calls, schema constructors, index
expressions, assignment and update targets, compound assignments,
atomic operations, and binding accessors.

Red, at the slice 1 sources: `cargo test --offline -p
subscript-typegpu-gen --test main evaluation_order` gave 12 failed
and 2 passed in 1.28 s. The two that passed are the controls (an
expression with no conflict keeps its text). Green: 14 passed in
1.30 s. The full generator suite gave 166 passed in 144.05 s. The
harness tests
`differential::every_program_matches_both_tiers_and_golden` and
`wgsl_goldens::every_pipeline_matches_its_golden_and_validates`
passed in 242.10 s: 39 program goldens on both tiers, 63 WGSL
modules.

No existing WGSL golden moved. The new programs are
`b32-evaluation-order` and `x28-live-evaluation-order`, over the
shared module `programs/evaluation-order.ts`. One kernel runs nine
cases at `k == 0` and `k == 1` and stores 18 words.

CL2 excludes a kernel that writes a `privateVar` from the host
lane. The program counter is therefore a storage word that a typed
shell increments on the host and on the device. The generator
fixtures use a `privateVar`.

The host run of `b32` gave the same 18 words before and after the
emitter change, because the host runs the source. Only the WGSL
order tests and the live comparison detect the defect.

### Slice 3 (rows 8 to 10)

Row 10: the host multiplies a line delta by 30 and by the scale
factor, and `examples/ui-demo` divides each delta by the factor. The
window test checks `scale(2); wheel(20, -12)` → `-10, 6`, scale 1 →
`-20, 12`, and two 3-pixel events at scale 2 → 1 and then 2 logical
pixels.

Row 9: `x29-live-sampler-lod` on Metal (yawgpu), 2026-11-28:

```text
A default device=0,1,1,1 host=0,1,1,1
A clamped device=0,1,1,1 host=0,1,1,1
B default device=1,0,0,1 clamped device=0,0,1,1
PASS
```

Case A is a one-level texture. Case B is a two-level device texture
with a blue level 1. The host cannot hold case B.

Row 8: UI15 Rev 10 states the contract, and the doc comment of
`UiRenderer.render` carries it.

### Live

`tools/live.sh` on Metal (yawgpu), 2026-11-28, with `x28` and `x29`
in the set: `live::every_x_program_passes_on_a_real_adapter`, 1
passed, 224.08 s. `x28-live-evaluation-order` printed `PASS`: the
18 device words equal the host words.

## Phase review (2026-11-28)

A no-context reviewer read the cumulative change and ran 13 probe
kernels through the generator. Result: no CRITICAL, 2 MAJOR, 1
unverified item, 9 MINOR.

| Finding | Disposition |
| --- | --- |
| MAJOR 1: `b32`, `x28`, `x29` had no T3 header, and the shared module had no `m<nn>` name, so the authored-spelling and ownership tests did not read it. | Fixed: `programs/m32-evaluation-order.ts`, T3 headers. |
| MAJOR 2: the record had no run citation for the wheel test and no window run. | Fixed in this record. The red requirement of the plan covers rows 1 to 7. The window run is the owner's. |
| Unverified: K35 stored a sampler in a `let` (`let _g_operand_0 = nearest;`). `naga` accepts it. WGSL requires a constructible type for a `let` *(docs)*. | Fixed: a texture or sampler binding field is pure (K35). |
| MINOR 1: gaps in the `_g_conditional_<n>` indices. The cause is a constructor name that takes an index and is then dropped. No part is emitted twice. | Fixed: the index is taken at use. |
| MINOR 2: the compute closure placed an array element after the members. | Fixed (K14 Rev 10 wording, `push_front`). |
| MINOR 3: PI6 Rev 1 did not state the `_` suffix or the shell consequence. | Fixed in PI6 Rev 1. |
| MINOR 4: three K34 branches had no red fixture. | Fixed: three fixtures. |
| MINOR 5: five K35 sites had no direct test. The probes gave correct WGSL. | Fixed: tests added. |
| MINOR 6: `x29` did not check the clamped sample of case B. | Fixed. |
| MINOR 7: `readBuffer` had no test for a mapped sub-range or a zero count. | Fixed in `b12`. An element size that is not a multiple of 4 stays untested. |
| MINOR 8: one comment described a change. | Fixed. |
| MINOR 9, outside this phase: a helper that assigns to a parameter emits `x = x + 1u;`, and WGSL parameters are immutable. No diagnostic. | Open. A candidate for the next phase. |

The reviewer found these areas clean: K35 over 13 probe forms (loop
conditions, steps, nested short-circuit, compound assignment,
atomics, texture store, defaults with conflicts), the K34 expansion
at every analysis site, the PI6 Rev 1 names against the support
module, the absence of a moved golden, residue, and rule ids.

## Close

Review fixes, targeted runs: the generator suite, 173 passed. The
harness tests `suite_structure::` (2 passed, 2.68 s),
`rejections::` (3 passed, 142 fixtures, 52.68 s), `wgsl_goldens::`
(2 passed, 64 modules, 72.25 s), `authored_spelling::` (1 passed,
34.30 s), and `differential::` (2 passed, 40 output goldens on both
tiers, 177.09 s). The differential run covers the window test of the
wheel unit and the `b12` sub-range read.

Gate: `tools/gate.sh --require-backend` on yawgpu (Noop),
2026-11-28: green, 417 passed, 1 ignored, 524.94 s wall. The
previous row is 521 s with 384 tests.

Live: `tools/live.sh` on Metal (yawgpu), 2026-11-28, after the
review fixes: `live::every_x_program_passes_on_a_real_adapter`, 1
passed, 207.35 s. `x28-live-evaluation-order` printed `PASS`.
`x29-live-sampler-lod` printed:

```text
A default host=0,1,1,1 device=0,1,1,1
A clamped host=0,1,1,1 device=0,1,1,1
B default device=1,0,0,1 clamped device=0,0,1,1
PASS
```

Window: the owner ran `tools/window.sh examples/ui-demo/main.ts
--frames 30` on Metal, 2026-11-28: `window:frames=30`, exit 0, no
`FAIL` line. The run does not check the scroll distance.

Open:

- review MINOR 9: a helper that assigns to a parameter emits invalid
  WGSL with no diagnostic;
- the subscript checker rejects `Context.fromBytes<u32>` with S100,
  so a program decodes one word as `FixedArray<u32, 1>`.

## Follow-up: element sizes (2026-11-28)

Review MINOR 7 left one case open: a mapped read of an element whose
size is not a multiple of 4. `b12-readback` now holds three cases
(BF3 Rev 2): a `Buffer<u16>` of 6 elements, a buffer of two 6-byte
elements, and a `Buffer<u16>` of 5 elements (10 bytes). The library
did not change. The first two cases return every element. In the
third, `mapAsync(READ, 0, 10)` returns false, `mapAsync(READ, 0, 8)`
returns true, and element 4 returns zero bytes.

The first gate run was red: 51 passed, 1 failed, 514.33 s.
`scopes::every_program_scopes_exactly_its_shader_and_pipeline_creation_calls`
gave `b12-readback.ts: expected no validation scope without a
creation call, found [Push, Pop]`. The program wrapped the 10-byte
map in a validation scope. The scope is removed. Both tiers print
the same lines with no scope and exit 0.

Gate: `tools/gate.sh --require-backend` on yawgpu (Noop),
2026-11-28: green, 450 passed, 1 ignored, 524.46 s wall. The
previous row is 677 s, a run with a second cargo process.
