# Block: library (LB-rules)

P15 contract. Rev 0, 2026-09-01. Rev 1 (LB1, LB3: the ui module),
2026-09-05. Rev 2 (LB1: import-reachable loading), 2026-09-05. Rev 3
(LB5: application modules), 2026-11-23. Rev 4 (LB5 Rev 1: one
flat directory), 2026-11-23. Plan §8 P10 slice 2, §8 P15, and §8 U1 govern the
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
- **LB5 — Application modules load from disk.** Rev 1, 2026-11-23.
  An application module is a source file that the program reaches
  through a relative import and that is not a registered module.
  The loaders (the harness and the generator) collect the
  application modules before the first check. The rules follow the
  subscript CLI loader, because subscript keys a module by the name
  after `./` in one flat namespace:
  1. The loaders read the import specifiers of the entry and of each
     application module, transitively.
  2. A specifier of the form `./<name>`, where `<name>` holds no
     `/`, resolves to `<name>.ts` in the entry's directory. The
     source file name is `<name>.ts`.
  3. A specifier that names a registered module (LB1) or the entry's
     support module `./<stem>.typegpu` never reads the disk. The
     registered module wins over a file with the same name.
  4. A specifier that is not of the form `./<name>` (for example
     `./a/b` or `../x`), or a `<name>.ts` that does not exist, adds
     no file. The compiler then reports S100 at the import site.
  5. The application modules load after the library modules and
     before the entry. The order is a depth-first post-order over the
     import declarations in source order: a module loads after the
     modules it imports. A module that the walk already reached is
     not visited again, so an import cycle ends at that module.
  6. The development tier, the shipping tier, the generator CLI, and
     the window host use the same loader.

  An application module follows LB2: the entry's pipeline can call
  a K2 helper or pass a kernel that an application module defines.
  A pipeline declaration in an application module is out of scope.
  Its `./<stem>.typegpu` import has no generated module and is S100.

  Programs: `b28-app-modules` imports `./m28-tile`, which imports
  `./m28-tile-size` (`testing.md` T1). The entry prints a value from
  each module on both tiers. Its compute pipeline calls a helper
  from `m28-tile.ts`, and its WGSL golden holds that helper. Reds: a
  missing application module and a specifier with a `/` after `./`
  each give S100 at the import site.

  Rev 1 measurement (2026-11-23, subscript `31e14f1c`): the checker
  resolves `./x` from `sub/a.ts` to a file `unrelated/x.ts`, and it
  rejects `./sub/a` from the entry with S100 even when `sub/a.ts` is
  present. Rev 0 required importer-relative paths with
  subdirectories, which the language does not express.
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
