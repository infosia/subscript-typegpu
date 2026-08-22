# Block: schema (SC-rules)

P1 contract. Rev 0, 2026-08-22. Plan §3 D1, D2, D4, D7 and §4 govern
this block. Layout arithmetic is `layout.md` (LY-rules).

## What a schema is

- **SC1 — A schema is a `@CStruct` class in the program.** No
  marker, no manifest, no sidecar file. The generator treats a value
  class as a schema when every field type is schema-legal (SC3) and
  the class is reachable from a schema use: a buffer creation
  (`Buffer<T>`), a binding wrapper (P2), or a field of another
  schema. A value class the program uses on the host only is not a
  schema and gets no diagnostic.
- **SC2 — Declaration order is layout order.** The generator never
  reorders fields. An author who wants fewer padding bytes reorders
  the source.
- **SC3 — Field types.** A schema field is `f32`, `i32`, `u32`,
  `f16`, a library vector or matrix class (SC5), `FixedArray<T, N>`
  of a schema-legal `T`, or another schema class. `boolean` is
  illegal (LY8). `string`, `T[]`, `Nullable`, a reference class, a
  function, `f64`, `i64`, `u64`, `i8`, `u8`, `i16`, and `u16` are
  illegal. Each rejection names the field, the type, and this rule.
- **SC4 — `FixedArray` length is a literal.** subscript resolves the
  length from an integer literal alone. The generator reads `N` from
  `Type::FixedArray`.

## The type library

- **SC5 — Vectors and matrices are `@CStruct` classes in
  `lib/typegpu-types.ts`**, hand-written, with the R33 alignment:
  `Vec2f`, `Vec2i`, `Vec2u` (`align: 8`), `Vec3f`, `Vec3i`, `Vec3u`,
  `Vec4f`, `Vec4i`, `Vec4u` (`align: 16`), `Vec2h` (`align: 4`),
  `Vec3h`, `Vec4h` (`align: 8`), `Mat2x2f` (two `Vec2f` columns,
  `align: 8`), `Mat3x3f` (three `Vec3f` columns, `align: 16`),
  `Mat4x4f` (four `Vec4f` columns, `align: 16`). Components are
  `x`, `y`, `z`, `w`. Columns are `c0` through `c3`.
- **SC6 — The library has real bodies.** Every vector and matrix
  method (`add`, `sub`, `mul`, `scale`, `dot`, `cross`, `length`,
  `normalize`, and the set `kernel.md` names in P2) is ordinary
  subscript that runs on the host. The generator maps each to the
  WGSL operator or builtin (P2). No method body is a stub.
- **SC7 — Constructors.** Each vector class has a constructor that
  takes its components in order. A free factory `v3f(x, y, z)` and
  family exists beside it. A matrix constructor takes its columns.
- **SC8 — `f16` vectors hold `f16` fields.** `f16` is storage-only
  in subscript, so `Vec2h`, `Vec3h`, `Vec4h` declare no arithmetic.

## Layout identity

- **SC9 — The schema class is the host type (D4).** The generator
  computes the C layout of every schema with subscript's rules
  (`layout.md` LY10) and the WGSL layout (LY1 through LY7). When the
  two differ at any field offset or at the total size, generation
  fails with a diagnostic that names the schema, the field, both
  offsets, and the fix: an alignment override on the field's class
  (R33), or a reordering. No padding field is ever generated.
- **SC10 — A uniform schema needs uniform-safe fields.** When a
  schema reaches a uniform binding (P2) or a uniform buffer, LY11
  applies. A violation is a diagnostic that names the member and
  the fix (`@CStruct({ align: 16 })` on the member's class, or a
  struct wrapper around the array element).

## Generated facts

- **SC11 — Layout constants.** For schema `X` the generator emits
  `X_SIZE`, `X_ALIGN`, and `X_OFFSET_<field>` as `u32` module
  constants in the support module, plus `X_STRIDE` for the array
  stride. A nested path joins with an underscore. A program prints
  these by name (testing.md T2), never a literal.
- **SC12 — The WGSL struct.** The generator emits one WGSL `struct`
  per schema, fields in order, types mapped per LY12, with no
  `@align` or `@size` attribute. The emitted text is committed as a
  golden beside the program (T6) from P2 on, when a kernel references
  it. In P1 the struct text appears in the support module as a
  string constant and the program prints its line count.
- **SC13 — The support module.** For program `<stem>.ts` the
  generator produces module `<stem>.typegpu.ts` in memory. The
  harness injects it. The CLI writes it. The program imports it as
  `./<stem>.typegpu`.

## Rejections

- **SC14 — Each rejection is a named diagnostic with a red fixture
  (T7).** The P1 set: a `boolean` field, an empty schema, a `string`
  field, a `T[]` field, a nullable field, a reference-class field, a
  64-bit field, a layout mismatch (SC9), a uniform violation (SC10).
  A diagnostic names its rule id and its owner: the author.
