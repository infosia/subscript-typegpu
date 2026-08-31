# P16 — texture arrays and the radiance cascades

Contract: `specs/blocks/texture.md` TX13, TX14, and
`specs/blocks/examples.md` EX1-EX7. Plan §8 P16. Upstream: TypeGPU
v0.12.0 (MIT, Software Mansion), the `@typegpu/radiance-cascades`
package and the documentation examples.

## Survey (2026-09-01)

The upstream sources were read at the file level. Facts that shape
the slices:

- The API layer already passes `depthOrArrayLayers`,
  `baseArrayLayer`, `arrayLayerCount`, and the `2d-array` view
  dimension. The TypeGPU layer and the generator are 2D-only.
- The radiance-cascades package and the `radiance-cascades`
  example write every cascade layer through a single-layer `2d`
  view (`baseArrayLayer`, `arrayLayerCount: 1`). WGSL array types
  appear at two sites only: the example's debug-overlay read
  (`texture_2d_array`) and the voronoi payload pack
  (`texture_storage_2d_array`, read and write).
- The upstream `radiance-cascades` example is a from-scratch
  implementation without package slots: the scene is a uniform
  struct of four disks and two boxes, and the ray march is inlined
  with 64 steps. This matches the compile-time architecture as
  written.
- The package's slots (`sdfSlot`, `colorSlot`, `rayMarchSlot`,
  `sdfResolutionSlot`) inject the scene at run time. The module
  port keeps the sizing, tiling, interval, and merge math, and
  every example supplies its own scene functions to its own
  kernel. `rc-docs-examples` tests the slot API itself and is
  dropped for that reason.
- `jump-flood-voronoi` packs color (layer 0) and the seed
  coordinate (layer 1) into one two-layer `rgba16float` array per
  ping-pong side. The step kernel reads a 3x3 neighborhood with a
  committed unroll and writes both layers. The halving loop runs
  on the host.
- Kernel-subset collisions known in advance: no shifts in K9 (the
  upstream tiling uses `2 << layer` — the module computes powers
  with a loop, the `bitonicSortStride` pattern), no `fwidth` (the
  edge smoothing takes a committed width), guarded dispatch at
  [16, 16] and [8, 8] workgroups exists.

## Status

2026-09-01: the plan section, TX13, TX14, and this record written.
Slice 1 (the probe, then the feature) is next.

## Slice 1 probe (2026-09-01)

Question: do the layers and the backend accept array-layer
creation, single-layer `2d` views, a layer-indexed
`texture_2d_array` load, and `texture_storage_2d_array` read and
write? Probe: a scratch headless program with raw WGSL through
the API layer, four steps inside validation error scopes, not
committed. Result: every step `ok` on yawgpu Noop and yawgpu
Metal (Apple M2). The API layer expresses every view, including
the `2d-array` dimension. Verdict: slice 1 is unblocked.

## Slice 1 (2026-09-01)

Landed: `Texture2dArray<f32>`, `ReadStorageTexture2dArray<F>`, and
`WriteStorageTexture2dArray<F>` with layer-indexed `load` and
`store` and real host bodies, the generator emission of
`texture_2d_array<f32>` and `texture_storage_2d_array<F, access>`
with a view-dimension discriminator on the layout entry, two
demonstrated-red access diagnostics, `b22-texture-array`
byte-identical on both tiers, and `x23-live-texture-array`
(a two-layer ping-pong read back through a single-layer `2d`
view). TX13 and TX14 joined the rule-id registry after the round,
so the two diagnostics cite TX11. The next round moves them to
TX13.

Evidence: gate green, 262 passed, 1 ignored, 190.54 s. Live lane
on Metal (yawgpu): `live::every_x_program_passes_on_a_real_adapter`
ok, 118.76 s, every x program through x23. Both new goldens
validate under `naga`. No existing golden moved.
