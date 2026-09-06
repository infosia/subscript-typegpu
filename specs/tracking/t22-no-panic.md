# t22-no-panic — library code has no panic site

2026-09-06. Owner instruction: the public API never panics. The
contract is `testing.md` T22. The comment pass
(`comments-pass.md`) listed the sites by hand. `cargo clippy` with the
restriction lints counted them per library crate before any change:

| Crate | `unwrap`/`expect` | `panic!` and `unreachable!` | indexing | slicing |
|---|---|---|---|---|
| `subscript-typegpu-gen` | 3 | 0 | 93 | 3 |
| `subscript-typegpu-harness` (lib) | 4 | 2 | 96 | 5 |
| `subscript-typegpu-webgpu-gen` | 41 | 13 | 26 | 4 |
| `subscript-typegpu-facade` | 0 | 0 | 2 | 2 |

`clippy::arithmetic_side_effects` counted 87, 90, 42, and 3 more.
Arithmetic overflow is outside T22 and stays recorded here.

The rounds: 1 `typegpu-gen`, 2 `harness` and `facade`, 3
`webgpu-gen`. Each round lands the lint denial in the crate root, so
the gate holds the crate from that commit on.
