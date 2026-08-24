# Block: texture (TX-rules)

P5 contract. Rev 0, 2026-08-23. Rev 1 (TX1, TX2, TX3, TX8),
Rev 2 (TX9, TX10 upload), 2026-08-24. Rev 3 (TX11, TX12 read
access), 2026-08-24.
2026-08-23. Plan §8 P5 governs this block.
Kernels are `kernel.md`, pipelines `pipeline.md`, render `render.md`.

## Binding wrappers

- **TX1 — The texture wrappers.** Rev 1. `Texture2d<f32>` emits
  `var name: texture_2d<f32>`. `Sampler` emits `var name: sampler`.
  `StorageTexture2d<F>` emits `var name: texture_storage_2d<F,
  write>` where `F` is a library format marker class with float
  channels (`Rgba8unorm`, `Rgba16float`, `R32float`, `Rgba32float`)
  and the access is `write`. `Texture2d<i32>`, `Texture2d<u32>`,
  an integer format, and a comparison sampler are diagnostics in
  this revision: each needs a typed `Vec4i`/`Vec4u` operation set
  that a later revision adds. A layout class field of any of these
  kinds is a binding like a buffer wrapper (PI3, PI5). The wrappers
  have real host bodies over a `Vec4f[]` image of known width and
  height, so a kernel runs on the host. An operation the generator
  rejects has a host body that traps with its rule id (RC-9).
- **TX2 — Groups.** Rev 1. `computePipeline2` through
  `computePipeline4` carry up to four layout classes. `renderPipelineL`
  carries one. Group index is parameter order. An empty layout class
  is a diagnostic.

## Kernel operations

- **TX3 — The texture calls.** Methods on the wrappers map to WGSL:
  `tex.dimensions(): Vec2u` → `textureDimensions(tex)`,
  `tex.load(coords: Vec2i, level: u32): Vec4f` → `textureLoad(tex,
  coords, level)`, `tex.sampleLevel(sampler, uv: Vec2f, level: f32):
  Vec4f` → `textureSampleLevel(tex, s, uv, level)`, `tex.sample(sampler,
  uv: Vec2f): Vec4f` → `textureSample(tex, s, uv)`,
  `storage.store(coords: Vec2i, value: Vec4f)` → `textureStore(st,
  coords, value)`. `sample` is legal in a fragment kernel only (it
  needs derivatives). In a compute or vertex kernel it is a
  diagnostic. `store` is a statement. The host bodies implement
  nearest sampling for `sampleLevel` and `sample` at level 0, and
  the live programs compare against that host body with a
  `filterMode: "nearest"` sampler.
- **TX4 — Bind group resources.** Rev 1 of PI9's `createBindGroup`:
  the positional list is `BindingResource[]`, a library `@Descriptor
  BindingResource { buffer?: GPUBuffer | null; textureView?:
  GPUTextureView | null; sampler?: GPUSampler | null }`, one per
  binding in declaration order, and free factories `bufferResource(b)`,
  `textureResource(v)`, `samplerResource(s)`. A resource kind that
  does not match the layout entry's kind traps with the binding
  index and both kinds.
- **TX5 — The layout entry spec.** `BindGroupLayoutEntrySpec` gains
  `kind` values `texture`, `storageTexture`, `sampler`,
  `comparisonSampler`, plus `sampleType`, `format`, and
  `samplerType` members the runtime maps to the API layer's
  descriptors. The generator fills them from the wrapper types.

## Programs

- **TX6 — Gate.** `b11-texture` binds a sampled texture, a sampler,
  a storage texture, and a uniform in a second group, in a compute
  kernel that loads, samples at level 0, and stores. Prints the
  layout entries' kinds by name and the WGSL line count.
- **TX7 — Live.** `x10-live-texture` uploads a 4×4 checkerboard
  through `queue.writeTexture`, runs the compute kernel that
  samples at pixel centers with a nearest sampler into a storage
  texture, copies the storage texture to a buffer, and compares
  every texel with the host body's result. `x11-live-fragment-sample`
  samples the same checkerboard in a fragment kernel over a
  full-screen quad with `sample` and compares with the host.

## Rejections

- **TX8 — The P5 set.** Rev 1. `sample` in a compute kernel (TX3),
  `store` on a sampled texture (TX3), a format marker outside TX1
  (TX1), a `Texture2d<T>` with `T` outside the set (TX1), an empty
  layout class (TX2), a resource kind mismatch (TX4, a trap). Each
  with a fixture. A diagnostic cites the rule in parentheses, never
  TX8.

## Upload (P11 slice 1)

- **TX9 — The upload helpers.** Rev 0, 2026-08-24. `lib/typegpu.ts`
  exports `writeTexturePixels(queue: GPUQueue, texture: GPUTexture,
  pixels: Vec4f[], width: u32, height: u32)` for the float formats
  (each component converted to the texture's format by the encode
  table this rule fixes: `rgba8unorm` scales to a byte with
  round-half-away-from-zero, the float formats pass bits through)
  and `writeTextureBytes(queue, texture, bytes: u8[], bytesPerRow:
  u32, width, height)` as the raw form. Both call the API layer's
  `writeTexture` with a full-extent single-mip destination. A
  `bytesPerRow` below the WebGPU 256-byte row alignment for a
  multi-row write traps with `TX9` and the numbers. The host
  `Texture2d` constructor path stays: `writeTexturePixels` and the
  host image hold the same values, so a live program uploads once
  and compares a sampled result against the host body.
- **TX10 — The gate and live programs.** `b18-texture-upload`
  writes a procedural gradient through both helpers, prints the
  first and last encoded bytes by value, and runs the kernel's host
  lane. `x19-live-texture-upload` uploads the gradient, samples it
  on the GPU, and compares against `simulateCompute` with the same
  host pixels. A `t`-style fixture reaches the TX9 row-alignment
  trap.

## Read access (P11 slice 3)

- **TX11 — Read-access storage textures.** Rev 0, 2026-08-24.
  `lib/typegpu.ts` adds `ReadStorageTexture2d<F>` (emits
  `texture_storage_2d<F, read>`, layout entry access `read-only`,
  methods `load(coords): Vec4f` and `dimensions(): Vec2u`) and
  `ReadWriteStorageTexture2d<F>` (emits `read_write`, access
  `read-write`, `load`, `store`, `dimensions`). `F` is the TX1
  format marker set. The backends accept the r32 formats without a
  feature (measured 2026-08-24, `p11-feature-gaps.md`); a program
  that binds a non-r32 `read-write` format first checks
  `hasFeature("texture-formats-tier2")` and reduces or exits with a
  message. Host bodies read and write the `Vec4f[]` image as TX1.
  A `load` on the write-only `StorageTexture2d` stays a diagnostic.
- **TX12 — The programs.** `b20-read-storage` binds one `read-only`
  and one `read-write` `r32float` storage texture, copies through a
  kernel, prints the layout entry kinds and access by name, and
  runs the host lane. `x21-live-read-storage` ping-pongs a small
  blur between a `read-only` source and a write-only target over
  two dispatches and compares against `simulateCompute`. Rejection
  fixtures: a `Vec*b`-style misuse stays with SC5, a `load` on the
  write-only wrapper (the TX11 diagnostic), red.
