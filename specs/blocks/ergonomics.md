# Block: ergonomics and diagnostics (EG-rules)

P6 contract. Rev 0, 2026-08-23. Plan §8 P6 governs this block. It
closes items earlier phases deferred: I12's coverage list, the
typed resources PI8 Rev 1 deferred to R35, the uniform binding-size
question (P2 review M6), and the diagnostic sweep.

## Typed resources

- **EG1 — A generated resources record per layout class.** For
  layout class `L` the support module exports `@Descriptor class
  LResources` with one required field per binding, named as the
  layout field, typed `GPUBuffer`, `GPUTextureView`, or
  `GPUSampler` by the wrapper kind, and a free function
  `create<Name>BindGroup<g>(device, pipeline, resources: LResources):
  GPUBindGroup`. The discovery check poisons both names (R35), so
  the program imports them like constants. PI8's constants stay.
  The positional `createBindGroup` stays for programs that prefer
  it.

## Buffers

- **EG2 — Field patches.** `Buffer<T>.patch(queue, elementIndex,
  fieldOffset: u32, bytes: u8[])` writes one field of one element:
  `fieldOffset` is an `X_OFFSET_<field>` constant read by name and
  `bytes` is `Context.bytesOf<F>(value)` of the field type. The
  method checks that `fieldOffset + bytes.length <= elementSize` and
  traps with the three numbers.
- **EG3 — Mapped reads in elements.** `readBuffer<T>` (BF3) gains
  `readOne<T>(readback, elementIndex): u8[]`.

## Coverage

- **EG4 — The reached-export list (I12 Rev 1).** The harness binary,
  in `dev` mode, counts the facade exports each program calls
  (through the symbol table: every `subscript_typegpu_*` export is
  wrapped by a counting thunk in the dev JIT symbol table). A test
  writes `specs/tracking/coverage.md` with the exports no `a`, `b`,
  or `x` program reaches. P6 closes the list: every unreached export
  gains a program or an exclusion row in `policy.toml` with a
  reason, and the test then fails on any unreached export.

## Measurements

- **EG5 — The Dawn run.** The owner runs `tools/live.sh` against a
  Dawn build (`SUBSCRIPT_TYPEGPU_BACKEND_LIB` names `libwebgpu_dawn`)
  once in P6, with `SUBSCRIPT_TYPEGPU_BACKEND` unset (Dawn ignores
  the yawgpu chain). The result decides the uniform binding-size
  question: if Dawn rejects a `minBindingSize` below 16 for a
  uniform struct, LY11 gains the rule and the layout engine rounds
  the uniform size. Otherwise the tracking entry records the
  measurement and LY11 stands.
- **EG6 — Timestamps, opt-in.** `ComputePipeline.dispatchTimed(...)`
  and a `TimestampPair` wrapper over the API layer's query set,
  resolved into a `u8[]` the program decodes with
  `fromBytes<FixedArray<u64, 2>>`. A program
  never prints the values (T2): it prints `timestamps:resolved`.
  Off by default, no hidden binding.

## Diagnostics

- **EG7 — The sweep.** A test enumerates every `diagnostic(`,
  `generator_diagnostic(`, and trap site in `crates/typegpu-gen`
  and `lib/typegpu.ts` and checks that each carries a rule id from
  `rule-ids.txt` and an owner. A fixture exists for each rule id
  the sweep finds (the corpus asserts one diagnostic per fixture),
  so no rule lacks a red.
- **EG8 — A runtime guard says what it guarded.** Every trap in
  `lib/typegpu.ts` names the rule id, the method, and the values
  that failed, before it traps (BF8 extended to every guard).

## Documentation

- **EG9 — README and one tutorial.** `README.md` states what the
  project is, the three layers, the environment variables, and the
  gate commands. `docs/tutorial.md` walks `b04-particles` from the
  schema to the dispatch in prose with the program's real lines. A
  test checks that every code line the tutorial quotes exists in the
  program it names (T12-style mechanical gate). The prose is
  reviewed once (CLAUDE.md "Two rounds").
