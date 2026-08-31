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

## Slice 3 (2026-09-01)

Landed: `lib/typegpu-color.ts` (the sRGB transfer pair, the HSV
pair, the Oklab matrix pair with a signed cube root, the
compositions, and the adaptive-L0.5 gamut clip with its private
cusp and intersection helpers, host bodies real), `oklab`
(windowed, hue on keys 1 and 2, the pointer as a probe ring, the
out-of-gamut checker — the kernel subset has no `fwidth`), and
`box-raytracing` (windowed, 343 flattened cells, a host look-at
basis instead of `mat4.aim`, zero alpha under the premultiplied
blend instead of `discard`). Registration follows the
`typegpu-sort` pattern, color ordered before noise.

Measured rejection recorded: a mutable kernel local requires an
initializer (S100, two sites in the gamut helpers).

Evidence: gate green, 261 passed, 1 ignored, 172.65 s. Both smoke
runs print `window:frames=30` on Metal (yawgpu) with zero `FAIL`
lines, read before this commit. Every `.wgsl` and `.expected`
golden stays byte-identical. The owner's visual runs and the EX2
comment review follow.

## Slice 4 probe (2026-09-01)

Question: do the layers and the backend accept
`texture_storage_2d<rgba16float, write>` and compute-stage
`textureSampleLevel` on an `rgba16float` sampled texture with a
linear sampler? Probe: a scratch headless program through the
public layers, six steps inside validation error scopes, not
committed. Result: every step `ok` on yawgpu Noop and yawgpu
Metal (Apple M2). The generator emits both forms. The Dawn run
waits for the owner's gated lane. Verdict: slice 4 is unblocked.

## Slice 4 (2026-09-01)

Landed: `stable-fluid` — nine 256x256 `rgba16float` fields with
host-side pair swaps, ten compute passes per frame (brush splat,
ink add, force add, velocity advection, ten viscosity Jacobi
iterations, divergence, pressure clear, ten pressure Jacobi
iterations, gradient subtraction, ink advection), native
compute-stage linear sampling for both advections, a 512-square
procedural Perlin background (EX6), keys 1, 2, and 3 for the
display modes, and the pointer brush. Twenty-seven examples
compile in the gate.

Measured rejections recorded: a non-literal compute workgroup
size (PI1), a scalar `Math.exp` through an `f64` cast (K12), and
a cast in a module constant initializer (K19).

Evidence: gate green, 261 passed, 1 ignored, 178.79 s. The smoke
run prints `window:frames=30` on Metal (yawgpu) with zero `FAIL`
lines, read before this commit. No golden moved. The owner's
visual run and the EX2 comment pass follow.

## Phase review (2026-09-01)

A fresh no-context reviewer read the cumulative diff. Findings:
0 CRITICAL, 4 MAJOR, 20 MINOR. The MAJOR set: the disco pattern
selection did not survive the host's per-frame key clear, the
box-raytracing fragment divided by a zero albedo channel, its
double gamma encode read as an error, and the sort module's trap
id `SORT1` existed in no block and outside the diagnostics sweep.

Closures: disco stores a pattern index on keys 1 to 3. The host
clamps every albedo channel to at least 0.004. A comment records
the upstream's deliberate double encode. `specs/blocks/library.md`
Rev 0 (LB1-LB4) documents the module set, the trap table, and the
module test duties, and the diagnostics sweep now covers every
library module. The trap has a demonstrated red
(`SORT1 bitonicSortPassCount length=3`). K14 Rev 8 parenthesizes
two different bitwise operators in one chain, from the WGSL
grammar *(docs)*, with an emission regression. The MINOR set
closed in the same round: headers gained the EX7 reductions,
five shutdowns dispose the bind group, the seeds lost the date
literal, and the perlin wrap comment no longer claims a seamless
loop. Two review questions stay recorded as questions: the
upstream density square, and the dev-tier f32 assumption behind
the byte-exact oracle claim (observed on Metal, not doubted).
