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

- **EG4 — The reached-export list (I12 Rev 1).** Rev 1, 2026-08-23.
  The harness binary's `dev` mode counts the facade exports a
  program calls through counting thunks in the dev JIT symbol table
  (ABI-preserving wrappers generated per export), and reports them
  after the program's output when `--coverage` is set. The
  differential test runs every `a` and `b` program with `--coverage`
  on, splits the report off the output, and compares the output
  bytes with the golden, so the counted run is the gated run. The
  `x` programs get one extra dev run each. The test writes
  `specs/tracking/coverage.md` and fails on any unreached export. An
  export leaves the facade only through an `[[export_exclude]]` row
  (F22) with a reason.

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
  `generator_diagnostic(`, and trap site under `crates/typegpu-gen/src`
  (recursively) and in `lib/typegpu.ts` and checks that each carries
  a rule id from `rule-ids.txt` and an owner. A fixture with
  `expected-rule` exists for each rule id the sweep finds (the
  corpus asserts one diagnostic per fixture), so no rule lacks a
  red. A site the checker makes unreachable is deleted, never kept
  behind a claim.
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
