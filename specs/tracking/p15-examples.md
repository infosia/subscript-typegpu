# P15 — the example ports, round 2

Contract: `specs/blocks/examples.md` EX1–EX7. Plan §8 P15.
Upstream: TypeGPU v0.12.0 (MIT, Software Mansion), the
documentation examples and the `@typegpu/sort` and `@typegpu/color`
packages.

## Survey (2026-08-31)

The remaining portable candidates were read at the source level.
Findings that shape the slices:

- The slot and accessor idiom appears in every candidate that
  imports a package: `compareSlot` and the scan slots (sort),
  `oklabGamutClipSlot` and the pattern slot (oklab), the perlin
  cache injection (perlin-noise). Each use resolves to a committed
  constant, a K2 helper, or one emitted variant per choice.
- `prefix-scan` calls GPU functions on the CPU for its expected
  arrays. This project's modules carry real host bodies, so the
  check holds without that mechanism.
- `box-raytracing` writes one helper as raw WGSL and uses
  `discard`. The port reimplements the helper (EX6) and returns
  alpha zero under the premultiplied blend.
- `stable-fluid` samples `rgba16float` storage textures in compute
  with a linear sampler. The backend probe is the slice 4 entry
  condition.
- The upstream sort package sorts `u32` only at workgroup size 256
  and pads to a power of two with copy passes. The module commits
  the power-of-two requirement instead and traps otherwise.

## Status

2026-08-31: the plan section and this record written. Slice 1 next.

## Slice 1 (2026-08-31)

Landed: `trippy-raymarching` (a twisted, repeated sphere-lattice
raymarch, the upstream slider defaults committed, the pointer
drives the lattice scroll), `disco` (three committed patterns —
rings, swirl, kaleidoscope — cycled on the key scalar), and
`perlin-noise` (the noise module's `perlin3d` over a committed
grid, one committed sharpening function, no compute pass).
Twenty-three examples compile in the gate.

Evidence: `tools/gate.sh --require-backend` green, 257 passed, 1
ignored, 157.50 s (the coding agent's timed run, counts confirmed
by a second run). The three smoke runs print `window:frames=30`
on Metal (yawgpu) with zero `FAIL` lines, read before this
commit. The EX2 comment pass and the owner's visual runs follow.
