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
