# imgui-survey — an immediate-mode GUI for subscript-typegpu

2026-09-05, branch `imgui`, workspace pin subscript `3677d1f`. This
survey decides how an immediate-mode GUI (the Dear ImGui idea) enters
this project. Repository facts come from a read of the tree at
`cbbab21`. Upstream facts come from the upstream sources as read on
2026-09-05 and are marked *(docs)*. Nothing upstream was run.

## The question

Three routes exist.

- **A. Bind Dear ImGui.** Load a C build of Dear ImGui (cimgui or
  dear_bindings) through the facade, and render with its own
  `imgui_impl_wgpu.cpp` over webgpu.h.
- **B. Author the GUI in subscript.** Port a small immediate-mode
  library to subscript over the TypeGPU layer, and render with one
  textured-quad pipeline.
- **C. A Rust GUI in the window host.** egui or imgui-rs inside
  `crates/window`, with a renderer over the facade.

## Route A — measured against the invariants

Facts *(docs)*, from `backends/imgui_impl_wgpu.cpp` at upstream
`master`:

- The backend compiles for exactly one header flavor, selected by
  `IMGUI_IMPL_WEBGPU_BACKEND_DAWN`, `_WGPU`, or `_WGVK`. yawgpu is
  none of the three. Its header is upstream webgpu-headers `a11ef44`,
  the same pin as this repository's `third_party/webgpu-headers`.
  Compatibility with yawgpu is unmeasured and is a third flavor to
  maintain.
- `wgpuDevicePopErrorScope` runs with
  `WGPUCallbackMode_AllowSpontaneous`. Design invariant 3 allows
  `AllowProcessEvents` only, and this callback lives in C++ outside
  the facade.
- The file is about 2,400 lines of C++, embeds WGSL and SPIR-V, and
  grows its vertex and index buffers with headroom each frame.

Facts *(docs)*, from dear_bindings: it emits `dcimgui.h`, keeps
`ImVec2` by value, converts callbacks such as `ImGuiInputTextCallback`
to C function pointers, and lists no wgpu backend among its tested
backends. Varargs functions (`ImGui::Text(fmt, ...)`) have no C form
that `subscript bind` maps. Function-pointer callbacks cross the
boundary that invariant 3 closes.

Consequences here:

- Invariant 7 (Rust only, generated C only) and the build-time rules
  exclude a C++ library in the tree. A run-time-loaded shared library
  is possible in principle, as the backend is, but the UI would be
  invisible to the differential gate: no subscript source, no
  goldens, no headless run, because Noop draws nothing.
- Invariant 1 (the bindable C subset) rejects varargs and callbacks
  at the surface. Every `igText` becomes `igTextUnformatted`, and
  every callback-shaped widget (input text with callbacks, draw
  callbacks) is unreachable.
- The window host carries one key scalar, the pointer, and three
  buttons (W2, W3). Dear ImGui needs key state, wheel, modifiers, and
  text. This gap is the same for every route.

Route A is rejected. Dear ImGui's draw-data model stays as the
reference for the renderer design below: vertices with position, uv,
and a packed color, `u16` indices, and draw commands split by clip
rect and texture.

## Existing WebGPU or TypeScript immediate-mode GUIs — measured

Question from the owner, 2026-09-05: does an immediate-mode GUI with a
WebGPU renderer exist in TypeScript, so that the port starts closer to
subscript? Search and source reads *(docs)*, nothing run.

| Candidate | What it is | Renderer | License | Fit |
|---|---|---|---|---|
| jsimgui (`@mori2003/jsimgui`) | Dear ImGui compiled to wasm with JS bindings | WebGL or WebGPU, selectable | MIT | wasm, no portable source |
| `@zephyr3d/imgui` | Dear ImGui through Emscripten inside the Zephyr3D engine | the engine's device layer, WebGL or WebGPU | MIT | wasm, no portable source |
| imgui-njs | a hand port of Dear ImGui about 1.70 to JavaScript, "work in progress", mixin-composed small files | Canvas 2D | MIT | Dear ImGui size, dynamic JavaScript idioms |
| `@thi.ng/imgui` | a functional immediate-mode GUI that emits hiccup-canvas shapes | Canvas 2D through hiccup, WebGL "early stages, unpublished" | Apache-2.0 | nine `@thi.ng` dependencies, functional style |
| microui-ts (jamesWalker55) | a TypeScript port of microui, about 1,350 lines, `Context` class, `Command[]` discriminated union, FNV-1a over `charCodeAt` | Canvas 2D demo with `measureText` | no license file in the repository | source unusable; the mapping decisions are readable |

Two findings decide the question.

1. **No TypeScript immediate-mode GUI with a WebGPU renderer exists
   as portable source.** The two WebGPU-capable packages are Dear
   ImGui in wasm. The TypeScript-source libraries render through
   Canvas 2D.
2. **The renderer is not the porting cost.** In every immediate-mode
   GUI the renderer is a few hundred lines against one draw-data
   contract: vertices with position, uv, and color, `u16` indices,
   and clip rects. Here the renderer is TypeGPU-shaped by design (a
   `@CStruct` vertex schema, kernels in the K subset, RN rules), so
   it is written once regardless of the source library. The porting
   cost is the core logic, and there the semantic distance counts,
   not the syntax. microui-ts uses generators for `iterCommands`,
   `structuredClone`, spread, destructuring, a discriminated union of
   command objects (S011 rejects a value union), `Record<Id,
   Container>` dynamic objects, and `toFixed`. Idiomatic TypeScript
   sits farther from subscript's accepted subset than microui's ANSI
   C, which has static arrays, explicit ids, no closures, and no
   dynamic maps.

Decision: the port source stays microui (C). microui-ts is a reading
reference for two mapping decisions this port also makes, pointer ids
to string labels and callbacks to constructor parameters, and nothing
from it is copied.

## Route C — rejected

The GUI logic would live in Rust in the host, not in scripts. The
project's product is script-facing libraries. Route C also adds a
large Rust dependency to the one workspace, against the build-time
rules, and the renderer would be written twice (egui's renderer
targets wgpu-rs, not webgpu.h).

## Route B — the decision

Port **microui** (https://github.com/rxi/microui, MIT) to subscript.
Facts *(docs)*, from `src/microui.h` version `2.02`, `demo/renderer.c`,
and `demo/atlas.inl`:

- About 1,100 lines of ANSI C. Widgets: windows, scrollable panels,
  buttons, checkboxes, textboxes, sliders, number fields, labels,
  word-wrapped text, tree nodes, headers, popups.
- The library draws nothing. It emits a command list of five kinds:
  `MU_COMMAND_JUMP`, `CLIP`, `RECT`, `TEXT`, `ICON`. The user renders
  the list. Text metrics come from two user functions, `text_width`
  and `text_height`.
- Widget ids are `unsigned`, from `mu_get_id(ctx, data, size)`, a
  hash over bytes, with an id stack.
- The demo atlas is `unsigned char atlas_texture[16384]`, 128 by 128,
  one alpha channel. Its rect table holds 100 entries: 4 icons, one
  3 by 3 white square, and 95 glyphs for ASCII 32 to 126. Text height
  is 18. The demo renderer batches quads (positions, uvs, colors, `u16`
  indices), clips with `glScissor`, and blends `src-alpha` over
  `one-minus-src-alpha`.

Every one of these maps onto what the tree has today:

| microui needs | This tree has |
|---|---|
| textured quads, per-vertex color | `renderPipelineL` with a `@CStruct` vertex schema (RN4), `Vec2f` and `u32` attributes (RN5), `programs/x11-live-fragment-sample.ts` samples a texture in a fragment kernel |
| `u16` indices, indexed draws | `setIndexBuffer`, `drawIndexed` (`lib/webgpu.ts`), `programs/b07-draw-variants.ts` |
| scissor per clip command | `setScissorRect` on the pass encoder, used by `examples/window-triangle` |
| `src-alpha` over blend | `RenderPipelineSpec.blend`, the one pair the host oracle also models (RN21, `programs/b21-blend.ts`) |
| an alpha atlas texture | `writeTextureBytes` with the 256-byte row rule (TX9), or `rgba8unorm` through `writeTexturePixels` |
| per-frame vertex upload | `Buffer<T>.write` over a pre-grown `u8[]` filled by `Context.bytesInto` (the `examples/fluid-with-atomics` pattern) |
| byte hashing of labels | `string.charCodeAt(i)` returns the byte 0 to 255 (stdlib §8), `u32` multiplication wraps (C3) |
| host-only containers, strings, maps | legal in host code (K5 forbids them in kernels only), and the UI core never runs on the GPU |

What does not map, and the resolution:

- **Function pointers** for `text_width`, `text_height`, and
  `draw_frame`. Closures cannot escape (C5). The port fixes the font
  to the atlas metrics module and makes the frame drawer a plain
  function. One font in version 1.
- **Pointer ids.** microui hashes a value's address for sliders and
  textboxes. subscript has no addresses. Every stateful widget takes
  a label or an explicit id string, and `pushId(string)` disambiguates.
- **The byte command buffer with jump pointers.** The port keeps
  typed command records in a host array and orders containers by
  z-index in a root list. No pointers.
- **Number formatting** (`%.2f`). Host code formats with integer
  arithmetic. Two decimals, no locale.
- **Bit shifts in kernels.** K9 has no `<<`, `>>`, `^`, and no
  `unpack4x8unorm`. The vertex kernel unpacks the `u32` color with
  `/ 256` and `% 256`, as `lib/typegpu-noise.ts` does.

### Architecture

Four pieces, in dependency order.

1. **`lib/typegpu-ui-atlas.generated.ts`.** microui's `demo/atlas.inl`
   pinned as a git submodule (`third_party/microui`) and converted by
   a tool under `tools/` to the alpha bytes, the 100 rects, and the
   text height. Generated, committed, byte-identical regeneration
   gate (core principle 7). MIT notice carried by the submodule.
2. **`lib/typegpu-ui.ts`, the core.** The microui port: context,
   input state, id stack, containers with scroll, layout (rows,
   widths, columns), the widget set, and the command list. Host code
   only. No GPU dependency. This is the part the headless gate covers
   completely.
3. **`lib/typegpu-ui-render.ts`, the renderer.** `UiVertex`
   (`position: Vec2f`, `uv: Vec2f`, `color: u32`), a uniform with the
   viewport size, one `renderPipelineL`, a `Texture2d` plus nearest
   `Sampler` layout, `src-alpha` over blend. A frame builder walks the
   command list into a vertex array, a `u16` index array, and draw
   ranges split at each `CLIP` command, then issues `setScissorRect`
   and `drawIndexed` per range. Fixed capacity chosen at `init`, the
   microui demo's 16,384 quads as the default, a named trap when a
   frame exceeds it.
4. **Window input, W2 Rev 3.** `frame` keeps its seven parameters, so
   the 25 windowed examples and the W2 signature test do not move.
   The host gains optional entries that it calls before `frame` when
   the script exports them: a wheel delta, a key bit set with press
   and release (backspace, return, shift, control, alt, arrows), and
   one call per text code point. R30 admits scalars only, so text
   arrives one `u32` per call. This is the one Rust change, in
   `crates/window`, and it is a window.md revision first.

### What is gated, and how

- **Headless, both tiers, byte-identical.** `programs/b23-ui-core.ts`
  drives the core with a scripted input sequence over several frames
  (pointer moves, presses, a key) and prints the command list and the
  container rects. `programs/b24-ui-render.ts` builds one frame into
  vertices and indices, prints the counts, the draw ranges, and a
  checksum, and emits the `.wgsl` golden of the pipeline. Noop draws
  nothing, so no pixel is read here (RN13). This is the same standard
  every other library module meets.
- **Live, gated run.** `programs/x24-live-ui.ts` renders one frame to
  an offscreen `rgba8unorm` target and reads back interior pixels of
  filled rects, which are exact because alpha is one there. Text
  pixels are not compared. `tools/live.sh` runs it.
- **Visual.** `examples/ui-demo/main.ts`, the microui demo port
  (windows, buttons, sliders, a textbox, a tree, a log panel), under
  `tools/window.sh --frames 30` with the smoke `grep FAIL`.
- **Regeneration.** The atlas module and the harness symbol table
  through `tools/regen.sh`.

### Costs and risks, in order

1. **Atlas source size.** 16,384 bytes as decimal literals is about
   80 KB of source that both tiers parse on every run. If the parse
   cost shows in the gate, the tool emits a hex string and the module
   decodes it with `charCodeAt`. Measure at U1.
2. **Input plumbing.** Optional host entries are a new W-rule and a
   host change through the coding agent. Whether the W2 signature test
   tolerates extra exports is unverified. Check it before U3.
3. **Module registration.** A new module needs LB1 and four Rust
   sites (`crates/typegpu-gen/src/lib.rs`, `main.rs`,
   `tests/support/mod.rs`, `tests/diagnostics/mod.rs`). Two modules
   (core, render) double that. One module with two files is not
   possible today, so the count stays two, or the render half lives
   in the core module.
4. **Value-copy semantics.** Widget state and containers are
   reference classes, not `@CStruct`, so that a helper can mutate
   them. `@CStruct` stays for the vertex schema and colors only. W004
   catches the write-only copy, not the read-after-write one.
5. **Text.** One bitmap font, ASCII only, 18 px. Proportional TTF
   atlases baked by a tool are a later phase and change nothing in the
   core.
6. **Kill criterion for the route.** If `b23-ui-core` cannot print a
   byte-identical command list on both tiers within U1, the route is
   wrong, because everything after it depends on that golden.

### Phases on branch `imgui`

| Phase | Deliverable | Exit |
|---|---|---|
| U0 | this survey, `specs/blocks/ui.md` contract (command model, id hashing, layout, widget set, renderer, input, trap table) | owner accepts the contract |
| U1 | `third_party/microui` pin, atlas tool and generated module, `lib/typegpu-ui.ts` core, `b23-ui-core` | golden byte-identical both tiers, gate wall time within budget |
| U2 | `lib/typegpu-ui-render.ts`, `b24-ui-render` with `.wgsl` golden, `x24-live-ui` | `x24` green on Metal (yawgpu) and Dawn |
| U3 | W2 Rev 3 optional input entries, host change, `examples/ui-demo` | 30-frame smoke green, every existing example untouched |
| U4 | textbox editing, wheel scroll, popups, phase review | review has no open CRITICAL or MAJOR |

Each phase ends with the phase review that CLAUDE.md requires.
