# Block: render (RN-rules)

P3 contract. Rev 0, 2026-08-22. Rev 3 (RN18–RN19 index format,
cull), 2026-08-23. Rev 4 (RN20 strip), 2026-08-24. Plan §8 P3 governs this block.
Kernels are `kernel.md` (K-rules) and apply to vertex and fragment
kernels. Pipelines are `pipeline.md` (PI-rules) where this block
does not say otherwise. Schemas are `schema.md`.

## The declaration

- **RN1 — A render pipeline is a module-level `const`.** The
  generator recognizes the declaration functions by declaring file
  (`typegpu.ts`) and name, never by name alone.
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
- **RN3 — Builtins.** Rev 1, 2026-08-23. `VertexInvocation {
  vertexIndex: u32; instanceIndex: u32 }` emits `@builtin(vertex_index)`
  and `@builtin(instance_index)` for the fields the kernel reads.
  `FragmentInvocation { frontFacing: boolean }` emits
  `@builtin(front_facing)` the same way. The fragment position is
  the varyings' `position` field (RN7), never a second builtin: WGSL
  admits one `@builtin(position)` per entry point.

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

- **RN7 — The varyings class.** Rev 1, 2026-08-23. A `@CStruct`
  class whose field named `position` of type `Vec4f` is
  `@builtin(position)`, and whose other fields are `@location(n)` in
  declaration order from 0, with `@interpolate(flat)` for integer
  types. A varying field is `f32`, `i32`, `u32`, a library `f32`,
  `i32`, or `u32` vector, or a library `f16` vector (which puts
  `enable f16;` on the module). `boolean`, a matrix, a schema class,
  `f16`, and a `FixedArray` are diagnostics. The class is not a
  schema: it is never laid out for a buffer and never gets layout
  constants, and the same class cannot also be a binding item or a
  vertex schema. A varyings class with no `position: Vec4f` field is
  a diagnostic.
- **RN8 — The fragment output.** A `Vec4f` return is `@location(0)
  vec4<f32>`. A multiple-target form (a `@CStruct` return with one
  `@location(n)` per `Vec4f` field) arrives with a library overload
  that types it, in a later phase. Until then every declaration
  types the fragment return as `Vec4f`, and a different return fails
  subscript's checker.

## Emission

- **RN9 — One module, two entries.** The render pipeline's module
  holds the structs, the module constants and private variables the
  two kernels reach (K19, K20), the bindings, the helpers, `@vertex
  fn <vert>`, and `@fragment fn <frag>`, in that order (K14). The kernels that
  reach a binding decide its visibility: vertex, fragment, or both.
  A binding no kernel reaches is a diagnostic. `enable f16;` depends
  on the types this module references. The golden is
  `programs/<stem>.<name>.wgsl` (K16).
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
  Rev 1, 2026-08-24. `x05-live-triangle` draws one flat-colored
  triangle into a 64×64 `rgba8unorm` target whose vertices put no
  pixel center on an edge, copies the texture to a buffer
  (`bytesPerRow` 256), maps, reads, and compares every pixel with
  the host's own point-in-triangle test at pixel centers: inside →
  the color, outside → the clear color. It prints `PASS` or `FAIL
  x=<n> y=<n> expected=<rgba> got=<rgba>`. No reference hash exists.
  A fragment color constant in a pixel-oracle program must not
  produce an exact `.5` product with 255. The float-to-unorm
  rounding of a tie is implementation-defined: for `0.5`, NVIDIA
  (Vulkan and D3D12) returns 127 and Apple (Metal) returns 128
  (measured 2026-08-24, `x17-live-indirect` pixel 1,2). `0.6` maps
  to 153 on both.
- **RN15 — Draw variants.** `b07-draw-variants` and
  `x06-live-draw-variants` cover `drawIndexed` with an index buffer
  and an instanced draw through `renderPipelineInstanced`, with the
  same host rasterizer: the host knows every triangle.
- **RN17 — Render bindings.** `b08-render-bindings` declares a
  pipeline through `renderPipelineL` with a uniform read by the
  vertex kernel and a storage buffer read by the fragment kernel,
  and prints each layout entry's visibility by name.
  `x07-live-render-uniform` offsets the triangle through the uniform
  and tints it through the storage buffer, compared with the host
  rasterizer.

## Index format and cull (P8 slice 2)

- **RN18 — The index format is a spec member.** `RenderPipelineSpec`
  gains `indexFormat?: GPUIndexFormat = "undefined"`. The generator
  emits `<name>_INDEX_FORMAT: GPUIndexFormat` when the literal names
  a format. `RenderPipeline` gains `setIndexBuffer(pass, buffer:
  GPUBuffer)`, which sets the buffer with the spec's format, offset
  0, and the buffer's full size. On a pipeline whose spec has no
  format the method traps with `RN18`. `pass.setIndexBuffer` and
  `pass.drawIndexed` stay available (RN11). `b07-draw-variants`
  moves to the spec form and prints `quad_INDEX_FORMAT` by name.
- **RN19 — Cull mode is proven live.** `x18-live-cull` draws one
  triangle twice with `cullMode: "back"` and `frontFace: "ccw"`:
  once with the indices `0, 1, 2` and once with `0, 2, 1`. Before
  the second draw the program reads the index buffer back and
  asserts it holds `0, 2, 1`, because a dropped write also yields
  an empty image. The host rasterizer (RN14) gains the same cull
  rule: it computes the signed area in clip space and drops a
  triangle whose winding the spec culls. The program compares both
  images with the host rasterizer and prints `PASS`. `b17-index-cull`
  prints the `_INDEX_FORMAT` constant and the spec's `cullMode` and
  `frontFace` by name on both tiers.

- **RN20 — The strip topology is proven.** Rev 0, 2026-08-24.
  `RenderPipelineSpec.topology` already reaches the pipeline
  descriptor (RN12). `b19-strip` declares `topology:
  "triangle-strip"`, draws four vertices as two triangles, prints
  the spec's topology by name, and `x20-live-strip` compares the
  strip image against the host rasterizer, whose triangle list for
  a strip of `n` vertices is the `n - 2` triangles with the even-odd
  winding flip. `stripIndexFormat` stays with the API layer.

## Rejections

- **RN16 — Each rejection is a named diagnostic with a red fixture.**
  Rev 1 adds: `setIndexBuffer` on a pipeline without an index format
  (RN18 trap, `t`-style) and a non-literal `indexFormat`.
  Rev 1, 2026-08-23. The generator's set: a vertex schema field
  outside RN5, a varyings class without `position: Vec4f`, a varying
  field outside RN7, a vertex kernel that writes any storage binding
  (every write, not the first), a binding no kernel reaches (RN9), a
  class that is both a varyings class and a schema. Two cases fail
  subscript's checker first, because the library's declaration
  types them: a fragment return outside `Vec4f` and a kernel
  signature outside RN2. Their fixtures assert the checker's
  diagnostic (PI13 Rev 1). A `textureSample` call is a K10 method
  outside the table until P5, and its fixture asserts K10.
