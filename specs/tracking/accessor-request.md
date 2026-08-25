# accessor-request — value accessors as method sugar (R37)

Goal: `res.params.$`, `privateOffset.$`, `privateOffset.$ = v`, and
`v.xy` become legal subscript. TypeGPU writes `layout.$.params`,
`privateVar.$`, and `v.xy`. The emitted WGSL does not change.

The request follows subscript's own precedent R29 (`compiler.md`
§58). There, a class index signature is sugar over a declared `get`
and `set` method. The checker does the rewrite. No new HIR form
exists, and no tier changes. This request asks for the same
treatment of a named accessor.

## Measured, 2026-08-25, at the workspace pin `f99d4cb`

Four probes ran through `subscript-typegpu-harness dev`. Each one
holds the smallest program that needs the construct.

| Probe | Diagnostic |
|---|---|
| `get $()` and `set $()` on a reference class | `S100: static methods and accessors are not decided`, once per accessor, then `S004` and `S100` at each use |
| `get sum()` on a `@CStruct` value class | the same `S100`, then `S100: Pair has no member sum` |
| `static make(...)` on a class | the same `S100`, then `S100: classes have no static member make` |
| `identity<T>(value: T): T` as a method | `S100: unknown type name T`, twice |

The checker reports one message for accessors and statics together
(`compiler/src/check/mod.rs`, the class-member loop). The parser
already carries the accessor kind, so the parse side needs no work.
Method type parameters parse and never bind.

## Consequence here

The change moves 49 authored sites and 3 library classes.

- 23 calls of `get()` on `Uniform`, `PrivateVar`, and `WorkgroupVar`
  become `.$`.
- 1 call of `set(v)` on a `PrivateVar` becomes `.$ = v`.
- 25 swizzle calls such as `v.xy()` become `v.xy`.
- `lib/typegpu.ts` gains an accessor on three wrapper classes.
  `lib/typegpu-types.ts` gains one per swizzle.

The generator keeps its structure. The checker rewrite produces the
call HIR that `crates/typegpu-gen/src/kernel.rs` already reads. Three
method-name arms take the accessor name in place of `get` and `set`:
the uniform arm, the private-variable arm, and the workgroup-variable
arm. The K10 swizzle table needs no change, because an accessor keeps
the swizzle name. Every `.wgsl` and `.expected` golden must stay
byte-identical, exactly as in P12. That equality is the acceptance
test on this side.

`Vec2f` through `Vec4h` are `@CStruct` value classes, so the swizzle
half needs a read accessor on a value class. R29 restricted index
signatures to reference classes. The request therefore splits the
two halves: a read accessor on both class kinds, a write accessor on
reference classes only.

## Not requested, and why

- **Static methods.** No site in this repository gets closer to
  TypeGPU with them. `Vec3f.splat(s)` is no nearer to `d.vec3f(s)`
  than the free `vec3fSplat(s)`, and the free `vec3f(x, y, z)`
  already matches `d.vec3f(x, y, z)`. Statics pay only together with
  overloads, as a `std` namespace object.
- **Method type parameters.** They allow
  `device.createBuffer<T>(...)`. `GPUDevice` is generated from the
  IDL, so a TypeGPU-layer method on it crosses the layer boundary of
  design invariant 8. The alternative is a `Root` class, and that is
  a decision for this side first.
- **Overloads.** They give `mul` one name for both kinds and
  collapse 27 factories into 12. The cost is a resolution rule in a
  language whose method table is one signature per name. This needs
  its own request and its own evidence.

## Owner decisions, 2026-08-25

Three points of the request needed a decision. The owner took all
three as proposed.

1. **The C name.** `sanitize` in subscript's `codegen/src/cemit.rs`
   maps every character outside `[A-Za-z0-9_]` to `_`, so an
   accessor named `$` collides with a member named `_`. `$` takes a
   distinct escape. Every other name keeps its current C spelling.
2. **Compound assignment.** `x.$ += 1` stays rejected, as R29
   rejects `a[i] += v`. This side writes `x.$ = x.$ + 1`. TypeGPU
   writes `+=`, and that difference stays.
3. **A write accessor on a value class.** Forbidden. A value class
   copies on assignment, so the write reaches a copy.

## Status

2026-08-25: request written and issued to the subscript repository
as R37. The measurements above are the evidence. Nothing on this
side waits for it.
