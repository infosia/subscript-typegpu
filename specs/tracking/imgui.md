# imgui — the immediate-mode GUI, U-phases

Branch `imgui`. Plan §8 U0–U4. The survey is
`specs/tracking/imgui-survey.md`. The contract is `specs/blocks/ui.md`.

## U0 (2026-09-05)

The owner accepted route B (a microui port over the TypeGPU layer) in
chat on 2026-09-05. The contract landed at `b588811` and `a0934c4`
(UI2: the atlas generator is a `subscript-typegpu-gen` subcommand,
not a node script, so that the regeneration gate has one form).

microui is pinned as `third_party/microui` at `0850aba`
(https://github.com/rxi/microui, `master` on 2026-09-05, MIT,
version 2.02, `src/microui.c` 1,208 lines). Pool sizes at the pin:
48 containers, 48 tree nodes, 16 row widths, a 256 KiB command list.

## U1

Handoff to the coding agent written 2026-09-05. Open.

### U1 round 1 (2026-09-05, `28e54c9`)

The atlas generator (`subscript-typegpu-gen ui-atlas`), the generated
module, the module registration, and the core's ids, input, hover,
layout, clipping, and commands. Evidence: `tools/gate.sh` green, 268
passed, 1 ignored, 159.1 s wall. Review findings, fixed in round 2:
the generated header carried the pin as a Rust constant, and `begin()`
allocated new arrays each frame.

### U1 round 2 (2026-09-05, `1527f0f`)

Containers, widgets, `programs/b23-ui-core.ts` (637 golden lines,
13 frames), the `UIT2` and `UIT3` reds, the header pin read from the
submodule gitfile, and the round 1 harness test replaced by the
program. Evidence: `tools/gate.sh` green, 268 passed, 1 ignored,
178.8 s wall. Both tiers print one byte sequence for `b23`.

Three contract deviations the coding agent reported, each taken into
the contract: storage grows to a maximum and stays (UI4 Rev 2), a
nested root's commands are grouped into one range (UI11 Rev 1), and
pool exhaustion is a trap of its own (UI18 Rev 1, `UIT4`, the code
still spells `UIT2` at that site and moves with U2).

One subscript diagnostic: `===` between two arrays is
`S100: operator not defined for `UiCommand[]` and `UiCommand[]``.
The program compares a retained element instead. No request.

### U1 phase review (2026-09-05)

A fresh Opus reviewer over `ee2e20a..91d1b11`, with execution of the
atlas and trap tests. CRITICAL 0, MAJOR 3, MINOR 14.

MAJOR, all fixed in the fix round: the tree node pool had no
exhaustion guard and evicted a live slot; `updateControl` focused on
a press over the widget where microui requires an earlier hover
frame, so a drag applied the press frame's whole pointer delta; the
scrollbars had no golden and the thumb drag no test.

MINOR fixed: the grouping sort runs only when an owner order
decrease was seen, dead range writes removed, the balance trap
reports six counters, the diagnostics arm lists `UIT1` to `UIT4` and
checks trap sites against fixtures, a debug print removed, `b23`
imports once and lists UI2, slider and number text centered as
microui. MINOR recorded in the contract, not changed in code: the
zero-sized command and the partial-rect clip pair (UI12), the extent
clamp (UI7), the free-command drop with a root present (UI13), UI10's
wording. MINOR unverified and left: the slider step cast width at
2^31, integer division sign on a negative numerator (C3 gives C
semantics).

Fix round evidence (`tools/gate.sh`, this host): green, 268 passed,
1 ignored, 180.2 s wall. `b23-ui-core.expected` is 883 lines, 22
frames. Four trap reds: `UIT2`, `UIT3`, `UIT4` container, `UIT4`
tree node.

## U1 closed (2026-09-05)

Exit met: the `b23` golden is byte-identical on both tiers, the gate
is within budget, the review has no open CRITICAL or MAJOR. U2 opens.

## U2 (2026-09-05)

### Round 1 and the design correction

The first delivery made `lib/typegpu-ui.ts` import a generated
support module of its own. The harness generated that module on
every program load through a ui-specific generator path. The gate
rose from 180 s to 255 s on this host. LB2 places pipeline
declarations in the program. The correction: the program declares
`uiPipeline` with `renderPipelineL` and passes its generated facts to
`UiRenderer` through `UiPipelineFacts` (UI14, UI15). The generator
and the harness carry no ui-specific path.

The same delivery kept a per-command clip snapshot array, so that the
renderer saw no stale scissor after a container exit. The correction
follows microui's `mu_set_clip(unclipped_rect)`. A partly clipped
command ends with a clip command for the unbounded rect (UI12 Rev 1),
and the renderer splits draw ranges at clip commands only. Two `b23`
clip lines and the `b24` ranges moved with it.

### Landed at `f40d04a`

The module gained `UiVertex`, `UiViewport`, `UiRenderLayout`,
`UiVarying`, `uiVertex`, `uiFragment`, `UI_BLEND`, `UiPipelineFacts`,
and `UiRenderer` (indices written once, capacity 1 to 16,384).
`programs/b24-ui-render.ts` prints 31 quads, 186 indices, two ranges,
a vertex checksum, and twelve checks over clip exits, nested roots,
root order, orphan commands, panel exits, a second window, and the
empty frame. `programs/x24-live-ui.ts` compares 1,664 pixels in three
patches with exact packed colors. The `UIT1` reds cover a frame over
capacity, a capacity of 0, and a capacity of 16,385. A generator test
reaches both kernels from a program import and validates the WGSL.

Evidence on this host: `tools/gate.sh` green with `pending 1` (the
backend lane, which holds the `UIT1` reds), 269 passed, 1 ignored,
218.6 s wall. `tools/live.sh` green on yawgpu Metal, 210.9 s, and on
Dawn, 206.2 s, with `x24` in the set. Every golden other than `b23`,
`b24`, and `x24` is byte-identical.

Deviation kept: UI15 said the index bytes are written per frame. They
are written once at creation. UI15 Rev 1 states it.

### U2 phase review (2026-09-05)

A fresh Opus reviewer over `523a667..fc30ffc`, with execution of the
generator, trap, diagnostics, and WGSL golden tests. CRITICAL 0,
MAJOR 2, MINOR 13.

MAJOR, fixed in fix round 2 (`a8ee882`): `UiRenderer` rebuilt its own
pipeline spec and ignored the program's declaration, and UI15's
facts list did not match the class. The renderer now takes the
program's `RenderPipelineSpec` in `UiPipelineFacts`, derives its two
strides from `Context.bytesOf`, and traps `UIT1` on a topology other
than `triangle-list` or an index format other than `uint16`.

MINOR fixed: `UI_BLEND` alpha is `src-alpha` over
`one-minus-src-alpha` as microui's renderer sets it, one unclipped
constant, the scissor is restored to the viewport after `render`, the
builder reuses its vertex bytes and range records, `b24` renders an
empty frame and creates the renderer at capacity 64, the scopes-lint
arm states its criterion, two spec reds. MINOR recorded in the
contract: `UiVarying` (UI14), `UiRoot` and the root methods as the
public root primitive (UI11), the topology justification (UI14).

### Build time and the loader fix

The reviewer named the per-program cost of a module loaded into every
program. The measurement and the fix are in
`specs/tracking/build-time.md` (U2 section) and `library.md` LB1
Rev 2, landed at `b5a4e1b`: one registered module table in
`crates/typegpu-gen/src/library.rs`, and every loader takes the core
set plus the modules the program's imports reach.

Evidence at `b5a4e1b` on this host: `tools/gate.sh --require-backend`
green, 274 passed, 1 ignored, 244.3 s wall. `tools/live.sh` green on
yawgpu Metal, 135.9 s, and on Dawn, 118.1 s.

## U2 closed (2026-09-05)

Exit met: `x24` green on Metal (yawgpu) and Dawn, the review has no
open CRITICAL or MAJOR, the backend gate is back under 1.2 times the
P16 row. U3 opens.

## U3 (2026-09-05)

Landed at the commit above `bb65ae2`: the window host queues wheel,
key, and text events and delivers them before `frame` through the
entries the script exports (W2 Rev 3, W3 Rev 2); the signature test
accepts the three required entries and any subset of the four
optional ones and names an unexpected export (W13 Rev 2);
`UiRenderer.create` and `UiRenderer.createHost` (UI15 Rev 3, the
host-owned device blocked the first delivery and the contract took
the two factories on the `createRenderPipelineHost` precedent);
`examples/ui-demo/main.ts`, a port of microui's demo windows with the
style editor omitted.

Evidence on this host: `tools/gate.sh --require-backend` green, 275
passed, 1 ignored, 263.1 s wall. `tools/window.sh` with `--frames 30`
on yawgpu Metal: `ui-demo` and `window-triangle` both print
`window:frames=30` with no `FAIL`. The visual result waits for the
owner's run. Every existing example is untouched.

Open for U4, from the example: the core exposes no
`currentContainer()` (the example reaches into `roots`), and no
scroll setter for a panel (microui's demo scrolls the log to its
end). The phase review follows.

### U3 phase review (2026-09-05)

A fresh Opus reviewer over `bb65ae2..9edcd77`, with execution of the
window and scopes tests and the host build. CRITICAL 0, MAJOR 2,
MINOR 15.

MAJOR, fixed in the fix round: the public `UiRenderer` constructor
was a third creation path with no validation (now private, and the
validation-scope lint rejects a direct `new UiRenderer` with a
fixture red); EX2 had no form for a port of another project (EX2
Rev 2, the header cites microui at the pinned commit).

MINOR fixed: `currentContainer()` (UI8 Rev 1) replaces the reach
into `roots` and scrolls the log to its end as microui does, the
log is capped at 8,000 bytes, fractional wheel deltas keep their
remainder, the host takes text from the key event's produced text
(W3 Rev 3), `deliver_input` iterates the taken queue, the rejection
test runs under a silent panic hook and gains two reds (W13 Rev 3),
the duplicate `checked_example` is gone, the export filter asserts
one file name, `createBufferHost` joins `lib/typegpu.ts` and the two
factories share a resource helper. MINOR recorded: the per-frame
`using queue` allocation is the pattern every windowed example uses.
MINOR unverified and closed by construction: the nested `@CStruct`
field write in the example is gone (whole-struct assignment).

Fix round evidence on this host: `tools/gate.sh --require-backend`
green, 276 passed, 1 ignored, 240.4 s wall. `tools/window.sh
--frames 30` on yawgpu Metal: `ui-demo` and `window-triangle` print
`window:frames=30` with no `FAIL`.

## U3 closed (2026-09-05)

Exit met: the 30-frame smoke is green, every existing example is
untouched, the review has no open CRITICAL or MAJOR. The visual
result of `ui-demo` waits for the owner's run. U4 opens with a
narrowed scope: the contract items the plan listed for U4 (textbox
editing, wheel scroll, popups) landed in U1 to U3. U4 holds the
example's EX2 comments, the README and `docs/from-typegpu.md`
entries, and the whole-branch review.
