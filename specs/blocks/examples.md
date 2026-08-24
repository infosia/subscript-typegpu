# Block: examples (EX-rules)

P10 contract. Rev 0, 2026-08-24. Rev 1 (EX2 header exemption),
2026-08-24. Plan §8 P10 governs this block.
The window host is `window.md` (W-rules). The example set is the
plan's P10 list.

## What an example is

- **EX1 — An example is a deliverable program, not a suite entry.**
  Each example lives in `examples/<name>/main.ts`, one directory per
  example. It has no golden and no `programs/` entry. A windowed
  example exports the W2 entries. A headless example exports `main`
  and prints its result. `tools/window.sh examples/<name>/main.ts`
  runs the first kind, `tools/example.sh examples/<name>/main.ts`
  (the harness dev lane) runs the second.
- **EX2 — The source is the document.** Each example reads top to
  bottom as a teaching artifact for a reader who knows TypeGPU. The
  header comment states: the example name, one sentence on what it
  shows, and the upstream sentence `Ported from TypeGPU's <name>
  example (https://github.com/software-mansion/TypeGPU).` Comments
  through the file explain what the code does, and, where the shape
  differs from TypeGPU, one comment states the difference in one or
  two sentences at the point of divergence. A comment never
  explains subscript syntax, never repeats the code in words, and
  never exceeds three lines. The header block is exempt from the
  line cap: it carries the name, the purpose, the reductions, and
  the citation (Rev 1).
- **EX3 — Examples use the public layers only.** The API layer, the
  TypeGPU layer, and `typegpu-types`. No facade name, no mirror
  name, no `Context.suspend` (the layers own the polling). The
  hygiene residue rules cover `examples/*`.

## Gates

- **EX4 — Every example compiles in the gate.** The W13 Rev 1
  harness test walks `examples/*/main.ts`, compiles each through the
  shared program loader on the dev tier with no device, and asserts
  success. A windowed example additionally passes the W2 signature
  test. Adding an example adds no Rust code.
- **EX5 — A headless example is checked by value.** A headless
  example ends with one `check:` line that states a computed
  invariant and `pass` or `fail`, so a reader who runs it knows the
  result without a golden. The gate does not run examples on a
  device (invariant 5); the owner's device runs are recorded in
  `specs/tracking/p10-examples.md`.
- **EX6 — Upstream is cited, never copied.** The port reimplements
  the example's idea in this library's shapes. No source line, no
  asset, and no shader text is copied from the upstream repository.
  Assets an example needs are generated in code (procedural data),
  never fetched.

## Interaction

- **EX7 — Interaction is the window host's surface.** A windowed
  example uses what W2 and W3 carry: the frame size, one key scalar
  per frame, and time derived from the frame count. An upstream
  example whose sliders or pointer input are essential is either
  reduced to key input with the reduction stated in the header, or
  deferred until the W-rules grow.
