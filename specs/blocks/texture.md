# Block: texture (TX-rules)

P5 contract. Rev 0, 2026-08-23. Plan §8 P5 governs this block.
Kernels are `kernel.md`, pipelines `pipeline.md`, render `render.md`.

## Binding wrappers

- **TX1 — Three more wrappers.** `Texture2d<T>` (`T` is `f32`,
  `i32`, or `u32`, the sample type) emits `var name: texture_2d<T>`.
  `Sampler` emits `var name: sampler`, `ComparisonSampler` emits
  `var name: sampler_comparison`. `StorageTexture2d<F>` emits
  `var name: texture_storage_2d<F, write>` where `F` is a library
  format marker class (`Rgba8unorm`, `Rgba8uint`, `Rgba16float`,
  `R32float`, `Rgba32float`) and the access is `write` in P5. A
  layout class field of any of these kinds is a binding like a
  buffer wrapper (PI3, PI5). The wrappers have real host bodies over
  a `T[]` image of known width and height, so a kernel runs on the
  host.
- **TX2 — Groups.** `computePipeline2` through `computePipeline4`
  and `renderPipelineL` carry up to four layout classes. Group index
  is parameter order, dense from 0.

## Kernel operations

- **TX3 — The texture calls.** Methods on the wrappers map to WGSL:
  `tex.dimensions(): Vec2u` → `textureDimensions(tex)`,
  `tex.load(coords: Vec2i, level: u32): Vec4<T>` → `textureLoad(tex,
  coords, level)`, `tex.sampleLevel(sampler, uv: Vec2f, level: f32):
  Vec4f` → `textureSampleLevel(tex, s, uv, level)`, `tex.sample(sampler,
  uv: Vec2f): Vec4f` → `textureSample(tex, s, uv)`,
  `storage.store(coords: Vec2i, value: Vec4f)` → `textureStore(st,
  coords, value)`. `sample` is legal in a fragment kernel only (it
  needs derivatives); in a compute or vertex kernel it is a
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

- **TX6 — Gate.** `b10-texture` binds a sampled texture, a sampler,
  a storage texture, and a uniform in a second group, in a compute
  kernel that loads, samples at level 0, and stores. Prints the
  layout entries' kinds by name and the WGSL line count.
- **TX7 — Live.** `x09-live-texture` uploads a 4×4 checkerboard
  through `queue.writeTexture`, runs the compute kernel that
  samples at pixel centers with a nearest sampler into a storage
  texture, copies the storage texture to a buffer, and compares
  every texel with the host body's result. `x10-live-fragment-sample`
  samples the same checkerboard in a fragment kernel over a
  full-screen quad with `sample` and compares with the host.

## Rejections

- **TX8 — The P5 set.** `sample` in a compute kernel, `store` on a
  sampled texture, a format marker outside TX1, a `Texture2d<T>`
  with `T` outside the set, a group gap, a resource kind mismatch
  (a trap). Each with a fixture.
