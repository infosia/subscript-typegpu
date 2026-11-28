# P23 — written parameters

## The defect (2026-11-28, revision `ef54ad6`)

A probe helper `function bump(x: u32, v: Vec2f): u32 { x = x + 1;
x++; v.x = 2.0; return x + (v.x as u32); }` went through
`subscript-typegpu-gen gen`. The generator exited 0 and emitted:

```wgsl
fn bump(x: u32, v: vec2<f32>) -> u32 {
  x = x + 1u;
  x += 1u;
  v.x = 2.0f;
  return x + u32(v.x);
}
```

A WGSL parameter is immutable *(docs)*. K8 Rev 1 is the contract.

## Evidence

Red, at `ef54ad6`: 12 of the 14 tests in
`kernel::written_parameters` failed. `naga` rejected each module
with `invalid left-hand side of assignment`, at the first write to a
parameter (for example `x = x + 1u;`, `v.x = 4.0f;`,
`a.item.value = 5u;`). The two that passed are the controls: a
helper that only reads, and a block local that shadows a parameter.

Green: 14 passed. `naga` accepts each module. No existing WGSL
golden moved (64 files). The new programs are
`b33-written-parameters` and `x30-live-written-parameters`, over the
shared module `programs/m33-written-parameters.ts`. `b33` prints the
helper result and the caller's value after the call:

```text
host:scalar=5,4
vector=7,5
item=9,6
```

The subscript checker rejects a local function that writes a
parameter of its enclosing function (S100), so that form never
reaches the generator.

## Phase review (2026-11-28)

A no-context reviewer read the change and ran probe kernels through
the generator. Result: no CRITICAL, 3 MAJOR, 5 MINOR.

| Finding | Disposition |
| --- | --- |
| MAJOR 1: a write to an invocation builtin (`ctx.instanceIndex = 3;`, `ctx.frontFacing = true;`, `ctx.globalId.x = 2;`) emitted `ctx.<field> = …;` with `ctx` undeclared, exit 0. The defect is older than this phase. | Fixed: a K8 diagnostic that names the builtin. Three red fixtures (compute, vertex, fragment). Before the fix each gave exit 0 and the invalid line. |
| MAJOR 2: K35 said that a read of a `var` local is a read, and the emitter keeps a read of a written parameter in place. No wrong WGSL follows. | Fixed in the spec: K35 Rev 1 states why the read stays in place. |
| MAJOR 3: this record had no red and no gate citation. | Fixed in this record. |
| MINOR 1, 2: K8 Rev 1 did not state the name rules, and two sentences were over the length limit. | Fixed in K8 Rev 1. |
| MINOR 3: no test for a write inside `switch`, `while`, `do…while`, for `p ||= v`, for a local function in the entry, or for an instance input. The probes gave correct WGSL. | Fixed: tests added. |
| MINOR 4, 5: the `m33` header, and two doc comments in `kernel.rs`. | Fixed. |
| Outside this phase, not run through `naga`: a field of `Mat2x2f` emits as a WGSL member (`n.c0.x = 1.0f;`), and a WGSL matrix has no `.c0` member *(docs)*. | Open. A candidate for the next phase. |

## Close

Review fixes, targeted runs: the generator suite, 190 passed,
93.51 s. The harness tests `rejections::` (3 passed, 145 fixtures,
55.98 s), `wgsl_goldens::` (2 passed, 66 modules, 74.54 s), and
`differential::` (2 passed, 182.30 s).

Gate: `tools/gate.sh --require-backend` on yawgpu (Noop),
2026-11-28: green, 434 passed, 1 ignored, 547.37 s wall. The
previous row is 525 s with 417 tests.

Live: `tools/live.sh` on Metal (yawgpu), 2026-11-28:
`live::every_x_program_passes_on_a_real_adapter`, 1 passed,
218.10 s. `x30-live-written-parameters` printed `PASS`: the six
device words equal the host words.
