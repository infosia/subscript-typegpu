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
