# Block: library (LB-rules)

P15 contract. Rev 0, 2026-09-01. Rev 1 (LB1, LB3: the ui module),
2026-09-05. Rev 2 (LB1: import-reachable loading), 2026-09-05. Plan §8 P10 slice 2, §8 P15, and §8 U1 govern the
module set. Kernel-subset rules are `kernel.md`
(K-rules).

- **LB1 — The module set is registered.** Rev 2. The script-facing
  library modules are `typegpu.ts`, `typegpu-types.ts`,
  `typegpu-color.ts`, `typegpu-noise.ts`,
  `typegpu-radiance-cascades.ts`, `typegpu-sdf.ts`, `typegpu-sort.ts`,
  `typegpu-ui.ts` and `typegpu-ui-atlas.generated.ts` (from U1,
  `ui.md`), and the generated mirrors. The generator and the harness
  register every module by name, in the harness load order. Rev 2: a
  program compiles with the core set (the two mirrors, `webgpu.ts`,
  `typegpu-types.ts`, `typegpu.ts`) and with the registered modules
  its import declarations reach, transitively, and with no other
  module. The loaders read the `from "./<module>"` specifiers of the
  program and of each loaded module. A program that imports no
  module beyond the core set compiles as it did before any module
  existed. Measured cause (U2, `specs/tracking/build-time.md`): with
  every module loaded, the ui module doubled the per-program compile
  on both tiers.
- **LB2 — Module GPU code is kernel-subset code with real host
  bodies.** A module can define K2 helpers, value classes, module
  constants, and kernels. A program imports a module kernel and
  passes it to `computePipeline` (measured, P15 slice 2).
- **LB3 — A module trap has a documented rule id.** A trap prints
  the rule id, the method, the values, and `(author)`, then stops.
  Rev 1. The trap table: `SORT1` — a sort driver received a length
  or a pass index outside its stated domain. `UIT1`, `UIT2`, `UIT3`, `UIT4` —
  the ui module's traps, defined in `ui.md` UI18. A trap id that is
  not in this table is a defect.
- **LB4 — Module tests.** Every exported GPU function is reached by
  a generator test that compiles and `naga`-validates a kernel.
  Every trap has a demonstrated red: a test drives the trap and
  observes the message.
