# Block: window (W-rules)

P9 contract. Rev 0, 2026-08-23. Rev 1 (W2, W6, W11 from the phase
review), 2026-08-23. Rev 2 (W8, W13 after the first `--frames` run),
2026-08-23. Rev 3 (W8 Rev 2 exiting path, W9 Rev 1 sRGB), 2026-08-24. Rev 4
(W2 Rev 2, W3 Rev 1 pointer input), 2026-08-24. Rev 5 (W2 Rev 3, W3
Rev 2, W13 Rev 2 optional input entries), 2026-09-05. Plan §8 P9 and
§8 U3 govern this block. The
facade side is `facade.md` L14 and `facade-generator.md` F23. The
script side is the API layer (`api-layer.md`) and the TypeGPU layer.

## The split

- **W1 — The host owns the window, the surface, and the loop.** A
  Rust binary, `subscript-typegpu-window` in `crates/window`, creates
  the window, the instance, the surface on that instance, the
  adapter, and the device. It pumps the event loop, acquires and
  presents every frame, and translates input into plain values. A
  script encodes one frame per call. A script never configures a
  surface, never presents, and never sees a window, a canvas, or a
  context object.
- **W2 — Three entries.** The host calls three script exports:
  `init(instance: SubscriptTypegpuInstance, device:
  SubscriptTypegpuDevice, format: GPUTextureFormat): void` once
  after the surface is configured, `frame(view:
  SubscriptTypegpuTextureView, width: u32, height: u32, key: u32,
  pointerX: f32, pointerY: f32, buttons: u32): void` once per
  presented frame (Rev 2: the pointer position in surface pixels,
  `-1, -1` before the pointer first enters the window, and a button
  bit set — bit 0 left, bit 1 right, bit 2 middle — sampled when
  the frame is called), and `shutdown(): void` once before
  the host releases the device. `GPUTextureFormat` is the wire enum
  alias in `lib/wire-enum-aliases.generated.d.ts`, and the host
  passes the configured format's wire value (Rev 1: no second
  alias).
  The script wraps the device with `hostOwnedGPUDevice(instance,
  device)` (a `GPUHostOwnedDevice` has neither `dispose` nor
  `destroy`) and the view with `new GPUTextureView(view)` and
  disposes neither. The host owns both. A harness test checks the
  three signatures of the example through the HIR.
  Rev 3: four optional entries. When the script exports them, the
  host calls `wheel(deltaX: f32, deltaY: f32): void`,
  `keyDown(key: u32): void`, `keyUp(key: u32): void`, and
  `textInput(codePoint: u32): void` before `frame`, once per queued
  event, in event order. A script that exports none of them behaves
  as before. The signature test accepts the three required entries
  plus any subset of the four optional ones, each with its exact
  signature, and no other export.
- **W3 — The host translates input, the script decides.** Rev 1.
  The host handles five window events: close ends the loop, resize
  reconfigures the surface, a key press stores the key's Unicode
  scalar in one slot that the next `frame` call receives and clears
  (`0` when none), a pointer move stores the position in surface
  pixels, and a pointer button press or release updates the button
  bit set. A second press before the next frame replaces the first
  key. Position and buttons are level state, not events: `frame`
  reads the latest value and nothing clears them. The meaning of
  every input is script code.
  Rev 2: the host queues three more event kinds between frames and
  delivers them through the W2 Rev 3 entries the script exports,
  then clears the queue. A wheel event carries pixel deltas: a
  line delta is 30 pixels per line, and a pixel delta passes as is.
  A key event for shift, control, alt, backspace, or return carries
  its bit (1, 2, 4, 8, 16) to `keyDown` on press and to `keyUp` on
  release, and a repeat counts as a press. A character key press
  carries each Unicode scalar of the character to `textInput`, and
  the space key carries 32. The `key` parameter of `frame` keeps its
  Rev 1 meaning. An event kind whose entry the script does not
  export is dropped.
- **W4 — One instance.** The host creates the instance through the
  facade's `subscript_typegpu_create_instance` with the L13 backend
  request, creates the surface on it, and requests the adapter and
  the device from it. The script receives that instance and that
  device. The release order at shutdown is: `shutdown()`, the
  device, the surface, the instance, the window.

## Frames

- **W5 — The frame statuses.** Before `frame`, the host calls
  `wgpuSurfaceGetCurrentTexture`. On `SuccessOptimal` or
  `SuccessSuboptimal` it creates the default view, calls `frame`,
  presents, and releases the view and the texture. On `Outdated` it
  reconfigures with the current window size and skips the frame. On
  `Timeout` it skips the frame. On `Lost` or `Error` it prints one
  message with the status name and exits with a non-zero code. The
  host never calls `frame` without a texture.
- **W6 — The host steps async work after every frame.** After
  `frame` returns and the host presents, the host pumps
  `subscript_typegpu_instance_process_events` and steps the session
  until no async work is pending, as the harness does for a
  program. A `frame` export is synchronous. An `init` export can be
  `async` and is drained the same way before the first frame. After
  every entry call and every drain the host writes the script's
  `print` output to its own stdout, so a script line is visible when
  it is printed (Rev 1).
- **W7 — The surface format.** The host configures the surface with
  `bgra8unorm` when the surface capabilities list it, and with the
  first listed format otherwise. `init` receives the configured
  format. A TypeGPU render pipeline in the script declares its
  `format` literally, so the example asserts that `_TARGET_FORMAT`
  equals the received format and exits through `print` and an early
  return otherwise. The host treats a script that returns from
  `init` without a pipeline as a normal run.
- **W8 — Errors exit once.** Rev 1. A failure in the host (window
  creation, surface creation, adapter or device request, a script
  entry that fails) prints one line that names the step and exits
  with a non-zero code. A compile failure prints the compiler's
  diagnostics first, then the one line. The host never prints per
  frame. Rev 2: the `window:frames=<n>` line and the
  shutdown sequence run on the event loop's exiting path, so an
  application quit (macOS Cmd+Q) reports like a close (measured: a
  Cmd+Q run printed nothing).

## The crate

- **W9 — Dependencies and platforms.** `crates/window` depends on
  `winit`, `raw-window-handle`, and on macOS the `objc2`,
  `objc2-app-kit`, and `objc2-quartz-core` crates, each pinned by
  exact version in the workspace manifest. macOS attaches a
  `CAMetalLayer` to the `NSView` and sets the layer's color space
  to sRGB (Rev 1): an unset layer is interpreted in the display's
  native gamut, while a browser color-matches canvas content as
  sRGB, measured as a visible saturation difference. Windows passes
  the `HWND` and `HINSTANCE`. `create_surface` is the only platform-conditional
  code. On another platform the binary prints one line and exits
  with a non-zero code. The crate has no cargo feature.
- **W10 — The dev tier runs in process.** The host runs the script
  through `ReloadSession` on the main thread inside the event loop,
  with the facade's native symbols. The ship tier is not part of
  P9. The open item is recorded in the plan.
- **W11 — The command.** `tools/window.sh <program.ts>` builds the
  binary with `cargo build --offline -p subscript-typegpu-window` and
  runs it with `SUBSCRIPT_TYPEGPU_BACKEND_LIB` and
  `SUBSCRIPT_TYPEGPU_BACKEND` as `tools/live.sh` reads them. A
  sandboxed shell sees no adapter, so Claude runs the command with
  the sandbox disabled, as for `tools/live.sh`. The host prints
  `window:frames=<n>` on close. An optional `--frames <n>` argument
  (Rev 1) closes the window after `n` presented frames, so a run
  without a person at the keyboard ends through the close path.
  `tools/window.sh` passes its remaining arguments to the host.
- **W12 — The example.** `examples/window-triangle/main.ts` draws
  one triangle through a TypeGPU render pipeline
  (`renderPipeline<Vertex, Varyings>`) whose clear color the space
  key advances, on the API layer and the TypeGPU layer only. No
  facade name appears in it. It is not a suite program: no golden,
  no `programs/` entry. The owner or Claude runs it and the tracking
  records the frame count and the visual result.

## Gates

- **W13 — What the headless gate holds.** Rev 2. The regeneration
  gate over the F23 host-only declarations, the W2 signature test
  over the example (Rev 2: the optional entries by exact signature), a harness test that compiles the example on the
  dev tier through the same path the host uses (support module
  generated in memory, every `lib/` file loaded) without a device,
  `cargo build --offline -p subscript-typegpu-window`, and `cargo
  clippy --workspace`. The host and the harness share one program
  loader, so the example cannot compile in one and fail in the
  other. The windowed run is never
  CI-required (invariant 5). The cold build row in
  `specs/tracking/build-time.md` records the increment the crate
  adds.

## Exit

`tools/window.sh examples/window-triangle/main.ts` opens a window on
macOS with the Metal backend, draws the triangle, advances the clear
color on space, resizes without a message, and prints
`window:frames=<n>` on close. The same on Dawn. W13 green. The
increment to the cold build is measured and recorded.
