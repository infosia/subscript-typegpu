# Block: library (LB-rules)

P15 contract. Rev 0, 2026-09-01. Rev 1 (LB1, LB3: the ui module),
2026-09-05. Rev 2 (LB1: import-reachable loading), 2026-09-05. Rev 3
(LB5: application modules), 2026-11-23. Rev 4 (LB5 Rev 1: one
flat directory), 2026-11-23. Rev 5 (LB5 Rev 2: module paths), 2026-11-23. Plan §8 P10 slice 2, §8 P15, and §8 U1 govern the
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
- **LB5 — Application modules load from disk.** Rev 2, 2026-11-23.
  An application module is a source file that the program reaches
  through a relative import and that is not a registered module.
  The loaders (the harness and the generator) collect the
  application modules before the first check. Subscript keys a
  module by its path (subscript `compiler.md` §354). The loaders use
  its public resolver, `subscript_compiler::resolve_module_specifier`,
  and its `module_key`. No other code here resolves a specifier.
  1. The loaders read the import specifiers of the entry and of each
     application module, transitively.
  2. **The root.** The walk starts with the entry's directory as the
     root. A relative specifier (`./` or `../`) resolves against the
     name of the importing file with the resolver. The result is a
     key, the file is `<root>/<key>.ts`, and its source file name is
     `<key>.ts`. If a key starts with `..`, the root moves up by the
     largest count of leading `..` segments, every name gains the
     matching directory prefix, and the walk starts again. When the
     walk ends, no key starts with `..`. This is the root rule of
     subscript §354 rule 8, so `subscript run` and these loaders give
     one program the same names.
  3. **The entry's directory.** Let `<dir>` be the entry's directory
     relative to the root, with a trailing `/`, or empty when the root
     is the entry's directory. The entry's name is `<dir><stem>.ts`.
     The registered modules (LB1) and the entry's support module sit
     in the entry's directory: the key of a registered module is
     `<dir>` followed by its `LIBRARY_ORDER` name without `.ts`, and
     the support key is `<dir><stem>.typegpu`. A resolved key equal
     to one of these keys never reads the disk. The registered module
     wins over a file with the same key. A file in another directory
     reaches them through the path to the entry's directory, for
     example `../typegpu` from `world/chunk.ts` when the entry is
     `main.ts`. Every check that identifies a registered module or the
     support module compares these keys.
  4. A specifier that is not relative, a key whose file does not
     exist, or a key that names a directory adds no file. The
     compiler then reports S100 at the import site.
  5. The application modules load after the library modules and
     before the entry. The order is a depth-first post-order over the
     import declarations in source order: a module loads after the
     modules it imports. A key that the walk already reached is not
     visited again, so an import cycle ends at that module.
  6. The development tier, the shipping tier, the generator CLI, and
     the window host use the same loader.
  7. **One name in two modules.** Kernel code can reach two free
     functions with one source name. If each one is declared in an
     application module, the generator emits each as
     `<name>_<key>` through the WGSL identifier mapping. In every
     other case (a declaration in the entry, a method, a module
     constant, a registered module, or an alias that equals another
     WGSL name) the generator reports K14. It never emits one WGSL
     declaration for two source declarations. Two value classes with
     one name are rejected for the whole program, whether or not a
     kernel reaches them.
  8. **The support module from an application module.** An
     application module can import the entry's support module through
     the path to the entry's directory, for example
     `../main.typegpu` from `sub/tile.ts`. The entry and any number of
     application modules can import it in one program. The generator
     treats every import whose resolved key is the support key as one
     support import.

  Open: one file that two spellings reach (another letter case on a
  file system that ignores case, or a symbolic link) gets two keys and
  loads twice. Subscript §354.4 open item 1 records the same case.

  An application module follows LB2: the entry's pipeline can call
  a K2 helper or pass a kernel that an application module defines.
  A pipeline declaration in an application module is out of scope.
  Its own `<stem>.typegpu` import has no generated module and is
  S100.

  Programs: `b28-app-modules` imports `./m28-tile`, which imports
  `./m28-tile-size` (`testing.md` T1). `b30-module-paths` imports
  `./m30/util` and `./m30/inner/util`. The two modules have one base
  name, and each declares a K2 helper with one name. `m30/inner/util.ts`
  imports `../util`, and `m30/util.ts` imports `../typegpu`. No
  committed file spells two `..` segments in a row. CLAUDE.md "No
  local or sibling paths" forbids a path that escapes the repository
  root, and `tools/hygiene.sh` approximates that rule by text. The
  text check does not resolve a path, so it also rejects a specifier
  that stays inside `programs/`. Subscript §354 tests the resolution
  of deeper paths, so one `..` level covers this repository's loader.
  The entry prints a value from each module on both tiers. Its compute pipeline calls both helpers,
  and its WGSL golden holds both. Harness fixtures cover an entry in
  a subdirectory that imports `../shared/x`, a module outside the
  entry's directory that imports the library and the entry's support
  module, and an application module that imports the support module
  while the entry imports it too. Reds: a missing module
  (`./sub/missing`) and `./typegpu` from a subdirectory each give
  S100 at the import site. A generator test covers rule 7 in a render
  pipeline. K14 reds cover an entry helper and a module helper with
  one name, and an alias that equals another WGSL name.

  Rev 1 measurement (2026-11-23, subscript `31e14f1c`): the checker
  keyed a module by its base name, so `./sub/a` was S100 and the
  importer's directory was not read. Rev 1 loaded `./<name>` files
  from the entry's directory only. Subscript §354 (change request
  R42) keys a module by its path, and Rev 2 adopts its resolver.
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
