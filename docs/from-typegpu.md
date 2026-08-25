# subscript-typegpu for TypeGPU users

This document compares [TypeGPU](https://github.com/software-mansion/TypeGPU)
with subscript-typegpu, topic by topic, with code from both sides.

The two libraries share concepts and names. They do not share source
code. A TypeGPU program does not compile as a subscript program, and a
subscript-typegpu program does not run in a browser. The reason is one
design decision: TypeGPU builds schemas and generates WGSL at run time,
and subscript-typegpu does both before the program runs.

Every pair of code blocks shows the same step in both libraries.
The subscript-typegpu side is quoted from `programs/`. The TypeGPU
side shows how TypeGPU 0.12 writes that same step. The TypeGPU code
is illustrative. This repository does not run it.

## Summary

| Topic | TypeGPU | subscript-typegpu |
|---|---|---|
| Language | JavaScript or TypeScript | [subscript](https://github.com/infosia/subscript) |
| Schema | `d.struct({ ... })`, a run-time object | `@CStruct class`, a compile-time declaration |
| Layout | computed at run time, `d.sizeOf(T)` | computed by the generator, `T_SIZE` and `T_STRIDE` constants |
| Kernel | a function with `'use gpu'` | a plain function named by `computePipeline<L>(fn, spec)` |
| WGSL | generated at run time from a compacted AST | generated before the program runs — one readable `.wgsl` file per pipeline |
| Bindings | `tgpu.bindGroupLayout({ ... })`, access by `layout.$.name` | a layout class with buffer, texture, and sampler binding fields |
| Buffer data | JavaScript objects, converted by TypeGPU | `Context.bytesOf<T>(value)`, the bytes of the value |
| Async | `Promise` | `await` over host-stepped futures |
| Lifetime | garbage collection and `root.destroy()` | `using` and `dispose()` |
| Errors | exceptions | `null` and `false` returns, error scopes, and traps that name a rule |
| CPU execution | call the function | `simulateCompute<L>(fn, resources, spec, workgroups, HOST_RUNNABLE)` |
| Execution targets | a browser or a WebGPU runtime for JavaScript | a JIT tier and a C tier, over a webgpu.h library chosen at run time |

## Initialization

TypeGPU creates a root. The root owns the device.

```ts
import { tgpu } from 'typegpu';

const root = await tgpu.init();
```

subscript-typegpu has no root. A program requests the adapter and the
device through `lib/webgpu.ts`, which follows the WebGPU JavaScript API.
Each request returns `null` on failure, and the program checks it.

```ts program=programs/b04-particles.ts
export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
```

Differences:

- `gpu` is an exported constant. There is no `navigator`.
- `await` yields to the host runtime through `Context.suspend()`. There is no
  JavaScript promise object and no callback. The API layer polls the
  facade's futures in a loop and yields at each poll.
- The adapter and the device are handles with `dispose()`. A program
  binds them with `using` so that the block end releases them.

## Data schemas

TypeGPU declares a schema as a value. The schema is a function that
constructs instances.

```ts
const Particle = d.struct({
  pos: d.vec3f,
  vel: d.vec3f,
});
```

subscript-typegpu declares a schema as a class with the `@CStruct`
decorator. The fields carry subscript types. The constructor is
ordinary code.

```ts program=programs/b04-particles.ts
@CStruct
class Particle {
  pos: Vec3f;
  vel: Vec3f;

  constructor(pos: Vec3f, vel: Vec3f) {
    this.pos = pos;
    this.vel = vel;
  }
}
```

Differences:

- `f32`, `u32`, and `i32` are distinct subscript types. TypeGPU uses
  the JavaScript `number` and the schema decides the WGSL type.
- `Vec3f` is a class from `lib/typegpu-types.ts`, declared
  `@CStruct({ align: 16 })`, so its C alignment equals the WGSL
  alignment. Its C size is 16 and its WGSL size is 12. `d.vec3f` is a
  schema object.
- There is no `d.Infer`. The class is the type.
- `@CStruct({ align: N })` raises the alignment of a class, the way
  `d.align` does for one field. `lib/typegpu-types.ts` declares `Vec3f`
  with `align: 16`. There is no `d.size`.

## Layout

TypeGPU computes a schema's size and alignment when a program first
asks for them.

```ts
d.sizeOf(Particle) // 32
d.alignmentOf(Particle) // 16
```

subscript-typegpu computes every layout before the program runs.
The generator emits the values as constants in a support module
`<program>.typegpu.ts`, and the program imports them.

```ts program=programs/b04-particles.ts
import {
  Particle_SIZE,
  ParticleLayoutResources,
  SimParams_SIZE,
  createParticleLayoutResources,
  createParticlesBindGroup0,
  particles_ENTRY,
  particles_HOST_RUNNABLE,
  particles_LAYOUT0,
  particles_WGSL,
  particles_WORKGROUP_X,
  particles_WORKGROUP_Y,
  particles_WORKGROUP_Z,
} from "./b04-particles.typegpu";
```

Differences:

- `T_SIZE` is the struct size. `T_STRIDE` is the array stride. Both
  are integer constants in generated code.
- The support module is not a file you keep. The runner —
  `subscript-typegpu-harness dev <program>` or `ship <program>` —
  generates it in memory before the program runs.
- The generator computes the C layout and the WGSL layout of every
  schema and rejects a difference (rule `SC9`).
- One WGSL layout has no C form: a scalar after a `vec3` member shares
  the vector's tail in WGSL. The generator rejects such a schema. The
  author adds an alignment override to the field's class, or reorders
  the fields.

## Buffers

TypeGPU creates a buffer from a schema and an initial value. It
converts JavaScript values to bytes.

```ts
const input = root
  .createBuffer(d.arrayOf(ReductionValue, count))
  .$usage('storage');
const output = root
  .createBuffer(ReductionCounter)
  .$usage('storage');

input.write(values);                // typed values, serialized by the schema
const result = await output.read(); // decoded value back on the CPU
```

subscript-typegpu creates a `Buffer<T>` from the device, an element
stride, a count, the WebGPU usage flags, and a label. The program
supplies the bytes.

```ts program=programs/x08-live-reduction.ts
    using input: Buffer<ReductionValue> = createBuffer<ReductionValue>(
      device,
      ReductionValue_STRIDE,
      count,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "x08-input",
    );
```

`Context.bytesOf<T>(value)` returns the bytes of a `@CStruct` value,
padding included. `Context.bytesInto<T>` writes them into an existing
`u8[]` at an offset.

```ts program=programs/x08-live-reduction.ts
    const queue = device.queue;
    input.write(queue, 0, inputBytes);
    output.writeOne(
      queue,
      0,
      Context.bytesOf<ReductionCounter>(new ReductionCounter(new AtomicU32(0))),
    );
```

`read` creates the staging buffer, copies, maps, and returns the
bytes. `Context.fromBytes<T>` rebuilds the value.

```ts program=programs/b12-readback.ts
    const bytes: u8[] = await buffer.read(device, 0, 4);
    const decoded: FixedArray<Particle, 4> = Context.fromBytes<FixedArray<Particle, 4>>(bytes, 0);
```

The explicit path through a mappable buffer, the same as in the
WebGPU JavaScript API, stays for a caller that owns the readback
buffer.

```ts program=programs/x08-live-reduction.ts
    const mapped: boolean = await readback.handle().mapAsync(
      GPUMapMode.READ,
      0,
      ReductionCounter_STRIDE as u64,
    );
    if (!mapped) { print("FAIL map"); return; }
    const result: ReductionCounter = Context.fromBytes<ReductionCounter>(
      readBuffer<ReductionCounter>(readback, 0, 1),
      0,
    );
```

Differences:

- `read(device, elementIndex, elementCount)` and `readOne` return the
  bytes, not a decoded value. A failed map traps. TypeGPU's `read()`
  returns the decoded JavaScript value.
- `Buffer<T>` has `write`, `writeOne`, `patch` (one field of one
  element), `copyTo`, `read`, and `readOne`. It has no `$usage`. The
  usage flags are constructor arguments, and a `read` on a buffer
  without `COPY_SRC` traps.
- The bytes come from the value's storage. There is no conversion
  step and no `d.InferInput`.
- A `FixedArray<T, N>` value also has bytes. `b06-render` writes three
  vertices with `Context.bytesOf<FixedArray<Vertex, 3>>(values)`.

## Bind group layouts

TypeGPU declares a layout as an object. The binding index follows the
property order. A kernel reads a binding through `layout.$.name`.

```ts
const saxpyLayout = tgpu.bindGroupLayout({
  params: { uniform: SaxpyParams },
  x: { storage: d.arrayOf(Item) },
  y: { storage: d.arrayOf(Item), access: 'mutable' },
});
```

subscript-typegpu declares a layout as a class. Each field is a binding
wrapper. The binding index follows the field order. A kernel receives
the layout as its first parameter.

```ts program=programs/b03-saxpy-uniform.ts
class SaxpyLayout {
  params!: Uniform<SaxpyParams>;
  x!: Storage<Item>;
  y!: MutStorage<Item>;
}
```

Differences:

- `Uniform<T>` binds `T` as a uniform. `Storage<T>` binds `array<T>`
  as read-only storage. `MutStorage<T>` binds `array<T>` as
  read-write storage. TypeGPU writes the same choice as `access:
  'mutable'`.
- A kernel reads each binding through its layout parameter.
  `res.params.$` reads a uniform. `res.items[i]` reads one element,
  and `res.items[i] = v` writes one element of a `MutStorage`. TypeGPU
  writes `layout.$.params` and `layout.$.items[i]`.
- A layout class is a parameter. A kernel with two layout parameters
  uses two bind groups, group 0 and group 1, in parameter order.
  `b11-texture` declares `computePipeline2<TextureLayout, ParamsLayout>`.
- The generator emits `<name>_LAYOUT0` with the binding entries and a
  typed factory `create<Name>BindGroup0(device, pipeline, resources)`.
- TypeGPU's fixed bindings (`root.createUniform`, `root.createMutable`)
  and the automatic catch-all bind group have no equivalent. Every
  binding belongs to a declared layout class.

## Kernels

A TypeGPU kernel is a function with the `'use gpu'` directive. A build
plugin extracts its AST, and TypeGPU generates WGSL from that AST when
the pipeline is first used.

```ts
const integrate = (particle: d.Infer<typeof Particle>, dt: number) => {
  'use gpu';
  const speed = std.length(particle.vel);
  if (speed > 0) {
    return Particle({
      pos: std.add(particle.pos, std.mul(particle.vel, dt)),
      vel: particle.vel,
    });
  }
  return particle;
};

const particleKernel = tgpu.computeFn({
  in: { gid: d.builtin.globalInvocationId },
  workgroupSize: [64],
})((input) => {
  const settings = layout.$.params;
  const i = input.gid.x;
  if (i < settings.count) {
    layout.$.particles[i] = integrate(layout.$.particles[i], settings.dt);
  }
});
```

A subscript-typegpu kernel is a plain function. Its parameters are the
layout classes and one `ComputeInvocation`. A module-level
`computePipeline<L>(fn, spec)` declaration names it as a kernel.

```ts program=programs/b04-particles.ts
function integrate(particle: Particle, dt: f32): Particle {
  const speed: f32 = particle.vel.length();
  if (speed > 0.0) {
    const pos: Vec3f = particle.pos.add(particle.vel.scale(dt));
    return new Particle(pos, particle.vel);
  }
  return particle;
}

function particleKernel(res: ParticleLayout, ctx: ComputeInvocation): void {
  const settings: SimParams = res.params.$;
  const i: u32 = ctx.globalId.x;
  if (i < settings.count) {
    res.particles[i] = integrate(res.particles[i], settings.dt);
  }
}

export const particles: ComputePipelineSpec = computePipeline<ParticleLayout>(particleKernel, {
  name: "particles",
  workgroupSize: [64, 1, 1],
});
```

Differences:

- There is no directive and no build plugin. The generator reads the
  typed HIR that the subscript compiler produces, starts at each
  `computePipeline` declaration, and walks the call graph. `integrate`
  is GPU code because the kernel calls it.
- The builtins are fields of `ComputeInvocation`: `globalId`,
  `localId`, `workgroupId`, `numWorkgroups`, and `localIndex`. TypeGPU
  declares them in the `in` object of `tgpu.computeFn`.
- The kernel subset is a named list of subscript constructs. A
  construct outside it is a generation error that names a rule
  (`K`-rules in `specs/blocks/kernel.md`). TypeGPU reports an
  unsupported construct when it generates the WGSL.
- The workgroup size is explicit. There is no default.
- The function is also host code. The "Running a kernel on the CPU"
  section shows how.

## Vector math

TypeGPU offers three spellings: `std` functions, methods on the
values, and JavaScript operators inside `'use gpu'` functions.

```ts
const value = loaded.add(sampled).mul(0.5);
// or: std.mul(std.add(loaded, sampled), 0.5)
// or, inside 'use gpu' code with tsover configured: (loaded + sampled) * 0.5
```

subscript-typegpu offers methods. The method bodies are real subscript
code, and the generator lowers each method to the WGSL operator or
builtin.

```ts program=programs/b11-texture.ts
  textures.target.store(coords, loaded.add(sampled).scale(0.5));
```

Differences:

- `Vec2f`, `Vec3f`, and `Vec4f` have `add`, `sub`, `mul`, `scale`,
  `dot`, `length`, `normalize`, and the componentwise builtins
  `abs`, `floor`, `ceil`, `fract`, `sqrt`, `exp`, `log`, `sin`,
  `cos`, `tan`, `sign`, `min`, `max`, `clamp`, `pow`, `mix`, `step`,
  `smoothstep`, `distance`, `reflect`, `refract`, and `faceForward`.
  `Vec3f` has `cross`. The integer vectors have `add`, `sub`, `mul`,
  `scale`, `dot`, `min`, `max`, and `clamp`, and the signed ones
  `abs`. The `f16` vectors declare no arithmetic.
- Comparisons are methods: `lt`, `le`, `gt`, `ge`, `eq`, and `ne`
  return a `Vec2b`, `Vec3b`, or `Vec4b` with `any`, `all`, and
  `not`. `select(other, mask)` picks per component. TypeGPU writes
  `std.lt(a, b)` and `std.select`.
- The free factories are `vec2f` through `vec4h` and take their
  components in order. TypeGPU writes `d.vec3f(x, y, z)`. `Vec3f` is
  the type annotation. `new Vec3f(x, y, z)` builds the same value.
- Swizzles are read accessors: `v.xy`, `v.xyz`, and the other in-order
  subsets.
  TypeGPU writes `v.xy`. Mixed constructors are free functions:
  `vec4fFrom3(v, w)`, `vec3fFrom2(v, z)`, `vec3fSplat(s)`, and the same
  shapes for the integer families. TypeGPU writes `d.vec4f(v, w)`.
- There is no operator on vectors. subscript has no operator
  overloading, and the library does not add a build step for it.
- `scale(s)` multiplies by a scalar. `mul(v)` multiplies component by
  component. TypeGPU uses `mul` for both. subscript has no overloads,
  so one name carries one signature.
- Matrices are `Mat2x2f`, `Mat3x3f`, and `Mat4x4f` with `mulVec`, `mul`,
  and `transpose`.

## Pipelines and dispatch

TypeGPU creates the pipeline from the kernel and dispatches with a
bind group attached.

```ts
const pipeline = root.createComputePipeline({ compute: vecAdd });

pipeline
  .with(bindGroup)
  .dispatchWorkgroups(Math.ceil(count / 64));
```

subscript-typegpu creates the pipeline from the generated WGSL and
layout constants. A validation error scope surrounds the creation.

```ts program=programs/b02-vecadd.ts
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      vecAdd_WGSL,
      vecAdd_ENTRY,
      [vecAdd_LAYOUT0],
      [vecAdd_WORKGROUP_X, vecAdd_WORKGROUP_Y, vecAdd_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const resources: VecAddLayoutResources = createVecAddLayoutResources(a, b, out);
    using bindGroup = createVecAddBindGroup0(device, pipeline, resources);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    using command = encoder.finishDefault();
    device.queue.submit([command]);
```

Differences:

- The pipeline exists when `createComputePipeline` returns. TypeGPU
  creates the WebGPU pipeline on first use.
- `dispatchThreads(encoder, groups, x, y, z)` takes the thread count and
  rounds up to whole workgroups. `dispatch` takes workgroup counts.
  Both take the encoder. The program records the commands, finishes the
  encoder, and submits.
- A backend that rejects the WGSL reports through the error scope. The
  program prints a failure and stops.
- `createTimestampPair` returns `null` when the device lacks
  `timestamp-query`. With a pair, `dispatchTimed` records two
  timestamps around the dispatch.
- A declaration with `guarded: true` gets a generated bounds guard
  on all three axes and one hidden uniform binding that the runtime
  owns. `dispatchThreads` writes the thread count before the pass.
  TypeGPU's `createGuardedComputePipeline` is the counterpart.
- `dispatchIndirect(encoder, groups, buffer, offset)` records an
  indirect dispatch. The argument blocks are the schemas
  `DispatchIndirectArgs`, `DrawIndirectArgs`, and
  `DrawIndexedIndirectArgs`, written with `Context.bytesOf`. Render
  indirect draws are `pass.drawIndirect` and
  `pass.drawIndexedIndirect` in the API layer.

## Workgroup variables, barriers, and atomics

TypeGPU declares address-space variables with `tgpu.privateVar` and
`tgpu.workgroupVar`, and reads them through `.$`.

```ts
const privateOffset = tgpu.privateVar(d.u32, 3);
const sharedValues = tgpu.workgroupVar(d.arrayOf(d.u32, 4));
const sharedCounter = tgpu.workgroupVar(d.atomic(d.u32));

const workgroupKernel = tgpu.computeFn({
  in: { lid: d.builtin.localInvocationIndex, wid: d.builtin.workgroupId },
  workgroupSize: [4],
})((input) => {
  privateOffset.$ += 1;
  sharedValues.$[input.lid] = input.lid + privateOffset.$;
  if (input.lid === 0) {
    std.atomicStore(sharedCounter.$, 0);
  }
  std.workgroupBarrier();
  std.atomicAdd(sharedCounter.$, sharedValues.$[input.lid]);
  std.workgroupBarrier();
  if (input.lid === 0) {
    std.atomicAdd(layout.$.counters[input.wid.x].total, std.atomicLoad(sharedCounter.$));
  }
});
```

subscript-typegpu declares them with `privateVar<T>`, `workgroupVar<T>`,
and `workgroupArray<T>` at module level. Atomics are the classes
`AtomicU32` and `AtomicI32`.

```ts program=programs/b10-workgroup.ts
const privateOffset: PrivateVar<u32> = privateVar<u32>(3);
const sharedValues: WorkgroupArray<u32> = workgroupArray<u32>(4);
const sharedCounter: WorkgroupVar<AtomicU32> = workgroupVar<AtomicU32>();

function workgroupKernel(res: WorkgroupLayout, ctx: ComputeInvocation): void {
  privateOffset.$ = privateOffset.$ + 1;
  sharedValues[ctx.localIndex] = ctx.localIndex + privateOffset.$;
  if (ctx.localIndex === 0) {
    sharedCounter.$.store(0);
  }
  workgroupBarrier();
  sharedCounter.$.add(sharedValues[ctx.localIndex]);
  workgroupBarrier();
  if (ctx.localIndex === 0) {
    res.counters[ctx.workgroupId.x].total.add(sharedCounter.$.load());
  }
}
```

Differences:

- A read-modify-write uses one statement: `x.$ = x.$ + 1`.
  subscript rejects `x.$ += 1`.
- Atomic operations are methods: `load`, `store`, `add`, `sub`, `min`,
  `max`, and `exchange`. TypeGPU uses `std.atomicAdd` and the others.
- `workgroupBarrier()` and `storageBarrier()` are functions with empty
  host bodies. The generator emits the WGSL builtin.
- The generator checks that every barrier is in uniform control flow.
  A barrier under a non-uniform condition, or after any `return` in
  source order, is a generation error that names rule `K22`. The WGSL validator `naga`
  does not report this case, and the backends reject the module at
  shader-module creation.

## Textures

TypeGPU binds a texture through the layout and samples with `std`.

```ts
const texturesLayout = tgpu.bindGroupLayout({
  source: { texture: d.texture2d(d.f32) },
  nearest: { sampler: 'non-filtering' },
  target: { storageTexture: d.textureStorage2d('rgba8unorm') },
});

// inside a compute function:
const uv = d.vec2f((gid.x + 0.5) / params.width, (gid.y + 0.5) / params.height);
const loaded = std.textureLoad(texturesLayout.$.source, coords, 0);
const sampled = std.textureSampleLevel(texturesLayout.$.source, texturesLayout.$.nearest, uv, 0);
std.textureStore(texturesLayout.$.target, coords, loaded.add(sampled).mul(0.5));
```

subscript-typegpu binds textures through layout fields of type
`Texture2d<f32>`, `Sampler`, and `StorageTexture2d<F>`. Sampling and
stores are methods on the field.

```ts program=programs/b11-texture.ts
class TextureLayout {
  source!: Texture2d<f32>;
  nearest!: Sampler;
  target!: StorageTexture2d<Rgba8unorm>;
}
```

```ts program=programs/b11-texture.ts
  const loaded: Vec4f = textures.source.load(coords, 0);
  const uv = new Vec2f(
    ((ctx.globalId.x as f32) + 0.5) / (params.width as f32),
    ((ctx.globalId.y as f32) + 0.5) / (params.height as f32),
  );
  const sampled: Vec4f = textures.source.sampleLevel(textures.nearest, uv, 0.0);
  textures.target.store(coords, loaded.add(sampled).scale(0.5));
```

Differences:

- The sampled texture type is `Texture2d<f32>` only. Integer textures
  and other dimensions are not in the library.
- The storage texture format is a type argument: `Rgba8unorm`,
  `Rgba16float`, `R32float`, or `Rgba32float`. `StorageTexture2d` is
  write-only; `ReadStorageTexture2d` and `ReadWriteStorageTexture2d`
  carry the `read` and `read_write` access. TypeGPU writes the access
  in the bind group layout entry.
- `writeTexturePixels(queue, texture, pixels, width, height)` uploads
  a `Vec4f[]` image with a fixed encode table, and `writeTextureBytes`
  is the raw form. TypeGPU uploads through `copyExternalImageToTexture`
  or its own write helpers.
- Every wrapper method has a host body over a `Vec4f[]` image with a
  width and a height. The host `Sampler` implements `nearest`
  filtering only.

## Render pipelines

TypeGPU declares a vertex function and a fragment function with typed
`in` and `out` objects.

```ts
const vert = tgpu.vertexFn({
  in: { position: d.vec2f, color: d.vec3f },
  out: { pos: d.builtin.position, color: d.vec3f },
})((input) => ({
  pos: d.vec4f(input.position, 0, 1),
  color: input.color,
}));

const frag = tgpu.fragmentFn({
  in: { color: d.vec3f },
  out: d.vec4f,
})((input) => d.vec4f(input.color, 1));

const Vertex = d.struct({ position: d.vec2f, color: d.vec3f });
const vertexLayout = tgpu.vertexLayout((n) => d.arrayOf(Vertex, n));

const tri = root.createRenderPipeline({
  attribs: vertexLayout.attrib,
  vertex: vert,
  fragment: frag,
  targets: { format: 'rgba8unorm' },
});
```

subscript-typegpu declares a vertex class, a varyings class, and two
functions. `renderPipeline<V, O>(vertex, fragment, spec)` names them.

```ts program=programs/b06-render.ts
function vert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.color,
  );
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, FRAGMENT_ALPHA);
}

export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, {
  format: "rgba8unorm",
});
```

The vertex buffer layout is generated from the vertex class. The pass
follows the WebGPU JavaScript API.

```ts program=programs/b06-render.ts
    pipeline.bind(pass, [], [vertices.handle()]);
    pass.draw(3);
    pass.end();
```

Differences:

- The vertex input is a `@CStruct` class, and the generator derives
  `<name>_VERTEX_LAYOUT0` with the stride, the formats, and the
  locations. TypeGPU derives the same from `tgpu.vertexLayout`.
- The varyings are a `@CStruct` class. The `Vec4f` field named
  `position` is the clip position.
- `renderPipelineL` adds a layout class for bindings in both stages.
  `renderPipelineInstanced` adds an instance class.
- `indexFormat` on the spec emits `<name>_INDEX_FORMAT`, and
  `RenderPipeline.setIndexBuffer(pass, buffer)` sets the buffer with
  that format. `cullMode`, `frontFace`, `topology` (the
  `triangle-strip` form included), and `blend` on the spec reach the
  pipeline descriptor. TypeGPU writes `withIndexBuffer` and
  `primitive: { cullMode }` and puts the blend on the target.
- One color target per pipeline. Depth, stencil, and multisample
  options are not in the library.

## Running a kernel on the CPU

A TypeGPU function with `'use gpu'` is also a JavaScript function.
A `tgpu.computeFn` entry has no direct CPU call form, so the CPU
path calls the helpers.

```ts
// integrate carries 'use gpu' and is still a JavaScript function:
const moved = integrate(particle, 0.5);
```

A subscript-typegpu kernel is also a subscript function.
`simulateCompute<L>` calls it once per invocation with a host layout
instance and a `ComputeInvocation`.

```ts program=programs/b02-vecadd.ts
    const hostLayout = new VecAddLayout();
    hostLayout.a = new Storage<Item>([new Item(1.0), new Item(2.0), new Item(3.0)]);
    hostLayout.b = new Storage<Item>([new Item(4.0), new Item(5.0), new Item(6.0)]);
    hostLayout.out = new MutStorage<Item>([new Item(0.0), new Item(0.0), new Item(0.0)]);
    simulateCompute<VecAddLayout>(
      vecAddKernel,
      hostLayout,
      vecAdd,
      [1, 1, 1],
      vecAdd_HOST_RUNNABLE,
    );
```

Differences:

- `Storage<T>`, `MutStorage<T>`, and `Uniform<T>` have host
  constructors over arrays and values. On the GPU the same classes are
  bindings.
- The host runs one invocation at a time. A kernel that uses a barrier,
  an atomic, a workgroup variable, or a written private variable is not
  host-runnable. The generator emits `<name>_HOST_RUNNABLE` as `false`,
  and `simulateCompute` traps with rule `CL2`.
- The live programs `x01`–`x04` and `x09` compare the GPU result with
  `simulateCompute` over the same kernel. The host side holds no second
  formula.

## Errors and lifetimes

TypeGPU throws exceptions and relies on garbage collection.
`root.destroy()` destroys the device.

```ts
root.destroy(); // <- frees up all the resources
```

subscript-typegpu has no exceptions and no garbage collection.

- A failed request returns `null`. A failed map returns `false`.
- A backend validation error reaches the program through
  `pushErrorScope` and `popErrorScope`, and through
  `nextUncapturedError()` after a pump.
- A misuse of the runtime library traps. The trap message names the
  rule, the method, and the values.
- Every handle has `dispose()`. A `using` binding calls it at block end.
  There is no finalizer and no reference count.

## Generating WGSL

TypeGPU links functions into a WGSL string at run time.

```ts
const resolved = tgpu.resolve([createSmallBoid, createBigBoid]);
```

subscript-typegpu generates one WGSL module per pipeline declaration
before the program runs. The support module holds it as
`<name>_WGSL`, and the generator also writes it out as one
readable `.wgsl` file per pipeline declaration, so you can open the
exact WGSL your kernel became. Run the generator again after a
kernel change and the file follows.

There is no `tgpu.resolve`, no template with externals, and no
`$uses`. A kernel refers to module-level constants, layout fields, and
functions by name, and the generator emits what the call graph reaches.

A WGSL shell is the escape hatch for a builtin or a construct the
generator does not emit. The function keeps a subscript body, which
the CPU lane runs, and a `wgslShell` declaration supplies the WGSL
body. `wgslDeclarations` adds module-level WGSL text.

```ts program=programs/b14-wgsl-shell.ts
wgslDeclarations("const SHELL_BIAS: u32 = 7u;");

function addBias(value: u32): u32 {
  return value + 7;
}

const addBiasShell: WgslShellSpec = wgslShell<(value: u32) => u32>(
  addBias,
  {
    body: "return value + SHELL_BIAS;",
  },
);
```

TypeGPU writes the same escape hatch as a WGSL-bodied function:

```ts
const SHELL_BIAS = tgpu.const(d.u32, 7);

const addBias = tgpu.fn([d.u32], d.u32)`(value) {
  return value + SHELL_BIAS;
}`.$uses({ SHELL_BIAS });
```

Differences:

- The WGSL `fn` line comes from the subscript signature. The body
  string holds statements only.
- A lexical fence rejects `@group`, `@binding`, `var<`, `override`,
  and the barrier builtins inside a shell body or the declarations.
- A `naga` error inside a shell body is reported with the shell's
  name.
- The subscript body is a second implementation. A live program
  compares the GPU result against it.

## Not in subscript-typegpu

These TypeGPU features have no equivalent.

- Run-time schema construction: `d.struct`, `d.arrayOf`, `d.align`,
  `d.size`, `d.Infer`.
- Fixed resources and the catch-all bind group: `root.createUniform`,
  `root.createReadonly`, `root.createMutable`.
- `tgpu.resolve`, `tgpu.const`, slots, derived values, and
  `tgpu.accessor`.
- Default workgroup sizes.
- `buffer.clear`, `buffer.copyFrom`, `common.writeSoA`.
- Operators on vectors, `std` as a namespace, and `tsover`.
- The derivative builtins `dpdx`, `dpdy`, `fwidth` and the pack and
  unpack builtins are not in the library yet.
- Integer and depth textures, texture dimensions other than 2D, sampler
  filters other than `nearest` on the host.
- Render pipelines with multiple color targets, depth-stencil, or
  multisample. Blending covers the `src-alpha` over and additive
  pairs in the host rasterizer; other factors run on the GPU only.
- `unplugin-typegpu`, `tgpu-gen`, and the browser.

## Not in TypeGPU

These subscript-typegpu properties have no equivalent.

- A C build for shipping: the program compiles with your platform's
  C compiler and reproduces the JIT's output exactly.
- A C layout equal to the WGSL layout, and `Context.bytesOf<T>`.
- The emitted WGSL as a readable file beside the program.
- A backend chosen at run time through `SUBSCRIPT_TYPEGPU_BACKEND_LIB`.
  yawgpu's default Noop backend runs headless on a machine with no
  GPU: copies execute, shaders do not.
- Generation-time rejections with rule ids, including the uniform
  control flow check for barriers.
- `simulateCompute` with the `HOST_RUNNABLE` constant.
- A WGSL shell with a host body that the CPU lane runs, and a
  `naga` error attributed to the shell by name.
- A window host outside the browser: `tools/window.sh` runs a
  script that exports `init`, `frame`, and `shutdown`, and the host
  owns the window, the surface, and the loop. `frame` carries the
  size, one key scalar, the pointer position, and the button bits.
  TypeGPU draws into a canvas through `GPUCanvasContext` and reads
  DOM events.

## Where to go next

- `docs/tutorial.md` walks `programs/b04-particles.ts` from schema to
  dispatch.
- `programs/` holds every subscript-typegpu example in this document. The `b` programs
  run headless. The `x` programs run on a real adapter through
  `tools/live.sh`.
- `specs/blocks/` holds the rules that the generator and the runtime
  enforce.
