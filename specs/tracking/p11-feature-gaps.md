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
