# Block: ui (UI-rules)

U0 contract. Rev 0, 2026-09-05. Rev 1 (UI4: records reused across frames, UI13: no-root dump), 2026-09-05. Rev 2 (UI4: growth to a maximum, UI11: nested roots, UI18: UIT4), 2026-09-05. Rev 3 (phase review: UI6 microui's focus order, UI7 extent, UI9 centered numbers, UI10, UI12, UI13), 2026-09-05. Rev 4 (UI17: the W2 Rev 3 entries), 2026-09-05. Rev 5 (U2 review: UI12 unbounded reset, UI14 the program declares the pipeline, UI15 facts and capacity), 2026-09-05. Rev 6 (U2 phase review: UI11 root primitive, UI14 Rev 2 the spec is read and the alpha pair, UI15 Rev 2 facts and scissor, UI18 UIT1), 2026-09-05. Rev 7 (UI15 Rev 3: the two factories, host-owned device), 2026-09-05. Rev 8 (U3 phase review: UI8 Rev 1 `currentContainer`, UI15 Rev 4 private constructor), 2026-09-05. Rev 9 (branch review: UI4 Rev 3, UI9 Rev 1, UI11, UI12 Rev 2, UI15 Rev 5, UI18, UI20), 2026-09-05. Plan §8 U-phases govern this block.
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
  (the C initializer of `atlas_texture` names 10,584 bytes and C
  zero-fills the rest of the 16,384-byte array, so the generator
  zero-fills too and rejects an initializer with more bytes),
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
  `begin()` and `end()` traps `UIT2`. Rev 2: the context keeps its
  command records, root records, and layout records across frames
  and overwrites them in place. Storage grows when a frame exceeds
  every earlier frame's count, and it stays at that size. Rev 3: the
  retained arrays do not grow in a steady frame. A steady frame
  allocates the strings of its text commands, the array that
  `drawOrder()` returns, and the lines that `dumpCommands()`
  returns.
  Nothing in the context depends on `Context.collect()`.
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
- **UI6 — Hover and focus.** Rev 1. For a widget with rect `r` and
  id `id`: the widget is under the pointer when the pointer is inside
  `r`, inside the current clip rect, and the widget's root container
  is the hover root. The rules apply in this order. If it is under
  the pointer and no button is down, hover is `id`. If focus is `id`
  and a press happened outside `r`, focus clears. If focus is `id`,
  no button is down, and the widget lacks `UI_OPT_HOLD_FOCUS`, focus
  clears. If hover is `id` and a press happened this frame, focus is
  `id`. If hover is `id`, no press happened, and the widget is not
  under the pointer, hover clears. A press therefore focuses a widget
  only when an earlier frame set hover on it with no button down, so
  a drag never applies the pointer delta of the press frame. The
  hover root is the root container with the highest z-index under the
  pointer, taken from the previous frame's containers (UI4).

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
  The layout tracks the content extent for scroll and autosize, and
  an extent with no item is 0 by 0. More than 16 widths trap `UIT3`.
- **UI8 — Containers.** Rev 1. `currentContainer(): UiRoot` returns
  the innermost open container's record, whose `rect`, `body`,
  `scrollX`, `scrollY`, `contentWidth`, and `contentHeight` a program
  reads and writes, as microui's `mu_get_current_container`. A call
  with no open container traps `UIT2`. A window is a root container: it has a
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
  appears when the content exceeds the body on an axis. Its thumb is
  at least 8 pixels. A wheel event scrolls the container under the
  pointer. A drag on the thumb scrolls in proportion.

## Widgets

- **UI9 — The widget set, version 1.** Rev 1. State crosses in
  holders, because subscript has no pointers: `UiState<T>` is a
  reference class with one field `value: T`. Each stateful widget
  (`button`, `buttonIcon`, `checkbox`, `slider`, `number`, `textbox`,
  `header`, `beginTreenode`) returns a `u32` response bit set: 1
  active, 2 submit, 4 change. `beginWindow` and `beginPopup` return
  `UI_RES_ACTIVE` when the container is open and 0 when it is
  closed. `label`, `text`, `beginPanel`, `openPopup`, and every
  `end*` call return nothing. Responses and options are `u32` bit
  sets under `UI_*` names, because the module has no namespace
  object and a bit set crosses every subscript boundary as one
  integer.
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
  down. A slider and a number widget center their text unless `opt`
  carries an alignment. A textbox appends the frame's text, removes
  one byte on backspace, and clears focus and reports submit on
  return.
  Number editing by shift-click (microui's `number_textbox`) is not
  in version 1.
- **UI10 — Text.** One font, the atlas glyphs for bytes 32 to 126,
  height `UI_TEXT_HEIGHT`. The width of a string is the sum of its
  glyph widths. A byte outside 32 to 126 has width 0 and draws
  nothing. A number displays with two decimals: the value times 100
  is rounded half away from zero to an integer, and the digits come
  from integer arithmetic, with a leading minus sign for a negative
  value.

## The command list

- **UI11 — Commands are typed records.** Rev 1. The core appends
  `UiCommand` records to one host array in call order: `kind` (1
  clip, 2 rect, 3 text, 4 icon), `x`, `y`, `w`, `h` (`i32`), `color`
  (`u32`, packed `0xAABBGGRR`, red in the low byte), `id` (the icon
  index for icon commands), and `text` (a `string`, empty except for
  text commands). Each root container owns one contiguous index
  range. A root container can open inside another, as a popup opens
  inside a window. Then `end()` groups each root's commands into its
  own range and keeps the call order inside the range. `end()` publishes
  `drawOrder(): i32[]`, the root containers by ascending z-index.
  There is no jump command: the renderer and the golden walk the
  ranges in draw order. `UiRoot`, `beginRoot`, and `endRoot` are the
  public root primitive: a program can open a root container of its
  own, and the suite programs use them to pin the range rules.
  `beginRoot` does not push the unbounded clip. `beginWindow` pushes
  it, as microui's `begin_root_container` does for a window.
- **UI12 — Clipping.** Rev 2. A clip stack starts at the unbounded rect.
  `pushClip(r)` intersects, `popClip()` restores. A rect or icon
  outside the clip emits nothing. A rect partly inside emits a clip
  command for the intersection, the rect, then a clip command for
  the unbounded rect (Rev 1: microui's `mu_set_clip(unclipped_rect)`,
  so that a later command that is fully visible under any clip draws
  with no scissor). Text partly inside emits the same pair around
  the text command. A rect or text whose visible part has a zero
  width or height emits nothing. A container body pushes its clip
  for its widgets. Rev 2. The divergences from microui, each
  deliberate: microui emits a zero-sized command, and this module
  emits none. microui emits the intersected rect with no clip pair
  for a partly clipped rect, and this module emits the pair. The
  first clip command of a pair carries the intersection, where
  microui's carries the current clip. `popLayout` clamps the extent
  at 0, where microui's `pop_container` keeps the sentinel. A slider
  with `high` equal to `low` places its thumb at 0, where microui
  divides by zero. The icon indices start at 0 (`UI_ICON_CLOSE`),
  where microui starts at 1 with 0 as no icon, so `buttonIcon` draws
  every icon.
- **UI13 — The golden form.** Rev 1. `dumpCommands(): string[]`
  renders one line per command in draw order. With no root container
  in the frame, the order is the call order. With a root container
  in the frame, a command emitted outside every root is dropped, as
  microui's jump chain skips it. The lines: `clip x y w h`, `rect x y w h
  #rrggbbaa`, `text x y #rrggbbaa "text"`, `icon n x y w h
  #rrggbbaa`, and after them one line per root container in draw
  order: `container id x y w h scrollX scrollY zindex`. A `b`
  program prints these lines. The two tiers print one byte sequence.

## The renderer

- **UI14 — One pipeline.** Rev 2. `UiVertex` is `@CStruct {
  position: Vec2f; uv: Vec2f; color: u32 }`. `UiViewport` is
  `@CStruct { width: f32; height: f32 }`. `UiVarying` is `@CStruct {
  position: Vec4f; uv: Vec2f; color: Vec4f }`. `UiRenderLayout` has
  `viewport: Uniform<UiViewport>`, `atlas: Texture2d<f32>`, and
  `nearest: Sampler`. The module exports the kernels `uiVertex` and
  `uiFragment` and the blend state `UI_BLEND`: color `src-alpha` over
  `one-minus-src-alpha` and alpha `src-alpha` over
  `one-minus-src-alpha`, operation `add`, the pair microui's demo
  renderer sets and the host oracle models (RN21). The program
  declares the pipeline, as LB2 states for every module kernel:
  `renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(uiVertex,
  uiFragment, { format, indexFormat: "uint16", blend: UI_BLEND })`,
  and the generator emits the program's `.wgsl` golden and facts. The
  renderer reads that spec (UI15) and requires `triangle-list` and
  `uint16`, because its index pattern reaches index 65535, which a
  strip topology reads as the restart value. The vertex kernel maps
  pixels to clip space (`x * 2 / width - 1`, `1 - y * 2 / height`) and
  unpacks the color with division and modulo by 256 (K9 has no
  shifts). The fragment kernel returns the color with its alpha
  multiplied by the atlas sample's red channel. The atlas texture is
  `rgba8unorm`, every channel the alpha byte, uploaded once through
  `writeTextureBytes`. No library module imports a generated support
  module.
- **UI15 — The frame builder.** Rev 3. `UiRenderer` is created by
  one of two static factories with the program's pipeline facts and
  a quad capacity, default 16,384: `UiRenderer.create(device:
  GPUDevice, facts, capacity)` for a device the script owns, and
  `UiRenderer.createHost(device: GPUHostOwnedDevice, facts,
  capacity)` for the window host's device (W2), on the precedent of
  `createRenderPipelineHost` and `createBindGroupHost`. Each factory
  creates the resources through its device class, takes the queue
  once (`device.queue` for a `GPUDevice`, `device.queue()` for a
  host-owned device, which the renderer then owns and disposes), and
  hands them to the constructor. The constructor is private: the
  factories are the only creation path, and the validation-scope
  lint rejects a direct `new UiRenderer`. `render` writes through
  that queue. `UiPipelineFacts` holds the WGSL, the two entry
  names, the bind group layout, the vertex buffer layout, and the
  program's `RenderPipelineSpec`. The renderer derives the two
  strides itself from `Context.bytesOf`. A capacity of 0 or above
  16,384, a spec whose topology is not `triangle-list`, or a spec
  whose index format is not `uint16` traps `UIT1`. The renderer owns
  a vertex buffer of four vertices per quad, a `u16` index buffer of
  six indices per quad written once at creation, the atlas texture,
  the sampler, the viewport uniform, one bind group, and the
  pipeline. `build(context)` walks the command list in draw order: a
  rect is one quad at the white atlas rect, an icon is one quad at
  its atlas rect centered in the command rect, text is one quad per
  glyph advanced by glyph width, and a clip command ends the current
  draw range and starts one with the new scissor. The builder keeps
  its vertex byte array and its range records across frames and
  overwrites them in place, as UI4 states for the context. Rev 5:
  the retained arrays do not grow in a steady frame, and each quad
  allocates the bytes that `Context.bytesOf` returns.
  `UiPipelineFacts` exists because LB2 places the pipeline
  declaration in the program, so the program hands the generated
  facts to the library.
  `render(context, pass, width, height)` calls `build`, writes the
  viewport and the retained vertex byte array through
  `Buffer<T>.write` (the array can carry an earlier frame's tail,
  which no draw range reaches, and a copy of the live part costs an
  allocation per frame), then for each range calls `setScissorRect` with the clip intersected with the
  viewport (an empty intersection skips the range) and
  `drawIndexed(indexCount, 1, firstIndex, 0, 0)`. After the last
  range it sets the scissor to the full viewport, so a caller that
  draws next in the same pass is not clipped. A frame that exceeds
  the quad capacity traps `UIT1` with the capacity and the count.
  The unbounded rect that the core and the renderer share is one
  named constant.
- **UI16 — The pixel oracle.** A live program draws one window
  with microui's default colors into an offscreen `rgba8unorm`
  target and reads back pixels inside filled rects, where alpha is
  one and the result is the packed color exactly. Text pixels and
  edges are not compared. `tools/live.sh` runs it.

## The window

- **UI17 — The host feeds the context.** Rev 1. A windowed example
  maps the host's values to UI5 calls each frame before `begin()`:
  the pointer position to `inputMouseMove`, button bit changes to
  `inputMouseDown` and `inputMouseUp`, and the W2 Rev 3 entries to
  their UI5 twins: `wheel` to `inputScroll`, `keyDown` and `keyUp` to
  `inputKeyDown` and `inputKeyUp`, and `textInput` to `inputText`.
  The example exports the four optional entries and ignores the
  `key` parameter of `frame`.

## Traps

- **UI18 — The trap table** (LB3). Rev 1. `UIT1` — the
  renderer's capacity, pipeline spec, or frame is outside its domain
  (a frame over the quad capacity, a capacity of 0 or above 16,384, a
  topology other than `triangle-list`, an index format other than
  `uint16`). `UIT2` — a widget or layout
  call outside `begin()` and `end()`, an `end*` call without its
  `begin*`, a second `begin()` inside a frame, `beginPanel` with
  `UI_OPT_CLOSED`, or `currentContainer()` with no open container. `UIT3` — `layoutRow` received more than 16 widths.
  The `UIT1` reds create a renderer and therefore run in the backend
  lane of the gate (`--require-backend`). The other reds run headless.
  `UIT4` — a frame needed a 49th container or tree node, so the
  pool had no slot to evict. Each trap has a demonstrated red.

## The public surface

- **UI20 — The exported names.** The module exports these names and
  no other. Constants: `UI_OPT_ALIGN_CENTER` 1, `UI_OPT_ALIGN_RIGHT`
  2, `UI_OPT_NO_INTERACT` 4, `UI_OPT_NO_FRAME` 8, `UI_OPT_NO_RESIZE`
  16, `UI_OPT_NO_SCROLL` 32, `UI_OPT_NO_CLOSE` 64, `UI_OPT_NO_TITLE`
  128, `UI_OPT_HOLD_FOCUS` 256, `UI_OPT_AUTO_SIZE` 512,
  `UI_OPT_POPUP` 1024, `UI_OPT_CLOSED` 2048, `UI_OPT_EXPANDED` 4096
  (microui's option order). `UI_MOUSE_LEFT` 1, `UI_MOUSE_RIGHT` 2,
  `UI_MOUSE_MIDDLE` 4. `UI_KEY_SHIFT` 1, `UI_KEY_CTRL` 2,
  `UI_KEY_ALT` 4, `UI_KEY_BACKSPACE` 8, `UI_KEY_RETURN` 16.
  `UI_RES_ACTIVE` 1, `UI_RES_SUBMIT` 2, `UI_RES_CHANGE` 4.
  `UI_ICON_CLOSE` 0, `UI_ICON_CHECK` 1, `UI_ICON_COLLAPSED` 2,
  `UI_ICON_EXPANDED` 3 (the atlas table indices, UI2). The fourteen
  `UI_COLOR_*` indices in microui's color order, `UI_COLOR_TEXT` 0
  to `UI_COLOR_SCROLL_THUMB` 13. `UI_BLEND`. Classes: `UiContext`,
  `UiState<T>`, `UiRect`, `UiCommand`, `UiRoot`, `UiStyle` (`width`,
  `height`, `padding`, `spacing`, `indent`, `titleHeight`,
  `scrollbarSize`, `thumbSize`, `colors: u32[]`), `UiVertex`,
  `UiViewport`, `UiVarying`, `UiRenderLayout`, `UiPipelineFacts`,
  `UiDrawRange`, `UiRenderer`. Functions: `uiVertex`, `uiFragment`,
  `uiNumberText`. The custom-widget surface of `UiContext` is the set
  microui exposes for the same purpose: `getId`, `setFocus`,
  `mouseOver`, `updateControl`, `getClip`, `pushClip`, `popClip`,
  `pushLayout`, `popLayout`, `layoutRow`, `layoutNext`,
  `layoutSetNext`, `layoutBeginColumn`, `layoutEndColumn`,
  `drawRect`, `drawIcon`, `drawText`, `textWidth`, `drawFrame`,
  `drawControlFrame`, `drawControlText`, `currentContainer`,
  `beginRoot`, `endRoot`, `drawOrder`, `dumpCommands`, and the
  readable state `style`, `hover`, `focus`, `lastId`, `lastRect`,
  `mouseX`, `mouseY`, `mouseDeltaX`, `mouseDeltaY`, `mouseDown`,
  `mousePressed`, `keyDown`, `keyPressed`, `scrollX`, `scrollY`,
  `textInput`, `frame`, `hoverRoot`, `currentRoot`, `commandCount`,
  `rootCount`, `commands`, `roots`. A program writes `style` and
  nothing else of that state. `UiRoot` exposes `id`, `rect`, `body`,
  `contentWidth`, `contentHeight`, `open`, `lastUpdate`, `scrollX`,
  `scrollY`, `zindex`, `start`, `end`. `UiRenderer` exposes
  `create`, `createHost`, `capacity`, `quadCount`, `indexCount`,
  `rangeCount`, `ranges`, `vertexBytes`, `build`, `render`,
  `dispose`. A name outside this rule is a defect, in the code or in
  the rule.

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
