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
  Rev 1, 2026-08-23: a second document, `docs/from-typegpu.md`,
  compares TypeGPU with this library topic by topic. Its TypeGPU
  examples are quotes from the TypeGPU documentation at a named
  version, and the document states once that this repository does not
  run them. Its subscript examples use the same `program=` fence, and
  the quote test scans every `docs/*.md` file. The document quotes
  TypeGPU's `tgpu` namespace, so `tools/hygiene.sh` exempts that one
  file from the banned-prefix check. `README.md` links both documents.
  Rev 2, 2026-08-23: `README.md` states that the library reinterprets
  TypeGPU's concepts for subscript and is not a port, quotes
  `b04-particles` with the same `program=` fence, and pairs each quote
  with its TypeGPU counterpart in one or two sentences. The quote test
  scans `README.md` with `docs/*.md`.

- **EG10 — The pipeline helpers accept a host-owned device.** Rev 0,
  2026-08-24. Rev 1, 2026-09-05: `createBufferHost<T>` joins them
  with the signature of BF1 and a `GPUHostOwnedDevice` first
  parameter. It creates the buffer through the host device and owns
  no queue. `lib/typegpu.ts` exports `createRenderPipelineHost`
  and `createComputePipelineHost` with the signatures of RN11 and
  PI9 and a `GPUHostOwnedDevice` first parameter, because subscript
  has no common interface over the two device classes and a windowed
  example otherwise repeats the descriptor by hand. The bodies stay
  single-sourced: the shared part lives in one helper over the
  values both device classes produce. A windowed example uses the
  host form.

- **EG11 — The authored spelling follows TypeGPU where subscript
  allows it.** Rev 0, 2026-08-25. Rev 1, 2026-08-25: subscript R37
  landed, so the accessor forms move too. Three forms change.

  The index form is the authored form for `Storage`, `MutStorage`,
  and `WorkgroupArray`. An author writes `res.items[i]` and
  `res.items[i] = v`. The methods `get(i)` and `set(i, v)` stay,
  because subscript's index signature needs them as its accessors.
  No file under `programs/`, `examples/`, or `docs/`, and not
  `README.md`, calls the two methods by name. A test over those
  four trees enforces it (T20). A direct test of the two methods
  stays, because they are public API. PI6 emits the same WGSL for
  both forms, and a generator test pins that equality (T21). A
  golden that moves under this rule is a defect, not a
  regeneration.

  The vector factory family is `vec2f`, `vec3f`, `vec4f`, and the
  same three shapes for the `i`, `u`, and `h` families. The `f`
  family adds six mixed and splat forms: `vec3fFrom2`,
  `vec4fFrom2`, `vec4fFrom3`, `vec2fSplat`, `vec3fSplat`, and
  `vec4fSplat`. The `i` and `u` families add the same six shapes.
  TypeGPU writes `d.vec3f`. The class names stay `Vec2f` through
  `Vec4h`, because a subscript program writes them as type
  annotations.

  The property form is the authored form for a value that takes no
  index. `Uniform<T>` declares the read accessor `$`. `PrivateVar<T>`
  and `WorkgroupVar<T>` declare the read accessor `$` and the write
  accessor `$`. Every swizzle declares a read accessor of its own
  name. An author writes `res.params.$`, `privateOffset.$ = v`, and
  `position.xy`. TypeGPU writes `layout.$.params`, `privateVar.$`,
  and `position.xy`.

  subscript R37 rejects `x.$ += 1`, so a read-modify-write reads and
  writes in one statement: `x.$ = x.$ + 1`. subscript R37 also
  forbids a write accessor on a `@CStruct` value class, so a swizzle
  is a read accessor and never an assignment target.

  A spelling that subscript forbids stays as it is. `scale` stays
  beside `mul`, because the language has no overloads.
  `createBuffer<T>` stays a free function, because a user-declared
  method takes no type parameters.
