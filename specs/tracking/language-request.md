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
