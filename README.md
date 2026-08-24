# subscript-typegpu

subscript-typegpu brings GPU programming to
[subscript](https://github.com/infosia/subscript) programs. It
rebuilds three ideas of
[TypeGPU](https://github.com/software-mansion/TypeGPU) for a
statically typed language without a JavaScript runtime:

- typed data schemas with automatic memory layout,
- GPU kernels written in subscript,
- typed bindings between the two.

It is not TypeGPU, and it is not a port of TypeGPU. TypeGPU builds
schemas and generates WGSL while your program runs. Here a
generator reads your typed program first, computes every memory
layout, and writes the WGSL and the layout constants your code
imports. Your program then runs on a development JIT or compiles to
C, over any webgpu.h implementation you point it at — no browser
involved.

The library has two layers:

- `lib/webgpu.ts` follows the WebGPU JavaScript API in names and
  shape: devices, buffers, textures, encoders, passes.
- `lib/typegpu.ts` is the TypeGPU-shaped layer on top: schemas,
  typed bindings, pipelines, and kernels that are plain functions.

## A first look

A counter that the GPU increments, from
`programs/b22-first-program.ts`.
[docs/first-gpu-program.md](docs/first-gpu-program.md) walks the
whole program step by step.

A kernel is a plain function, and a module-level `computePipeline`
declaration marks it. The generator finds the declaration and
writes the WGSL for you. TypeGPU marks the same function with
`'use gpu'` and generates its WGSL at run time.

```ts program=programs/b22-first-program.ts
function incrementCounter(res: CounterLayout, ctx: ComputeInvocation): void {
  const state: State = res.state.get(0);
  state.counter += state.incrementBy;
  res.state.set(0, state);
}

export const firstProgram: ComputePipelineSpec = computePipeline<CounterLayout>(
  incrementCounter,
  {
    name: "firstProgram",
    workgroupSize: [1, 1, 1],
  },
);
```

The kernel's state lives in a buffer. A `@CStruct` class shapes the
bytes, and a layout class names the bindings of one bind group.
TypeGPU writes `d.struct({ ... })` and `root.createMutable(...)`
for the same two roles.

```ts program=programs/b22-first-program.ts
@CStruct
class State {
  counter: u32;
  incrementBy: u32;

  constructor(counter: u32, incrementBy: u32) {
    this.counter = counter;
    this.incrementBy = incrementBy;
  }
}
```

```ts program=programs/b22-first-program.ts
class CounterLayout {
  state!: MutStorage<State>;
}
```

You never count bytes by hand: the generator emits the sizes and
offsets as constants — `State_STRIDE`,
`State_OFFSET_incrementBy` — and `Context.bytesOf<State>(value)`
turns a typed value into the bytes a buffer takes. Reading back is
explicit: a copy through a staging buffer, then `Context.fromBytes`.

```ts program=programs/b22-first-program.ts
    const readbackBytes: u8[] = await stateBuffer.readOne(device, 0);
    const readback: State = Context.fromBytes<State>(readbackBytes, 0);
    print(`readback:counter=${readback.counter} incrementBy=${readback.incrementBy}`);
```

The same kernel body also runs on the CPU through
`simulateCompute`, so you can test kernel logic with no GPU at
hand. One program serves development and shipping: the JIT runs it
as is, and the C tier compiles it with your platform's C compiler —
with identical results.

## How this differs from TypeGPU

| | TypeGPU | subscript-typegpu |
|---|---|---|
| Schema | `d.struct({ ... })`, a run-time value | `@CStruct class`, a declaration |
| Memory layout | computed at run time | computed ahead of time, importable as constants |
| WGSL | generated at run time from a compacted AST | generated ahead of time — the emitted WGSL sits next to your program as a readable file |
| Kernel marker | `'use gpu'` directive and a build plugin | `computePipeline<L>(fn, spec)` declaration |
| Buffer data | JavaScript values converted by the library | `Context.bytesOf<T>(value)`, the bytes of the value |
| Lifetime | garbage collection | `using` and `dispose()` |
| Errors | exceptions | `null` and `false` returns, error scopes, and traps that name a rule |
| Execution | a JavaScript runtime with WebGPU | a JIT for development and a C tier for shipping, over a webgpu.h library loaded at run time |

There is no source compatibility between the two. A TypeGPU program
does not compile as a subscript program. The concepts carry over.
Most names do not.
[docs/from-typegpu.md](docs/from-typegpu.md) compares the two
libraries topic by topic, with code from both sides.

## Running a program

Two environment variables select the GPU backend:

- `SUBSCRIPT_TYPEGPU_BACKEND_LIB` names the webgpu.h shared library
  to load — for example a [yawgpu](https://github.com/infosia/yawgpu)
  or Dawn build.
- `SUBSCRIPT_TYPEGPU_BACKEND` picks the adapter backend: `metal`,
  `vulkan`, `gles`, `d3d11`, or `d3d12`. Leave it unset for the
  library's default.

Run a headless program:

```sh
tools/example.sh examples/matrix-multiplication/main.ts
```

Run a windowed program (opens a window; add `--frames <n>` to close
it after `n` frames):

```sh
tools/window.sh examples/boids/main.ts
```

A windowed program exports three functions the host calls: `init`
once, `frame` once per displayed frame (with the size, one key
scalar, and the pointer), and `shutdown` once.

## Examples

`examples/` holds twenty programs ported from TypeGPU's example
set — boids, a grid fluid, slime mold, Conway's life, ray marching,
clouds, and more. Each file states what it shows and where it
differs from the TypeGPU original.

## Documents

- [docs/first-gpu-program.md](docs/first-gpu-program.md) builds the smallest compute program: a GPU counter from buffer creation to readback.
- [docs/tutorial.md](docs/tutorial.md) walks `programs/b04-particles.ts` from the schema to the dispatch.
- [docs/from-typegpu.md](docs/from-typegpu.md) compares TypeGPU with this library, topic by topic.

## Development

This section is for working on the library itself.

Commands:

```sh
tools/gate.sh              # the full test suite, headless
tools/gate.sh --require-backend
tools/regen.sh             # regenerate all generated files
```

CAUTION: Run the live lane only with a real adapter. The command executes every `x` program on the device.

```sh
tools/live.sh
```

All Cargo commands in these tools use offline mode.
`SUBSCRIPT_TYPEGPU_UPSTREAM_DIR` names a TypeGPU checkout for the
layout-vector tool, which records TypeGPU's computed memory layouts
as reference values for the layout tests.

Test programs live in `programs/` — `a` programs cover the WebGPU
API layer, `b` programs cover the generated TypeGPU modules on both
compilation tiers, and `x` programs run on a real device. Each has
an `.expected` file holding the output both tiers must reproduce
exactly, and each generated pipeline has its emitted `.wgsl` beside
it.

Repository layout:

- `crates/facade` contains the generated C ABI facade and its loader.
- `crates/webgpu-gen` contains the facade and WebGPU API generator.
- `crates/typegpu-gen` contains the schema, layout, and WGSL generator.
- `crates/harness` contains the dev, ship, coverage, documentation, and live test lanes.
- `crates/window` contains the window host.
- `examples` contains the ported examples, outside the test suite.
- `lib` contains the script libraries and generated ambient files.
- `programs` contains the test programs with their expected outputs.
- `specs` contains the contracts and tracking records.
- `tools` contains the commands above.
