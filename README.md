# subscript-typegpu

subscript-typegpu rebuilds three ideas of
[TypeGPU](https://github.com/software-mansion/TypeGPU) for
[subscript](https://github.com/infosia/subscript). It is not TypeGPU,
and it is not a port of TypeGPU. It is a new library for a statically
typed language without a JavaScript runtime. The three ideas:

- typed data schemas with automatic memory layout,
- GPU kernels written in subscript,
- typed bindings between the two.

TypeGPU does this work at run time. subscript-typegpu does it before
the program runs: a generator reads the typed program, computes every
memory layout, and emits the WGSL. The program then runs on a JIT tier
or compiles to C, over the webgpu.h implementation that
`SUBSCRIPT_TYPEGPU_BACKEND_LIB` names.

The project has two script layers and one Rust facade:

- `lib/webgpu.ts` is the WebGPU API layer. It follows the WebGPU
  JavaScript API in names and shape.
- `lib/typegpu.ts` is the TypeGPU layer: binding wrappers, pipelines,
  buffers, and textures. `lib/typegpu-types.ts` holds the vector and
  matrix classes. A schema is a `@CStruct` class in the program itself.
- `crates/facade` loads one webgpu.h implementation at run time.

## A first look

This section follows `programs/b04-particles.ts`. Each block is a
quote from that program. The sentence after each block names the
TypeGPU counterpart. `docs/from-typegpu.md` holds the full comparison.

A schema is a class with the `@CStruct` decorator.

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

TypeGPU writes `d.struct({ pos: d.vec3f, vel: d.vec3f })`, a value
that exists at run time. Here the class is the schema, and the
generator computes its memory layout before the program runs.

A bind group layout is a class whose fields are binding wrappers. The
binding index is the field order. `MutStorage<Particle>` binds a
read-write `array<Particle>`.

```ts program=programs/b04-particles.ts
class ParticleLayout {
  params!: Uniform<SimParams>;
  particles!: MutStorage<Particle>;
}
```

TypeGPU passes the same shape as an object to its `bindGroupLayout`
function: `{ params: { uniform: SimParams }, particles: { storage:
d.arrayOf(Particle), access: 'mutable' } }`.

A kernel is a plain function. Its leading parameters are the bind
group layout classes, one per bind group, and its last parameter
carries the invocation builtins.

```ts program=programs/b04-particles.ts
function particleKernel(res: ParticleLayout, ctx: ComputeInvocation): void {
  const settings: SimParams = res.params.get();
  const i: u32 = ctx.globalId.x;
  if (i < settings.count) {
    res.particles.set(i, integrate(res.particles.get(i), settings.dt));
  }
}

export const particles: ComputePipelineSpec = computePipeline<ParticleLayout>(particleKernel, {
  name: "particles",
  workgroupSize: [64, 1, 1],
});
```

TypeGPU marks a kernel with the `'use gpu'` directive and a build
plugin extracts its AST. Here the `computePipeline` declaration marks
the kernel, and the generator walks the typed call graph from it.
`integrate` is a second plain function that the kernel calls.

The generator emits a support module with the schema sizes, the bind
group layout specs, the WGSL, the entry point, the workgroup size, and
typed factories. The program imports them.

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

TypeGPU has no such module. `d.sizeOf(Particle)` and the WGSL string
are computed when a program asks for them.

The host side creates the pipeline inside a validation error scope,
builds the bind group from the typed factory, and dispatches.

```ts program=programs/b04-particles.ts
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      particles_WGSL,
      particles_ENTRY,
      [particles_LAYOUT0],
      [particles_WORKGROUP_X, particles_WORKGROUP_Y, particles_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    const resources: ParticleLayoutResources = createParticleLayoutResources(
      params,
      particlesBuffer,
    );
    using bindGroup = createParticlesBindGroup0(device, pipeline, resources);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], count, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
```

TypeGPU writes `root.createComputePipeline({ compute })` and
`.with(bindGroup).dispatchWorkgroups(n)`, and creates the WebGPU
pipeline on first use. Here every handle is explicit: `using` releases
it at the block end, and a rejected shader is a visible failure.

A host-runnable kernel also runs on the CPU with host-side binding
values. The generator decides host-runnability and emits it as the
`_HOST_RUNNABLE` constant.

```ts program=programs/b04-particles.ts
    const hostLayout = new ParticleLayout();
    hostLayout.params = new Uniform<SimParams>(new SimParams(2.0, 1));
    hostLayout.particles = new MutStorage<Particle>([
      new Particle(new Vec3f(1.0, 2.0, 3.0), new Vec3f(0.5, 0.0, 0.0)),
    ]);
    simulateCompute<ParticleLayout>(
      particleKernel,
      hostLayout,
      particles,
      [1, 1, 1],
      particles_HOST_RUNNABLE,
    );
```

A TypeGPU `'use gpu'` function is also a JavaScript function. Here
`simulateCompute` calls the kernel once per invocation, and the live
programs `x01`–`x04` and `x09` compare the GPU result against it.

## How this differs from TypeGPU

| | TypeGPU | subscript-typegpu |
|---|---|---|
| Schema | `d.struct({ ... })`, a run-time value | `@CStruct class`, a declaration |
| Memory layout | computed at run time | computed by the generator, emitted as constants |
| WGSL | generated at run time from a compacted AST | generated before the program runs, committed as a golden |
| Kernel marker | `'use gpu'` directive and a build plugin | `computePipeline<L>(fn, spec)` declaration |
| Buffer data | JavaScript values converted by the library | `Context.bytesOf<T>(value)`, the bytes of the value |
| Lifetime | garbage collection | `using` and `dispose()` |
| Errors | exceptions | `null` and `false` returns, error scopes, and traps that name a rule |
| Execution | a JavaScript runtime with WebGPU | a JIT tier and a C tier over a webgpu.h library loaded at run time |

There is no source compatibility between the two. A TypeGPU program
does not compile as a subscript program. The concepts carry over.
Most names do not.

## Environment variables

`SUBSCRIPT_TYPEGPU_BACKEND_LIB` must name the backend shared library for backend-required gates and device runs.

`SUBSCRIPT_TYPEGPU_BACKEND` selects a yawgpu backend. The accepted values are `metal`, `vulkan`, and `gles`.

If `SUBSCRIPT_TYPEGPU_BACKEND` is absent, the library selects its default. The yawgpu default is Noop. The gate never sets this variable.

`tools/live.sh` also accepts `default`. This value sends no selection chain to a library that ignores the yawgpu extension.

`SUBSCRIPT_TYPEGPU_UPSTREAM_DIR` names a TypeGPU checkout for the optional golden-vector tool.

## Commands

Run the standard gate:

```sh
tools/gate.sh
```

Run the gate with a required backend:

```sh
tools/gate.sh --require-backend
```

Regenerate all committed generator outputs:

```sh
tools/regen.sh
```

CAUTION: Run the live lane only with a real adapter. The command executes every `x` program.

```sh
tools/live.sh
```

All Cargo commands in these tools use offline mode.

## Programs

Programs use a letter, a two-digit number, and a short name.

- `a` programs test the WebGPU API and ergonomics.
- `b` programs test generated TypeGPU modules on both gate tiers.
- `x` programs test live device results against host expectations.

Each gate program has an `.expected` file with the same stem. Generated pipelines have `.wgsl` files with the same stem.

`simulateCompute` runs a host-runnable kernel through its script body and wrapper storage. `simulateComputeThreads` uses the same thread counts as `dispatchThreads`.

## Repository layout

- `crates/facade` contains the generated C ABI facade and its loader.
- `crates/webgpu-gen` contains the facade and WebGPU API generator.
- `crates/typegpu-gen` contains the schema, layout, and WGSL generator.
- `crates/harness` contains the dev, ship, coverage, documentation, and live test lanes.
- `lib` contains the script libraries and generated ambient files.
- `programs` contains gate and live programs with their goldens.
- `specs` contains the contracts and tracking records.
- `tools` contains regeneration, gate, hygiene, and live commands.

## Documents

- [docs/tutorial.md](docs/tutorial.md) walks `programs/b04-particles.ts` from the schema to the dispatch.
- [docs/from-typegpu.md](docs/from-typegpu.md) compares TypeGPU with this library, topic by topic, with code from both sides.
