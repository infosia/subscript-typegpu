# Block: pipeline (PI-rules)

P2 contract. Rev 0, 2026-08-22. Plan §3 D1, D3, D10 and §4 govern
this block. Kernels are `kernel.md`. The runtime classes live in
`lib/typegpu.ts`.

## The declaration

- **PI1 — A pipeline is a module-level `const`.**
  `export const stepPipeline = computePipeline<StepLayout>(step,
  { workgroupSize: [64, 1, 1] });`. `computePipeline` is a library
  generic function in `lib/typegpu.ts`, with a real body that returns
  a `ComputePipelineSpec` (the kernel name and the workgroup size).
  The generator finds every call to `computePipeline` at module
  level, reads the kernel from the `FuncRef` argument, and reads the
  workgroup size from the descriptor literal. A non-literal size, a
  non-`FuncRef` kernel, or a declaration inside a function is a
  diagnostic.
- **PI2 — The kernel signature by layout count.** `computePipeline<L0>`
  takes `(res: L0, ctx: ComputeInvocation) => void`.
  `computePipeline2<L0, L1>`, `computePipeline3<L0, L1, L2>`, and
  `computePipeline4<L0, L1, L2, L3>` take the two-, three-, and
  four-layout forms. Group index is parameter order. The names are
  provisional until the RC-2 probe (plan §9) confirms that the
  checker accepts a named function as a generic function argument
  typed by a function type. The fallback is a string kernel name.
- **PI3 — A layout class is a plain class of binding fields.** Every
  field is a binding wrapper (PI5). Binding index is declaration
  order from 0. No other member is legal. The class is not
  `@CStruct` and not `@Descriptor`. A layout class is never
  instantiated by the author.
- **PI4 — `ComputeInvocation` carries the builtins.** A library
  class with `globalId: Vec3u`, `localId: Vec3u`, `workgroupId:
  Vec3u`, `numWorkgroups: Vec3u`, `localIndex: u32`. The generator
  emits a `@builtin` parameter for each field the kernel reads and
  no other. The host never constructs one.

## Binding wrappers

- **PI5 — Three wrappers in P2.** `Uniform<T>`: `get(): T`. WGSL
  `var<uniform> name: T`. `Storage<T>`: `[index: u32]: T` readonly
  through `get(i)`, `length(): u32`. WGSL `var<storage, read> name:
  array<T>`. `MutStorage<T>`: `[index: u32]: T` through `get(i)` and
  `set(i, v)`, `length(): u32`. WGSL `var<storage, read_write>`.
  `T` is a schema class, a library vector or matrix, or `f32`,
  `i32`, `u32`. The wrappers have real bodies over a `T[]` so a
  kernel runs on the host (P7).
- **PI6 — Binding access emits the variable.** `res.params.get()` →
  `params` (a copy into a `let`). `res.particles[i]` → `particles[i]`.
  `res.particles[i] = v` → `particles[i] = v;`.
  `res.particles.length()` → `arrayLength(&particles)`. `res` itself
  never appears in WGSL. A layout class passed anywhere but the
  kernel's parameter is a diagnostic.
- **PI7 — Uniform schemas.** A schema behind `Uniform<T>` obeys
  LY11 (SC10). The generator runs the uniform check on every such
  schema.

## Generated artifacts

- **PI8 — The support module.** For each declaration `<name>` the
  generator emits: `<name>_WGSL: string`, `<name>_ENTRY: string`,
  three `u32` constants `<name>_WORKGROUP_X`, `_Y`, `_Z`, and, per
  layout class
  `L` of the declaration, `<name>_LAYOUT<g>: BindGroupLayoutSpec`
  (a library `@Descriptor` with an entries array of `{ binding,
  visibility, kind, minBindingSize }`) plus a typed factory
  `create<Name>BindGroup<g>(device, layout, resources:
  <L>Resources): GPUBindGroup` where `<L>Resources` is a generated
  `@Descriptor` with one `GPUBuffer` field per binding, named as
  the layout field. Schema constants per SC11 continue.
- **PI9 — The runtime class.** `lib/typegpu.ts` exports
  `ComputePipeline` with `create(device, wgsl, entry, layouts:
  BindGroupLayoutSpec[]): ComputePipeline` (a free function
  `createComputePipeline`, because subscript has no static methods),
  `bindGroupLayout(g): GPUBindGroupLayout`, `dispatch(encoder,
  groups: GPUBindGroup[], x, y, z)`, `dispatchThreads(encoder,
  groups, count)` (rounds up by the workgroup size), and `dispose()`.
  It builds on `lib/webgpu.ts` only.
- **PI10 — The WGSL golden.** Per K16. The harness module
  `wgsl_goldens` compares every `<stem>.<name>.wgsl` and validates
  it with naga.

## Programs

- **PI11 — Gate programs print structure.** A `b` program creates
  the pipeline through the generated constants, creates buffers
  sized from schema constants, creates bind groups through the
  typed factory, encodes a dispatch, submits, and prints markers,
  constants by name, and the WGSL line count. Noop executes no
  shader, so a `b` program never prints a result buffer.
- **PI12 — Live programs compute.** An `x` program writes input
  data, dispatches on a real adapter, reads back, compares with a
  host computation, and prints `PASS`. Until R34 lands the input
  bytes are built with `Math.f32ToBits` in the program, and the
  output is decoded the same way. After R34 the program uses
  `Context.bytesOf` and `Context.fromBytes`.

## Rejections

- **PI13 — Each rejection is a named diagnostic with a red fixture.**
  The P2 set: a layout field that is not a wrapper, a wrapper `T`
  outside PI5, a non-literal workgroup size, a kernel that is
  `async`, a kernel with a non-`void` return, a declaration inside a
  function, a layout class used as a local, a uniform schema that
  violates LY11.
