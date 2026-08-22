# P5 — textures and samplers

Status: **in progress**. Opened 2026-08-23. Plan §8 P5. Contract:
`specs/blocks/texture.md` (TX1–TX8).

## Slice 1 — the wrappers, the texture calls, `b11-texture`, `x10`, `x11`

Delivered 2026-08-23, committed at `4b34f4a`. Planner verification:
`tools/gate.sh --require-backend` green, 214 tests in six
executables (facade 4, typegpu-gen 7 and 35, harness 18 in 41.21
seconds plus the ignored live test, webgpu-gen 3 and 147).
`b11-texture` equal on both tiers with its golden: two groups, a
sampled texture, a sampler, a write-only storage texture, a uniform
in group 1, `textureLoad`, `textureSampleLevel`, `textureStore`.
Five TX8 fixtures and the resource-kind trap. Live run outside the
sandbox at `4b34f4a` (Metal): ok, 23.60 seconds, `x01`–`x11`
`PASS` — the compute sampling of the checkerboard and the fragment
`textureSample` both equal the host sampling body. Measurement:
44 s / 0.2 s / 54 s / 51 s / 6.

Noted for the phase review: the emitter copies a uniform binding
into a local `var` of the same name (`var params = params;`) when
the author's local shares the binding's name. Valid WGSL, but a
shadowing shape.

## Phase review (2026-08-23)

A fresh reviewer ran the gate (green, 51.14 s) and found CRITICAL 1,
MAJOR 5, MINOR 15. CRITICAL: a kernel local that shares a binding's
name shadows it in WGSL, so a later read of the binding resolves to
the local — valid WGSL, silent divergence from the host. MAJOR: the
integer sample types and `Rgba8uint` generated ill-typed WGSL with
no diagnostic; every P5 diagnostic cited the meta-rule TX8; PI9 was
never revised for `BindingResource[]`; TX2 claimed multi-group
render pipelines the library does not declare; `Texture2d.store`
had an empty host body. Resolutions in the specs: K14 Rev 4 (the
emitter reserves the module-scope name set and suffixes colliding
locals), TX1 Rev 1 (float sample type and float formats only, the
rest diagnostics, host bodies trap for rejected operations), TX2 Rev
1, TX8 Rev 1 (cite the enforced rule), PI9 Rev 1, PI6 wording, the
meta-rule line in `rule-ids.txt`. The code findings go to the
coding agent.

## Exit criteria

| # | Criterion | Evidence |
|---|---|---|
| 1 | `b11-texture` gate-green with its WGSL golden | Slice 1, `--require-backend` green |
| 2 | `x10-live-texture` and `x11-live-fragment-sample` `PASS` against the host sampling body | Metal at `4b34f4a`, 23.60 s |
| 3 | Every TX8 rejection has a red fixture | Five fixtures and one trap, one diagnostic each |
| 4 | Budgets hold | 44 s / 0.2 s / 54 s / 51 s / 6 |
