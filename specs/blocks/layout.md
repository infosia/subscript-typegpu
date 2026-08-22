# Block: layout (LY-rules)

P1 contract. Rev 0, 2026-08-22. Plan §3 D4 and D6 govern this
block. The rules restate the WGSL memory layout rules that TypeGPU
implements. Evidence lands in `specs/tracking/p1-layout.md`.

## Arithmetic

- **LY1 — One default layout.** The layout engine implements the
  WGSL default layout as arithmetic over a type tree. It reads no
  address space. A uniform check (LY11) runs after it.
- **LY2 — Scalars.** `f32`, `i32`, `u32`: align 4, size 4. `f16`:
  align 2, size 2.
- **LY3 — Vectors.**

  | Type | Align | Size |
  |---|---|---|
  | `vec2f`, `vec2i`, `vec2u` | 8 | 8 |
  | `vec3f`, `vec3i`, `vec3u` | 16 | 12 |
  | `vec4f`, `vec4i`, `vec4u` | 16 | 16 |
  | `vec2h` | 4 | 4 |
  | `vec3h` | 8 | 6 |
  | `vec4h` | 8 | 8 |

- **LY4 — Matrices.** `mat2x2f`: align 8, size 16. `mat3x3f`: align
  16, size 48. `mat4x4f`: align 16, size 64. Column-major. A
  `mat3x3f` column is a `vec3f` with a 4-byte tail.
- **LY5 — Arrays.** For `N` elements of `T`: stride
  `roundUp(sizeOf(T), alignOf(T))`, align `alignOf(T)`, size
  `N * stride`.
- **LY6 — Structs.** Walk the members in declaration order with a
  cursor from 0. Member offset is `roundUp(cursor, alignOf(member))`.
  The cursor becomes `offset + sizeOf(member)`. Struct align is the
  maximum member align. Struct size is `roundUp(cursor, align)`.
- **LY7 — roundUp.** `roundUp(value, modulo)` raises `value` to the
  next multiple of `modulo`. Every alignment is a power of two.

## Rejections

- **LY8 — No `bool` in a schema.** WGSL `bool` is not host-shareable.
  An author uses `u32`.
- **LY9 — Empty schemas are illegal.** A struct with no member has no
  alignment.

## The C side

- **LY10 — The C layout.** For a value class: each field at
  `roundUp(cursor, cAlign(field))`, class align
  `max(cAlignOverride, max field align)`, size rounded to the class
  align. `cAlign` of a scalar is its size. `cAlign` of a value class
  is its R33 override when present, else its natural align. A
  `FixedArray<T, N>` has align `cAlign(T)` and size `N * cSize(T)`.
  The generator computes this itself and never depends on
  `subscript-codegen`. The ship tier's `offsetof` proof in subscript
  is the oracle for the C numbers.
- **LY11 — The uniform address space adds two rules.** An array
  element stride must be a multiple of 16. A struct-typed member's
  offset must be a multiple of 16. Nothing else changes. Because the
  schema class is the host type (SC9), the generator does not emit
  `@align`. It reports the violation and the author aligns the
  source (SC10). The resulting layout is identical in storage and
  uniform, so one layout serves both.

## WGSL

- **LY12 — Type spelling.** `f32` → `f32`, `i32` → `i32`, `u32` →
  `u32`, `f16` → `f16`, `Vec3f` → `vec3<f32>`, `Vec2h` →
  `vec2<f16>`, `Mat4x4f` → `mat4x4<f32>`, `FixedArray<T, N>` →
  `array<T, N>`, a schema → its class name. A module that names
  `f16` opens with `enable f16;`.

## Verification

- **LY13 — Golden vectors are committed.**
  `tools/gen-layout-vectors.mjs` computes size, alignment, offsets,
  and strides with upstream TypeGPU at the pinned revision and writes
  `specs/layout-vectors.json`. A test compares the layout engine to
  the committed file and never runs Node. The script reads the
  checkout from `SUBSCRIPT_TYPEGPU_UPSTREAM_DIR` and records the
  TypeGPU version in the file.
- **LY14 — naga reads the offsets back.** A test parses every
  emitted WGSL struct with `naga` (dev-dependency), validates it, and
  compares `span`, each member `offset`, and each array `stride`
  with the engine. For a uniform schema the test wraps the struct in
  `var<uniform>` and asserts that `naga` accepts it.
- **LY15 — Every emitted WGSL module validates.** `naga` with
  `ValidationFlags::all()` and the capability set the module needs
  (`SHADER_FLOAT16` when `enable f16;` is present, else empty). A
  failure names the program and quotes the diagnostic with its cause
  chain.
- **LY16 — The C numbers are proven end to end.** A program prints
  `X_SIZE` and `X_OFFSET_<field>` by name on both tiers. The ship
  tier compiles the schema class as C with `_Alignas`, so a printed
  number that differs from the engine's fails the golden.
