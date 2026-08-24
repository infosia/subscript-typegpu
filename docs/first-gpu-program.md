# Your first GPU program

A compute kernel runs an ordinary calculation on the GPU, in
parallel, with no triangle in sight. This page builds the smallest
useful one: a counter that the GPU increments. Every code block is
a quote from `programs/b22-first-program.ts`, and a test checks the
quotes against the program.

A GPU function needs more ceremony than a host function: the code
compiles for a different processor, the memory is allocated ahead,
and handles cross between the two sides. TypeGPU performs that
ceremony at run time. This library performs it before the program
runs: a generator reads the typed program and emits the WGSL and
the layout constants the host code imports.

## The smallest kernel

A kernel is a plain function. A module-level `computePipeline`
declaration marks it, and the generator finds every such
declaration.

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

TypeGPU writes `root.createGuardedComputePipeline(() => { 'use gpu';
... })` and extracts the function with a build plugin. Here the
declaration is the marker, and the generator emits one WGSL module
and a support module `./b22-first-program.typegpu` with the entry
name, the workgroup size, and the layout constants.

## State lives in a buffer, and a schema shapes it

The kernel reads and writes `res.state` — a binding, not a host
variable. A schema class gives the state its layout, and a layout
class names the bindings of one bind group.

```ts program=programs/b22-first-program.ts
@CStruct
class State {
  counter: u32;
  incrementBy: u32;

  constructor(counter: u32, incrementBy: u32) {
    this.counter = counter;
    this.incrementBy = incrementBy;
  }
```

```ts program=programs/b22-first-program.ts
class CounterLayout {
  state!: MutStorage<State>;
}
```

TypeGPU builds the same shapes at run time with `d.struct({ ... })`
and `root.createMutable(...)`. Here the generator computes the
layout before the program runs and emits it as constants:
`State_STRIDE` is the schema's stride, and
`State_OFFSET_incrementBy` is one field's byte offset. No byte
count is written by hand.

```ts program=programs/b22-first-program.ts
    using stateBuffer: Buffer<State> = createBuffer<State>(
      device,
      State_STRIDE,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST,
      "b22-state",
    );
```

The initial value crosses as the bytes of a typed value, padding
included:

```ts program=programs/b22-first-program.ts
    const initialState = new State(0, 10);
    stateBuffer.write(
      device.queue(),
      0,
      Context.bytesOf<State>(initialState),
    );
```

## Run it

The pipeline builds from the generated WGSL and layout constants,
inside a validation error scope, so a rejected shader is a visible
failure instead of a silent zero. The typed factory builds the bind
group, and one dispatch runs one invocation.

```ts program=programs/b22-first-program.ts
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      firstProgram_WGSL,
      firstProgram_ENTRY,
      [firstProgram_LAYOUT0],
      [
        firstProgram_WORKGROUP_X,
        firstProgram_WORKGROUP_Y,
        firstProgram_WORKGROUP_Z,
      ],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }

    const resources: CounterLayoutResources = createCounterLayoutResources(
      stateBuffer.handle(),
    );
    using bindGroup = createFirstProgramBindGroup0(
      device,
      pipeline,
      resources,
    );

    using firstEncoder = device.createCommandEncoderDefault();
    pipeline.dispatch(firstEncoder, [bindGroup], 1, 1, 1);
    using firstCommand = firstEncoder.finishDefault();
    device.queue().submit([firstCommand]);
```

TypeGPU compiles the pipeline lazily on the first
`dispatchThreads()`. Here `createComputePipeline` returns a created
pipeline, and the program checks the scope before it uses it.

## Change one field, read the result

`patch` writes one field of one element, so the counter's current
value stays untouched on the GPU. `readOne` copies the element back
through a staging buffer, and `Context.fromBytes` rebuilds the
typed value.

```ts program=programs/b22-first-program.ts
    stateBuffer.patch(
      device.queue(),
      0,
      State_OFFSET_incrementBy,
      Context.bytesOf<FixedArray<u32, 1>>([25]),
    );

    using secondEncoder = device.createCommandEncoderDefault();
    pipeline.dispatch(secondEncoder, [bindGroup], 1, 1, 1);
    using secondCommand = secondEncoder.finishDefault();
    device.queue().submit([secondCommand]);

    const readbackBytes: u8[] = await stateBuffer.readOne(device, 0);
    const readback: State = Context.fromBytes<State>(readbackBytes, 0);
    print(`readback:counter=${readback.counter} incrementBy=${readback.incrementBy}`);
```

TypeGPU spells the same three steps `buffer.patch(...)`,
`buffer.read()`, and schema-driven deserialization. One difference
matters when this program runs as a gate test: the headless gate
substrate executes copies but no shaders, so the committed output
records `counter=0`. A live device records the two increments. The
host lane below proves the arithmetic on every test run, device or
not.

## Why a plain variable cannot hold GPU state

```ts
let counter: u32 = 0;

function badKernel(res: CounterLayout, ctx: ComputeInvocation): void {
  counter += 1;
}
```

The kernel compiles for the GPU, so it cannot close over host
memory. The generator rejects a kernel that reads a mutable global
with a `K19` diagnostic that names the global, and a committed red
fixture holds that rejection. State the GPU can write lives in a
buffer behind a `MutStorage` binding, and the host reads it back as
bytes.

## The kernel also runs on the CPU

The same function body is host code. `simulateCompute` runs it over
host-side bindings, one invocation at a time, so the arithmetic is
proven without a device.

```ts program=programs/b22-first-program.ts
    const hostLayout = new CounterLayout();
    hostLayout.state = new MutStorage<State>([new State(0, 10)]);
    simulateCompute<CounterLayout>(
      incrementCounter,
      hostLayout,
      firstProgram,
      [1, 1, 1],
      firstProgram_HOST_RUNNABLE,
    );
    const hostState: State = hostLayout.state.get(0);
    hostState.incrementBy = 25;
    hostLayout.state.set(0, hostState);
    simulateCompute<CounterLayout>(
      incrementCounter,
      hostLayout,
      firstProgram,
      [1, 1, 1],
      firstProgram_HOST_RUNNABLE,
    );
    print(`host:counter=${hostLayout.state.get(0).counter}`);
```

Two increments — 10, then 25 — leave the host counter at 35, and
the gate compares that line against the committed golden on both
compilation tiers.

## Where to go next

- `docs/tutorial.md` builds a particle pipeline with vectors and a
  larger schema.
- `docs/from-typegpu.md` maps every TypeGPU concept to this
  library, topic by topic.
- `examples/` holds twenty ported TypeGPU examples, headless and
  windowed.
