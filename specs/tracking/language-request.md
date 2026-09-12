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
