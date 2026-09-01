# Block: library (LB-rules)

P15 contract. Rev 0, 2026-09-01. Plan §8 P10 slice 2 and §8 P15
govern the module set. Kernel-subset rules are `kernel.md`
(K-rules).

- **LB1 — The module set is registered.** The script-facing library
  modules are `typegpu.ts`, `typegpu-types.ts`, `typegpu-color.ts`,
  `typegpu-noise.ts`, `typegpu-radiance-cascades.ts`,
  `typegpu-sdf.ts`, `typegpu-sort.ts`, and the
  generated mirrors. The generator and the harness register every
  module by name, in the harness load order.
- **LB2 — Module GPU code is kernel-subset code with real host
  bodies.** A module can define K2 helpers, value classes, module
  constants, and kernels. A program imports a module kernel and
  passes it to `computePipeline` (measured, P15 slice 2).
- **LB3 — A module trap has a documented rule id.** A trap prints
  the rule id, the method, the values, and `(author)`, then stops.
  The trap table: `SORT1` — a sort driver received a length or a
  pass index outside its stated domain. A trap id that is not in
  this table is a defect.
- **LB4 — Module tests.** Every exported GPU function is reached by
  a generator test that compiles and `naga`-validates a kernel.
  Every trap has a demonstrated red: a test drives the trap and
  observes the message.
