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
