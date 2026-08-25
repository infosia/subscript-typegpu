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

## R37 landed (2026-08-25)

subscript `a2228d9` carries it: contract `2bc4a4f`, amended
`0840f51` after a phase review, implementation `f29c4c5`.
`specs/blocks/compiler.md` §65 is the rule, and `collisions.md` C12
records the JS divergence. The workspace re-pins to `a2228d9`.

The shape landed as asked, with one divergence and four additions
that this side must read.

- **The divergence.** The request asked for one method per accessor
  name. The pair records as two methods: the read as `name` with no
  parameters, and the write as `name=` with one parameter and the
  return type `void`. A method table holds one signature per name,
  and both tiers key a method by name, so one name for both
  collides. An identifier holds no `=`, so `name=` collides with no
  declared method. The emitter therefore reads the method name `$`
  for a read and `$=` for a write.
- The read accessor declares no parameter and an explicit return
  type. The write accessor declares one typed parameter, no default,
  and no return type.
- The read and the write of one name must share one type (§65 rule
  1a).
- A write accessor without a read accessor is illegal (§65 rule 6).
- The C name gained two escapes, `$` to `_dollar_` and `=` to
  `_set_`, plus a per-namespace deduplication. Point 1 of the owner
  decisions landed wider than it was asked, because the phase review
  measured a collision that needs no `$` at all.

## P13 — the sweep (2026-08-25)

`Uniform<T>` took the read accessor `$`. `PrivateVar<T>` and
`WorkgroupVar<T>` took the read and the write accessor `$`. Every
swizzle became a read accessor, 39 of them over six vector classes.
The three index families did not move: their `get(i)` and
`set(i, v)` are the accessors that the index signature needs.

The emitter reads the new names at six sites in
`crates/typegpu-gen/src/kernel.rs`: the host-block predicate, the
global root, the wrapper reference, the private and workgroup arms,
the uniform arm, and the barrier validator. One predicate changed
shape. It read `name == "get" && args.len() <= 1`, which covered a
scalar read and an array read together. It now reads
`(name == "$" && args.is_empty()) || (name == "get" && args.len() ==
1)`, which is the same set after the rename.

The sweep moved 23 reads, 1 write, and 25 swizzle calls, as the
consequence section predicted.

Evidence: `tools/gate.sh --require-backend` green, 256 passed, 1
ignored, 117 s. Every `programs/*.wgsl` and `programs/*.expected`
file stayed byte-identical, so the checker rewrite fed the emitter
the call HIR it already read. `tools/live.sh` on Metal (yawgpu)
green, 67.36 s.

## Phase review (2026-08-25)

CRITICAL 0, MAJOR 5, MINOR 2.

Four MAJOR findings were stale prose that the change made false: the
uniform-read sentence and the swizzle sentence in
`docs/from-typegpu.md`, "swizzles as properties" in that document's
list of features with no equivalent, and K20 Rev 2, which named the
method `x` where the accessor is `$`. The fifth MAJOR is open below.
Both MINOR findings were in the same document: the `+=` difference
was unlisted, and the bare word "accessors" now reads as two things.

## Remaining defect, recorded not fixed

`docs/from-typegpu.md` swizzle bullet now reads "Swizzles are read
accessors: `v.xy` ... TypeGPU writes `v.xy`." The second sentence
repeats the first, because the two spellings are equal now. The
document had its verification pass and its fix pass, so CLAUDE.md
"Two rounds" stops here.

## Owner decision: the WebGPU layer moves (2026-08-25)

The owner chose the accessor move over a restated reason. J14 is the
contract and plan §8 P14 is the phase.

## The finding that scoped P14

`crates/webgpu-gen/policy.toml` holds 14 `attribute-method`
deviations, and each one gives "user-defined accessors are
unavailable" as its reason. The generator copies that reason into
`lib/webgpu.ts`, so a shipped artifact carries it 14 times. R37
makes the reason false.

The members are `GPUDevice.queue`, `GPUBuffer.size`, `usage`, and
`mapState`, `GPUTexture.width`, `height`, `depthOrArrayLayers`,
`mipLevelCount`, `sampleCount`, `dimension`, `format`, and `usage`,
and `GPUQuerySet.type` and `count`.

Design invariant 8 says the WebGPU API layer follows the WebGPU
JavaScript API in naming and shape, and that a deviation is a
recorded policy deviation. The recorded cause is gone, so the
deviation has no cause. Two answers exist. The layer moves to read
accessors, which deletes all 14 rows and matches the JavaScript API.
Or the rows keep a different and true cause. No true cause is known
today.

The move costs a generator change, a regeneration, and a sweep of
about 200 call sites, most of them `device.queue()`. It is a phase,
not a round. The owner decides.

## A defect P14 must close

`examples/matrix-next/main.ts` line 205 and
`examples/dispatch/main.ts` line 123 bind an owned device's queue
with `using`. The owned `GPUDevice` caches one `GPUQueue` and
disposes it in its own `dispose()`, so the caller disposes a wrapper
it does not own. `GPUQueue.dispose()` guards on a private flag, so
the second disposal is silent, and neither example reads the queue
after the scope ends. The defect is latent today.

A property makes the misuse unwritable: `using queue = device.queue`
has no method call to bind. P14 rewrites both lines to read the
property at each use.

`GPUHostOwnedDevice.queue()` is the opposite case and is correct
today. It returns a new owned wrapper per call, and every windowed
example binds it with `using`. J14 keeps it a method for that
reason.
