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
