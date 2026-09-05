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
support module of its own (`typegpu-ui.typegpu`), which the harness
generated on every program load through a ui-specific generator path
(`generate_ui_support`, a poison entry, a render-definition filter,
a `resource_helpers` flag). The gate rose from 180 s to 255 s on this
host, and LB2 places pipeline declarations in the program. The
correction: the program declares `uiPipeline` with `renderPipelineL`
and passes its generated facts to `UiRenderer` through
`UiPipelineFacts` (UI14 Rev 1, UI15 Rev 1). The generator and the
harness carry no ui-specific path. The same delivery kept a
per-command clip snapshot array to protect the renderer from a stale
scissor after a container exit. The correction follows microui's
`mu_set_clip(unclipped_rect)`: a partly clipped command ends with a
clip command for the unbounded rect (UI12 Rev 1), and the renderer
splits draw ranges at clip commands only. The two `b23` clip lines and
the `b24` ranges moved with it.

### Landed at `f40d04a`

`UiVertex`, `UiViewport`, `UiRenderLayout`, `UiVarying`, `uiVertex`,
`uiFragment`, `UI_BLEND`, `UiPipelineFacts`, `UiRenderer` (indices
written once, capacity 1 to 16,384), `programs/b24-ui-render.ts`
(31 quads, 186 indices, two ranges, a vertex checksum, twelve printed
checks over clip exits, nested roots, root order, orphan commands,
panel exits, a second window, and the empty frame), its `.wgsl`
golden, `programs/x24-live-ui.ts` (1,664 pixels in three patches,
exact packed colors), the `UIT1` reds (a frame over capacity, a
capacity of 0, a capacity of 16,385), and a generator test that
reaches both kernels from a program import and validates the WGSL.

Evidence on this host: `tools/gate.sh` green, 269 passed, 1 ignored,
218.6 s wall. `tools/live.sh` green on yawgpu Metal, 210.9 s, and on
Dawn, 206.2 s, with `x24` in the set. Every golden other than `b23`,
`b24`, and `x24` is byte-identical.

Deviation kept: UI15 said the index bytes are written per frame. They
are written once at creation. UI15 Rev 1 states it.
