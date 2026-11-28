# P24 — matrix columns

## The defect (2026-11-28, revision `0cff83e`)

A probe kernel held `const m = new Mat2x2f(…); let n = m; n.c0.x =
9.0; n.c1 = new Vec2f(5.0, 6.0); res.out[0] = m.c0.x + n.c1.y +
res.mats[0].c1.x;`. `subscript-typegpu-gen gen` exited 0 and
emitted:

```wgsl
  var m = mat2x2<f32>(vec2<f32>(1.0f, 2.0f), vec2<f32>(3.0f, 4.0f));
  var n = m;
  n.c0.x = 9.0f;
  n.c1 = vec2<f32>(5.0f, 6.0f);
  out[0u] = m.c0.x + n.c1.y + mats[0u].c1.x;
```

`naga` rejects the member with `invalid field accessor` (the red
below). No committed program
reads a matrix column in a kernel, so no golden holds the form. K36
is the contract.

subscript rejects `res.mats[1].c0 = v` (S100, a write through a copy
of the element), so a column write to a binding element never
reaches the generator.

## Evidence

Red, at `0cff83e`: `cargo test --offline -p subscript-typegpu-gen
--test main matrix_columns::` exited 101 in 2.27 s. 10 of the 11
tests failed. `naga` rejected each module at its first column
access, for a read and for a write:

| Test | First rejected line | `naga` |
| --- | --- | --- |
| `mat2_reads`, `mat3_reads`, `mat4_reads` | `var first = m.c0;` | `invalid field accessor` `c0` |
| `local_writes` | `n.c0 = v;` | `invalid field accessor` `c0` |
| `written_parameter` | `m.c0.x = 9.0f;` | `invalid field accessor` `c0` |
| `schema_field` | `s.transform.c1.x = 9.0f;` | `invalid field accessor` `c1` |
| `binding_reads` | `output[0u] = transform.c3.w + mats[1u].c0.x;` | `invalid field accessor` `c3` |
| `call_result` | `var v = makeMatrix().c1;` | `invalid field accessor` `c1` |
| `capture_before_conflict` | `let _g_operand_0 = m.c1;` | `invalid field accessor` `c1` |
| `fragment_column` | `return m.c2;` | `invalid field accessor` `c2` |

The test that passed is the control: a user schema with a field
`c0` keeps `.c0`.

Green: 11 passed in 3.27 s. `naga` accepts the component writes
`m3[1u].y = 20.0f;` and `m4[3u].w += 4.0f;`. No existing WGSL golden
moved (66 files). The new programs are `b34-matrix-columns` and
`x31-live-matrix-columns`, over the shared module
`programs/m34-matrix-columns.ts`. The kernel stores nine words, one
weighted sum per column of the three classes:
`17, 11, 87, 118, 129, 349, 409, 469, 561`.

## Phase review (2026-11-28)

A no-context reviewer read the change and ran five probe kernels
through the generator. Result: no CRITICAL, 1 MAJOR, 6 MINOR.

| Finding | Disposition |
| --- | --- |
| MAJOR 1: this record had no red and no gate citation. | Fixed in this record. |
| MINOR 1, 2: `field_suffix` had no doc comment, and `Place::field` appended a suffix that is not always a field. | Fixed: the doc comment, `push_suffix`. |
| MINOR 3: no test for a module-constant root, a private or workgroup variable root, an object that needs parentheses, or a `FixedArray` element. The probes gave correct WGSL. | Fixed: five tests. |
| MINOR 4, 5: K36 listed fewer roots than the code covers, and cited *(docs)* for a claim that `naga` now proves. | Fixed in K36. |
| MINOR 6: the `m34` header did not cite LB5. | Fixed. |

The reviewer found these areas clean: every emission of a field
goes through one function, the class test (a program class named
`Mat3x3f` is a K14 diagnostic, and a user schema with `c0` to `c3`
keeps its members), the parentheses of `(m * m)[0u].x`, the nine
expected words by hand, and residue. A column swap, a component
swap, and a row-for-column transposition each change a word.

## Close

Review fixes, targeted runs: the generator suite, 206 passed. The
harness tests `wgsl_goldens::` (2 passed) and `suite_structure::`
(2 passed).

Gate: `tools/gate.sh --require-backend` on yawgpu (Noop),
2026-11-28: green, 450 passed, 1 ignored, 535.66 s wall. The
previous row is 547 s with 434 tests.

Live: `tools/live.sh` on Metal (yawgpu), 2026-11-28:
`live::every_x_program_passes_on_a_real_adapter`, 1 passed,
221.36 s. `x31-live-matrix-columns` printed `PASS`: the nine device
words equal the host words, the component writes included.
