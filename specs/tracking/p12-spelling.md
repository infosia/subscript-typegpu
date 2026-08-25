# P12 — the TypeGPU spelling parity

Contract: `specs/blocks/ergonomics.md` EG11. Plan §8 P12.
Cross-references: `specs/blocks/pipeline.md` PI5 Rev 1 and PI6,
`specs/blocks/kernel.md` K9, K20, K27, `specs/blocks/schema.md` SC7.

## Why

The owner asked on 2026-08-25 how much of the distance from TypeGPU
is a subscript rule and how much is a free choice. The measurement
below answers it. Kind one is forced. Kind two and kind three are
the P12 work.

## Measured: what forces each divergence

The measurement reads subscript's checker at the workspace pin
`f99d4cb`. The upstream project is
https://github.com/infosia/subscript. Every path in the table below
is relative to that repository.

| TypeGPU form | Here | Cause |
|---|---|---|
| `layout.$.x`, `v.xy`, `privateVar.$` | `get()`, `v.xy()` | `compiler/src/check/mod.rs:2344` reports S100 `static methods and accessors are not decided` |
| `std.mul(v, s)` over both kinds | `mul` and `scale` | one signature per name: the method table is a map keyed by name (`mod.rs:2397`) |
| `d.vec4f(v, w)` | `vec4fFrom3(v, w)` | the same rule |
| `std` as a namespace | methods | a namespace object needs static methods |
| `root.createBuffer<T>(...)` | `createBuffer<T>(device, ...)` | type parameters exist on classes (`mod.rs:1101`) and functions (`mod.rs:1171`) only |
| `d.struct({...})`, `tgpu.bindGroupLayout({...})` | `@CStruct class`, a layout class | no Proxy, no Reflect, no eval, plus D1 |
| `a + b` | `a.add(b)` | no operator overloading (plan §6) |
| `buffer.write(values)` | `Context.bytesOf<T>(...)` | `bytesOf` needs a statically sized type (BF2) |

Two divergences have no such cause, and P12 closes them.

1. `Storage` and `MutStorage` carry an index signature already
   (`lib/typegpu.ts:566` and `583`). `x08-live-reduction.ts:57` and
   `x12-live-uniform-reread.ts:60` prove the read and the write on a
   binding. The programs and the documents still call `get(i)` and
   `set(i, v)`.
2. The vector factory root is `v3f`. TypeGPU's is `vec3f`.
   TypeGPU spells the instance type `v3f` and the schema `vec3f`,
   so the current names invert the pair.

subscript's index signature is sugar over `get` and `set`
(`corpus/accept/a136-index-signature.ts`), and it rejects the class
without them (`corpus/reject/r129-index-signature-no-get.ts`). The
two methods therefore stay in the library. EG11 makes the index
form the authored form instead.

## Status

2026-08-25: contract written (EG11, PI5 Rev 1, K9, K20, K27, SC7,
plan §8 P12).

2026-08-25: both slices landed. Slice 1 moved every program, every
example, both tutorial documents, `README.md`, the inline sources
under `crates/`, and five reject fixtures to the index form. Slice 2
renamed the factory family in `lib/typegpu-types.ts` and in
`free_function` in `crates/typegpu-gen/src/mapping.rs`.
`lib/typegpu.ts` kept `get(index)` and `set(index, value)` as the
accessors, and `crates/harness/tests/runtime/mod.rs` now covers both
the accessors and the index form.

The reject fixture `pi6-binding-method.ts` writes through a
read-only `Storage`. Its diagnostic changed text and kept its rule
and its owner:

```text
error[S100]: `a[i] = v` cannot write through a readonly index signature
 --> pi6-binding-method.ts:6:62
```

Evidence, on the reference machine: `tools/gate.sh
--require-backend` green, 254 passed, 1 ignored, 108 s.
`git status --short` listed no `programs/*.wgsl` file and no
`programs/*.expected` file, so the goldens held. `tools/live.sh` on
Metal (yawgpu) green, `live::every_x_program_passes_on_a_real_adapter`
ok, 67.56 s. The test count matched the count before the change, so
the two slices added no test and removed none.

## Phase review (2026-08-25)

A fresh no-context review read the whole phase diff. Result:
CRITICAL 0, MAJOR 1, MINOR 12. Every finding held under a check.

MAJOR 1. The slice-1 sweep rewrote both halves of the one test that
proved the two spellings emit one WGSL
(`crates/typegpu-gen/tests/kernel/mod.rs`, the `hist` reads), so its
assertion compared the index form with itself. No program reached
the emitter's four method arms after the sweep, so the arms lost
their cover. T21 replaces the lost proof with a direct test: two
sources that differ only in the access spelling must produce equal
WGSL.

The twelve MINOR findings, and what each one changed:

1. EG11 claimed that no program calls the two methods by name. The
   direct test in `crates/harness/tests/runtime/mod.rs` does, and it
   must. EG11 now scopes the claim to `programs/`, `examples/`,
   `docs/`, and `README.md`, and T20 gates it.
2. EG11 gave the cause of the free `createBuffer<T>` as "a method
   takes no type arguments". `Context.bytesOf<T>` disproves the
   sentence as written. The cause is that a user-declared method
   takes no type parameters.
3. An EG11 sentence ran 26 words. Split.
4. The plan's P12 paragraph ran seven sentences. Split.
5. This file described its own method with a passive fragment and an
   `-ing` verb form. Rewritten.
6. This file claimed every path below one sentence belongs to
   subscript. Three belong to this repository. The claim now scopes
   itself to the table.
7. The plan's §4 sketch still imported `v3f`. Renamed.
8. `docs/from-typegpu.md` said `res.items[i] = v` writes one
   element, without naming `MutStorage`. A `Storage` index signature
   is read-only.
9. `docs/from-typegpu.md` carried two wordings for the missing
   accessor. One concept takes one wording.
10. The reject fixture `pi6-binding-method.ts` holds an index write.
    Renamed to `pi6-storage-readonly-write.ts`.
11. The index line added to the runtime test reached no assertion.
    The test now prints the value it writes, and it covers all three
    classes.
12. K9, K20, K27, and SC7 changed their normative identifier lists
    with no Rev marker, while PI5 took one in the same diff. All
    four now carry Rev 1.

The MAJOR finding adds T21 and MINOR 1 adds T20. `rule-ids.txt` also
gained T19, which the previous round left out.

## Close (2026-08-25)

The review fixes landed. T20 is
`crates/harness/tests/authored_spelling/`. It walks the checked HIR
of every program and example, and it reads the source line to tell
an authored method call from the index sugar, because subscript
lowers `x[i]` to a `get` call. T21 is
`binding_methods_and_indices_emit_identical_wgsl` in the generator's
kernel tests. It generates two sources that agree in every name, and
it asserts equal WGSL plus the five expected expressions, so the two
forms cannot pass by failing alike.

Reds: the T21 scratch added one to the emitted index of the
`WorkgroupArray` method arm, and the test failed with the missing
expression. The T20 scratch restored `programs/b14-wgsl-shell.ts`
line 54 to `set(0, v)`, and the test failed with
`EG11: use `x[i] = v` for MutStorage<u32> instead of `set``. Both
sources were restored.

Close evidence: `tools/gate.sh --require-backend` green, 256 passed,
1 ignored, 116 s. The count grew by the two new tests. Every
`programs/*.wgsl` and `programs/*.expected` file stayed
byte-identical. `tools/live.sh` on Metal (yawgpu) green, 67.49 s.
