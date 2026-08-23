# P8 — library breadth, slice 1

Contracts: `specs/blocks/buffer.md` Rev 1 (BF9–BF11),
`specs/blocks/kernel.md` Rev 5 (K10 Rev 1, K25–K28). Plan §8 P8.

## Status

2026-08-23: contracts written. No code yet.

2026-08-23: BF9 is blocked on subscript. Measured on the dev tier at
pin `bb9dadc` with one-file probes: an `async` method on a generic
class is `S100: async methods on generic class templates are not in
the decided surface`; a generic `async` free function is `S100: is
not a directly declared async function`; `u8[] | null` is `S011:
unions are limited to Ref | null`; an `async` method on a
non-generic class runs. Escalated as subscript request R36 (generic
`async`). Proposed BF9 Rev 1 for the owner's decision: the return
type is `Promise<u8[]>` and a failed map traps with `BF9`, so no
nullable array is needed. Until the decision, step 1 holds, steps 2
and 3 proceed, and `x14` reads through the BF3 explicit path. The
plan's exit item for `x14` through `read` moves to the step 1 close.

## Decisions

- Swizzles are methods (`v.xy()`), because subscript has no
  user-defined accessors. The set is the in-order subsets, so the
  method count stays at 3 per `Vec3*` and 10 per `Vec4*`.
- A scalar `select` is the K9 conditional. No function is added.
- Derivatives (`dpdx`, `dpdy`, `fwidth`) and the pack/unpack
  builtins are not in this slice. Their host bodies need a
  rasterizer-side definition that RN14's host rasterizer does not
  have.
- `Vec*b` classes are value classes, never schemas (SC5).

## Steps 2 and 3 (2026-08-23)

K25–K27 in `lib/typegpu-types.ts` with host bodies, the K10 table in
`crates/typegpu-gen/src/mapping.rs`, the K10 Rev 1 table-vs-HIR test
(scratch red both ways: an extra row `scratchExtra`, a missing row
for `abs`, a scratch class method `scratchMissing`), K28 fixtures
(SC5 `Vec2b` field, PI5 `Vec*b` wrapper, checker `Vec3h has no
method abs`), `b13-vector-builtins` with five `host:` lines,
`x14-live-vector-builtins` through the BF3 explicit path.

Evidence: `tools/gate.sh --require-backend` green, 235 passed, 137 s
under load (see `build-time.md`). `tools/live.sh` x01–x14 PASS:
Metal (yawgpu) 40.57 s, Dawn 37.12 s. The budget check for the
program gate (120 s) is open until an idle re-measurement.

## R36 landed (2026-08-23)

subscript `ac9436f` (contract `4652964`, compiler.md §64) admits an
`async` method on a generic class and a generic `async` function.
The workspace re-pins to `ac9436f`. BF9 Rev 1: the return type is
`Promise<u8[]>`, and a failed map traps with `BF9`. Step 1 proceeds.

## Step 1 (2026-08-23)

`Buffer<T>.read` and `readOne` own the staging buffer (BF9 Rev 1),
`createBuffer<T>` stores the usage and BF10 traps, `b12-readback`
(both tiers), `x14` reads through `read`. Scratch reds: BF9 through
a one-byte staging map that Noop refuses (`BF9 Buffer.read
elementIndex=0 elementCount=1 count=1`), BF10 `read` (`usage=8`),
BF10 `write` (`usage=4`).

Evidence: `tools/gate.sh --require-backend` green, 235 passed, 132 s
at load average 6.2 (the owner's subscript work shares the machine).
`tools/live.sh` x01–x14 PASS: Metal 34.79 s, Dawn 32.15 s. Per plan §7 (owner decision
2026-08-23), the rows under load stand as recorded and do not block
the close. The per-module measurement found no module that doubled
against HEAD with the diff applied.

## Phase review (2026-08-23)

Fresh no-context review (Opus) of `1a3af20..43c6269`: 1 CRITICAL,
3 MAJOR, 14 MINOR. CRITICAL: `smoothstep` emitted and computed with
the receiver as WGSL's `low` edge. MAJOR: `step` the same, the
host-line count assertion relaxed with a program-name special case,
the budget rows under load. Cause of the CRITICAL: K25 Rev 0 said
"receiver as the first argument" for every builtin, and host body
and emission shared the error, so the `x14` comparison could not
see it. Resolutions: K25 Rev 1 (argument order follows the WGSL
signature, order-sensitive inputs in the gate program, one hand
check recorded here), K27 six factories, K10 Rev 1 wording, PI14
Rev 1, BF9 bound check, BF10 on every write path, plan §7 budget
reading, plan P8 item 2 and exit (1). Code fixes in one Codex
round, all 18 findings closed.

Hand check of the order-sensitive golden values against the WGSL
specification (`b13-vector-builtins.expected`):

| Method | Inputs | Golden | WGSL definition |
|---|---|---|---|
| `step` | x `[-1,.25,.75,2]`, edge `[0,.5,1,3]` | `[0,0,0,0]` | `x >= edge ? 1 : 0`; reversed order gives `[1,1,1,1]` |
| `smoothstep` | x `[.25,.5,.75,1]`, low 0, high 1 | `[.15625,.5,.84375,1]` | `t*t*(3-2t)`, `t = (x-low)/(high-low)` |
| `mix` | x `[1,2,3,4]`, y `[5,6,7,8]`, a `.25` | `[2,3,4,5]` | `x*(1-a) + y*a` |
| `clamp` | x `[-2,.5,2,8]`, low `[0,1,2,3]`, high `[1,2,3,4]` | `[0,1,2,4]` | `min(max(x, low), high)` |
| `refract` | I `[1,0,0,0]`, N `[0,1,0,0]`, eta `.5` | `[.5,-.8660254,0,0]` | `k = 1 - eta²(1 - dot(N,I)²) = .75`; `eta*I - (eta*dot + sqrt(k))*N` |
| `faceForward` | e1 `[1,2,3,4]`, e2 `[1,0,0,0]`, e3 `[-1,0,0,0]` | `[1,2,3,4]` | `dot(e2,e3) < 0 ? e1 : -e1` |
| `select` | f `[1,7,3,9]`, t `[0,8,2,10]`, mask `[F,T,F,T]` | `[1,8,3,10]` | `mask ? t : f` per component |

All seven agree.

Evidence at the close: `tools/gate.sh --require-backend` green, 235
passed, 114 s at load average 2.9. `tools/live.sh` x01–x14 PASS:
Metal (yawgpu) 35.55 s, Dawn 32.88 s. `.wgsl` goldens for `b13` and
`x14` regenerated with the corrected order and validated by `naga`.

## Close

The workspace re-pins subscript to `f99d4cb` (tracking commits after
`1438b76`, no code change), as the R36 response asks.

P8 slice 1 COMPLETE 2026-08-23. Open for a later slice: derivatives,
pack/unpack, `Vec*i.abs` at the `i32` minimum (K25 states the
domain).
