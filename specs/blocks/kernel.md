# Block: kernel (K-rules)

P2 contract. Rev 0, 2026-08-22. Rev 1 (K9, K14, K15, K17),
2026-08-22. Rev 2 (K18–K24), 2026-08-23. Rev 3 (K18, K19, K22),
2026-08-23. Rev 4 (K14 shadowing), 2026-08-23. Rev 5 (K10, K25–K28),
2026-08-23. Rev 6 (K25 Rev 1 argument order, K27 count), 2026-08-23. Rev 7
(K29–K31 WGSL shells), 2026-08-23. Rev 8 (K14 Rev 5 one order),
2026-08-23. Rev 9 (K19 Rev 4 FixedArray constants), 2026-08-24. Rev 10 (K14
Rev 6 literal suffixes), 2026-08-24. Rev 11 (K14 Rev 6 logic
parentheses), 2026-08-24. Rev 12 (K5 Rev 1 `using`, absence test),
2026-08-30. Plan §3 D2, D3, D7, D9 and §4
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
  a diagnostic that names the construct and this rule. Rev 1: a
  `using` declaration in a kernel body (the HIR `Stmt::Let` with
  `dispose` true) is a K5 rejection with a fixture. The HIR
  `ExprKind::AbsenceTest` is a K5 rejection in the emitter without a
  fixture: the checker emits the node only for a descriptor member,
  and it rejects `undefined` on a kernel local first (`S012`, measured
  2026-08-30).
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

- **K9 — The expression set.** Rev 1, 2026-08-25 (the factory root,
  EG11). Literals, locals, parameters, field access, `FixedArray`
  index, binding access (PI6), unary `-`, `!`,
  `~`, binary `+ - * / %`, comparisons, `&&`, `||`, `&`, `|`, the
  conditional `?:` (lowered to `if`/`else` over a `var` placed
  where the expression is evaluated — inside the loop for a loop
  condition, inside the chosen branch for a nested conditional — so
  both sides keep short-circuit evaluation), `as` casts among `f32`,
  `i32`, `u32`, calls to
  helpers, calls to library methods (K10), `new` of a library
  vector or matrix, `new` of a schema class with all fields set by
  its constructor, and the `vec3f` family of factories. Any other
  expression is a diagnostic: a template string, an array literal,
  a lambda, `await`, `yield`, a `Math` call outside K11, a
  `JSON`/`Date`/`RegExp` call, `Length` of a `T[]`.
- **K10 — Library methods map to WGSL.** Rev 1. The generator
  carries one table from (receiver type, method) to an emission:
  `Vec*f.add` → `a + b`, `sub` → `-`, `mul` → `*` (componentwise),
  `scale` → `v * s`, `dot` → `dot(a, b)`, `cross` → `cross(a, b)`,
  `length` → `length(a)`, `normalize` → `normalize(a)`, `Mat*.mul` →
  `a * b`, `mulVec` → `m * v`, `transpose` → `transpose(m)`. Integer
  vectors map the same operators. K25, K26, and K27 add rows. The
  table is the one place a method gets GPU meaning. A method outside
  the table is a diagnostic. A generator test reads the method set
  of every vector, matrix, and atomic class in `lib/typegpu-types.ts`
  from the HIR and asserts that the table has a row for each method
  and no row for a method that does not exist.

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

- **K14 — The emitted WGSL is deterministic.** Rev 6. Declaration
  order, the same for every module: `enable` directives, the raw
  declarations text (K30) when the program has one, the schema
  structs the module references in first-use order, shells (K29) in
  declaration order, binding declarations in group and binding
  order, module constants and private and workgroup variables in
  declaration order, helpers in dependency order, the entry
  function. Rev 4 and earlier placed module variables before the
  bindings. Rev 5 moved them after, so one order serves a module
  with and without shells, and every `.wgsl` golden was regenerated
  under it. Every emitted integer literal carries its
  WGSL suffix, `u` for `u32` and `i` for `i32` (Rev 6): an
  unsuffixed literal concretizes as an abstract `i32`, `naga`
  accepted `2147483648` in a `u32` context and Tint refused it with
  `value cannot be represented as 'i32'` (measured 2026-08-24, the
  atomic fluid example on Metal and recorded in
  `specs/tracking/p10-examples.md`). The emitter spells `i32`
  minimum as `(-2147483647i - 1i)`, because WGSL reads
  `-2147483648i` as negation of a literal above the maximum, `naga`
  accepted it, and Tint refused it (the same example, the second
  measurement). The emitter parenthesizes every mixed `&&` and
  `||` chain, because Tint requires the parentheses and `naga` does
  not (`mixing '&&' and '||' requires parenthesis`, the slime-mold
  tie condition, the third measurement). Identifiers keep
  their subscript names. A name that collides with a WGSL reserved
  word or a builtin function or type gets a `_` suffix, through one
  function applied to every identifier the emitter writes: struct
  names, field names, bindings, locals, helpers, the entry, and the
  `_ENTRY` constant. The list of reserved words and builtins is
  committed in `mapping.rs`, and a test compares it with naga's
  lists. A mangled name that collides with an author identifier gets
  a further `_`. Rev 4, 2026-08-23: the emitter fixes the
  module-scope name set before it emits a body — struct names,
  binding names, module constants, private and workgroup variables,
  helpers, the entry — and a kernel local, a `for` variable, or a
  block-scoped local whose emitted name is in that set gets a `_`
  suffix, repeated until the name is free, on its declaration and
  every reference. A binding read never resolves to a local. Parentheses appear where precedence needs them,
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

## P4 — kernel depth (Rev 2, 2026-08-23)

- **K18 — More statements.** `switch` over an integer with a
  required `default`, `break`, `continue`, and a nested block join
  K7. A `switch` case body ends with `break`, `continue`, or
  `return`, or is empty and shares the next case's body, `default`
  included: the emitter writes `case a, b:` or `case a, default:`.
  A case body that falls through with statements is a diagnostic.
  `break` and `continue` inside a `for`, `while`, or `switch` emit
  the WGSL statement of the same name. A `continue` inside a
  `switch` inside a loop targets the loop, as in WGSL.
- **K19 — Module constants.** Rev 5, 2026-08-29. A module-level
  `const` of a scalar, a library vector, or a library matrix type
  reaches a kernel as a WGSL `const` of the same name when the
  generator folds its initializer: a literal, a unary or binary
  expression of literals and other module constants, or a vector
  factory of such. The fold uses checked arithmetic in the
  constant's type. A division by zero, an overflow, any other
  initializer, and a mutable global are K19 diagnostics when a
  kernel reads the constant. A rejected `privateVar` initializer is
  a K20 diagnostic. Rev 5: a cycle between module constants is not a
  K19 case. subscript rejects a module initializer that reads a
  binding declared after it (`compiler.md` §67.1 rule 4c, from pin
  `f43e3b2`), and every cycle contains one such read. `tsc` rejects
  the same shape as TS2448. The generator therefore never folds a
  cycle, and it keeps no fixture for one. Rev 4: a module-level `const` of a `FixedArray` of a
  foldable scalar type reaches a kernel as a WGSL `const` array of
  the same name when every element folds under the same rules. The
  noise module's permutation tables are the first use.
- **K20 — Private and workgroup variables.** Rev 1, 2026-08-25 (the
  access forms, EG11). A module-level `const` whose initializer is
  `privateVar<T>(init)`, `workgroupVar<T>()`,
  or `workgroupArray<T>(n)` (library generic functions with real
  bodies over a `T` or a `T[]`, `n` a literal) declares a variable
  the kernel reaches by name. The emitter writes `var<private> x: T
  = init;`, `var<workgroup> x: T;`, or `var<workgroup> x: array<T,
  n>;`. `T` is a scalar, a library vector or matrix, or a schema
  class. Access is `x.$` and `x.$ = v` for a scalar variable, `x[i]`
  and `x[i] = v` for an array, and `x.length()`, as the binding
  wrappers (PI6 and EG11). Rev 2, 2026-08-25: the scalar forms are
  accessors on subscript R37. The accessor is named `$`, so a read
  records as the method `$`, and a write records as the method `$=`. A private or
  workgroup variable initialized with a value in WGSL's forbidden
  positions (a workgroup variable with an initializer) is a
  diagnostic.
- **K21 — Atomics.** `AtomicU32` and `AtomicI32` are library value
  classes (`@CStruct`, one field, size 4, align 4) legal as a schema
  field and as a workgroup variable type. Their methods `load()`,
  `store(v)`, `add(v)`, `sub(v)`, `min(v)`, `max(v)`, `exchange(v)`
  (each returning the old value where WGSL does) have real host
  bodies and emit `atomicLoad(&p)`, `atomicStore(&p, v)`,
  `atomicAdd(&p, v)` and family, where `p` is the emitted place of
  the receiver. The receiver must be a place inside a storage
  binding or a workgroup variable. A schema that holds an atomic
  cannot be copied to a local or written as a whole: both are
  diagnostics.
- **K22 — Barriers.** Rev 1, 2026-08-23. `workgroupBarrier()` and
  `storageBarrier()` are library free functions with empty host
  bodies, legal only as a statement in the kernel body, never in a
  helper. The generator enforces uniform placement itself, because
  `naga` did not reject a barrier after a non-uniform early return
  (measured 2026-08-23, `x08`). Both backends reject such a module
  at shader-module creation — Tint: "'workgroupBarrier' must only be
  called from uniform control flow" — and a program without an
  error scope then reads zeros from an invalid pipeline (corrected
  2026-08-23, plan §10 C4). The rule: a
  barrier statement is legal at the kernel body's top level, or
  inside `while`, `for`, and `if` statements whose conditions are
  uniform. A uniform expression reads only literals, module
  constants, `Uniform<T>` bindings, `length()` of a binding, and
  locals that were assigned only uniform expressions under uniform
  control. A builtin, a storage binding read, a workgroup or private
  variable read, and a helper result are non-uniform. A local
  assigned under a non-uniform condition is non-uniform from then
  on, and a `break` or `continue` under a non-uniform condition
  makes every local the loop writes non-uniform. A `for` step runs
  under the loop's condition. No `return` statement precedes a
  barrier in source order. No `break` or `continue` under a
  non-uniform condition leaves a loop that contains a barrier. A
  violation is a K22 diagnostic that names the statement and the
  non-uniform value. The analysis is conservative: it rejects some
  uniform programs and accepts no non-uniform barrier. The harness
  still runs naga. The idiom for a bounds check before a barrier is
  a conditional load: `partials[local] = global < n ?
  input[global].value : 0.0;`.
- **K23 — The workgroup builtins.** `ComputeInvocation.localId`,
  `workgroupId`, `numWorkgroups`, and `localIndex` emit their
  `@builtin` parameters when read (PI4).
- **K24 — The P4 rejections.** A `switch` with no `default`, a
  fallthrough with statements, a mutable global read in a kernel, a
  workgroup variable with an initializer, an atomic schema copied to
  a local, a barrier in a helper, an atomic method on a local, a
  `return` before a barrier, a barrier inside an `if` on a builtin,
  a barrier inside a loop whose condition reads a binding. Each with
  a fixture and one diagnostic.

## Vector builtins (P8)

- **K25 — Componentwise builtins.** Rev 1. `Vec2f`, `Vec3f`, and
  `Vec4f` gain the methods `abs()`, `floor()`, `ceil()`, `fract()`,
  `sqrt()`, `exp()`, `log()`, `sin()`, `cos()`, `tan()`, `sign()`,
  `min(other)`, `max(other)`, `clamp(low, high)`, `pow(other)`,
  `mix(other, amount: f32)`, `step(edge)`, `smoothstep(low, high)`,
  `distance(other): f32`, `reflect(normal)`, `refract(normal, eta:
  f32)`, and `faceForward(incident, reference)`. `Vec*i` gain
  `abs()`, `min(other)`, `max(other)`, and `clamp(low, high)`.
  `Vec*u` gain `min(other)`, `max(other)`, and `clamp(low, high)`.
  Each maps to the WGSL builtin of the same name. The receiver is
  the builtin's value argument, so the argument order follows the
  WGSL signature: `v.step(edge)` → `step(edge, v)`,
  `v.smoothstep(low, high)` → `smoothstep(low, high, v)`,
  `v.mix(other, a)` → `mix(v, other, a)`, `v.clamp(low, high)` →
  `clamp(v, low, high)`, `v.refract(n, eta)` → `refract(v, n, eta)`,
  `v.faceForward(i, r)` → `faceForward(v, i, r)`, and the rest with
  the receiver first. Each has a real host body over the scalar
  operation, so the CPU lane runs it. The host bodies use `Math`
  members from K11 and the scalar free functions, never a second
  formula. `Vec*i.abs()` of `i32` minimum is outside the domain: the
  host returns a value the `i32` cannot hold, WGSL returns the
  minimum. A gate program prints `step`, `smoothstep`, `mix`,
  `clamp`, `refract`, `faceForward`, and `select` on inputs whose
  result differs for every argument order, and the phase tracking
  records one hand check of those golden values against the WGSL
  specification.
- **K26 — Comparisons, bool vectors, `select`.** `lib/typegpu-types.ts`
  adds `Vec2b`, `Vec3b`, and `Vec4b` with `boolean` fields `x`, `y`,
  `z`, `w`, the methods `any(): boolean`, `all(): boolean`, and
  `not()`. They are value classes, not schemas: a `Vec*b` field in a
  `@CStruct` class is an SC5 diagnostic, because WGSL gives `bool`
  no host-shareable layout. Every float and integer vector gains
  `lt(other)`, `le(other)`, `gt(other)`, `ge(other)`, `eq(other)`,
  and `ne(other)`, each returning the `Vec*b` of the same width,
  and `select(other, mask: Vec*b)`, which takes this vector where
  the mask is `false` and `other` where it is `true`. Emission:
  `a < b`, `a <= b`, `a > b`, `a >= b`, `a == b`, `a != b`,
  `any(v)`, `all(v)`, `!v`, `select(a, b, mask)`. A scalar select
  is the conditional `?:` of K9 and gets no function.
- **K27 — Swizzles and factories.** Rev 1, 2026-08-25 (the factory
  root, EG11). Rev 2, 2026-08-25: every swizzle is a read accessor on
  subscript R37, so an author writes `v.xy`. Its read records as the
  method of the same name, and the K10 table needs no change. Every
  float and integer vector gains the in-order
  swizzle accessors: on a `Vec3*`, `xy`, `xz`,
  `yz`; on a `Vec4*`, `xy`, `xz`, `xw`, `yz`, `yw`,
  `zw`, `xyz`, `xyw`, `xzw`, `yzw`. Each returns a new
  vector and emits `v.xy` and family. A swizzle outside this set is
  not an accessor, so subscript's checker rejects it before the
  generator runs. `lib/typegpu-types.ts` adds the factories
  `vec3fFrom2(v: Vec2f, z: f32)`, `vec4fFrom2(v: Vec2f, z: f32, w:
  f32)`, `vec4fFrom3(v: Vec3f, w: f32)`, `vec2fSplat(s: f32)`,
  `vec3fSplat(s)`, `vec4fSplat(s)`, and the same six shapes for the
  `i` and `u` families. They emit `vec3<f32>(v, z)`, `vec4<f32>(v,
  z, w)`, `vec4<f32>(v, w)`, `vec2<f32>(s)`, `vec3<f32>(s)`, and
  `vec4<f32>(s)`, with `i32` and `u32` for the other families. The factories join
  the K9 `vec3f` family. A swizzle is never an assignment target,
  because subscript R37 forbids a write accessor on a `@CStruct`
  value class.
- **K28 — The P8 rejections.** Rev 1 adds the slice 2 set: a
  shell with a non-literal body, a shell whose function is a kernel,
  a fence token in a body (`@group`), unbalanced braces, a second
  `wgslDeclarations` call, a shell name that collides with a schema,
  and a `naga` error inside a shell body attributed by name. Rev 0: A `Vec*b` field in a schema (SC5), a
  `Vec*b` in a binding wrapper (PI5), a K25 method on a `Vec*h`
  (the checker, because SC8 declares no arithmetic), and a method
  of a vector class that has no table row (K10, through a scratch
  class edit recorded and reverted). Each with a fixture and one
  diagnostic, except the checker case, whose fixture asserts the
  checker's diagnostic.

## WGSL shells (P8 slice 2)

- **K29 — A WGSL shell is a source function with two bodies.** A
  module-level declaration `export const addBias: WgslShellSpec =
  wgslShell(addBiasFn, { body: "return input + SHELL_BIAS;" })`
  marks the module-level function `addBiasFn` as a shell.
  `wgslShell` is a library function in `lib/typegpu.ts` with a real
  body that returns the spec. The generator finds every module-level
  `wgslShell` call (as PI1 finds `computePipeline`), reads the
  function from the `FuncRef` argument and the WGSL statements from
  the literal `body`. A shell is a helper under K2: value-type
  parameters, a value-type or `void` return, no layout class, no
  `ComputeInvocation`. The generator writes the WGSL `fn` line from
  the function's typed signature (K4 types, K12 spellings) and
  inserts the `body` statements, indented, as the function body. It
  never walks the function's subscript body. The subscript body is
  the host implementation: the CPU lane runs it (CL1), and a live
  program compares the GPU result against it. A kernel or a helper
  calls a shell by its name. A shell with no caller in any kernel's
  call graph is not emitted. A non-literal `body`, a non-`FuncRef`
  function, a declaration inside a function, and a shell whose
  function is a kernel (PI1 names it) are diagnostics.
- **K30 — Raw declarations and the fence.** A module-level
  `wgslDeclarations("const SHELL_BIAS: u32 = 7u;")` call adds WGSL
  text above the generated declarations of every WGSL module of the
  program. A program has at most one such call. Every shell body
  and the declaration text pass one lexical fence before emission:
  the generator tokenizes with WGSL's blankspace set and rejects the
  identifiers `override`, `workgroupBarrier`, `storageBarrier`,
  `textureBarrier`, the attribute pairs `@group` and `@binding`, the
  sequence `var<`, and a body whose braces do not balance. A shell
  or declaration name equal to a generated declaration (a schema, a
  binding, a kernel, a constant) is a diagnostic. The fence is
  lexical and nothing more. `naga` validates the composed module
  (K15).
- **K31 — A `naga` error inside a shell is attributed.** The emitter
  records the line span of every shell body and of the declaration
  text inside the emitted module. When `naga` reports an error on a
  line inside a span, the harness prefixes the diagnostic with the
  shell name (`shell addBias:`) or `declarations:`. A line outside
  every span stays a generator defect (K15). The module order is K14's.

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
