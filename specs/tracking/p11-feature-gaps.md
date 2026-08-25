# P11 — the low-cost feature gaps

Plan §8 P11. Ranked from the P10 survey. Slices: (1) texture upload
and strip, (2) pointer input, (3) read-access storage textures with
a backend probe first, (4) blending. Deferred: depth, multisample,
integer textures, texture arrays, 3D textures.

## Status

2026-08-24: plan written. Slice 1 starts after P10 slice 1 round 1
lands.

## Slice 3 probe (2026-08-24)

Question: do the backends accept read-access storage textures?
Sources: the yawgpu tree at `13ac0b4` (read), and a probe program
through the facade (run).

Probe: a compute module with `texture_storage_2d<r32float, read>`
and with `read_write`, the matching bind group layouts, and the
pipelines, each inside a validation error scope. Result: every stage
`ok` on yawgpu Noop, yawgpu Metal (Apple M2), and Dawn. `PASS` on
all three.

From the yawgpu source: the Tint shim allows the
`readonly_and_readwrite_storage_textures` language feature, the bind
group layout validation accepts `ReadOnly` and `ReadWrite` and
rejects non-`read` access in the vertex stage, and the MSL path
carries Tint's store fence. One defect reported to the yawgpu owner
(`HANDOFF.md` there, 2026-08-24): `texture-formats-tier2` is
advertised without the `readWriteTextureSupport` device query that
Dawn performs, so a below-tier-2 device would accept an invalid
`read_write` layout on a tier-2 format. The r32 baseline this
project ships first is unaffected.

Verdict: slice 3 is unblocked. The r32 formats first, tier-2
formats behind `hasFeature("texture-formats-tier2")`.

## yawgpu response (2026-08-24)

The tier-2 report landed in yawgpu: `a3b1db0` gates
`texture-formats-tier2` on `MTLDevice.readWriteTextureSupport`, and
`feed066` adds the Vulkan twin with
`shaderStorageImageExtendedFormats`. The slice 3 caveat on tier-2
formats reduces to `hasFeature("texture-formats-tier2")`, which now
reflects the device on both backends.

## Slice 1 contract (2026-08-24)

`texture.md` TX9 (the two upload helpers over the API layer's
`writeTexture`, the encode table, the row-alignment trap) and TX10
(`b18-texture-upload`, `x19-live-texture-upload`), `render.md` RN20
(`b19-strip`, `x20-live-strip`, the host rasterizer's strip
expansion with the even-odd winding flip). Unblocks the `clouds`
port and the strip reductions.

## Slice 1 (2026-08-24)

Landed: `writeTexturePixels` (the encode table, round half away from
zero for `rgba8unorm`), `writeTextureBytes`, the 256-byte
row-alignment trap (red recorded), `b18-texture-upload` and
`b19-strip` byte-identical on both tiers, `x19-live-texture-upload`
against the host pixels, `x20-live-strip` against the host
rasterizer's strip expansion with the winding flip before the cull.

Evidence: gate green, 249 passed, 198 s at load 2.3.
`tools/live.sh` x01–x20 PASS: Metal (yawgpu) 59.65 s, Dawn 55.53 s.
The program-change gate grew from 86 s (P8 open) to 198 s with the
program count; the growth tracks the suite size, not a regression.
A future phase can parallelize the differential lane if the growth
starts to hurt iteration.

## Slice 2 contract (2026-08-24)

`window.md` Rev 4: W2 Rev 2 (`frame` gains `pointerX`, `pointerY`
in surface pixels with `-1, -1` before entry, and a `buttons` bit
set) and W3 Rev 1 (pointer position and buttons are level state the
host stores and `frame` samples). Every windowed example adds the
three parameters and reads them or not. The W13 signature test
follows. Unlocks `fluid-with-atomics` unreduced and the drawing
examples.

## Slice 2 (2026-08-24)

Landed: the host stores the pointer position in surface pixels
(`-1, -1` before entry) and the button bit set, `frame` carries
`pointerX`, `pointerY`, `buttons` after `key` (W2 Rev 2), the
thirteen windowed examples take the new signature, the W13 test
checks it, and `fluid-double-buffering` follows the pointer x while
the left button is down with the keys kept.

Evidence: gate green, 249 passed, 196 s. Smoke runs `triangle`,
`fluid-double-buffering`, `boids` print `window:frames=30` on
Metal, read before this commit.

## Slice 3 (2026-08-24)

Landed: `ReadStorageTexture2d<F>` and `ReadWriteStorageTexture2d<F>`
with host bodies, the `read`/`read_write` emission and layout
access, the bind-group path, and the TX11 diagnostic for a `load`
on the write-only wrapper (red recorded). `b20-read-storage` prints
the kinds and access by name on both tiers; `x21-live-read-storage`
ping-pongs a blur over two dispatches against the host lane.

Evidence: gate green, 250 passed, 205 s. Live x01–x21 PASS: Metal
(yawgpu) 62.71 s, Dawn 58.56 s. Remaining slices: blending (4).

## Slice 4 first live run (2026-08-24)

`x22-live-blend` failed at `x=41 y=35 expected=15,46,138,153
got=48,54,142,255`, byte-identical on Metal and Dawn. The GPUs
agree with each other and carry the both-triangles alpha
(`one`/`one` saturates at 255), so the host oracle is wrong: its
local edge test misses the second triangle at a pixel center on or
near an edge. The fix moves the coordinates so every pixel center
keeps a margin from every edge, the practice of the other live
render programs. The blending code itself is not implicated.

## Slice 4 close (2026-08-24)

Blending landed: `RenderPipelineSpec.blend` through both device
forms, the host rasterizer's two factor pairs with the `RN21` trap
(red recorded), `b21-blend` on both tiers, and `x22-live-blend`
fixed by coordinates with a checked `0.0025` edge margin for every
pixel center (the failing pixel sat on the first triangle's `bc`
edge at 3e-8). Gate green, 251 passed, 216 s. Live x01–x22 PASS:
Metal (yawgpu) 66.09 s, Dawn 61.54 s.

P11 slices 1–4 are complete. Deferred remains: depth, multisample,
integer textures, texture arrays, 3D textures.

## Correction to the slice 4 close (2026-08-25)

The slice 4 close records `Live x01–x22 PASS: Metal (yawgpu) 66.09
s, Dawn 61.54 s`. Both runs used macOS. The claim did not hold on
every adapter. A Vulkan live run on an NVIDIA adapter failed
`x20-live-strip` and `x22-live-blend`. Both programs carried a
unorm tie in the pixel oracle. `specs/tracking/windows.md` W5 holds
the defect, the rule change, and the fix.

`24ca42e` changed both programs, so the times above measure a tree
that no longer exists.

Corrected claim, at `bea7da5` on the reference machine (Apple M2,
macOS): `tools/live.sh` x01–x22 PASS on both tiers, Metal (yawgpu)
65.81 s, Dawn 61.55 s. The gate is green at the same tree, 253
passed and 1 ignored. `specs/tracking/build-time.md` row 53 holds
the times.
