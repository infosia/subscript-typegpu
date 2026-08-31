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

## Slice 2 (2026-09-01)

Landed: `lib/typegpu-sort.ts` (an ascending `u32` bitonic step
kernel with a host pass enumerator that traps on a
non-power-of-two length, and a 256-lane exclusive `f32` Blelloch
scan pair with a one-level host driver, limit 65536, host oracle
included), `bitonic-sort` (windowed, one comparator step per
frame, keys 1 and 2), and `prefix-scan` (headless, three lengths
against the host oracle byte for byte). Kernels defined in a
library module compile through `computePipeline` imports — the
measured answer to the slice's design question. Registration
touches four Rust files, the `typegpu-noise` pattern.

Measured rejections recorded: compound assignment through an index
signature, shifts outside K9 (the module computes the stride with
a loop), and a duplicate function name across checked modules.

The fourth acceptance gap: Tint requires parentheses when a
bitwise operator mixes with an arithmetic operator, and `naga`
does not. The Metal smoke caught `mixing '&' and '-' requires
parenthesis` in the bitonic module before the commit. K14 Rev 7
parenthesizes mixed bitwise chains, with a red-then-green
regression. No committed golden carries such a chain.

Evidence: gate green, 260 passed, 1 ignored, 171.51 s. On Metal
(yawgpu): `check:scan8 pass`, `check:scan123 pass`,
`check:scan4096 pass` (byte-exact), and the bitonic smoke prints
`window:frames=30` with zero `FAIL` lines after the fix. On Noop
the three checks print `noop`. The owner's visual run of
`bitonic-sort` and the EX2 comment review follow.
