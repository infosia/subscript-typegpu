# subscript-typegpu

subscript-typegpu provides WebGPU and typed GPU programs for subscript.

The project has two script layers over one Rust facade:

- `lib/webgpu.ts` provides the WebGPU API layer.
- `lib/typegpu.ts` provides schemas, typed resources, pipelines, textures, and buffers.
- `crates/facade` loads one webgpu.h implementation at run time.

The generators create the facade, the ambient mirror, the WebGPU API layer, TypeGPU support modules, and WGSL modules.

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

See [docs/tutorial.md](docs/tutorial.md) for a typed particle pipeline example.
See [docs/from-typegpu.md](docs/from-typegpu.md) for a topic-by-topic comparison with TypeGPU.
