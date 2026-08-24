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

This section follows `programs/b22-first-program.ts`: a counter
that the GPU increments. Each block is a quote from that program.
[docs/first-gpu-program.md](docs/first-gpu-program.md) walks the
whole program step by step.

A kernel is a plain function, and a module-level `computePipeline`
declaration marks it. The generator finds the declaration, walks
the typed code, and emits the WGSL and the layout constants before
the program runs. TypeGPU marks the same function with `'use gpu'`
and generates its WGSL at run time.

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

The generator emits the sizes and offsets as constants —
`State_STRIDE`, `State_OFFSET_incrementBy` — so the host writes the
buffer with `Context.bytesOf<State>(value)` and patches one field
without a hand-written byte count. Reading back is explicit: a copy
through a staging buffer, then `Context.fromBytes`.

```ts program=programs/b22-first-program.ts
    const readbackBytes: u8[] = await stateBuffer.readOne(device, 0);
    const readback: State = Context.fromBytes<State>(readbackBytes, 0);
    print(`readback:counter=${readback.counter} incrementBy=${readback.incrementBy}`);
```

The same kernel body runs on the host through `simulateCompute`,
one invocation at a time, so the arithmetic is proven on every test
run with no device. The program passes both compilation tiers — the
JIT and the emitted C — with byte-identical output.

## How this differs from TypeGPU

| | TypeGPU | subscript-typegpu |
|---|---|---|
| Schema | `d.struct({ ... })`, a run-time value | `@CStruct class`, a declaration |
| Memory layout | computed at run time | computed by the generator, emitted as constants |
| WGSL | generated at run time from a compacted AST | generated before the program runs, committed, and compared byte for byte by the tests |
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

`SUBSCRIPT_TYPEGPU_BACKEND` selects the adapter backend. The accepted values are `metal`, `vulkan`, `gles`, `d3d11`, and `d3d12`.

If `SUBSCRIPT_TYPEGPU_BACKEND` is absent, the library selects its default. The yawgpu default is Noop. The gate never sets this variable.

`tools/live.sh` and `tools/window.sh` also accept `default`. This value sends no backend request.

On Windows, place `vulkan-1.dll` and `d3dcompiler_47.dll` beside `webgpu_dawn.dll`. Dawn loads them from the library directory, not from `System32`. A missing library fails with `DynamicLib.Open: ... Windows Error: 87`.

`SUBSCRIPT_TYPEGPU_UPSTREAM_DIR` names a TypeGPU checkout for the optional layout-vector tool, which records TypeGPU's computed memory layouts as reference values for the layout tests.

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

CAUTION: Run the window host only with a real adapter and a display. The command opens a window.

```sh
tools/window.sh examples/window-triangle/main.ts
```

`tools/window.sh` reads the same two environment variables as `tools/live.sh`. The host owns the window, the surface, the instance, the device, and the event loop, and calls three exports on the script: `init(instance, device, format)` once, `frame(view, width, height, key)` once per frame, and `shutdown()` once. `--frames <n>` closes the window after `n` frames. On close the host prints `window:frames=<n>`.

All Cargo commands in these tools use offline mode.

## Programs

Programs use a letter, a two-digit number, and a short name.

- `a` programs test the WebGPU API and ergonomics.
- `b` programs test generated TypeGPU modules on both gate tiers.
- `x` programs test live device results against host expectations.

Each gate program has an `.expected` file with the same stem: the committed reference output that both tiers must reproduce byte for byte. Each generated pipeline has a committed `.wgsl` file with the same stem, which a test regenerates and validates on every run.

`simulateCompute` runs a host-runnable kernel through its script body and wrapper storage. `simulateComputeThreads` uses the same thread counts as `dispatchThreads`.

## Repository layout

- `crates/facade` contains the generated C ABI facade and its loader.
- `crates/webgpu-gen` contains the facade and WebGPU API generator.
- `crates/typegpu-gen` contains the schema, layout, and WGSL generator.
- `crates/harness` contains the dev, ship, coverage, documentation, and live test lanes.
- `crates/window` contains the window host.
- `examples` contains the window example, outside the program suite.
- `lib` contains the script libraries and generated ambient files.
- `programs` contains the gate and live programs with their committed reference outputs.
- `specs` contains the contracts and tracking records.
- `tools` contains regeneration, gate, hygiene, live, and window commands.

## Documents

- [docs/first-gpu-program.md](docs/first-gpu-program.md) builds the smallest compute program: a GPU counter from buffer creation to readback.
- [docs/tutorial.md](docs/tutorial.md) walks `programs/b04-particles.ts` from the schema to the dispatch.
- [docs/from-typegpu.md](docs/from-typegpu.md) compares TypeGPU with this library, topic by topic, with code from both sides.
