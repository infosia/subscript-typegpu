# P10 — the example ports

Contract: `specs/blocks/examples.md` EX1–EX7. Plan §8 P10. Upstream:
TypeGPU v0.12.0 (MIT, Software Mansion), the documentation examples.

## Survey (2026-08-24)

77 upstream examples were classified against this library's surface.
Portable now: boids, fluid-double-buffering, confetti,
xor-dev-centrifuge-2, triangle, gradient-tiles, square,
matrix-multiplication, matrix-next, dispatch. Portable with an sdf
and a noise module: ray-marching, caustics, smoky-triangle,
vaporrave, fluid-with-atomics, prng-cpu-gpu. Deferred by feature:
read-only storage textures, image upload, pointer input, depth,
blending and multisample. Out of scope: webcam and video external
textures, network fetches, ONNX, WebGL, react, three.js, runtime
shader text, render bundles, the TypeGPU debug logger. Upstream
sliders reduce to committed literals, and pointer input reduces to
the key scalar where the example survives it (EX7).

## Status

2026-08-24: contracts and the plan section written. Slice 1 next.

## Slice 1 round 1 (2026-08-24)

Landed: `tools/example.sh`, headless `matrix-multiplication`,
`matrix-next`, `dispatch`, windowed `triangle`, `gradient-tiles`,
`square`, `xor-dev-centrifuge-2`. EX4 compiles all seven in the gate.

Evidence: `tools/gate.sh --require-backend` green, 247 passed,
169 s. Headless on Metal (yawgpu) through `tools/example.sh`:
`check:product pass`, `check:naive pass`, `check:tiled pass`,
`check:dispatch pass` (counts 13, 35, 45 over non-multiple thread
counts). On Noop the three print `check:<name> noop`. Windowed on
Metal with `--frames 60`: all four print `window:frames=60`, exit 0.
`triangle` the same on Dawn. Comments per EX2 follow in a separate
commit by the planning side.

## Round 1 comment review (2026-08-24)

EX2 comments were drafted with an upstream cross-check and applied
by the planning side. The review also found: the `xor` shell's
subscript body computes a different image than its WGSL body (a K29
violation — the CPU lane can never match); dead `frameCount` state
in three windowed examples; four copies of the render-pipeline
descriptor because RN11's helper takes `GPUDevice` and a windowed
example holds `GPUHostOwnedDevice` (resolved as EG10); one
formula-vs-literal check in `dispatch`; a no-effect barrier in
`matrix-next`'s tiled kernel; `read` versus `readOne` spelling
drift. The upstream slider freezes are stated in each header (EX7),
and the unused `key` parameter is the W2 signature, kept. A cleanup
round fixes the code findings.

## Slice 1 round 2 (2026-08-24)

Landed: `confetti` (64 particles, compute update, the storage buffer
doubles as the instance stream, six vertices for the upstream
strip), `boids` (96 boids, guarded compute, A-B storage pair with a
bind-group swap, velocity-oriented instanced triangles),
`fluid-double-buffering` (32x32 grid, three passes ping-ponged, the
obstacle x on the key scalar per EX7). The EX2 comments for round 1
were applied with an upstream cross-check; three comments avoid the
banned prefix by naming the TypeGPU construct without its namespace.

Evidence: `tools/gate.sh --require-backend` green, 247 passed,
175 s. Windowed with `--frames 60` on Metal (yawgpu): all three
print `window:frames=60`, exit 0. `boids` the same on Dawn.

## Owner's visual runs (2026-08-24, Metal)

`boids` (387 frames), `confetti` (238), `fluid-double-buffering`
(2378, the A and D keys moved the obstacle), `xor-dev-centrifuge-2`
(1590): every run opened, animated, and closed through the close
path. Verdict: the first three look as intended.
`xor-dev-centrifuge-2` renders black. Open defect: either the
tone-map keeps the color near 0.1, or the frame uniform does not
reach the shader (an aspect of zero makes the uv NaN). The round-3
K29 fix gives the shell one shared formula, so the host lane can
evaluate it and split the two causes.

## Round 3 verification (2026-08-24)

Gate green, 247 passed, 172 s. Live x01–x18 PASS on the updated
yawgpu (`feed066`, Metal 51.70 s) and Dawn (47.82 s). `triangle`,
`confetti`, and `fluid-double-buffering` run under `--frames`.
`boids` trapped `PI15 ComputePipeline.dispatchThreads x=96 y=1 z=1`
on a fresh per-frame encoder: `writeGuard` compared raw handles and
the allocator reused a disposed address (PI15 Rev 2 fixes the
comparison to wrapper identity). The xor black screen is the
tone-map: the shell peaks at 0.03 at scale 0.09; with the spoke
clamped at zero and scale 2.0 the numeric peak is 0.53 and the mean
0.20 (computed over a radius-angle grid). A fix round carries both.

## Round 4 (2026-08-24)

`writeGuard` compares the encoder wrapper by reference (PI15 Rev 2):
the two-frames-two-encoders accept check passes and the
double-dispatch fixture stays red. The xor tunnel clamps the spoke
at zero and tone-maps at 2.0 in both shell bodies. Evidence: gate
green 248 passed 172 s; `boids`, `xor-dev-centrifuge-2`, `confetti`,
`fluid-double-buffering` each print `window:frames=60` on Metal.
The owner's visual re-check of xor is the open item for the slice 1
close.

## Owner's visual re-check (2026-08-24, Metal)

`xor-dev-centrifuge-2` after the round-4 tone-map fix (216 frames):
the tunnel renders as intended. Every slice 1 windowed example now
has a passed visual run.

## Slice 1 close (2026-08-24)

Round 5 landed: the fluid uniform holds only the live value, the
constants derive from `GRID_SIZE`, one staged grid write, confetti
on a guarded `dispatchThreads`, one shared vector for the angle
trigonometry, the boids guard bound through the library so the
`size: 16` literal left, and `createBindGroupHost` joined EG10.
Comment review for the three simulation ports applied with an
upstream cross-check.

Evidence: gate green, 248 passed, 173 s. `confetti`, `boids`,
`fluid-double-buffering` print `window:frames=60` on Metal. The
owner's visual runs cover all eight windowed examples, and the
three headless examples print their `check:` lines `pass` on Metal.

P10 slice 1 COMPLETE 2026-08-24: ten ports, `tools/example.sh`,
EG10 with the host bind-group form. Next: slice 2 (the sdf and
noise modules, five ports).

## Slice 2 (2026-08-24)

Landed: `lib/typegpu-sdf.ts` (seven functions from the mathematical
definitions, a generator test compiles and `naga`-validates a kernel
that calls each), `lib/typegpu-noise.ts` (a 16-bit LCG with a
documented state shape, `perlin3d` over committed permutation
tables through K19 Rev 4 `FixedArray` constants), the headless
`prng-cpu-gpu` differential, and the windowed `ray-marching`,
`caustics`, `smoky-triangle`, `vaporrave`.

Evidence: gate green, 249 passed, 177 s. `prng-cpu-gpu` prints
`check:prng pass` on Metal (byte-exact GPU versus host over 64
samples). The four windowed ports print `window:frames=60` on
Metal, and `ray-marching` also on Dawn. The owner's visual runs and
the comment review are the open items for the slice close.

## Slice 2 visual runs and close (2026-08-24)

The owner ran the four windowed ports on Metal: `ray-marching`
(171 frames after one run that ended without the frames line —
Cmd+Q bypasses `CloseRequested`, recorded below), `caustics` (278),
`smoky-triangle` (313), `vaporrave` (574). Verdict: all four look
as intended. The owner sees color differences against the upstream
browser examples. Expected part: the ports commit their own
palettes, and this `perlin3d` is the classic algorithm, not
upstream's seeded gradients (the headers state both). Open part,
recorded for the host round: the macOS layer sets no color space,
so the display interprets the pixels in its native gamut, while a
browser color-matches canvas content as sRGB. Two host items queue
for the next window round: the layer's sRGB color space (W9), and
the frames line on the `exiting` path so Cmd+Q reports like a close
(W8).

P10 slice 2 COMPLETE 2026-08-24.

## The color question resolved (2026-08-24)

The owner decided: the color differences against the upstream
browser examples come from the ports' own palettes and reductions,
and no color matching is pursued. The sRGB layer setting (W9 Rev 1)
stays because it makes the output independent of the display's
gamut. The exiting path (W8 Rev 2) closes the Cmd-Q report gap.

## Slice 3 start (2026-08-24)

Order decision (the owner delegated the P10/P11 order): P10 slice 3
(clouds, strip de-reductions) first because it proves the P11 slice
1 features in examples, then P11 slice 2 (pointer input), P10 slice
4 (fluid-with-atomics), P11 slice 3 (read-access storage textures),
P10 slice 5 (slime-mold and the jump-flood pair as far as the
features reach), P11 slice 4 (blending).

## Slice 3 (2026-08-24)

Landed: `clouds` (a CPU `perlin3d` noise texture uploaded with
`writeTexturePixels`, six marched density layers, committed
constants in the header), and the strip de-reductions — `confetti`
and `fluid-double-buffering` draw the upstream four-vertex
`triangle-strip` through RN20, with their comments reworded to
match. Sixteen examples compile in the gate.

Evidence: gate green, 249 passed, 197 s. `clouds`, `confetti`, and
`fluid-double-buffering` print `window:frames=30` on Metal. The
owner's visual run of `clouds` is batched with the pointer-input
round.

## Slice 4 (2026-08-24)

Landed: `fluid-with-atomics` — a 64x64 `AtomicU32` water grid,
gravity then sideways equalization through `add`/`sub`, a pointer
brush over the `sdLine` segment while the left button is down, the
`1`/`2`/`0` keys for water/wall/erase (the header states the
reduction), the fullscreen strip render. Eighteen examples compile
in the gate (fourteen windowed, four headless).

Evidence: gate green, 249 passed, 197 s. The smoke run prints
`window:frames=30` on Metal, read before this commit.

## Visual runs (2026-08-24, Metal)

The owner ran `clouds` (185 frames), `confetti` after the strip
change (343), and `fluid-double-buffering` with the pointer (794):
all three look as intended. `fluid-with-atomics` printed
`FAIL validation shader.wgsl:31:38 error: value cannot be
represented as 'i32'` — the PI14 scope made the backend rejection
visible — and drew nothing for 1555 frames. The run raced the fix
round's working tree, so whether the committed WGSL carries the
defect is open until the round lands. If the committed module does,
the lesson is that `naga` accepted a literal Tint refuses, and the
generator gains a rule.

## Fix round (2026-08-24)

Landed: the atomic fluid moves to a read-only current buffer plus a
cleared atomic delta buffer with a finalize pass and a swap, the
brush protects walls, the stroke clears on button release, the
tileable four-sample torus blend and the wrapped time in clouds,
and the impossible checks left both files. Gate green, 250 passed,
203 s. Smoke: clouds prints `window:frames=30`; `fluid-with-atomics`
still prints `FAIL validation shader.wgsl:31:38 error: value cannot
be represented as 'i32'` on Metal — the failure is in the committed
module, not the round's working tree. Diagnosis follows.

## The i32 diagnosis (2026-08-24)

`WALL_LEVEL: u32 = 2147483648` (2^31). The emitter writes integer
literals without a WGSL suffix, an unsuffixed literal is an
abstract int that concretizes as `i32`, and 2^31 does not fit:
Tint refuses the module, `naga` infers `u32` from the context and
accepts it. The same acceptance gap as the K22 barrier case. K14
Rev 6: every emitted integer literal carries its suffix. Every
`.wgsl` golden regenerates under the rule.

## K14 Rev 6 landed; the fluid failure persists (2026-08-24)

Every emitted integer literal carries its suffix, 40 goldens
regenerated and validated, the 2^31 fold test red then green. Gate
green, 251 passed, 203 s. Live x01–x21 PASS on Metal (62.49 s) and
Dawn (57.97 s). `fluid-with-atomics` still fails at
`shader.wgsl:31:38` with the same message, so the unsuffixed
literal was not this failure's cause: the suspect is a `u32` value
at or above 2^31 crossing into an `i32` constant context in one of
the example's four modules. The next round dumps the emitted
modules and fixes the site.
