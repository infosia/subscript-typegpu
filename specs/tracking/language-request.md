# language-request — nine language items escalated as R39

2026-09-02. Escalated to the subscript owner as R39, written to
subscript's `HANDOFF.md` (https://github.com/infosia/subscript).
Every diagnostic was measured with `subscript check` and
`subscript run` built from the workspace pin `e1c2be1`.

| Item | Ask | Status at escalation |
|---|---|---|
| R39.1 | `Ref<T>` opt-in reference parameter for a `@CStruct` value | requested. The `struct` keyword form is withdrawn: `tsc` rejects it |
| R39.2 | operators on value classes | withdrawn: `tsc` rejects `a + b` on class operands (TS2365) |
| R39.3 | `x.name op= v`, `a[i] op= v`, `++`, `--` as checker sugar | requested |
| R39.4 | stable S-codes for unknown name, duplicate declaration, unknown member | requested |
| R39.5 | `??` and `?.` over `Ref \| null` | requested |
| R39.6 | method type parameters, on instance and static methods | requested |
| R39.7 | `tools/downstream.sh` in subscript, a downstream gate run before a re-pin | requested |
| R39.8 | static members: a static read accessor with no write accessor | requested, low weight. §71 statics are implemented at `e1c2be1`. Statics on generic classes and in `declare class` are not requested |
| R39.9 | overloads by parameter type, one body monomorphized per signature, the union legal only in the implementation signature | requested. Overloads by arity are recorded, not requested: the `undefined` guard crosses S012 |

## Evidence from this tree

- R39.1: a write through a value parameter that the function reads
  back compiles with no diagnostic and does not reach the caller.
  W004 covers the write-only case only. The P16 drag defect had this
  shape (`specs/tracking/p16-texture-arrays.md`, fix `01fa15e`).
- R39.2: 260 `.add(`, 77 `.scale(`, 38 `.sub(`, 21 `.dot(` sites in
  `programs/`, `examples/`, and `lib/`.
- R39.3: six hand-spelled rewrites, one on an accessor and five on
  fields or indices.
- R39.4: `S100` is cited twelve times in `specs/tracking/` for four
  different rules.
- R39.5: 279 `=== null` comparisons. 103 of them print a failure and
  return, and neither operator shortens that shape.
- R39.6: 58 `createBuffer<T>(` calls and 172 `Context.*<T>` calls,
  every one a free function or a `Context` static.
- R39.7: the candidate pin `c45d164` stopped every program
  (`specs/tracking/lowering-request.md`). The path-patch recipe
  resolves with `cargo metadata --offline`.
- R39.8: a static method and a `static readonly` field on a
  `@CStruct` value class run at `e1c2be1`. Two `policy.toml` reasons
  are false since §71: "static fields and user-defined namespaces are
  unavailable" (five namespace singletons, 469 constant reads) and
  "static methods are unavailable" (the `GPUDevice` constructor
  shape). The 30 free vector and matrix factories (92 call sites)
  can become static methods after the kernel generator emits a
  static call. This is downstream work, open.
- R39.9: the overload probe (method `mul(f32)` and `mul(V2)`, free
  `abs(f32)` and `abs(i32)`, an `instanceof` branch in the body)
  passes `tsc --strict` under subscript's `prelude/` with exit 0.
  `subscript check` rejects it at the duplicate name. `scale` and
  `mul` unify into one name at 80 call sites. The 27 vector factories
  wait on arity overloads and do not move.

## Downstream acceptance

R39.3, R39.5, R39.6, R39.8, and R39.9 land here as a sweep with every
`.wgsl` and `.expected` golden byte-identical, as R37 did. R39.1 has
no sweep today. R39.4 changes citations in `specs/tracking/` only.

## Downstream work opened by §71, not by R39

The two false `policy.toml` reasons and the factory sweep are a phase
on this side, the same shape as P14 after R37. Not started.

## R39 landed (2026-09-02), re-pin (2026-09-03)

subscript decided the nine items at `25c9437` (contract §82) and
landed six of them by `d45c0c1`. The record is subscript's
`specs/tracking/r39-six-requests.md` and
`specs/tracking/r39-overloads-deferred.md`.

| Item | Decision |
|---|---|
| R39.1 `Ref<T>` | deferred (owner). Zero downstream sites |
| R39.2 operators | withdrawn |
| R39.3 compound assignment sugar | landed, §82.1 |
| R39.4 three codes | landed, §82.2: S016 unknown name, S017 duplicate declaration, S018 unknown member |
| R39.5 `??`, `?.` | landed narrowed, §82.3. `?.` is legal as the whole left operand of `??` and as a call statement only, because `tsc` types a chain as `T \| undefined` |
| R39.6 method type parameters | landed, §82.4, instance and static. Type arguments are explicit |
| R39.7 downstream tool | landed, subscript `tools/downstream.sh` |
| R39.8 static read accessor | landed, §82.5 |
| R39.9 overloads | deferred (owner). The request's resolution rule 4 fails for two numeric signatures: `i32` and `f32` are one `number` to `tsc`. The narrowed shape (signatures differ by a kind `tsc` cannot assign across) is recorded there |

### The re-pin

The workspace moved from `e1c2be1` to `d45c0c1`. Two fixtures moved
their expected code, and each move was measured against the
`d45c0c1` CLI before the gate ran:

| Fixture | From | To | Measured message |
|---|---|---|---|
| `k28-half-builtin` | S100 | S018 | `` `Vec3h` has no method `abs` `` |
| `pi4-invocation-field` | S100 | S018 | `` `Ctx` has no member `unknown` `` |

A correction. The four `sc3-*` fixtures import a constant that the
rejected schema never emits, and the missing-export diagnostic is
S016 at `d45c0c1`. They were moved to S016 and the gate went red: the
first diagnostic of each fixture is the value-class field whitelist
message, which stays S100, and the harness matches the first
diagnostic. The four fixtures stay at S100.

Every `.wgsl` and `.expected` golden is byte-identical across the
re-pin. subscript's downstream run stopped at `k28`, the first
mismatch, so `pi4` was found here.

Evidence at `d45c0c1`: `tools/gate.sh` green, 265 passed, 1 ignored,
146.5 s wall. `tools/gate.sh --require-backend` with yawgpu Noop
green, 265 passed, 1 ignored, 215.7 s wall.

### Downstream work opened by R39, not started

- One `x.$ = x.$ + v` site can become `x.$ += v` (R39.3).
- The `S100` citations in this directory for "unknown type name",
  "duplicate declaration", and "has no method" name S016, S017, and
  S018 at `d45c0c1`. They stay as written, because each names the pin
  it was measured at.
- The two false `policy.toml` reasons and the factory sweep (§71,
  recorded above).
- A `Root`-shaped class with `create<T>` is now possible (R39.6). It
  is a design decision for `specs/blocks/library.md` first.

### Two more re-pins (2026-09-03)

The workspace moved from `d45c0c1` to `db3449d`, then to `587d6da`.
Both re-pins carry subscript changes that this project did not ask
for. No fixture and no golden moved. `specs/tracking/windows.md`
records the gate evidence at `ada9e24`.

## R40 — §108 and the layout class (2026-09-12)

The workspace pin moves from `ed7a668` to `403f8fc`. subscript §108
(`specs/blocks/compiler.md` at `403f8fc`) requires every class field
to hold a value before the constructor returns: an initializer, or a
top-level constructor assignment. A `!` assertion does not count.
Ambient declarations and `@Descriptor` members are exempt.

Measured at `403f8fc` before any change: `subscript-typegpu-gen`
9 passed, 59 failed, 2.91 s. `subscript-typegpu-harness` 20 passed,
24 failed, 8.05 s. The harness stops at the first diagnostic of
`lib/typegpu.ts` (`VertexInvocation.vertexIndex!`). With the three
`!` fields of the builtin carriers given initializers, the next
diagnostics were `UiRenderLayout`, `SaxpyLayout`, and the ten schema
classes of `programs/b01-layout.ts`: 28 passed, 16 failed, 15.14 s.

Two shapes fail. **A**: the PI3 layout class, `name!: Wrapper<T>`
fields and no constructor. 33 program classes, 4 library classes,
54 example classes, and the two builtin carriers. Stock `tsc`
accepts the shape. **B**: a `@CStruct` schema with bare fields and no
constructor. The ten classes of `programs/b01-layout.ts` and the
schema fixtures inside the generator tests. Stock `tsc` answers
TS2564, so subscript's invariant 5 forbids a language-side exemption
for B. The 93 schema classes with an assigning constructor, and the
72 `!` members inside `@Descriptor` classes, pass.

The request to subscript (2026-09-12) asked for A1: a construction-site
form of rule 2, where `const x = new C()` followed by a top-level
assignment of every `!` field counts as assigned. It named A2 (exempt a
class whose every field carries `!`) as weaker, and A3 (a layout class
becomes a `@Descriptor`) as the downstream-only alternative.

**Reply (2026-09-12): A1 and A2 declined. No subscript change.** The
language has one form for members the use site supplies, the
`@Descriptor` literal with R17 `!` members, and A1 adds a second
spelling that is legal at one syntactic position only. The barrier is
PI3, which says "never instantiated by the author" while the CPU lane
instantiates the class in 24 programs. The reply recommends A4: a
constructor that assigns every field from its parameters, the spelling
the S100 diagnostic names. Probed by the subscript side at `403f8fc`:
the A4 form is `tsc`-clean, passes `subscript check`, and runs on the
dev tier. The reply carries no subscript commit. Cite
`specs/blocks/compiler.md` §108.1 rule 1 at `403f8fc`.

**Decision (owner, 2026-09-12): A4.** `specs/blocks/pipeline.md` PI3
Rev 1: a layout class declares one constructor that takes one
parameter per field in declaration order and assigns each. The
generator reads the field list and never reads the constructor. Shape
B takes initializers or constructors in the program and the fixtures.

### The round (2026-09-12)

The generator's `layout()` accepts one constructor of the PI3 Rev 1
form and reports the first departure of seven kinds: no constructor,
no parameter for a field, a parameter type that differs, a missing
assignment, a statement that is not `this.<field> = <parameter>` in
order, an extra parameter, an extra statement. Three reject fixtures
demonstrate the red: `pi3-constructor-order`, `pi3-constructor-extra`,
`pi3-constructor-missing`, each with one diagnostic that names the
departure. `pi3-non-field` keeps its method and stays red.

Changed: 34 program layout classes, 54 example layout classes, 4
library layout classes, the two builtin carriers, 31 CPU-lane
construction sites, the ten schema classes of `b01-layout` (six by
initializer, four by constructor), 78 fixture classes, 59 inline test
classes, 7 document quotes. Measured: a `@CStruct` schema class accepts
a field initializer, and a `FixedArray<T, N>` constructor parameter is
legal at `403f8fc`.

Every `.expected` and `.wgsl` golden is byte-identical across the
re-pin.

Evidence at `403f8fc`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
224.15 s wall. The gate needs `SUBSCRIPT_TYPEGPU_BACKEND_LIB` as an
absolute path: the harness tests run with `crates/harness` as the
working directory, and a relative path fails every backend test at
`dlopen`.

### Re-pin to `b8b9739` (2026-09-12)

The workspace pin moves from `403f8fc` to `b8b9739`. The five subscript
commits between them split `cemit.rs`, `lower/func.rs`, `lir.rs`, and
`check/expr.rs` into child modules under 2,000 lines and split the
compiler contract into one file per section. No language rule changed.
No fixture and no golden moved.

Evidence at `b8b9739`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
245.8 s wall. The wall time includes a cold compile of the four
subscript crates.

### Re-pin to `082657b` (2026-09-20)

The workspace pin moves from `b8b9739` to `082657b`. The subscript
range holds 125 commits. It adds the sandbox compile profile (§109),
one reservation for each dev-JIT module (§110), a callback
registration with an explicit end (§111), and the reserved position
id 0 (§112). `082657b` is the last commit in the range that changes a
crate. The `swc_ecma_parser` fork moves from `c603b41` to `affcb6e`,
the revision in subscript's `Cargo.lock`.

One API change reaches this repository.
`subscript_compiler::parse_import_specifiers` takes a `Profile` as
its second argument. `crates/typegpu-gen/src/library.rs` passes
`Profile::Default`, because library sources are trusted first-party
code. No fixture and no golden moved.

Evidence at `082657b`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
236 s wall on a warm build.

### Re-pin to `93a4041` (2026-09-21)

The workspace pin moves from `082657b` to `93a4041`. The subscript
range holds 13 commits. It removes the sandbox compile profile (§113,
which supersedes §109) and the compile thread (§114). A compile runs
on the thread of the caller. `93a4041` is the last commit in the range
that changes a crate. The `swc_ecma_parser` fork stays at `affcb6e`.

One API change reaches this repository.
`subscript_compiler::parse_import_specifiers` takes the source file
only, because `Profile` does not exist at this pin.
`crates/typegpu-gen/src/library.rs` returns to the one-argument call.
No fixture and no golden moved.

Evidence at `93a4041`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
242.8 s wall. The wall time includes a cold compile of the four
subscript crates.

### Re-pin to `2fa77ec` (2026-09-29)

The workspace pin moves from `93a4041` to `2fa77ec`. The subscript
range holds 75 commits, §115 through §130 and stdlib batches 1 to 5b.
The `swc_ecma_parser` fork stays at `affcb6e`.

Changes that reach this repository:

- §130: the value class decorator is `@ValueType`. `@CStruct` is S100.
  Every tracked file outside `specs/tracking/` uses the new spelling.
  The mirror and `lib/webgpu.ts` do not contain the decorator.
- §129: a program with several source files names its entry module.
  The harness and `subscript-typegpu-gen` mark the entry. The entry
  module exports functions only, so pipeline, render pipeline, and
  shell declarations are module-level `const` without `export`
  (PI1, RN1, K29 examples updated). The harness reads the host
  entries from `module.host_entries`.
- §125: callees and references carry a module symbol. The generator
  matches `Function.symbol` and `Global.symbol`, and WGSL and
  diagnostics use the source name. K14 Rev 9 rejects two program
  declarations or two value classes that share one source name.
  Fixtures: `k14-shared-source-name.ts`, `k14-shared-class-name.ts`.
  Red for the class case: before the check, the generator used the
  library `RandomF32` layout for a program `RandomF32` and emitted a
  three-argument call to a two-field constructor, with no diagnostic.
- §115: `Stmt::Using`, `Stmt::Try`, and `Stmt::Throw` exist. Every
  walker descends into them. A kernel rejects `throw` and `try` under
  K7 (fixtures `k17-throw.ts`, `k17-try.ts`).
- HIR: `ExprKind::JsonResultValue` is removed, and `ExprKind::Local`
  carries its type.
- §124: `ComputePipeline.guard` in `lib/typegpu.ts` copies
  `guardQueue` into a local before the null check and the loop.

No WGSL golden and no `.expected` golden moved.

Open: the shell scan in `crates/typegpu-gen/src/shell.rs` does not
visit lambda bodies, async call arguments, or template parts, so a
`wgslShell` call inside a lambda gets no K29 diagnostic. This
predates the re-pin.

Evidence at `2fa77ec`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
397.2 s wall on a warm build (harness `tests/main.rs` 348.9 s).

### Typed symbols: re-pin to `5c5ec98` (2026-09-30)

Request: the HIR carried §125 identities as text in `String` fields,
so a comparison with a source name compiled and missed at run time.
Evidence from the `2fa77ec` re-pin: `cargo check` found none of the
§125 breaks, the survey found them by reading, and a program
`RandomF32` resolved to the library class with no diagnostic. The
request went to subscript on 2026-09-29. It landed as §131 (`hir::Symbol`,
`ClassDef.symbol`), with no language change and no golden move.
The subscript range `2fa77ec..5c5ec98` also amends §130: `@CStruct`
is the general unknown-decorator S100.

Downstream changes: the generator and the harness tests compare
`Symbol` with `Symbol`, and read `source_name()` only for output
names. Schema, layout, and varyings lookups use `ClassDef.symbol`.
The text helpers that parsed identity markers are removed. Lookups
that stay by name: support-module import names (an import carries a
name only), WGSL struct text keyed by the output struct name, the
constructed matrix column class name in `typegpu-types.ts`, and the
library class classifications. K14 Rev 9 adds layout classes:
fixture `k14-shared-layout-name.ts`, red before the check (the support
module exported `BitonicSortResourcesResources` twice), green after.

Evidence at `5c5ec98`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
383.8 s wall on a warm build.

### Re-pin to `86fd064` (2026-10-01)

The workspace pin moves from `5c5ec98` to `86fd064`. The subscript
range holds 24 commits, §132 through §141. The `swc_ecma_parser` fork
stays at `affcb6e`. No source in this repository changes.

- §132, §133, §134, §138, §139, §140, §141: no source here uses the
  affected forms (no `Worker` containers, no `import type`, no
  top-level blocks, no regex literals).
- §135: the generic bodies in `lib/typegpu.ts` and `lib/typegpu-ui.ts`
  read no member of a type parameter. All 87 programs and examples
  check clean at the new pin.
- §136: every mirror constant is a small `uint64_t` value, and
  `tools/regen.sh` leaves no diff.
- §137: the lib import graph has no cycle and no module initializer
  prints. No golden moved.
- Rust API: `ExprKind::Lambda` gains `id`, and `Module.top_level` is
  in module run order. Both match sites use `..`, so the workspace
  compiles without an edit.

Evidence at `86fd064`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
407.7 s wall.

### Re-pin to `de41409`, and `==` / `!=` (2026-10-02)

The workspace pin moves from `86fd064` to `de41409`. The subscript
range holds §142 through §144. The `swc_ecma_parser` fork stays at
`affcb6e`. §142 and §143 need no source change here.

§144 makes `==` and `!=` legal, with the type rules, the result, and
the lowering of `===` and `!==`. Owner decision of 2026-10-02: every
script source in this repository writes `==` and `!=`. The change
covers `lib/`, `programs/`, `examples/`, test fixtures, script
sources in Rust tests, the `lib/webgpu.ts` emitter in
`crates/webgpu-gen/src/api.rs`, and the code quotes in `docs/`. The
diff is 1,272 changed lines, and each one differs from its old line
only by the operator spelling. `tools/gen-layout-vectors.mjs` is a
Node.js script and keeps `===`. No hygiene rule rejects `===`.

The library host-runner tests in `crates/typegpu-gen/tests/library`
link the newest harness rlib in `target/debug/deps`. Run alone after
a pin change, they use a stale rlib until a workspace build replaces
it. The gate is not affected.

Evidence at `de41409`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 286 passed, 0 failed, 1 ignored,
389.5 s wall. No WGSL or `.expected` golden moved.

### Re-pin to `a502cf1`, through `5e708e4` (2026-10-05)

The workspace pin moves from `de41409` to `a502cf1`. The subscript
range holds §145 through §160. The `swc_ecma_parser` fork stays at
`affcb6e`. The intermediate pin `5e708e4` (§145 through §159) was
green but slow, so this commit carries both steps.

- §150: `ExprKind::Assign` gains `update`. The kernel emitter writes
  an assignment or an update only in statement position. An
  assignment or an update used as a value is now a K7 diagnostic
  (`kernel.md` Rev 16). Before the change, `res.output[index++] = v`
  emitted invalid WGSL with exit 0. The fixtures
  `k17-postfix-value.ts` and `k17-assignment-value.ts` record that
  red run and are green now.
- §158: `ExprKind::Unassigned` is the value of `let x: T;`. The
  kernel emitter writes `var x: T;`.
- §155: the checker enforces `private`. The scope lint check for
  `new UiRenderer` duplicated that rule and is removed. The fixture
  test now asserts the checker message (`ui.md` Rev 13).
- §160: checker performance only. No source change.

Evidence at `5e708e4`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 287 passed, 0 failed, 1 ignored,
1079.8 s wall.

Evidence at `a502cf1`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 287 passed, 0 failed, 1 ignored,
539.4 s wall. No WGSL or `.expected` golden moved.

### Re-pin to `888b68d` (2026-10-05)

The workspace pin moves from `a502cf1` to `888b68d`. The subscript
range holds §161, a checker performance change. The
`swc_ecma_parser` fork stays at `affcb6e`. No source change.

Evidence at `888b68d`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 287 passed, 0 failed, 1 ignored,
489.9 s wall. No WGSL or `.expected` golden moved.

### Re-pin to `41f593b` (2026-10-06)

The workspace pin moves from `888b68d` to `41f593b`. The subscript
range holds §162 and §163. The `swc_ecma_parser` fork stays at
`affcb6e`.

- §162: `hir::ExprKind::Local` gains a third field, the written
  annotation flag. The ten match sites in `crates/typegpu-gen` bind it
  as `_`. No behavior changes.
- §163: a null check of `xs[k]` does not narrow `xs[k]`. No source in
  this repository relies on that narrowing, and every program checks
  clean.

Evidence at `41f593b`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 287 passed, 0 failed, 1 ignored,
472.9 s wall. No WGSL or `.expected` golden moved.

### Re-pin to `e8cf6e0` (2026-10-07)

The workspace pin moves from `41f593b` to `e8cf6e0`. The subscript
range holds §164 through §175. The `swc_ecma_parser` fork stays at
`affcb6e`. The workspace compiles without an edit: the new HIR forms
(`Lambda.is_async`, `TaskGroup`, `AsyncAll`, `ArrayClear`) reach this
repository only through `..` patterns and `_` arms.

- §167: async functions are first-class values. The checker still
  rejects an `async` kernel with S100, now as the function type
  mismatch that PI13 states. The fixture `pi13-async-kernel.ts`
  asserts the new message.
- §164, §166, §168 through §175: no source change. No `.expected`
  golden moved, so the §171 and §172 release changes do not alter
  program output.

Evidence at `e8cf6e0`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 287 passed, 0 failed, 1 ignored,
512.2 s wall. No WGSL or `.expected` golden moved.

### Re-pin to `2867fd1` (2026-10-11)

The workspace pin moves from `e8cf6e0` to `2867fd1`. The subscript
range holds §176 through §189. The `swc_ecma_parser` fork stays at
`affcb6e`. The new crate `subscript-boundary` has no dependency, so
the `subscript-typegpu-webgpu-gen` closure still excludes
`subscript-compiler`.

- `Stmt::For.step` is now `Vec<Stmt>`. At the pin, `i++` and `i += 1`
  lower to one expression statement, and an empty step lowers to an
  empty list. An update of an indexed target with a computed index,
  for example `res.out[k + 0]++`, lowers to a `Let` and an expression
  statement. The kernel emitter accepts an empty step and one
  expression statement. Any other step is K7 (`kernel.md` Rev 17).
  The fixture `k7-for-step-statements.ts` records the red run.
- `Stmt::GeneratorForOf` is new. A kernel rejects it with K7. The
  fixture `k7-generator-for-of.ts` records the red run. The shell,
  pipeline, and render scanners and the harness walkers visit its
  subject and body.
- `Stmt::Using` gains `finalizer`. The scanners visit it.
- §179: the bindgen mirror records const members and const
  parameters. `tools/regen.sh` adds 59 `@subscript-c-member` and
  `@subscript-c-parameter` lines to
  `lib/subscript-typegpu.generated.d.ts`.

Evidence at `2867fd1`: `tools/gate.sh --require-backend` with the
yawgpu library, `gate: green`, 290 passed, 0 failed, 1 ignored,
475.7 s wall. No WGSL or `.expected` golden moved.
