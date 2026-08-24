# subscript-typegpu — project plan

Rev 0, 2026-08-22. `CLAUDE.md` holds the invariants. When this plan
and `CLAUDE.md` disagree, `CLAUDE.md` wins. When evidence disagrees
with either, fix both.

## 1. What this project is

subscript-typegpu gives subscript programs two things that
[TypeGPU](https://github.com/software-mansion/TypeGPU) gives
JavaScript programs: typed data schemas with automatic memory layout,
and GPU kernels authored in the host language. TypeGPU's
implementation rests on `Proxy`, function-object reflection, and
TypeScript type-level programs. subscript rejects all three
permanently. Therefore this project moves that machinery to compile
time (invariant 9).

The project is self-contained. It owns its facade over webgpu.h, its
runtime library, its generator, and its gates. It consumes subscript
as a pinned crate and a webgpu.h implementation as a shared library
at run time. Nothing else is consumed.

## 2. What the proof of concept measured

A prior proof of concept built the same two layers. Its findings are
restated here as rules. The code is not copied.

| Finding | Rule in this project |
|---|---|
| The full gate took 3.5 hours. The differential suites inside it took under 5 minutes. The rest was 52 test executables × 3 feature configurations, plus build scripts that reran on env changes | `CLAUDE.md` "Build time", rules 1 through 5 |
| The generator pulled the Cranelift stack for one layout function | `subscript-typegpu-gen` depends on `subscript-compiler` only. C layout is computed in the generator and cross-checked at ship tier |
| Committed generated TypeScript for 36 programs moved on every emitter change, so phases ran in 4 to 5 slices each | Generated support modules are not committed. WGSL goldens are (`CLAUDE.md` core principle 4) |
| A WGSL text escape hatch with a lexical fence cost more review rounds than any feature | No raw WGSL enters through the authoring surface in P0 through P5. A later phase adds it only with a WGSL parser, never a fence |
| Noop executes no compute shader | Numeric truth comes from the live lane. The plan adds a CPU lane (P7) as a second numeric oracle |
| `run_jit_with_native_libraries` forks without `exec`. Objective-C refuses to initialize in that child | The gate lane keeps the fork (Noop needs no Objective-C). The live lane uses `ReloadSession` |
| Three independent layout oracles caught real defects: naga offsets, C `offsetof`, TypeGPU golden vectors | All three stay (§5 D6) |
| A diagnostic that named the wrong owner recurred in four phases | Diagnostics name their cause and their owner from P2 on (§5 D9) |
| Per-type arithmetic functions (`addV3f`) read acceptably but not well | Vector arithmetic is methods on value classes with real bodies (§5 D7) |
| Flattened host mirrors (`position_x`, `position_y`) | The schema class is the host type. Layout identity through R33 (§3 D4, risk RC-1) |
| The WebGPU substrate generator was reviewed clean: zero debt markers, two-way policy validation, byte-gated outputs | The generator is imported and reshaped, not rewritten (§3 D11, §5) |
| The generator carried 1,556 lines and a `chdir` for an engine example and a measurement backend | Dropped at import. The facade has one consumer, the API layer |
| The generator re-parsed its own generated Rust with `syn` to build the symbol table | The symbol table is emitted from the plan. `syn` is not a dependency |

## 3. Design decisions

Each decision names its kill evidence. A killed decision
is revised in §10, never patched silently.

- **D1 — Compile time replaces run time.** Schemas, kernels, and
  layouts are compile-time artifacts. Kill evidence: a workflow that
  requires run-time schema construction. None is known.
- **D2 — The typed HIR is the input.** `subscript-typegpu-gen` calls
  `check_program` and walks `hir::Module`. WGSL emission is
  type-directed. No second parser exists. Kill evidence: a required
  fact absent from the HIR.
- **D3 — One source file holds host code and GPU code.** A kernel is
  a plain module-level function. A schema is a plain value class. No
  sidecar module, no manifest const, no `.gpu.ts` split. The
  generator discovers kernels through pipeline declarations (§4).
  Kill evidence: a program the generator cannot partition into host
  and GPU parts without a marker the language lacks.
- **D4 — The schema class is the host type.** The author writes one
  `@CStruct` class. Host code constructs it, fills `FixedArray<T, N>`
  of it, and passes it to a buffer. The generator checks that the C
  layout equals the WGSL layout and emits a named diagnostic when it
  does not. Layout identity for vector members needs an alignment
  override on `@CStruct`. The owner decided on 2026-08-22 that
  subscript gains it (request R33). Kill evidence: a WGSL layout that
  no aligned C struct can reproduce. None is known for the planned
  schema set.
- **D5 — The backend loads at run time.** The facade resolves `wgpu*`
  symbols from `SUBSCRIPT_TYPEGPU_BACKEND_LIB` on `subscript_typegpu_create_instance`. One
  build configuration exists. Kill evidence: a platform where a
  shared library cannot be loaded from a path. None is in scope
  (macOS, Linux, Windows).
- **D6 — Three independent layout checks.** Golden vectors computed
  by upstream TypeGPU, naga's reported offsets for the emitted WGSL,
  and subscript's ship-tier `offsetof` verification of the
  `@CStruct` class. Kill evidence: two checks that cannot disagree by
  construction.
- **D7 — The kernel library is ordinary subscript with real bodies.**
  `Vec3f.add`, `dot`, `normalize`, and the rest have CPU bodies and
  are host-callable. The generator maps a fixed set of them to WGSL
  builtins and lowers everything else from the HIR. Kill evidence: a
  library function whose CPU body cannot match WGSL semantics and
  cannot be marked GPU-only.
- **D8 — Two verification lanes, one gate.** The gate lane runs on
  yawgpu Noop and proves structure, validation, layout, and both
  tiers. The live lane runs on a real adapter and proves numerics.
  Both are mandatory per phase exit. Only the gate lane runs on every
  test run. Kill evidence: a numeric defect class the live lane
  cannot catch.
- **D9 — A diagnostic names its cause and its owner.** An author
  defect reports as an author defect with the rule id. A generator
  defect reports as a generator defect. A backend diagnostic keeps
  its cause chain. Kill evidence: none. This is a policy.
- **D10 — The facade surface is what the API layer needs.** It is a
  policy-chosen subset of webgpu.h shaped for `subscript bind`. The
  subset grows by area, in the order the TypeGPU layer needs. Kill
  evidence: a TypeGPU feature that needs a facade call the subset
  lacks. The fix is one more policy row, not a change of policy.
- **D11 — The substrate generator is imported, then reshaped.** The
  proof of concept's generator (webgpu.yml × policy → facade, header,
  mirror, API layer, symbol table) enters without history. It keeps
  its model, plan, patterns, and API join. It loses every part that
  served a sibling crate, and it changes where §5 says. Kill evidence:
  a reshape item that costs more than the module it reshapes. Then
  that module is rewritten, and the rest stays imported.

## 4. Authoring surface (sketch)

The block specs own the details. This sketch fixes the shape so that
P0 and P1 can start. Names are provisional until `schema.md` and
`kernel.md` land.

```ts
import { Vec3f, v3f, Uniform, Storage, MutStorage, GlobalInvocationId,
         computePipeline } from "./typegpu";

@CStruct class Params { dt: f32; count: u32; }
@CStruct class Particle { pos: Vec3f; vel: Vec3f; }

class StepLayout {                       // one class per bind group
  params: Uniform<Params>;               // binding 0
  particles: MutStorage<Particle>;       // binding 1, array<Particle>
}

function step(res: StepLayout, id: GlobalInvocationId): void {
  if (id.x < res.params.count) {
    const p: Particle = res.particles[id.x];
    p.pos = p.pos.add(p.vel.scale(res.params.dt));
    res.particles[id.x] = p;
  }
}

export const stepPipeline = computePipeline(step, { workgroupSize: [64, 1, 1] });
```

What the generator does with this program:

1. It finds every `computePipeline(fn, desc)` and `renderPipeline(...)`
   call at module level. Each names a kernel entry function.
2. It walks the kernel's call graph. Every reachable function is GPU
   code and must stay inside the kernel subset. Everything else is
   host code and is left alone.
3. A layout class is a parameter whose fields are binding wrappers.
   Binding index is declaration order. Group index is parameter order.
4. It emits one WGSL module per pipeline declaration and one support
   module that holds the WGSL string, the layout entries, the
   workgroup size, and a typed bind-group factory per layout class.
5. The runtime library (`lib/typegpu.ts`) supplies the generic classes:
   `ComputePipeline<L0>`, `BindGroup<L>`, `Buffer<T>`, and the queue
   and encoder wrappers. Generated code is data plus thin factories.

Host code uses the result:

```ts
using buf = device.createBuffer<Particle>(particles);   // FixedArray<Particle, 1024>
using bg  = stepPipeline.createBindGroup0(device, { params: paramsBuf, particles: buf });
stepPipeline.dispatchThreads(encoder, bg, 1024);
```

## 5. Architecture

```
                 webgpu.yml ─┐
                policy.toml ─┼─ subscript-typegpu-webgpu-gen ─▶ crates/facade/src/generated.rs
                gpuweb IDL  ─┘        │                          crates/facade/subscript-typegpu.h
                                      │ subscript bind            lib/subscript-typegpu.generated.d.ts (mirror)
                                      └─────────────────────────▶ lib/webgpu.ts, lib/wire-enum-aliases.generated.d.ts
                                                                  crates/harness/src/native_symbols.generated.rs

program.ts ─check_program─▶ hir::Module ─ subscript-typegpu-gen ─▶ WGSL + support module (in memory)
                                                                    │ imports
lib/typegpu.ts  (TypeGPU layer, hand-written subscript) ────────────┘
      │ calls
lib/webgpu.ts   (WebGPU API layer, generated subscript)
      │ calls the mirror
crates/facade   extern "C" subscript_typegpu_* ─ function table ─▶ libyawgpu.dylib (loaded at run time)
```

Repository layout:

```
Cargo.toml                 workspace, [profile.dev] debug = "line-tables-only", no [features]
rust-toolchain.toml        channel pin, rustfmt, clippy
crates/facade/             subscript-typegpu-facade: lib + staticlib. Deps: libloading.
                           src/generated.rs (generated), src/runtime.rs (hand-written),
                           subscript-typegpu.h (generated)
crates/webgpu-gen/         subscript-typegpu-webgpu-gen: lib + bin. Deps: serde, serde_yaml,
                           toml, weedle2, subscript-bindgen. policy.toml lives here
crates/typegpu-gen/        subscript-typegpu-gen: lib + bin. Deps: subscript-compiler.
                           Dev-deps: naga, serde_json
crates/harness/            subscript-typegpu-harness: bin + one test target. Deps: facade,
                           typegpu-gen, subscript-compiler, subscript-codegen, libc
crates/window/             subscript-typegpu-window: the winit host (W-rules). Deps: the harness
                           crate for the session, winit, raw-window-handle, objc2 on macOS.
examples/window-triangle/  the windowed example, outside the program suite (W12)
lib/                       webgpu.ts (generated), wire-enum-aliases.generated.d.ts,
                           subscript-typegpu.generated.d.ts, typegpu.ts, typegpu-types.ts
programs/                  aNN-*.ts (API layer), bNN-*.ts (TypeGPU), .expected goldens,
                           bNN-*.<kernel>.wgsl goldens, xNN-*.ts (live)
specs/                     this plan, blocks/, tracking/
tools/                     gate.sh, hygiene.sh, regen.sh, live.sh, gen-layout-vectors.mjs
third_party/webgpu-headers git submodule, pinned (webgpu.yml, webgpu.h)
third_party/gpuweb         git submodule, pinned (the IDL)
```

External inputs: `SUBSCRIPT_TYPEGPU_BACKEND_LIB` names the backend
shared library. `SUBSCRIPT_TYPEGPU_UPSTREAM_DIR` names a TypeGPU
checkout for `tools/gen-layout-vectors.mjs`, which the owner runs.
Nothing else. Each crate builds at most two test executables: one
integration, one unit. Binaries build none.

### The substrate generator (imported)

What enters: `model.rs`, `policy.rs`, `naming.rs`, `plan.rs`,
`emit_header.rs`, `emit_rust.rs`, the sixteen `patterns/*` modules,
`idl.rs`, `api_model.rs`, `api.rs`, `lib.rs`, `policy.toml`, the
synthetic fixtures, and the tests that read only the policy, the
pins, or inline fixtures (about 135 of 178). The facade's hand-written
`runtime.rs` (slot table, deferred release, `callback_guard`) enters
with it.

What does not enter: `engine_header.rs`, `engine_facade_abi.rs`,
`measure_backend.rs`, the `chdir` in the driver, the surface host-only
policy row, and the 36 tests that served them.

What changes at import, each with its gate:

| Id | Change | Where | Gate |
|---|---|---|---|
| I4 | The proof of concept's four-letter C prefix becomes `subscript_typegpu_` (snake_case, from the yml names), its type prefix becomes `SubscriptTypegpu`, the header becomes `subscript-typegpu.h`, the mirror file name follows | `naming.rs`, `emit_header.rs` preamble | Regenerated outputs carry no old prefix token. A hygiene grep |
| I5 | The `extern "C"` block becomes generated shims over a function table that `runtime.rs` fills from `SUBSCRIPT_TYPEGPU_BACKEND_LIB` with `libloading` on `subscript_typegpu_create_instance`. Call sites do not change | `emit_rust.rs` and the 17 `rust_*_extern` renderers. The three hard-coded names (`wgpu{Pascal}Release` ×2, `wgpuAdapterInfoFreeMembers`) | `cargo tree -p subscript-typegpu-facade` lists `libloading` alone. `a01` runs with the variable set. The variable absent gives one stderr line that names the variable, then a null instance |
| I6 | The symbol table is emitted from the plan's export list, not by parsing `generated.rs`. `syn` and the `include_str!` of a sibling crate leave | `native_symbols.rs` | `syn` is not a direct dependency of any workspace crate (`cargo tree -e normal --depth 1`). The harness table equals the facade's export set, checked by a test that links both |
| I7 | The driver takes the repository root as an argument and writes the six outputs of this repository | `bin/` | `tools/regen.sh` regenerates. The regen test is byte-identical |
| I8 | Tests merge into `tests/main.rs`, one module per former file. The hard-coded pin canaries move into one `pins` module next to the pin table | `tests/` | One executable. The test count before and after is recorded |
| I9 | Every in-crate mention of the proof of concept's names is rewritten to this repository's names or to the upstream URL | comments, docs | Hygiene grep |
| I10 | Measured, then decided: if the CEnum alias list is computable from policy and yml alone, the two-pass facade generation collapses to one pass and `generate_with_cenum_aliases` leaves | `lib.rs`, `api.rs` | A before-and-after byte comparison of every output. If the outputs differ, the two-pass stays and the reason is recorded |

The policy file keeps its two-way validation (unknown, dead,
duplicate, unpoliced, invalid all abort). The subset it names at P0 is
the proof of concept's subset minus the surface family. Later phases
add policy rows by area.

The mirror is produced by `subscript_bindgen::generate_for_header`
in process. That call loads libclang at run time, so the regen test
needs libclang on the machine. The test prints one `pending` line and
passes when libclang is absent, the way the backend-absent case does.

### The facade

`runtime.rs` gains the loader: one `OnceLock` table, filled once,
each symbol resolved by name from the pinned function list. A missing
symbol names itself and the library path in the stderr line. Async
keeps the proof of concept's shape: `subscript_typegpu_x_begin(...)
-> future id`, `subscript_typegpu_future_status(instance, id) ->
i32`, `subscript_typegpu_x_take(instance, id) -> handle`,
`subscript_typegpu_instance_process_events`. Scripts never see
callbacks.

### The WebGPU API layer

`lib/webgpu.ts` is generated from the IDL join. It is the product's
first layer and the TypeGPU layer's substrate. The policy deviations
stay listed at the top of the file. Programs `aNN-*` exercise it
directly, one per area, with the same golden discipline as `bNN-*`.

### The TypeGPU generator

Crate `subscript-typegpu-gen` exposes `generate(files: &[SourceFile])
-> Result<Generated, Vec<Diagnostic>>`. `Generated` holds the WGSL
per pipeline and the support module source. The harness calls it in
process before the JIT. The CLI (`subscript-typegpu-gen gen
program.ts --lib lib/ -o dir/`) writes the same artifacts for
ship-tier users.

Layout engine: a pure module over a schema type tree. One layout,
the WGSL default, plus the uniform check of LY11 (which changes no
number and reports a violation). A C-layout function for the same
tree, with the R33 override. A diagnostic where they differ (D4).

Kernel emitter: type-directed, from `hir::Expr.ty`. From P4 on the
emitter carries TypeGPU's origin model — where a value lives
(uniform, storage, workgroup, private, local, argument) — because
barrier uniformity needs it. P2 carries no field it does not read.

### The harness

One binary. One test target with modules: `differential` (every `a`
and `b` program, both tiers, Noop, coverage counted), `wgsl_goldens`
(every `.wgsl` golden equals the generator's output and validates
under naga), `c_layout`, `rejections` (every fixture under
`crates/typegpu-gen/tests/fixtures/reject/` fails with its named
diagnostic), `traps`, `coverage`, `simulate`, `docs`, `uniformity`,
`api`, `live` (`#[ignore]`, `x` programs on a real adapter through
`ReloadSession`). The regeneration gates live in the generator
crates' own tests.

Ship tier: emit C, compile with the platform C compiler, link the
facade staticlib and the subscript runtime staticlib, run. The
facade staticlib carries `libloading` and nothing else, so the
`--release` build of it is seconds.

## 6. Constraints inherited

| Constraint (source) | Consequence here |
|---|---|
| No Proxy, Reflect, Symbol, eval (subscript, permanent) | All TypeGPU run-time machinery becomes generator output (D1) |
| Generics: explicit type arguments, no inference, constraints not enforced (subscript) | Every generic use is spelled `Buffer<Particle>`. Library functions that need a type per instantiation are methods on the type |
| `@CStruct` fields: sized numerics, bool, value classes, `FixedArray`, enums (subscript) | Vectors and matrices are `@CStruct` value classes. Layout identity needs alignment control — risk RC-1 |
| No typed arrays, no `ArrayBuffer` (subscript) | The host-to-GPU path is a `FixedArray<T, N>` or `T[]` of schema values. The facade copies from the value's bytes |
| No operator overloading (subscript) | `a.add(b)`, `a.scale(s)`, `a.dot(b)` (D7). Scalars use native operators |
| Imports: `./sibling` only, one global class namespace (subscript) | The runtime library is two files beside the program. Library class names carry no prefix. The harness injects them as `SourceFile`s |
| Ambient prelude is hardcoded in the checker (subscript) | The library is ordinary source, not a `.d.ts`. Only the mirror is ambient |
| Diagnostics: S001–S014 plus S100, line and column only (subscript) | Generator diagnostics are `S100` with this project's rule id in the message, rendered by `render_diagnostics` |
| `hir` items are `#[non_exhaustive]` (subscript) | Every `match` has a wildcard arm that emits a named "unsupported construct" diagnostic |
| Async is host-stepped (subscript Q34) | Buffer readback polls futures around `Context.suspend()` |
| `run_jit*` forks on Unix (subscript) | Gate lane: fork is fine on Noop. Live lane: `ReloadSession` |
| Noop validates WGSL through Tint and executes no compute (yawgpu) | Gate proves structure. Live proves numerics (D8) |
| `libyawgpu.a` does not link standalone (yawgpu) | The backend is the dylib plus `libtint_shim`, loaded at run time (D5) |
| f16 is storage-only (subscript) | f16 legal in schemas, illegal in kernel arithmetic |
| `bool` is not host-shareable (WGSL) | `bool` is illegal in a schema. A flag is `u32` |

## 7. Build-time budgets

Measured in P0 on the reference machine (Apple M2, 16 GB), warm
cargo cache, `CARGO_BUILD_JOBS=4`. Recorded in
`specs/tracking/build-time.md` with the command and the wall time.

| Measurement | Budget |
|---|---|
| Cold `cargo build --workspace --tests` | 8 minutes |
| Warm no-op `cargo test --workspace --no-run` | 5 seconds |
| Warm full gate (`tools/gate.sh`) after a one-line change in `crates/typegpu-gen/src/lib.rs` | 4 minutes |
| Warm full gate after a program-only change | 2 minutes |
| Number of test executables | 2 per crate at most: one integration, one unit. 4 at the P0 close, 6 from P1 |

A budget is a gate. If a phase needs more, the phase spec states the
new number and the cause before the work starts.

**How a budget is read (owner decision, 2026-08-23).** The budgets
exist for iteration speed. A gate that takes a few minutes is
acceptable. A row is red when it is about twice the previous
comparable row and the cause is in the diff, located by a
per-module measurement. A measurement under load is recorded with
the load condition and does not block a phase close. A budget
number is revised with its cause stated, never waited out.

## 8. Phases

Exit criteria are pre-registered. If a criterion is unreachable
without an upstream change or a scope change, the phase escalates. It
does not work around.

### P0 — seed and generator import (~15%)

Slice 1, the import. Workspace, pins, profile, `tools/`, the two
submodules, the substrate generator with reshape items I4 through I10,
the facade crate with `runtime.rs` and the loader, the six
regenerated outputs committed, the generator's test executable green.

Slice 2, the first program. `lib/webgpu.ts` as regenerated, the
harness with one test target, program `a01-smoke` (adapter, device,
buffer write, map read, dispose) through the API layer, no TypeGPU
generator yet.

Exit: (1) `a01` output is byte-identical across dev JIT, ship AOT,
and the committed golden on yawgpu Noop, with
`SUBSCRIPT_TYPEGPU_BACKEND_LIB` set. (2) The regeneration gate covers
every committed generated file, demonstrated red then green. (3)
`specs/tracking/build-time.md` records the five measurements of §7.
Each is inside budget, or the budget is revised with the cause
stated. (4) `cargo tree -p subscript-typegpu-facade` lists
`libloading` alone. `syn` is not a direct dependency of any
workspace crate. (5) The
workspace has no `[features]` table and no `build.rs`, proven by a
hygiene check. (6) I4 through I9 are green by their gates. I10 is
measured and decided. (7) The generator's test count before and
after I8 is recorded and equal.

### P1 — schemas and layout (~20%)

Vector and matrix value classes with real bodies, the layout engine
(default and uniform modes), the C-layout function, WGSL struct
emission, golden vectors from TypeGPU, naga cross-check, the layout
identity diagnostic, `Buffer<T>` with `write` from `FixedArray<T, N>`
and `read` into it.

R33 is at the pin from P0 slice 2 on. The layout engine computes C
layout with the override. The phase's first program proves identity
for every vector and matrix class, because subscript's own corpus
entry pins values and not alignment numbers.

Exit: (1) every committed golden vector passes. (2) `b01-layout`
lays out every vector and matrix class and prints the constants by
name, equal on both tiers and the golden. (3) naga's offsets equal
the engine's for every emitted struct. (4) The harness's C probe and
`value_class_layouts` equal the engine for every schema (LY16 Rev
1). (5) No schema in the corpus holds a padding field. (6) `Buffer<T>`
writes a `FixedArray<T, N>` and reads it back through R34, proven by
a program.

Status: see `specs/tracking/p1-layout.md`.

### P2 — compute kernels (~20%)

Pipeline declarations, layout classes, kernel discovery through the
call graph, the kernel subset (expressions, `let`/`const`, `if`,
`for`, `return`, swizzles, type-directed literals, the std set that
maps to WGSL builtins), support-module emission, `ComputePipeline`
and `BindGroup` in the runtime library, WGSL goldens, rejections with
named diagnostics and demonstrated reds.

Exit: (1) `b02-vecadd`, `b03-saxpy-uniform`, `b04-particles` are
gate-green on both tiers, with WGSL goldens. (2) `x01`–`x03` print
`PASS` on a real adapter with host-computed expectations. (3) Every
rejection rule has a red fixture. (4) Every generator diagnostic names
its rule id and its owner (D9). (5) Build-time budgets hold.

### P3 — render (~15%)

Vertex and fragment entries, IO structs with `@location` and
`@builtin(position)`, vertex layouts from schemas, `RenderPipeline`,
render pass encoding, indexed and instanced draws.

Exit: (1) `b06-render`, `b07-draw-variants`, and
`b08-render-bindings` gate-green with WGSL goldens. (2)
`x05-live-triangle`, `x06-live-draw-variants`, and
`x07-live-render-uniform` print `PASS` against the host rasterizer
(RN14, RN17). (3) Every RN16 rejection has a red fixture. (4)
Budgets hold.

### P4 — kernel depth (~10%)

`while`, `switch`, `break`, `continue`, helper functions with an
acyclic call graph, module constants, private and workgroup
variables, atomics, barriers under uniform control flow.

Exit: (1) `b09-kernel-depth` (K18, K19) and `b10-workgroup` (K20,
K21, K22, K23) gate-green with WGSL goldens. (2) `x08-live-reduction`
sums across several workgroups with a workgroup array, a barrier,
and an atomic add, and prints `PASS` — a result only correct
barriers and atomics produce. (3) `x09-live-switch` decides a
result through `switch`, `break`, and `continue`, and prints
`PASS`. (4) Every K24 rejection has a red fixture, and a barrier
under non-uniform control flow is a K22 generator diagnostic with a
red fixture (corrected, §10 C1). (5) Budgets hold.

### P5 — textures and samplers (~10%)

Texture and sampler bindings, storage textures, `textureLoad`,
`textureStore`, `textureSampleLevel` in compute, `textureSample` in
fragment only, multiple bind groups.

Exit: (1) `b11-texture` gate-green with its WGSL golden (TX6). (2)
`x10-live-texture` and `x11-live-fragment-sample` print `PASS`
against the host sampling body (TX7). (3) Every TX8 rejection has a
red fixture. (4) Budgets hold.

### P6 — ergonomics and diagnostics (~5%)

Buffer range writes and field patches in elements, guarded dispatch
(`dispatchThreads`), timestamp queries opt-in, the diagnostics sweep.

Exit: (1) EG1 typed resources used by one `b` program. (2) EG4's
list is closed. (3) EG5's Dawn run is recorded with its decision.
(4) EG7's sweep finds no diagnostic without a rule id, an owner, and
a fixture. (5) `README.md` and the tutorial exist with their quote
gate. (6) Budgets hold.

### P7 — the CPU lane (~5%, optional, owner decides at P6 close)

Because the library has real bodies (D7), a kernel can run on the
dev-tier JIT with its wrappers holding host data. Contract draft:
`specs/blocks/cpu-lane.md` (CL1–CL6): `simulateCompute` loops the
invocations on the host, barrier kernels are out until a phased
revision, the live programs take the kernel body as their oracle.

Exit: `x01`–`x04` and `x09` use `simulateCompute` as their oracle
and print `PASS` on Metal. One `b` program's host golden is
committed and compared on both tiers. `CL2` has a fixture. Budgets
hold. The lane is a gate module, not a new executable.

### P8 — library breadth, slice 1 (readback, vector builtins, swizzles)

The plan's phases are executed. P8 starts the breadth work that
`docs/from-typegpu.md` lists under "Not in subscript-typegpu", in
the order a TypeGPU user meets the gap. Slice 1 is three items.
Contracts: `buffer.md` Rev 1 (BF9–BF11) and `kernel.md` Rev 5
(K10 Rev 1, K25–K28).

1. `Buffer<T>.read` and `readOne` own the staging buffer (BF9,
   BF10). `b12-readback` is the gate program.
2. Componentwise vector builtins, comparisons with `Vec*b`, and
   `select` (K25, K26). `b13-vector-builtins` runs every new method
   once on the host through `simulateCompute`, on a two-, a three-,
   and a four-wide receiver, and prints one `host:` line (CL4) with
   the method families joined.
3. In-order swizzles and the `From`/`Splat` factories (K27).
   `b13` covers them.

`x14-live-vector-builtins` dispatches one kernel that applies every
K25–K27 method to inputs with exact `f32` results where the WGSL
builtin is exact (`abs`, `floor`, `ceil`, `fract`, `sign`, `min`,
`max`, `clamp`, `step`, `select`, the comparisons, the swizzles, the
factories) and compares byte-exactly against `simulateCompute`. The
transcendental methods (`sqrt`, `exp`, `log`, `sin`, `cos`, `tan`,
`pow`, `mix`, `smoothstep`, `distance`, `normalize`-based `reflect`,
`refract`, `faceForward`) compare within a relative tolerance of
`2^-16`, stated in the program header. `x14` reads its result
through `read` (BF11).

Exit: (1) `b12` and `b13` goldens are byte-identical on both tiers,
and `b13`'s `.wgsl` golden validates. (2) `x14` prints `PASS` on Metal
and on Dawn. (3) The K10 Rev 1 table test exists and a scratch
mismatch is recorded red. (4) K28 and BF10 fixtures are red. (5)
`docs/from-typegpu.md` "Vector math", "Buffers", and the two "Not
in" lists state the new surface, and the quote gate is green. (6)
Budgets hold, measured and recorded.

### P8 — library breadth, slice 2 (shells, guarded dispatch, indirect, index format, cull)

The proof of concept carried five generator features that slice 1
did not: WGSL shells with raw declarations, guarded dispatch through
a hidden binding, indirect dispatch and draw wrappers, an index
format on the render entry, and a cull-mode golden. Slice 2 builds
each for this library's shapes. Contracts: `kernel.md` Rev 7
(K29–K31), `pipeline.md` Rev 5 (PI15–PI18), `render.md` Rev 3
(RN18–RN19, RN16 Rev 1).

Design differences from the proof of concept, stated once: a shell's
subscript body is the host implementation and the CPU lane runs it
(the proof of concept required a dead `return`); a `naga` error
inside a shell is attributed by name through recorded line spans;
the guard covers three axes and rides the layout spec as a `guard`
entry, so the runtime owns the hidden buffer without a generated
class; render indirect draws stay in the API layer; the indirect
argument blocks are `@CStruct` schemas.

Programs: `b14-wgsl-shell`, `b15-guarded-dispatch`, `b16-indirect`,
`b17-index-cull`, `x15-live-shell` (GPU shell result against the
host body through `simulateCompute`), `x16-live-guarded` (no author
guard, thread count below the workgroup multiple, sentinel slots
untouched), `x17-live-indirect` (dispatch and draw arguments from
`PI17` schemas), `x18-live-cull`.

Exit: (1) the four `b` goldens are byte-identical on both tiers and
their `.wgsl` goldens validate. (2) `x15`–`x18` print `PASS` on
Metal and on Dawn. (3) K28 Rev 1, PI18, and RN16 Rev 1 fixtures are
red, including one `naga` error attributed to a shell by name. (4)
`docs/from-typegpu.md` states the new surface, and the quote gate is
green. (5) Budgets hold per §7.

### P9 — the window host (after P8 slice 2)

The proof of concept ran a windowed triangle through a host-owned
engine and three pushed entries. P9 builds that split as a crate:
`crates/window`, a winit host that owns the window, the surface, the
instance, the device, and the loop, and calls `init`, `frame`, and
`shutdown` on a script. Contract: `window.md` (W1–W13), `facade.md`
L14, `facade-generator.md` F23. Design differences from the proof of
concept, stated once: the surface family is generated by a policy
row kind, never hand-written; the ten symbols resolve on demand so
a backend without them still loads; the frame statuses `Outdated`
and `Timeout` skip a frame instead of ending the run; the host
prints one message on failure and exits.

Open after P9: the ship tier for the window host, Linux surfaces
(Wayland, X11), more than one window event, and a surface-format
list on the render spec.

Exit: W13 green. `tools/window.sh examples/window-triangle/main.ts`
on Metal and on Dawn: the window opens, the triangle draws, space
advances the clear color, resize reconfigures without a message,
close prints `window:frames=<n>`. The cold build increment is
recorded.

### P10 — the example ports (after P9)

TypeGPU's documentation examples (MIT, Software Mansion) are ported
as far as the library reaches, per `examples.md` (EX1–EX7). A port
reimplements the idea (EX6). The survey of all 77 upstream examples
and their feature needs is `specs/tracking/p10-examples.md`.

Slice 1, no new library surface. Headless: `matrix-multiplication`,
`matrix-next` (workgroup tiles against a CPU reference), `dispatch`
(thread-count checks). Windowed: `triangle`, `gradient-tiles`,
`square`, `xor-dev-centrifuge-2`, `confetti`, `boids`,
`fluid-double-buffering`. `tools/example.sh` runs a headless example
through the harness dev lane.

Slice 2, two library modules. `lib/typegpu-sdf.ts` and
`lib/typegpu-noise.ts`: pure functions and module constants in the
kernel subset with real host bodies, K10 untouched (a kernel calls
them as K2 helpers). Ports: `ray-marching`, `caustics`,
`smoky-triangle`, `vaporrave`, `fluid-with-atomics` (the brush on
the key scalar, EX7), and `prng-cpu-gpu` as a headless differential
check of the noise module.

Slice 3, on the P11 slice 1 features. `examples/clouds/main.ts`:
the host fills an `rgba8unorm` noise texture from `perlin3d` on the
CPU, uploads it with `writeTexturePixels` (TX9), and the fragment
raymarches layered density from the sampled noise. The strip
reductions leave: `confetti` draws its card as a four-vertex
`triangle-strip` (RN20), and `fluid-double-buffering` draws the
upstream four-vertex strip quad. The affected comments change with
the code.

Deferred, by the missing feature (the survey ranks these gaps):
read-only storage textures (slime-mold, game-of-life, the jump-flood
pair), image decode and byte-to-texture upload (blur, image-tuning,
clouds), pointer input (stable-fluid, the drawing examples), depth
(the 3D set), blending and multisample (the geometry set). Webcam,
network, ONNX, WebGL, react, and three.js examples are out of scope.

Exit per slice: every ported example compiles in the gate (EX4), a
headless example prints its `check:` line `pass` on both tiers, a
windowed example runs `--frames 120` to `window:frames=120` on Metal
and on Dawn, and the owner's visual run is recorded. Comments obey
EX2, written or revised by the planning side. Budgets hold.

### P11 — the low-cost feature gaps (interleaves with P10)

The P10 survey ranked the missing features by the examples they
unlock. P11 takes the low-difficulty half. Measured base: the API
layer already carries `GPUQueue.writeTexture` over `u8[]`,
`GPUStorageTextureAccess` with `read-only` and `read-write`, the
`triangle-strip` topology, and `stripIndexFormat`. Each slice
revises its block before implementation.

Slice 1 — texture upload and strip (very low). A TypeGPU-layer
helper writes a `Vec4f[]` or `u8[]` image into a sampled texture
through `writeTexture` (TX Rev), with a `b` program and a live
program that samples an uploaded gradient. One render program with
`topology: "triangle-strip"` proves the pass-through (RN). Unlocks
the `clouds` port and removes the strip reductions of EX7 ports.

Slice 2 — pointer input (low). W2 Rev: `frame` gains the pointer
position in pixels and a button bit set, from winit events, one
state slot as the key (W3). Unlocks `fluid-with-atomics` unreduced,
`stable-fluid`, the drawing examples, and `oklab`'s probe.

Slice 3 — read-access storage textures (low-medium). `StorageTexture2d`
gains a read-only and a read-write form (TX Rev), the generator
emits `texture_storage_2d<F, read>` and `read_write`, the layout
entry carries the access, the host bodies read. Risk named in
advance: backend support. The slice starts with a probe program on
yawgpu Metal and Dawn; a yawgpu gap becomes an IB-rule escalation
like the backend request. Unlocks `slime-mold`, `game-of-life`
(without `textureGather`), the jump-flood pair, and the
radiance-cascades docs pair later.

Slice 4 — blending (medium). `RenderPipelineSpec` gains a blend
member the runtime passes through, and the host rasterizer blends
the compared pixels. Unlocks `box-raytracing`, `genetic-racing`,
`wind-map` partially. Depth, multisample, integer textures, texture
arrays, and 3D textures stay in the deferred list with their
dependents.

Exit per slice: the block Rev, the programs and goldens on both
tiers, the live lane on Metal and Dawn, the rejection fixtures, the
unlocked P10 example ported, budgets recorded.

## 9. Risk register

| Id | Risk | Mitigation / trigger |
|---|---|---|
| RC-1 | ~~`@CStruct` has no alignment control~~ **Closed 2026-08-22.** R33 landed in subscript at `ba6aa2e` (compiler.md §62): `@CStruct({ align: N })`, `N` in `{2, 4, 8, 16}`, both tiers, `offsetof` proof for `Vec3f` 16/16, `Mixed` 32/16, `Mat3x3f` 48/16, `Vec2f` 8/8, measured on clang and MSVC | P0 slice 2 re-pins subscript to `ba6aa2e`. P1's layout gate is the end-to-end check, because `a141` pins values, not alignment numbers |
| RC-2 | ~~`computePipeline(fn, desc)` needs a function value of a named function and a descriptor literal in one expression~~ **Closed 2026-08-22.** The checker accepts `computePipeline<L>(step, { workgroupSize: [64, 1, 1] })` with a function-typed generic parameter | — |
| RC-3 | A generic runtime class (`Buffer<T>`) needs per-`T` size and layout facts with no inference | The generator emits one `Layout` const per schema and the class takes it as a constructor argument |
| RC-4 | `hir` is `#[non_exhaustive]`. A subscript re-pin adds a construct the emitter does not know | Wildcard arms emit a named diagnostic. The re-pin procedure runs the full gate |
| RC-5 | Run-time loading fails on a platform or with a backend that needs a companion library (`libtint_shim`) | The loader reports the `dlopen` error text and the path it tried. The live lane checks `otool -L` before a Metal run |
| RC-6 | The one-binary harness serializes tests that the old layout ran in parallel | Programs run in parallel inside one process where the tier allows. Measure in P0 |
| RC-7 | 16 GB memory with SWC and Cranelift in one link | `CARGO_BUILD_JOBS=4`, one cargo command at a time, `line-tables-only`. Measure in P0 |
| RC-8 | Library class names share the program's global namespace | Library names are short and documented. A collision is an author diagnostic from the checker, not from this project |
| RC-9 | Methods with real bodies let an author call a GPU-only intrinsic on the host | GPU-only intrinsics have a body that traps with a named message. The generator lowers the call |
| RC-10 | The live lane cannot run in a sandboxed agent shell | `tools/live.sh` is owner-run. `CLAUDE.md` forbids sandboxed device runs |
| RC-11 | The mirror regen test needs libclang at run time (`subscript-bindgen` loads it) | Loud `pending` when absent. The owner's machine has it through Xcode |
| RC-12 | The imported generator's hard-coded pin canaries (IDL block count, policy accounting tuple) need a manual edit at every re-pin | They move into one `pins` module (I8). The re-pin procedure lists them |
| RC-13 | A reshape item (I5, I10) costs more than the module it touches | D11's kill evidence. The module is rewritten, the rest stays imported |
| RC-15 | ~~The program imports its support module before it exists~~ **Closed 2026-08-22.** R35 landed at subscript `bb9dadc`: `check_program_with` poisons one missing module and records the imported names | SC1a Rev 1. The stub and the import scan leave |
| RC-14 | ~~No subscript construct yields the bytes of a value class~~ **Closed 2026-08-22.** R34 landed at subscript `bb9dadc`: `Context.bytesOf`, `bytesInto`, `fromBytes`, padding zeroed on both tiers | `buffer.md` BF-rules. P1 slice 2 |

## 10. Corrections

A killed decision from §3 or a changed exit criterion is recorded
here with the evidence, the date, and the corrected claim.

- **C1 — naga does not enforce barrier uniformity (2026-08-23).**
  P4 exit criterion 4 said a barrier under non-uniform control flow
  "fails the harness with naga's diagnostic". Measured: naga 29
  accepted `x08`'s barrier after a non-uniform early return.
  Corrected claim: the generator enforces uniform placement (K22 Rev
  1), and the criterion names a generator diagnostic. Evidence:
  `specs/tracking/p4-kernel-depth.md`. The first record of this
  correction also said that yawgpu's Tint compiled the module and
  Metal read zeros; C4 corrects that.
- **C2 — D4 cannot reproduce a tail-packed WGSL struct
  (2026-08-23).** WGSL places a scalar after a `vec3` member in the
  vector's tail (`{ p: vec3f; x: f32 }` is 16 bytes). R33 only raises
  an alignment, so the C layout puts `x` at 16 and SC9 rejects the
  schema. D4 stands for every schema whose members start at their
  natural WGSL offsets, and the limitation is the rule: an author
  reorders or pads such a schema. Evidence: the retrospective
  review at `82175df`.
- **C3 — A spec revision a tracking entry cites must exist
  (2026-08-23).** Four closes cited a revision that had not landed
  in the block (P1 exit criteria, PI8 Rev 1, PI9 Rev 1, PI8 Rev 2),
  because a failed text replacement went unnoticed. Rule: before a
  tracking entry cites `<rule> Rev <n>`, the block carries that
  marker. Evidence: the retrospective review at `82175df`.
- **C4 — Both backends rejected the non-uniform barrier; the
  program could not see it (2026-08-23).** The P4 record said
  yawgpu's Tint compiled `x08`'s module and Metal read the workgroup
  memory as zero, and it marked Dawn's rejection *(docs)*. Measured
  with a validation error scope around the creation calls: yawgpu
  (Metal) and Dawn both fail `createShaderModule` with
  "'workgroupBarrier' must only be called from uniform control
  flow", and the compute pipeline is then an error object. The
  zeros came from an invalid pipeline, not from workgroup memory.
  Corrected claims: no backend defect exists, K22 Rev 1 stays (the
  generator rejects first, with a named rule), and every program
  wraps its creation calls in a validation error scope so a backend
  rejection is a visible failure (PI14). Evidence:
  `specs/tracking/p4-kernel-depth.md`.

## 11. Open questions for the owner

1. ~~**Alignment control in `@CStruct`** (RC-1).~~ Landed
   2026-08-22 as R33 at subscript `ba6aa2e`.
2. **The kernel marker** (RC-2). The plan uses a pipeline declaration
   as the marker. A `'use gpu'` directive is valid TypeScript and
   TypeGPU's own spelling, but subscript's treatment of a string
   expression statement is not measured. P0 measures it.
3. **Library packaging.** Flat `./sibling` imports force the two
   library files beside every program. A subscript library path is a
   language-side change. Not blocking through P7.
