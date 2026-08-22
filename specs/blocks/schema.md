# Block: schema (SC-rules)

P1 contract. Rev 0, 2026-08-22. Plan §3 D1, D2, D4, D7 and §4 govern
this block. Layout arithmetic is `layout.md` (LY-rules).

## What a schema is

- **SC1 — A schema is a `@CStruct` class in the program.** No
  marker, no manifest, no sidecar file. The generator treats a value
  class as a schema when every field type is schema-legal (SC3) and
  the class is reachable from a schema use: a buffer creation
  (`Buffer<T>`), a binding wrapper (P2), a field of another schema,
  or, in P1, a generated constant the program imports. A value class
  the program uses on the host only is not a schema and gets no
  diagnostic.
- **SC1a — The discovery check.** Rev 1, 2026-08-22. The program
  imports its support module before the module exists. The generator
  calls `subscript_compiler::check_program_with` with
  `poison_missing_modules = ["./<stem>.typegpu"]` (R35, at subscript
  `bb9dadc`). The discovery HIR carries the classes, the functions,
  the globals, and one `PoisonedImport` record with the imported
  names. The generator reads the class declarations, the pipeline
  declarations, and the imported names from it, computes layouts
  from its own type tree, generates, and lets the harness check the
  complete set. The generator never lowers a discovery HIR and never
  calls subscript's layout on it. No stub module exists. The one
  text access left reads a source line to name the construct in a
  relabelled checker diagnostic. An imported name that no schema or
  pipeline produces is an SC3 diagnostic when its class has an
  illegal field, and an SC1 diagnostic ("not a schema") otherwise.
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
  `lib/typegpu-types.ts`**, hand-written, with the R33 alignment.
  The generator recognizes them by declaring file and class name,
  never by name alone:
  `Vec2f`, `Vec2i`, `Vec2u` (`align: 8`), `Vec3f`, `Vec3i`, `Vec3u`,
  `Vec4f`, `Vec4i`, `Vec4u` (`align: 16`), `Vec2h` (`align: 4`),
  `Vec3h`, `Vec4h` (`align: 8`), `Mat2x2f` (two `Vec2f` columns,
  `align: 8`), `Mat3x3f` (three `Vec3f` columns, `align: 16`),
  `Mat4x4f` (four `Vec4f` columns, `align: 16`). Components are
  `x`, `y`, `z`, `w`. Columns are `c0` through `c3`.
- **SC6 — The library has real bodies.** Every method is ordinary
  subscript that runs on the host. The generator maps each to the
  WGSL operator or builtin (P2). No method body is a stub. The P1
  method set, per kind:

  | Kind | Methods |
  |---|---|
  | `Vec2f`, `Vec3f`, `Vec4f` | `add`, `sub`, `mul`, `scale`, `dot`, `length`, `normalize`. `Vec3f` adds `cross` |
  | `Vec2i`, `Vec3i`, `Vec4i`, `Vec2u`, `Vec3u`, `Vec4u` | `add`, `sub`, `mul`, `scale`, `dot` |
  | `Vec2h`, `Vec3h`, `Vec4h` | none (SC8) |
  | `Mat2x2f`, `Mat3x3f`, `Mat4x4f` | `mul` (matrix × matrix), `mulVec` (matrix × vector), `transpose`, and a free `identity` factory |

  `kernel.md` extends the set in P2.
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
  stride and `X_STRIDE_<member>` for each array-typed member. A
  nested path joins with an underscore. A field name that contains
  an underscore is rejected, so names stay unique. A program prints
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
