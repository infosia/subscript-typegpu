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

## Owner's run (2026-08-23)

Dawn (`SUBSCRIPT_TYPEGPU_BACKEND` unset, `libwebgpu_dawn.dylib`),
`tools/window.sh examples/window-triangle/main.ts` at `938d6fd`: the
window opened, the triangle drew, space advanced the clear color,
resize kept the drawing, close printed `window:frames=207` and the
host exited. An earlier run printed no `window:frames` line and its
end was not observed. The Metal run and the `--frames` run follow
the review fixes.

## `--frames` run 1 (2026-08-23)

After the review fixes, `tools/window.sh examples/window-triangle/main.ts
--frames 120` failed on Metal and Dawn with `window:compile: rejected
with 13 diagnostic(s)`: `imported module ./main.typegpu is not among
the program's files`. The host compiled the program without the
TypeGPU support module. The W13 signature test reads the HIR with
poisoned imports, so it could not see it. Resolution: W8 Rev 1
(diagnostics before the one line), W13 Rev 1 (one program loader
shared by the host and the harness, and a gate test that compiles
the example through it).

## Close (2026-08-23)

Review fixes landed: all 19 items and `--frames`. Round 2: one
program loader in the harness crate shared by the dev lane, the
coverage lane, the trap tests, and the window host; the W13 Rev 1
gate test compiles the example through it (scratch red: `imported
module './broken-typegpu' is not among the program's files`); W8
Rev 1 prints the diagnostics before the one line.

Evidence: `tools/gate.sh --require-backend` green, 246 passed, 168 s.
`tools/window.sh examples/window-triangle/main.ts --frames 120`:
Metal (yawgpu) `window:frames=120` exit 0, Dawn `window:frames=120`
exit 0. The owner's interactive Dawn run (above) covers the triangle,
space, resize, and close. `tools/live.sh` x01–x18 PASS on Metal
(50.93 s) and Dawn after the loader change.

P9 COMPLETE 2026-08-23. Open: the ship tier for the window host,
Linux surfaces, a surface-format list on the render spec.

## sRGB round defect (2026-08-24)

The commit `b842e58` aborts at window start: objc2's type check
rejects `setColorspace:` with a `*mut c_void` argument (`^v` against
`^{CGColorSpace=}`). The gate stayed green because it never runs the
window, and the commit landed before the smoke run's result was
read. Two records: the fix types the pointer with an opaque
`CGColorSpace` and `RefEncode`; and a window-host change is
committed only after a `--frames` smoke run passes.
