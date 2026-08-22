# Block: facade runtime (L-rules)

P0 contract. Rev 0, 2026-08-22. CLAUDE.md invariants 1 through 4 and
7 govern this block. Plan §5 "The facade" names the shape. The
generator's own rules (F-, S-, and area rules) live in
`facade-generator.md`. This block holds the loader, the pump, and
the export shape, under the letter L so that the two sets never
collide.

## The loader

- **L1 — One library, named by one variable.**
  `subscript_typegpu_create_instance` reads
  `SUBSCRIPT_TYPEGPU_BACKEND_LIB`. The value is a path to a shared
  library that exports the webgpu.h functions the plan uses. The
  facade loads it with `libloading` on the first call and keeps it
  loaded for the process lifetime.
- **L2 — Loud absence.** If the variable is absent, the function
  prints one line to stderr: `subscript-typegpu: set
  SUBSCRIPT_TYPEGPU_BACKEND_LIB to the webgpu.h shared library`, and
  returns a null instance. If the library fails to load, the line
  carries the path and the loader's error text. If a symbol is
  missing, the line carries the symbol name and the path. In every
  case the return is a null instance and no other export is called.
- **L3 — Every symbol resolves before any call.** The table is
  filled completely or not at all. A partial table never exists.
- **L4 — One backend name in the facade.** Rev 1, 2026-08-22. The
  facade knows webgpu.h and one companion extension: the Tier-1
  backend's instance backend-select chained struct (yawgpu,
  `YAWGPU_STYPE_INSTANCE_BACKEND_SELECT`, a `u32` backend id). It
  knows nothing else about yawgpu, Dawn, or wgpu-native. A library
  that does not implement the extension ignores the chain, as
  webgpu.h specifies for an unknown `sType`.
- **L13 — The backend request.** `subscript_typegpu_create_instance`
  reads `SUBSCRIPT_TYPEGPU_BACKEND`. Absent means no chain: the
  library's default (yawgpu: Noop). `metal`, `vulkan`, `gles` select
  that backend through the L4 extension. Any other value prints one
  stderr line with the value and the accepted set, and returns a
  null instance. A null instance from the library after a request
  prints one line that names the request and the library path. The
  gate never sets the variable. The live lane sets it.
- **L5 — The ship tier links the same crate.** The facade builds as
  `lib` and `staticlib`. The staticlib carries `libloading` and the
  platform's dynamic loader library. No other link input exists.

## Async

- **L6 — Futures are integers.** An async export returns a `u64`
  future id. `subscript_typegpu_future_status(instance, id)` returns
  `0` pending, `1` success, a negative backend status on failure,
  and `-100` for an unknown id. `subscript_typegpu_future_drop`
  releases a slot. A typed `*_take` export transfers the result once.
- **L7 — AllowProcessEvents only.** Every future callback info the
  facade registers uses `WGPUCallbackMode_AllowProcessEvents`. The
  uncaptured-error callback info has no mode field in the pinned
  header. It only records into the device's queue (facade-generator.md
  G3). A generated
  callback runs inside `runtime::callback_guard`, copies every string
  out, records the outcome in the slot, and returns. It never calls a
  webgpu.h function and never unwinds. An unwind inside the guard
  aborts the process.
- **L8 — The pump order.** `subscript_typegpu_instance_process_events`
  calls `wgpuInstanceProcessEvents`, then releases the handles the
  callbacks deferred. Scripts pump, poll, then take.

## Exports

- **L9 — Null in, zero out.** Every export checks each handle and
  pointer parameter for null first and returns the typed zero:
  nothing for `void`, `0` for ids and counts, `false` for probes,
  null for handles.
- **L10 — Panic-free bodies.** No export body can panic. The only
  `catch_unwind` is `callback_guard`. Every `unsafe` block carries a
  `// SAFETY:` comment the generator emits.
- **L11 — Create owns.** A creation export records the created
  handle's owning instance. `*_release` exports are the only path
  that frees. No finalizer exists.
- **L12 — The header is the contract.** `subscript-typegpu.h` is
  generated with the facade and committed. Every construct in it is
  mapped by `subscript bind` without loss (CLAUDE.md invariant 1).
  The mirror regen test proves it on every run.
