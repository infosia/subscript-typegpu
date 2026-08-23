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

## Phase review (2026-08-23)

Fresh no-context review (Opus) of `23ad3a3..938d6fd`: 1 CRITICAL,
4 MAJOR, 14 MINOR. CRITICAL: `await_future` looped forever on a
failure status (only `1` returned, only negatives failed). MAJOR: the
host never wrote the script's `print` output; the example lacked the
PI14 scope; `crates/window` depended on `subscript-codegen` directly
(Build time rule 4); `surface.rs` structs and signatures were literal
text in the generator, so the regeneration gate could not see a pin
change. Resolutions: W2 Rev 1 (`GPUTextureFormat`, no second alias),
W6 Rev 1 (the host writes script output after every entry call),
W11 Rev 1 (`--frames <n>`), F23 (derived from the yml), plan layout
(`crates/window` through the harness crate, `examples/`). Code fixes
in one Codex round. The owner's first Dawn run printed no
`window:frames` line; the `--frames` path makes the close path
measurable without a person at the keyboard.
