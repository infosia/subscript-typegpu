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
