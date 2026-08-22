# Block: render (RN-rules)

P3 contract. Rev 0, 2026-08-22. Plan §8 P3 governs this block.
Kernels are `kernel.md` (K-rules) and apply to vertex and fragment
kernels. Pipelines are `pipeline.md` (PI-rules) where this block
does not say otherwise. Schemas are `schema.md`.

## The declaration

- **RN1 — A render pipeline is a module-level `const`.**
  `export const tri = renderPipeline<Vertex, Varyings>(vert, frag,
  { format: "rgba8unorm" });`. `renderPipeline` is a library generic
  function with a real body that returns a `RenderPipelineSpec` (the
  target format, the primitive topology, the cull mode). The
  generator finds the call at module level, reads the two kernels
  from the `FuncRef` arguments, and reads the descriptor literal.
  `renderPipelineL<L, Vertex, Varyings>` takes a layout class first
  for pipelines with bindings. A second vertex schema for instance
  data is `renderPipelineInstanced<Vertex, Instance, Varyings>`.
- **RN2 — Kernel signatures.** The vertex kernel is `(v: Vertex,
  ctx: VertexInvocation): Varyings`, with a layout class before `v`
  under `renderPipelineL` and an instance schema after `v` under
  `renderPipelineInstanced`. The fragment kernel is `(input:
  Varyings, ctx: FragmentInvocation): Vec4f`, with the same layout
  class first under `renderPipelineL`. The generator checks the
  types against the declaration's type arguments (PI2).
- **RN3 — Builtins.** `VertexInvocation { vertexIndex: u32;
  instanceIndex: u32 }` emits `@builtin(vertex_index)` and
  `@builtin(instance_index)` for the fields the kernel reads.
  `FragmentInvocation { position: Vec4f; frontFacing: boolean }`
  emits `@builtin(position)` and `@builtin(front_facing)` the same
  way.

## Vertex input

- **RN4 — The vertex schema is a `@CStruct` class.** Its fields are
  the attributes, in declaration order, at `@location(n)` from 0.
  The vertex buffer layout is the schema's layout: `arrayStride` is
  `X_STRIDE`, each attribute's `offset` is `X_OFFSET_<field>`, and
  its `format` is the field type's format (RN5). The step mode is
  `vertex` for the vertex schema and `instance` for the instance
  schema, whose locations continue after the vertex schema's.
- **RN5 — Formats.** `f32` → `float32`, `Vec2f` → `float32x2`,
  `Vec3f` → `float32x3`, `Vec4f` → `float32x4`, `u32` → `uint32`,
  `Vec2u`…`Vec4u` → `uint32x2`…`uint32x4`, `i32` → `sint32`,
  `Vec2i`…`Vec4i` → `sint32x2`…`sint32x4`, `Vec2h` → `float16x2`,
  `Vec4h` → `float16x4`. A matrix, a nested schema, a `FixedArray`,
  `f16`, and `Vec3h` are not vertex attributes in P3 and are a
  diagnostic.
- **RN6 — The generated vertex layout.** The support module exports
  `<name>_VERTEX_LAYOUT<s>: VertexBufferLayoutSpec` per vertex
  buffer slot `s` (0 for the vertex schema, 1 for the instance
  schema), a library `@Descriptor` with `arrayStride`, `stepMode`,
  and `attributes: VertexAttributeSpec[]` (`format`, `offset`,
  `shaderLocation`). Constants only, as PI8.

## Inter-stage data

- **RN7 — The varyings class.** A `@CStruct` class whose field named
  `position` of type `Vec4f` is `@builtin(position)`, and whose
  other fields are `@location(n)` in declaration order from 0, with
  `@interpolate(flat)` for integer types. It is not a schema: it is
  never laid out for a buffer and never gets layout constants. A
  varyings class with no `position: Vec4f` field is a diagnostic.
- **RN8 — The fragment output.** A `Vec4f` return is `@location(0)
  vec4<f32>`. A `@CStruct` class return with `Vec4f` fields is one
  `@location(n)` per field in declaration order, for multiple
  targets (P3 declares one target; the rule allows more).

## Emission

- **RN9 — One module, two entries.** The render pipeline's module
  holds the structs, the bindings, the helpers, `@vertex fn <vert>`,
  and `@fragment fn <frag>`, in that order (K14). Binding visibility
  is derived from which kernel reaches the binding: vertex, fragment,
  or both. The golden is `programs/<stem>.<name>.wgsl` (K16).
- **RN10 — The support module.** `<name>_WGSL`, `<name>_VERTEX_ENTRY`,
  `<name>_FRAGMENT_ENTRY`, `<name>_LAYOUT<g>` as PI8, the RN6 vertex
  layouts, and `<name>_TARGET_FORMAT: GPUTextureFormat`.

## The runtime

- **RN11 — `createRenderPipeline`.** `lib/typegpu.ts` exports
  `createRenderPipeline(device, wgsl, vertexEntry, fragmentEntry,
  layouts: BindGroupLayoutSpec[], vertexLayouts:
  VertexBufferLayoutSpec[], spec: RenderPipelineSpec): RenderPipeline`
  over `lib/webgpu.ts`, with `bindGroupLayout(g)`, `bind(pass,
  groups: GPUBindGroup[], vertexBuffers: GPUBuffer[])` (sets the
  pipeline, the bind groups, and each vertex buffer with its full
  size), and `dispose()`. Draw calls, index buffers, render passes,
  and attachments stay with the API layer, because they are
  WebGPU-shaped already.
- **RN12 — The spec.** `RenderPipelineSpec { format:
  GPUTextureFormat; topology?: GPUPrimitiveTopology =
  "triangle-list"; cullMode?: GPUCullMode = "none"; frontFace?:
  GPUFrontFace = "ccw" }`. The generator reads the literal members
  it needs (`format`) and the runtime reads the rest.

## Programs

- **RN13 — Gate programs print structure.** `b06-render` creates the
  pipeline, one vertex buffer through `Buffer<Vertex>`, an
  offscreen `rgba8unorm` texture, a render pass with a clear, binds,
  draws three vertices, submits, and prints markers, the vertex
  layout constants by name, and the WGSL line count. Noop draws
  nothing, so no pixel is read.
- **RN14 — Live programs compare pixels with a host rasterizer.**
  `x05-live-triangle` draws one flat-colored triangle into a 64×64
  `rgba8unorm` target whose vertices put no pixel center on an edge,
  copies the texture to a buffer (`bytesPerRow` 256), maps, reads,
  and compares every pixel with the host's own point-in-triangle
  test at pixel centers: inside → the color, outside → the clear
  color. It prints `PASS` or `FAIL x=<n> y=<n> expected=<rgba>
  got=<rgba>`. No reference hash exists.
- **RN15 — Draw variants.** `b07-draw-variants` and
  `x06-live-draw-variants` cover `drawIndexed` with an index buffer
  and an instanced draw through `renderPipelineInstanced`, with the
  same host rasterizer: the host knows every triangle.

## Rejections

- **RN16 — Each rejection is a named diagnostic with a red fixture.**
  A vertex schema field outside RN5, a varyings class without
  `position: Vec4f`, a fragment kernel that returns a non-`Vec4f`
  non-class type, a kernel signature outside RN2, a `textureSample`
  call (P5), a vertex kernel that writes a storage binding.
