# P21 — sharp UI text

Plan §8 P21. Contracts: `window.md` W2 Rev 6 and W13 Rev 4, `ui.md`
UI15 Rev 8 and UI17 Rev 2 (slice 1), UI2 Rev 3 (slice 2).

## Survey (2026-11-23, Opus subagent, code read at `d9a7582`)

HiDPI path:

- The host opens the window at `LogicalSize(960, 640)` and configures
  the surface from `inner_size()`, which is physical
  (`crates/window/src/main.rs`). On a scale-2 display the surface is
  1920x1280.
- `CursorMoved` positions pass to `frame` unchanged, in physical
  pixels. The host reads no scale factor and has no
  `ScaleFactorChanged` arm.
- The UI core lays out in atlas pixels. `render` writes the target
  size into the viewport uniform and intersects clips with the
  physical target.
- The vertex kernel maps pixels to clip space through the viewport
  uniform. A logical viewport therefore needs no WGSL change.

Atlas route:

- `ab_glyph` 0.2.32 with `ab_glyph_rasterizer`, `owned_ttf_parser`,
  and `ttf-parser` is in `Cargo.lock` through `sctk-adwaita` and in
  the local registry. Licenses: Apache-2.0, and MIT OR Apache-2.0 for
  `ttf-parser`. `fontdue` is not in the lock.
- The harness depends on `subscript-typegpu-gen`. A rasterizer there
  compiles into every gate build. A separate generator crate keeps it
  out.
- No facility in `lib/` reads a file at run time. The atlas stays a
  compiled constant.
- U2 measured the whole ui module at +1.6 s dev and +2.1 s ship per
  importing program (`build-time.md`). No measurement isolates the
  share of the 32,768-digit atlas string.
- Estimate *(not measured)*: 95 glyphs of a sans face at about 15
  pixels make a 2x atlas of about 80,000 pixels. As one hex digit
  per 4-bit pixel with zero runs coded, the string stays near its
  current size.

Owner decisions (2026-11-23): fix both causes. The font is Roboto
(Apache-2.0).

## Slice 1

Round 1 (Codex, 2026-11-23). The host calls `scale` after `init` and
on `ScaleFactorChanged`, before the queued input. `uiScissor` maps a
logical clip to physical pixels, and `render` takes the scale.
`ui-demo` divides the pointer by the factor. `b24` gains four
`uiScissor` lines: scale 2 inner `8 12 20 16`, odd edges
`6 10 16 24`, target edge `0 70 101 9`, and scale 1.5 `4 7 13 19`.
No earlier line moved. Reds: `UIT1 UiRenderer.render scale=0
(author)`, and the W13 signature test rejects a wrong `scale`
signature.

Window run (owner, Metal, scale-2 display): `tools/window.sh
examples/ui-demo/main.ts` printed `window:frames=3954` and exited 0.
The owner reports that the text reads more easily and that input
lands where the pointer is.

## Slice 2

Round 2 (Codex, 2026-11-23). `crates/ui-atlas` rasterizes
`Roboto-Medium.ttf` at the submodule commit `77fe458` with
`ab_glyph`. `Cargo.lock` gains only the new workspace member. The
atlas is 256x308 texels, and `UI_ATLAS_ALPHA_CODE` holds 21,560
characters (the hex string held 32,768).

Compile time of `b24` (median of three, scratch harness, front end
through link, no GPU work):

| Atlas | Dev (s) | Ship (s) |
| --- | ---: | ---: |
| microui hex string | 6.95 | 5.29 |
| microui, empty string | 6.93 | 5.30 |
| Roboto code string | 6.86 | 5.44 |

The atlas string has no cost above the run-to-run variation. The
encoding is kept, because it is shorter and costs nothing.

Moved goldens: 20 `b23` lines, each decided by a glyph width (the
centered slider text x, the caret x, the clip width of `large`), and
the `b24` line `vertices fnv1a` (1253674997 to 3079871005). The
`b24` and `x24` WGSL goldens rename the sampler binding.

## Phase review (2026-11-23)

A no-context Opus reviewer read the working tree: 0 CRITICAL,
2 MAJOR, 7 MINOR.

- MAJOR: `docs/windowed-program.md`, `docs/from-typegpu.md`, and
  `README.md` did not name the `scale` entry. Fixed.
- MAJOR: the slice 2 evidence was not recorded. Recorded here.
- MINOR, fixed: the sampler name `filter` is reserved in WGSL and
  emitted as `filter_`. UI14 Rev 3 names it `atlasSampler`.
- MINOR, fixed: reds `UIT1 UiRenderer.render scale=NaN (author)` and
  `UIT1 uiScissor scale=0 (author)`.
- MINOR, fixed: a test asserts that every atlas rect size is a
  multiple of `UI_ATLAS_SCALE` and that every glyph rect is 36
  texels high.
- MINOR, fixed: spec dates, a sentence fragment, a stale plan clause,
  and the order of `rule-ids.txt`.
- MINOR, accepted: the cell-width error of the tool has no test,
  because Roboto cannot reach it.
- MINOR, accepted: the provenance test runs `git` to read the
  gitlink, so the gate needs `git` and a `.git` directory. The
  previous generator read the submodule's `HEAD` file instead.
- MINOR, accepted: the host's `scale` call order has no headless
  test. W13 covers signatures only, and the owner's window run
  covers the order.

Evidence: `tools/live.sh` on Metal green, 196 s, 1 passed (the live
suite, `x24` included). `tools/gate.sh --require-backend` green,
499 s, 384 passed, 1 ignored (live). Windows regeneration of the
atlas is open.

Window run (owner, Metal, scale-2 display): `tools/window.sh
examples/ui-demo/main.ts` printed `window:frames=576` and exited 0.
The owner reports no defect in the text, the frames, or the icons.
Open: a visual check at scale 1 and the Windows regeneration run.

P21 is COMPLETE.
