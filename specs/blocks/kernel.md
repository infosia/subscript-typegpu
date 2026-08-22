# Block: kernel (K-rules)

P2 contract. Rev 0, 2026-08-22. Plan §3 D2, D3, D7, D9 and §4
govern this block. The pipeline declaration, the layout classes,
and the binding wrappers are `pipeline.md` (PI-rules). Schemas are
`schema.md`.

## What a kernel is

- **K1 — A kernel is a module-level function.** It has a name, no
  `export` requirement, no decorator, no directive. The generator
  finds it through a pipeline declaration (PI1). Its signature is
  `(res: L, ctx: ComputeInvocation): void` for one bind group, or
  the two-, three-, and four-layout forms PI2 names. It is not
  `async`, not a generator, and returns `void`.
- **K2 — The call graph is the GPU code.** Every function the kernel
  calls, directly or through a helper, is GPU code and obeys this
  block. The graph is acyclic. A cycle is a diagnostic that names
  the cycle. A helper is a module-level function with value-type
  parameters and a value-type or `void` return. A helper never takes
  a layout class or `ComputeInvocation`.
- **K3 — Everything else is host code.** A function the kernel does
  not reach is never inspected. Host code and GPU code live in one
  file.

## Types in a kernel

- **K4 — Value types only.** A kernel local, parameter, or return
  is `f32`, `i32`, `u32`, `boolean`, a library vector or matrix, a
  schema class, or a `FixedArray` of those. `f16` is storage-only
  (LY11 spirit): a kernel reads and writes an `f16` field through an
  `f32` conversion only when `kernel.md` Rev 1 admits it. In P2 an
  `f16` field access in a kernel is a diagnostic.
- **K5 — Rejected types.** `string`, `T[]`, `Map`, `Set`, a reference
  class, `Nullable`, a function value, `f64`, `i64`, `u64`, `i8`,
  `u8`, `i16`, `u16`, `Date`, `RegExp`, `object`, a `Worker`. Each is
  a diagnostic that names the construct and this rule.
- **K6 — Literals are typed by context.** The HIR types every
  literal (subscript R26). The emitter writes the WGSL suffix from
  `Expr.ty`: `1u`, `1i`, `1.0f`. A `boolean` literal is `true` or
  `false`.

## Statements

- **K7 — The statement set.** `let`, `const`, expression statement,
  assignment (including `+=` and family), `if`/`else`, `for` with an
  initializer, a condition, and a step, `while`, `return`. P4 adds
  `switch`, `break`, `continue`. A `for...of` over a `FixedArray` is
  emitted as an index loop. Any other statement is a diagnostic.
- **K8 — `let` and `const` map to `var` and `let`.** A subscript
  `const` binding emits a WGSL `let`. A subscript `let` binding emits
  a WGSL `var`. A value class local is always `var`, because the
  kernel assigns its fields.

## Expressions

- **K9 — The expression set.** Literals, locals, parameters, field
  access, `FixedArray` index, binding access (PI6), unary `-`, `!`,
  `~`, binary `+ - * / %`, comparisons, `&&`, `||`, `&`, `|`, the
  conditional `?:` (lowered to `if`/`else` over a `var` placed
  where the expression is evaluated — inside the loop for a loop
  condition, inside the chosen branch for a nested conditional — so
  both sides keep short-circuit evaluation), `as` casts among `f32`,
  `i32`, `u32`, calls to
  helpers, calls to library methods (K10), `new` of a library
  vector or matrix, `new` of a schema class with all fields set by
  its constructor, and the `v3f` family of factories. Any other
  expression is a diagnostic: a template string, an array literal,
  a lambda, `await`, `yield`, a `Math` call outside K11, a
  `JSON`/`Date`/`RegExp` call, `Length` of a `T[]`.
- **K10 — Library methods map to WGSL.** The generator carries one
  table from (receiver type, method) to an emission: `Vec*f.add` →
  `a + b`, `sub` → `-`, `mul` → `*` (componentwise), `scale` → `v *
  s`, `dot` → `dot(a, b)`, `cross` → `cross(a, b)`, `length` →
  `length(a)`, `normalize` → `normalize(a)`, `Mat*.mul` → `a * b`,
  `mulVec` → `m * v`, `transpose` → `transpose(m)`. Integer vectors
  map the same operators. The table is the one place a method gets
  GPU meaning. A method outside the table is a diagnostic.
- **K11 — Scalar builtins.** `Math.abs`, `min`, `max`, `floor`,
  `ceil`, `sqrt`, `pow`, `exp`, `log`, `sin`, `cos`, `tan`, `fround`
  map to the WGSL builtin of the same name (`fround` to nothing: the
  value is already `f32`). Library free functions `clamp`, `mix`,
  `step`, `smoothstep`, `fract`, `sign` in `typegpu-types.ts` have
  real bodies and map to their WGSL names. Any other `Math` member
  is a diagnostic.
- **K12 — Casts.** `x as f32`, `x as i32`, `x as u32` emit `f32(x)`,
  `i32(x)`, `u32(x)`. A cast to or from any other type is a
  diagnostic.
- **K13 — Value semantics are preserved.** A schema or vector local
  assigned from a binding or another local copies, in WGSL as in
  subscript. The emitter never synthesizes a pointer in P2. A helper
  receives arguments by value and returns by value.

## Emission

- **K14 — The emitted WGSL is deterministic.** Declaration order:
  `enable` directives, the schema structs the module references in
  first-use order, binding declarations in group and binding order,
  helpers in dependency order, the entry function. Identifiers keep
  their subscript names. A name that collides with a WGSL reserved
  word or a builtin function or type gets a `_` suffix, through one
  function applied to every identifier the emitter writes: struct
  names, field names, bindings, locals, helpers, the entry, and the
  `_ENTRY` constant. The list of reserved words and builtins is
  committed in `mapping.rs`, and a test compares it with naga's
  lists. A mangled name that collides with an author identifier gets
  a further `_`. Parentheses appear where precedence needs them,
  and the need is judged on the emitted WGSL operator, not on the
  subscript expression kind: a K10 method that lowers to an
  operator, a `Math.fround` that lowers to its argument, and a
  lowered `?:` are operands like any other.
  Whitespace is fixed: two-space indentation, one statement per
  line, no trailing space. A struct declaration carries no trailing
  semicolon.
- **K15 — Every emitted module validates.** Rev 1, 2026-08-22.
  `naga` is a dev-dependency (CLAUDE.md "Build time" rule 4), so the
  generator does not run it. The generator runs its own structural
  checks before it returns: every referenced name is declared, every
  binding has a unique group and binding pair, and every statement
  and expression is in the K7 and K9 sets. The harness validates
  every emitted module with `naga` on every test run (LY15, PI10).
  A naga failure there is a generator defect: the test names the
  generator as the owner and quotes naga's cause chain, because the
  author's program passed the checker and the generator's checks.
- **K16 — The WGSL golden.** For program `<stem>.ts` and pipeline
  declaration `<name>`, the module text is committed as
  `programs/<stem>.<name>.wgsl`. The harness compares the
  generator's text with the file byte for byte (T6). A program's
  golden output prints the module's line count by name, never the
  text.

## Diagnostics

- **K17 — Each rejection is a named diagnostic with a red fixture
  (T7).** A diagnostic cites the rule it enforces (K5, K9, K10, K11,
  K12, K7, K4), never this rule. A fixture header names that rule and
  the fixture reaches that rule's check and no earlier one. The P2
  set: a `string` local, a `T[]` parameter, a
  reference-class local, a lambda, a recursive helper, a helper that
  takes a layout class, a `Math` member outside K11, a cast to `f64`,
  a template string, an `f16` field read, a method outside K10, a
  statement outside K7, a kernel signature outside K1, a literal
  outside K6, a builtin outside PI4, a binding access outside PI6.
  Each names its rule id and the author as the owner. `await` in a
  kernel fails subscript's checker before the generator runs (an
  `async` function is not a function value), and its fixture asserts
  the checker's diagnostic.
