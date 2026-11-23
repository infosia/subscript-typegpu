# P20 — UI quads write in place

Plan §8 P20. Contract: `ui.md` UI15 Rev 7.

## Base

A downstream game report (2026-11-23) states that `UiRenderer.quad`
packs four `UiVertex` values with `Context.bytesOf`. It then copies
the bytes one by one into `vertexBytes`. The report measured an
isolated CPU-only ship-tier test on Apple M2: 512 quads per
iteration, 480 iterations, the median of six launches. The result was
32.2 ms for that path and 4.1 ms for `bytesInto` at the quad offset.
Both buffers held 40,960 bytes with FNV-1a checksum 2498469553
*(reported, not run here)*.

Read at `4f3a838`: `lib/typegpu-ui.ts` `quad` matches the report.
Subscript stdlib §18 rule 6 (pin `ddc0f0e2`): `bytesInto` traps when
`offset + sizeof(T)` exceeds the array length. The builder grows the
array before the write.

## Slice 1

Round 1 (Codex, 2026-11-23). `UiRenderer` stores `quadBytes`, four
times the vertex stride. `quad` appends zero bytes up to
`offset + quadBytes` and writes with
`Context.bytesInto<FixedArray<UiVertex, 4>>`. `b24` gains three
checks: a 64-quad frame grows `vertexBytes` to `quadCount * 4 *
stride`, its last quad reads back, and a one-quad frame after it
keeps the length. Every earlier `b24` line is unchanged, including
`vertices fnv1a 1253674997`.

Timing (Codex, ship tier, harness with the yawgpu Noop backend,
`Date.now()` around command construction and `build`). Each run has
480 frames: 479 frames of 512 rects and eight glyphs, then one frame
of 128 rects and eight glyphs. FNV-1a covers the whole retained array
after the last frame, the earlier tail included.

| Builder | Five runs (ms) | Median (ms) | Retained bytes | FNV-1a |
| --- | --- | ---: | ---: | ---: |
| `4f3a838` (`bytesOf`) | 81, 67, 66, 65, 64 | 66 | 49,920 | 1652744397 |
| P20 (`bytesInto`) | 38, 25, 28, 29, 29 | 29 | 49,920 | 1652744397 |

The interval includes `UiContext` command construction, so the ratio
is smaller than the isolated packing ratio in the report.

## Phase review (2026-11-23)

A no-context Opus reviewer read the working tree: 0 CRITICAL,
1 MAJOR, 4 MINOR.

- MAJOR: the timing and the gate evidence were not in this record.
  They are now recorded here.
- MINOR, accepted: `quadBytes` is four times the vertex stride, not
  `sizeof(FixedArray<UiVertex, 4>)`. The two are equal because
  `sizeof` includes the tail padding (subscript §18 rule 4), and the
  UI15 Rev 7 text states the stride form.
- MINOR, accepted: the field comment on `vertexBytes` no longer names
  which fields `build` refreshes.
- MINOR, accepted: "short frame reuse" checks the length, not the
  array identity. The earlier "frame storage reuse" check holds the
  alias.
- MINOR, fixed: two sentences in the plan and in this record broke
  the STE length limit.

Evidence (phase end): `tools/gate.sh --require-backend` green, 479 s,
380 passed, 1 ignored (live). No WGSL golden moved. `b24` gains three
lines after `frame storage reuse` and before `PASS`.

P20 is COMPLETE.
