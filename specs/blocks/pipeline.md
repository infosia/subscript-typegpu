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
  four-layout forms. Group index is parameter order. Measured
  2026-08-22: the checker accepts a named function as a generic
  function argument typed by a function type (RC-2 closed). The
  generator reads the layout classes from the kernel's parameter
  types and checks that each equals the declaration's type argument.
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

- **PI5 — The buffer wrappers.** (P5 adds the texture wrappers, TX1.) `Uniform<T>`: `get(): T`. WGSL
  `var<uniform> name: T`. `Storage<T>`: `[index: u32]: T` readonly
  through `get(i)`, `length(): u32`. WGSL `var<storage, read> name:
  array<T>`. `MutStorage<T>`: `[index: u32]: T` through `get(i)` and
  `set(i, v)`, `length(): u32`. WGSL `var<storage, read_write>`.
  `T` is a schema class, a library vector or matrix, or `f32`,
  `i32`, `u32`. The wrappers have real bodies over a `T[]` so a
  kernel runs on the host (P7).
- **PI6 — Binding access emits the variable.** `res.params.get()` →
  `params` (a copy into the local the author declared, `var` for a
  value class per K8). `res.particles[i]` → `particles[i]`.
  `res.particles[i] = v` → `particles[i] = v;`.
  `res.particles.length()` → `arrayLength(&particles)`. `res` itself
  never appears in WGSL. A layout class passed anywhere but the
  kernel's parameter is a diagnostic.
- **PI7 — Uniform schemas.** A schema behind `Uniform<T>` obeys
  LY11 (SC10). The generator runs the uniform check on every such
  schema.

## Generated artifacts

- **PI8 — The support module's exports.** Rev 2, 2026-08-23. For
  each declaration `<name>` the generator emits `<name>_WGSL:
  string`, `<name>_ENTRY: string`, three `u32` constants
  `<name>_WORKGROUP_X`, `_Y`, `_Z`, `<name>_HOST_RUNNABLE: boolean`
  (CL2), and, per layout class of the declaration,
  `<name>_LAYOUT<g>: BindGroupLayoutSpec` (a library `@Descriptor`
  with an entries array of `{ binding, visibility, kind,
  minBindingSize }`, every size from the layout engine, a type the
  engine cannot size is a diagnostic). Per layout class `L` it
  emits `@Descriptor class LResources` and the factories
  `createLResources(...)` and `create<Name>BindGroup<g>(device,
  pipeline, resources)` (EG1). Schema constants per SC11 continue.
  The discovery check (SC1a Rev 1) poisons every generated name. Rev
  0 and Rev 1 of this rule allowed constants only; the record of
  those revisions is `specs/tracking/p2-kernels.md` and
  `p6-ergonomics.md`.
- **PI9 — The runtime class.** `lib/typegpu.ts` exports
  `createComputePipeline(device, wgsl, entry, layouts:
  BindGroupLayoutSpec[], workgroup: FixedArray<u32, 3>):
  ComputePipeline` (a free function, because subscript has no static
  methods), `ComputePipeline.bindGroupLayout(g): GPUBindGroupLayout`,
  `dispatch(encoder, groups: GPUBindGroup[], x, y, z)`,
  `dispatchThreads(encoder, groups, x, y, z)` (rounds each axis up by
  its workgroup size), `dispose()`, and `createBindGroup(device,
  layout: GPUBindGroupLayout, spec: BindGroupLayoutSpec, buffers:
  GPUBuffer[]): GPUBindGroup` (positional, binding order equals
  declaration order, a count mismatch traps with both counts). It
  builds on `lib/webgpu.ts` and `lib/typegpu-types.ts` only.
- **PI10 — The WGSL golden.** Per K16. The harness module
  `wgsl_goldens` compares every `<stem>.<name>.wgsl` and validates
  it with naga.

## Programs

- **PI11 — Gate programs print structure.** A `b` program creates
  the pipeline through the generated constants, creates buffers
  sized from schema constants, creates bind groups through the
  typed factory or the positional call, encodes a dispatch, submits,
  runs `simulateCompute` when the pipeline is host-runnable and
  prints its `host:` line (CL4), and prints markers,
  constants by name, and the WGSL line count. Noop executes no
  shader, so a `b` program never prints a result buffer.
- **PI12 — Live programs compute.** An `x` program writes input
  data through `Context.bytesOf` (BF7), dispatches on a real
  adapter, reads back through `Context.fromBytes`, compares with a
  host computation (`simulateCompute` where CL3 applies), and prints
  `PASS`.

## Rejections

- **PI13 — Each rejection is a named diagnostic with a red fixture.**
  Rev 1, 2026-08-22. The generator's set: a layout field that is not
  a wrapper, a wrapper `T` outside PI5, a non-literal workgroup size,
  a declaration inside a function, a layout class used as a local, a
  uniform schema that violates LY11, a declaration type argument
  that differs from the kernel's parameter type (PI2). Two cases
  never reach the generator: an `async` kernel and a kernel with a
  non-`void` return fail subscript's checker first, because the
  declaration's function type does not match. Their fixtures assert
  the checker's diagnostic. The generator never relabels a checker
  diagnostic and never reads the program text.
