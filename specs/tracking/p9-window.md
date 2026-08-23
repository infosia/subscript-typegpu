# P9 — the window host

Contracts 2026-08-23: `specs/blocks/window.md` W1–W13, `facade.md`
L14, `facade-generator.md` F23, plan §8 P9. Reference: the proof of
concept's `windowed-example.md` W1–W9, `host-embedding.md` H1–H3,
`push-entries.md` Z1–Z6, and its two unfixed defects (per-frame error
print, transient acquire statuses treated as failure), both closed
by W5 and W8.

## Status

2026-08-23: contracts written. Starts after P8 slice 2 closes.

## Landed (2026-08-23)

F23 `[[host_only]]` rows (22) and the generated
`crates/facade/src/surface.rs` with its regeneration gate (scratch
red: `crates/facade/src/surface.rs differs; run tools/regen.sh`).
`crates/window`: winit host, exact-version pins, `create_surface`
the only platform code, the W3–W8 loop. `examples/window-triangle/main.ts`
and `tools/window.sh`. W13: the signature test through the HIR,
clippy, the window build in the gate. Cold build 45 s → 48 s.

Smoke run by Claude (sandbox disabled, 10 s each, then terminated):
Metal (yawgpu) and Dawn both ran without a failure line. The
interactive check (triangle visible, space advances the clear color,
resize, close prints `window:frames=<n>`) is the owner's run.
