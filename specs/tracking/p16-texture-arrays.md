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

## Slice 2 (2026-09-01)

Landed: the two array access diagnostics moved from TX11 to TX13
with both reds re-quoted, and `examples/jump-flood-voronoi` — a
512-square two-layer ping-pong (color, seed coordinate) over the
TX13 wrappers, seeds from the noise module at the committed
0.999 threshold, one halving step per frame from offset 256, keys
1 and 2 for the upstream buttons, and a single-layer `2d` render
view. Twenty-nine examples compile in the gate.

Evidence: gate green, 262 passed, 1 ignored, 198.90 s. The smoke
run prints `window:frames=30` on Metal (yawgpu) with zero `FAIL`
lines, read before this commit. No golden moved. The owner's
visual run follows.

## Slice 3 (2026-09-01)

Landed: `lib/typegpu-radiance-cascades.ts` (the CPU sizing and
ping-pong rule, seven kernel-subset helpers for the tiling, the
interval schedule, the ray angle, and the merge and gather sample
positions — powers of two by loop, K9 has no shifts),
`sdDisk` and `sdBox2d` in `lib/typegpu-sdf.ts`, and
`examples/radiance-cascades` — six cascade layers over two array
textures through per-layer single-layer views, a 64-step
sphere-trace of the uniform scene, the top-down merge, the
field build, the ACES render, and pointer dragging of the
nearest scene element. Thirty examples compile in the gate.

Measured rejection recorded: scalar trigonometry through an
`f64` cast (K12) — the mapped vector methods carry it.

Evidence: gate green, 263 passed, 1 ignored, 200.03 s. The smoke
run prints `window:frames=30` on Metal (yawgpu) with zero `FAIL`
lines, read before this commit. No golden moved. The owner's
visual run follows.

## Slice 4 (2026-09-01)

Landed: `examples/radiance-cascades-drawing` — painted strokes at
a committed radius into a 512-square scene, a full jump-flood
halving sequence per changed frame over the two-layer payload
pair with per-step immutable parameter buffers, derived SDF and
color textures, the slice 3 cascade kernel at the upstream
quarter resolution (128, five layers), and keys 1 and 2 for the
lit and signed-distance views. Thirty-one examples compile in the
gate.

Evidence: gate green, 263 passed, 1 ignored, 204.29 s. The smoke
run prints `window:frames=30` on Metal (yawgpu) with zero `FAIL`
lines, read before this commit. No golden moved. The phase
review and the owner's visual runs follow.

## Phase review (2026-09-01)

A fresh no-context reviewer read the cumulative diff. Findings:
0 CRITICAL, 4 MAJOR, 9 MINOR. The MAJOR set: the two array host
traps still cited TX11, LB1 and the trap sweep lagged the
radiance-cascades registration, the ACES denominator read 0.01
for Narkowicz's 0.14, and the module's two host functions had no
direct test.

Closures: the host traps cite TX13. LB1 lists the module and the
sweep scans it. Both examples carry the 0.14 constant. Direct
host tests pin `cascadeDimensions` (512 gives 256/512/6, 128
gives 64/128/5) and the `cascadeWriteSide` parity. The MINOR set
closed in the same round: the ninth flood candidate assigns its
distance, the program headers cite TX13 and TX14, key 0 joined
the drawing header, the `sample` arms exclude array receivers
with a new red pair, `Texture2dArray` gained the trapping `store`
and `dimensions`, the parity coupling and the vector-trigonometry
sites carry comments, and the style items closed. The plan's
layer-loop sentence now matches the module: the loop stays with
each example's bind-group table.

Evidence: gate green, 264 passed, 1 ignored, 205.48 s. The three
smoke runs print `window:frames=30` on Metal (yawgpu) with zero
`FAIL` lines, read before this commit. No golden moved. The
owner's visual runs close the phase.
