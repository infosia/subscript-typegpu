# Block: ui (UI-rules)

U0 contract. Rev 0, 2026-09-05. Rev 1 (UI4: records reused across frames, UI13: no-root dump), 2026-09-05. Plan §8 U-phases govern this block.
`specs/tracking/imgui-survey.md` records the route decision. Render
rules are `render.md`, textures `texture.md`, buffers `buffer.md`,
the window host `window.md`, modules `library.md`.

The block delivers an immediate-mode GUI authored in subscript: a
core that turns widget calls and input into a command list, and a
renderer that turns the command list into one indexed draw per clip
rect over the TypeGPU layer. The core is host code and runs on both
tiers with no GPU. The renderer is the only GPU code.

## Source and data

- **UI1 — The source library.** microui version 2.02
  (https://github.com/rxi/microui, MIT) is pinned as the submodule
  `third_party/microui`. The core reimplements its behavior in
  subscript. No C from it compiles, and no line of it is copied
  (EX6). Where this block states a number or a rule, microui's
  `src/microui.c` at the pin is the reference. Where this block is
  silent, the pin decides.
- **UI2 — The atlas module is generated.** `subscript-typegpu-gen
  ui-atlas <repo-root>` reads `third_party/microui/demo/atlas.inl`
  and writes `lib/typegpu-ui-atlas.generated.ts` with:
  `UI_ATLAS_WIDTH` and `UI_ATLAS_HEIGHT` (128), `UI_ATLAS_ALPHA_HEX:
  string`, the 16,384 alpha bytes as 32,768 lowercase hex digits,
  `uiAtlasAlpha(): u8[]` which decodes it, `UI_ATLAS_RECT_X`,
  `UI_ATLAS_RECT_Y`, `UI_ATLAS_RECT_W`, `UI_ATLAS_RECT_H: i32[]`, the
  100 rects in microui's table order (the four icons, the white
  rect, the 95 glyphs for bytes 32 to 126), `UI_ATLAS_WHITE` (the
  white rect's index), `UI_ATLAS_FONT` (the index of the glyph for
  byte 0, so that byte `b` is at `UI_ATLAS_FONT + b`), and
  `UI_TEXT_HEIGHT` (18). The module header names the source path and
  the pinned commit. `tools/regen.sh` runs the subcommand. A
  generator test regenerates the module in memory and compares it
  byte for byte with the committed file (core principle 7).
- **UI3 — Ids.** An id is a `u32`, FNV-1a over the label's bytes:
  the seed is the id stack's top, or 2166136261 when the stack is
  empty, and each byte `b` from `charCodeAt` applies
  `h = (h ^ b) * 16777619` with wrapping. Every stateful widget takes
  a label string. `pushId(label)` and `popId()` scope the ids of the
  calls between them. Two widgets with one label in one scope share
  one id, and that is the author's responsibility, as in microui.

## The frame and the input

- **UI4 — One context, one frame.** `UiContext` is a reference
  class. A frame is `begin()`, widget and layout calls, `end()`.
  `begin()` clears the command list and the root list, computes the
  pointer delta, and takes the hover root from the previous frame.
  `end()` orders the root containers by z-index, clears the
  press, key-press, and text-input edge state, and stores the
  pointer position for the next delta. A widget call outside
  `begin()` and `end()` traps `UIT2`. Rev 1: the context keeps its
  command records, root records, and layout records across frames
  and overwrites them in place, so that a frame allocates only the
  strings of its text commands. Nothing in the context depends on
  `Context.collect()`.
- **UI5 — Input is pushed as plain values, before `begin()`.**
  `inputMouseMove(x: i32, y: i32)`, `inputMouseDown(x, y, button:
  u32)`, `inputMouseUp(x, y, button)`, `inputScroll(dx: i32, dy:
  i32)`, `inputKeyDown(key: u32)`, `inputKeyUp(key: u32)`, and
  `inputText(codePoint: u32)`. Buttons are bits: 1 left, 2 right, 4
  middle. Keys are bits: 1 shift, 2 control, 4 alt, 8 backspace, 16
  return. `inputText` appends one byte when the code point is 32 to
  126 and ignores every other code point (UI10). Down and press are
  distinct: down is level state, press is set by `inputMouseDown`
  or `inputKeyDown` and cleared by `end()`.
- **UI6 — Hover and focus.** For a widget with rect `r` and id `id`:
  the widget is under the pointer when the pointer is inside `r`,
  inside the current clip rect, and the widget's root container is
  the hover root. If it is under the pointer and no button is down,
  hover is `id`. If it is under the pointer and a press happened this
  frame, focus is `id`. If focus is `id` and no button is down and
  the widget lacks `UI_OPT_HOLD_FOCUS`, focus clears. If focus is
  `id` and a press happened outside `r`, focus clears. If it is not
  under the pointer, hover clears when hover was `id`. The hover root
  is the root container with the highest z-index under the pointer,
  taken from the previous frame's containers (UI4).

## Layout

- **UI7 — Rows and columns.** `layoutRow(widths: i32[], height:
  i32)` starts a row with up to 16 items. `layoutNext(): UiRect`
  returns the next item's rect: a width of 0 is the default width
  (68 plus two paddings of 5), a negative width extends to that
  many pixels from the body's right edge, a height of 0 is the
  default height (10 plus two paddings). Items advance by width
  plus spacing (4). The row advances by the tallest item plus
  spacing. `layoutBeginColumn()` and `layoutEndColumn()` nest a
  layout whose position and extents fold back into the outer one.
  `layoutSetNext(rect, relative)` places the next item explicitly.
  The layout tracks the content extent for scroll and autosize.
  More than 16 widths trap `UIT3`.
- **UI8 — Containers.** A window is a root container: it has a
  rect, a z-index, and a scroll offset, and a press inside it brings
  it to the front. The title bar (height 24) drags the window, the
  close box closes it, the resize handle at the bottom-right
  corner resizes it, and `UI_OPT_AUTO_SIZE` fits the rect to the
  content. A panel is a non-root container that takes the next
  layout rect and clips to it. A popup is a root container that
  opens at `openPopup(label)` and closes at a press outside it.
  Options are bits named `UI_OPT_*`, one per microui option.
  Containers live in a pool keyed by id, with least-recent
  eviction, sized as microui's pools. A scrollbar of width 12
  appears when the content exceeds the body on an axis; its thumb is
  at least 8 pixels; a wheel event scrolls the container under the
  pointer; a drag on the thumb scrolls in proportion.

## Widgets

- **UI9 — The widget set, version 1.** State crosses in holders,
  because subscript has no pointers: `UiState<T>` is a reference
  class with one field `value: T`. Every widget returns a `u32`
  response bit set: 1 active, 2 submit, 4 change.
  `button(label, opt = 0)`, `buttonIcon(icon, opt = 0)`,
  `checkbox(label, state: UiState<boolean>)`,
  `slider(label, state: UiState<f32>, low, high, step = 0, opt = 0)`,
  `number(label, state: UiState<f32>, step, opt = 0)`,
  `textbox(label, state: UiState<string>, opt = 0)`,
  `label(text)`, `text(text)` with word wrap,
  `header(label, opt = 0)`, `beginTreenode(label, opt = 0)` and
  `endTreenode()`, `beginWindow(title, rect, opt = 0)` and
  `endWindow()`, `beginPanel(label, opt = 0)` and `endPanel()`,
  `beginPopup(label)`, `endPopup()`, `openPopup(label)`. A slider
  reads its value from the pointer's x inside the base rect and
  rounds to `step` when `step` is not 0. A number widget adds the
  pointer's x delta times `step` while focused with the left button
  down. A textbox appends the frame's text, removes one byte on
  backspace, and clears focus and reports submit on return.
  Number editing by shift-click (microui's `number_textbox`) is not
  in version 1.
- **UI10 — Text.** One font, the atlas glyphs for bytes 32 to 126,
  height `UI_TEXT_HEIGHT`. The width of a string is the sum of its
  glyph widths. A byte outside 32 to 126 has width 0 and draws
  nothing. A number displays with two decimals, computed with
  integer arithmetic, rounded half away from zero, with a leading
  minus sign for a negative value.

## The command list

- **UI11 — Commands are typed records.** The core appends
  `UiCommand` records to one host array in call order: `kind` (1
  clip, 2 rect, 3 text, 4 icon), `x`, `y`, `w`, `h` (`i32`), `color`
  (`u32`, packed `0xAABBGGRR`, red in the low byte), `id` (the icon
  index for icon commands), and `text` (a `string`, empty except for
  text commands). Each root container records the index range it
  emitted. `end()` publishes `drawOrder(): i32[]`, the root
  containers by ascending z-index. There is no jump command: the
  renderer and the golden walk the ranges in draw order.
- **UI12 — Clipping.** A clip stack starts at the unbounded rect.
  `pushClip(r)` intersects, `popClip()` restores. A rect or icon
  outside the clip emits nothing. A rect partly inside emits a clip
  command for the intersection, the rect, then a clip command for
  the current clip. Text partly inside emits the same pair around
  the text command. A container body pushes its clip for its
  widgets.
- **UI13 — The golden form.** Rev 1. `dumpCommands(): string[]`
  renders one line per command in draw order (with no root container
  in the frame, every command in call order): `clip x y w h`, `rect x y w h
  #rrggbbaa`, `text x y #rrggbbaa "text"`, `icon n x y w h
  #rrggbbaa`, and after them one line per root container in draw
  order: `container id x y w h scrollX scrollY zindex`. A `b`
  program prints these lines. The two tiers print one byte sequence.

## The renderer

- **UI14 — One pipeline.** `UiVertex` is `@CStruct { position:
  Vec2f; uv: Vec2f; color: u32 }`. `UiViewport` is `@CStruct {
  width: f32; height: f32 }`. `UiRenderLayout` has `viewport:
  Uniform<UiViewport>`, `atlas: Texture2d<f32>`, and `nearest:
  Sampler`. `uiPipeline` is `renderPipelineL<UiRenderLayout,
  UiVertex, UiVarying>` with `triangle-list`, `uint16` indices, no
  culling, and blend color `src-alpha` over `one-minus-src-alpha`,
  alpha `one` plus `one`, operation `add` (the pair RN21 models). The
  vertex kernel maps pixels to clip space (`x * 2 / width - 1`,
  `1 - y * 2 / height`) and unpacks the color with division and
  modulo by 256 (K9 has no shifts). The fragment kernel returns the
  color with its alpha multiplied by the atlas sample's red channel.
  The atlas texture is `rgba8unorm`, every channel the alpha byte,
  uploaded once at `init` through `writeTextureBytes`.
- **UI15 — The frame builder.** `UiRenderer` is created at `init`
  with a quad capacity, default 16,384, and owns a vertex buffer of
  four vertices per quad, a `u16` index buffer of six indices per
  quad, the atlas texture, the sampler, the viewport uniform, and
  one bind group. `render(context, pass, width, height)` walks the
  command list in draw order: a rect is one quad at the white atlas
  rect, an icon is one quad at its atlas rect centered in the
  command rect, text is one quad per glyph advanced by glyph width,
  and a clip command ends the current draw range and starts one
  with the new scissor. The builder writes the vertex and index
  bytes once per frame through `Buffer<T>.write`, then for each
  range calls `setScissorRect` with the clip intersected with the
  viewport (an empty intersection skips the range) and
  `drawIndexed(indexCount, 1, firstIndex, 0, 0)`. A frame that
  exceeds the quad capacity traps `UIT1` with the capacity and the
  count.
- **UI16 — The pixel oracle.** A live program draws one window
  with microui's default colors into an offscreen `rgba8unorm`
  target and reads back pixels inside filled rects, where alpha is
  one and the result is the packed color exactly. Text pixels and
  edges are not compared. `tools/live.sh` runs it.

## The window

- **UI17 — The host feeds the context.** A windowed example maps
  W2 values to UI5 calls each frame before `begin()`: the pointer
  position to `inputMouseMove`, button bit changes to
  `inputMouseDown` and `inputMouseUp`, and the key scalar to
  `inputText` when it is 32 to 126. Wheel, key down and up, and
  text beyond one scalar per frame need W2 Rev 3 (plan §8 U3) and
  are not in version 1 of the example.

## Traps

- **UI18 — The trap table** (LB3): `UIT1` — a frame exceeded the
  renderer's quad capacity. `UIT2` — a widget or layout call
  outside `begin()` and `end()`, or an `end*` call without its
  `begin*`. `UIT3` — `layoutRow` received more than 16 widths.
  Each trap has a demonstrated red.

## Tests

- **UI19 — The gate.** `programs/b23-ui-core.ts` builds a
  `UiContext`, plays a scripted input sequence over several frames
  (a pointer move onto a button, a press, a release, a drag on a
  slider, a key), runs a window with a button, a checkbox, a slider,
  a label, a tree node, and a panel, and prints `dumpCommands()`
  after each frame. `programs/b24-ui-render.ts` builds one frame,
  runs the frame builder against Noop, and prints the quad count,
  the index count, the draw ranges, and an FNV-1a checksum of the
  vertex bytes, and its `.wgsl` golden is committed. Both are
  byte-identical across the two tiers. `programs/x24-live-ui.ts` is
  UI16. The generator test set reaches both kernels (LB4). The
  atlas module has the byte-identical regeneration gate.
