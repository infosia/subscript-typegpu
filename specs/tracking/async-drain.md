# async-drain — what one `mapAsync` costs the host

The async surface is `async-request`: a begin call, a status poll, a
facade pump, and `await Context.suspend()`. `render_async_method` in
`crates/webgpu-gen/src/api.rs` emits it, and `lib/webgpu.ts` carries
one copy for each Promise-returning member.

`mapAsync` is the member a real-time program wants inside a frame.
This record holds its measured drain cost and one generator fix.

## Measured, 2026-08-28, at the pin `a2228d9`

`programs/a06-map-async-cost.ts` maps a buffer and reads it back.
`subscript-typegpu-harness dev <program> --measure-map-async` counts
the host `async_step` calls from the suspension to the empty pending
set, and times them. The mode writes to stderr, so no golden moves.
Default runs do not measure.

Three runs for each row.

| Backend | Emission | `async_step` calls | Wall time |
|---|---|---|---|
| yawgpu, Noop | before the fix | 1 | 6083, 6333, 8167 ns |
| yawgpu, Noop | after the fix | 0 | 41, 42, 83 ns |
| yawgpu, Metal | before the fix | 1 | 12292, 12292, 12791 ns |
| yawgpu, Metal | after the fix | 0 | 42, 42, 42 ns |

The Noop future resolves on the first pump, so the Noop pair is a
floor. Metal answers on the first pump too: the pump inside the
generated loop resolves the map future, and the script suspends zero
times.

One `mapAsync` therefore costs the Metal host one round trip of about
12.3 us before the fix, and none after it. A 60 Hz frame is 16.7 ms,
so one call costs 0.07 per cent of a frame. The cost is per async
call, and it grows with the number of calls in a frame.

A sandboxed shell enumerates zero adapters, so every Metal run above
came from an unsandboxed shell.

One constraint holds for every phased measurement program. The ship
tier emits `subscript_kick_async_exports`, which calls each async
root export after `main` (`codegen/src/cemit.rs:1271` in subscript).
The program therefore runs each phase a second time, on handles the
first run disposed. `a06-map-async-cost.ts` holds one flag that makes
the second run a no-op. Without the flag the ship tier faults, and
the measured red is `SIGSEGV` with 0 bytes of output.

## The fix — the loop suspends only while the future is pending

The emitted loop read a non-zero status and then suspended once more
before it exited. That suspension cost one host round trip and held
the root pending for one more step. The emission now guards it:

    status = subscript_typegpu_future_status(this.instance, future);
    if (status === 0) {
      await Context.suspend();
    }

Every pending iteration still returns control to the host, because
the guard suspends whenever the status is zero.

Two sites in `crates/webgpu-gen/src/api.rs` carry the loop:
`render_async_method`, and `render_error_scope_pop_method` in its
non-host-owned form. The host-owned form of the second one holds a
synchronous status loop and no suspension, and its output stays
byte-identical. A third site, `render_record_drain_method`, emits one
pump and no loop.

## The facade pump, by caller

`lib/webgpu.ts` holds 12 script call sites: the adapter and device
requests, four pipeline creations, two error-scope pops, the buffer
map, the submitted-work completion, and two device-lost record
reads. `specs/tracking/coverage.md` reports 157 facade exports
reached and 0 unreached, and each site's begin call is one of them.

`crates/window/src/main.rs` holds three host call sites. Two serve
the adapter and device futures at startup, before `init`, and no
script loop owns those. The third is `drain_async`, which pumps
before the loop and after each host async step.

No caller is unreachable. `drain_async` can pump for a script future
that the script's own loop already pumps.

`drain_async` runs `while async_pending() != 0`. It has no time
budget, no step budget, and no sleep, and it drains every pending
root. No `examples/` entry declares `async function frame`, so
nothing with a frame budget has entered that loop yet. A bounded
drain needs a contract in `specs/blocks/` first, and this round did
not change the drain.
