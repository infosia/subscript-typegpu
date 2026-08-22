# CLAUDE.md — subscript-typegpu permanent development rules

This file holds only what is *invariant* — roles, boundaries, and
conventions. `specs/subscript-typegpu-project-plan.md` holds design
and phasing. When the plan and this file disagree, this file wins.
When evidence disagrees with either, fix both.

## What this project is

**subscript-typegpu** brings WebGPU and the ideas of
[TypeGPU](https://github.com/software-mansion/TypeGPU) to
[subscript](https://github.com/infosia/subscript) programs. It is a
library project, not a language project. It delivers two script-facing
layers over one substrate:

1. the **WebGPU API layer** (`lib/webgpu.ts`): a subscript-source
   library presenting the WebGPU **JS** API shape (`GPUDevice`,
   `createBuffer`, gpuweb IDL naming) adapted to subscript semantics;
2. the **TypeGPU layer** (`lib/typegpu.ts` plus `subscript-typegpu-gen`): typed
   data schemas with automatic memory layout, and GPU kernels authored
   in subscript and compiled to WGSL by a generator that walks the
   typed HIR;
3. the **facade**: a Rust crate exporting a C-ABI surface (`subscript-typegpu.h`)
   over a webgpu.h implementation that the facade loads at run time;
4. the **generated ambient mirror** (`.d.ts`) the WebGPU API layer sits
   on — plumbing, not the product;
5. **`subscript-typegpu-webgpu-gen`**: the generator that emits the facade, `subscript-typegpu.h`,
   the mirror, `lib/webgpu.ts`, and the harness symbol table from the
   pinned webgpu.yml, the pinned gpuweb IDL, and a policy file;
6. the **program suite and gates** proving all of it, headless.

A prior proof of concept built the same layers. Its generator (item 5)
is imported into this repository without history and reshaped to this
repository's rules. Its measured findings are restated in
`specs/` as first-party rules. Every other part is designed anew. The
proof of concept's code is reference material, never a source to copy.

## Roles (read first)

Implementation is done by a **separate coding agent**: Codex, reached
through the `codex` MCP server. **Claude plans and orchestrates** — it
authors `specs/`, emits task handoffs, reviews the coding agent's
diffs against acceptance criteria, runs builds and tests, and manages
git (`init`/`add`/`commit`). Claude does not write production code.
The coding agent does not plan, edit `specs/`, change scope, or
commit.

Claude's own subagents (surveys, reviews, phase reviews) run on the
Opus model wherever the task allows. The top-level model's usage limit
is the scarce resource.

## Design invariants (read second)

1. **The script-visible surface crosses subscript's bindable C subset
   only.** Every construct in `subscript-typegpu.h` must be mapped by
   `subscript bind` without loss. A construct the mirror cannot
   express is redesigned in the facade — never worked around in the
   language, never leaked to scripts.
2. **Backend-agnostic over webgpu.h, chosen at run time.** The facade
   loads the webgpu.h implementation from a shared library named by
   `SUBSCRIPT_TYPEGPU_BACKEND_LIB` and resolves the `wgpu*` symbols it needs. No
   backend is linked. No cargo feature selects a backend. The
   workspace has exactly one build configuration.
   [yawgpu](https://github.com/infosia/yawgpu) is the Tier-1 backend
   and its Noop backend is the headless gate substrate. Dawn is the
   conformance oracle (gated runs, never CI-required).
3. **Async is futures + ProcessEvents, only.** Every future callback
   the facade registers uses `WGPUCallbackMode_AllowProcessEvents` and
   terminates inside the facade. The one callback the pinned header
   defines without a mode, the uncaptured-error callback, only
   records into a queue that scripts drain after a pump. Scripts poll
   futures around `Context.suspend()`. No facade callback ever
   unwinds or calls back into webgpu.h.
4. **The pinned webgpu-headers revision is the shape of truth.** When
   an implementation disagrees with the pin, the pin wins.
5. **Headless-first.** Every required gate passes with no GPU, no
   window, no external device. Real-device runs are gated, never
   CI-required.
6. **Scripts are trusted; lifetimes are manual.** Create-owns plus
   explicit `dispose()` and `using`. No finalizers, no reference
   tracking on behalf of scripts.
7. **The implementation language is Rust.** The only C in the tree is
   generated: `subscript-typegpu.h`, and subscript's ship-tier output. Panics
   never unwind across the C ABI.
8. **The public API is JS-shaped and TypeGPU-shaped; the substrate is
   not public.** The WebGPU API layer follows the WebGPU JS API
   (gpuweb IDL) in naming and shape. The TypeGPU layer follows
   TypeGPU's concepts and naming. Where subscript semantics forbid a
   shape, the resolution is a recorded policy deviation or a subscript
   change request — never an improvisation. The facade and the mirror
   are internal plumbing.
9. **Compile time replaces run time.** What TypeGPU does at run time
   with `Proxy`, reflection, and `new Function`, this project does in
   `subscript-typegpu-gen`. No run-time schema construction exists.
10. **Build time is a gate.** The rules are in "Build time" below.
    A change that breaks a build-time budget is red, the same as a
    failed test.

## Non-goals (permanent unless the plan is revised with evidence)

- **WebGPU coverage ahead of need.** The WebGPU API layer grows by
  area, in the order the TypeGPU layer needs. Coverage of the rest of
  the IDL is a planned phase, never a reason to delay a TypeGPU phase.
- **JS semantics emulation.** No promise objects, no GC-driven
  lifetimes, no exceptions, no DOM-coupled surface.
- **Running TypeGPU's own tests or the upstream CTS as an oracle.**
  TypeGPU's layout rules serve as golden vectors. Implementation
  conformance below the facade is
  [webgpu-native-cts](https://github.com/infosia/webgpu-native-cts)'s
  job.
- **Implementing WebGPU.** GPU semantics belong to the backend.
- **Upstreaming to external projects.** When this project must change
  a dependency, it forks and pins the fork, cited by URL and pinned
  by commit.
- **Adversarial hardening.** Scripts are trusted first-party logic.
  Effort goes to clear, early errors for honest mistakes.

## Build time

The proof of concept's full gate took 3.5 hours. The measured causes
were one executable per integration-test file, three feature
configurations, build scripts that reran on environment changes, and
full debuginfo. These rules close each cause.

1. **Zero cargo features in the workspace.** Backend selection,
   optional legs, and measurement modes are run-time inputs.
2. **No `build.rs` in the workspace.** Generated artifacts are
   committed and regenerated by tools under `tools/`.
3. **One integration-test executable per crate.** Integration tests
   live in `tests/main.rs` with one module per topic. A library crate
   adds at most one unit-test executable. Binaries carry no tests.
   Suites are data-driven over `programs/`, so a new program adds no
   Rust code.
4. **Dependency closure is a contract per crate.**
   `subscript-typegpu-facade` depends on no subscript crate and links no webgpu.h
   implementation. `subscript-typegpu-webgpu-gen` depends on `subscript-bindgen` and
   its model crates, never on `subscript-compiler`. `subscript-typegpu-gen`
   depends on `subscript-compiler` and never on `subscript-codegen`.
   `naga` is a dev-dependency only. The harness is the only crate
   that depends on `subscript-codegen`.
5. **`[profile.dev] debug = "line-tables-only"`** in the workspace
   manifest.
6. **Budgets are measured, recorded, and gated.** P0 measures the
   cold build, the warm no-op build, and the warm full gate on the
   reference machine, and records them in
   `specs/tracking/build-time.md`. The plan sets the budgets. Every
   phase close re-measures and records.
7. **Reference machine: Apple M2, 16 GB.** Memory is the limit. Run
   one cargo command at a time. `tools/gate.sh` sets
   `CARGO_BUILD_JOBS` itself.

## Toolchain and dependencies

- **subscript** is consumed as a pinned dependency (git URL + rev),
  never a sibling path. Dev-tier execution goes through subscript's
  Rust API. Ship tier goes through the emitted C and the platform C
  compiler.
- **webgpu-headers** is a git submodule pinned by commit. The FFI
  declarations the facade uses are generated from it by a tool, not
  by a build script, and committed.
- **The backend library** resolves via `SUBSCRIPT_TYPEGPU_BACKEND_LIB` (a path to
  the shared library) → loud failure. Machine-local settings live only
  in gitignored files.
- **`rust-toolchain.toml`** pins the channel. `cargo fmt --check` and
  `cargo clippy --workspace -- -D warnings` are standing gates.

### When subscript is the blocker

subscript is first-party and changeable. When a missing or awkward
subscript capability forces a costly or ugly shape here, the default
is **not** to design around it. Claude escalates a subscript change
request to the owner — what is missing, the evidence, the consequence
for this project, and the proposed language-side fix — and the owner
decides which side moves. Contorting the library to absorb a language
gap is a decision the owner makes, never a silent default.

## Language

- **All repository documentation, specs, comments, and identifiers:
  English.**
- Conversation with the user (chat responses): Japanese.

## Writing style (applies to everything: specs, tracking, comments,
## commit messages, and chat reports)

**State fact, evidence, and consequence. Nothing else.**

Forbidden: rhetorical or dramatic framing; narrating a mistake as a
story; restating a lesson in more than one document; self-referential
commentary; emphatic repetition.

Required: a lesson worth keeping is written **once**, as a one-line
rule. Reports state what changed, what was measured, what is next. A
correction states what was wrong, the evidence, and the corrected
claim.

**Rule: a claim about another system's behaviour requires running that
system.** Claims taken from documentation alone are marked *(docs)*
where they appear. The same standard applies to claims about this
repository: read the code before you describe it.

### Form: Simplified Technical English (ASD-STE100)

The rules above govern *what* to say. These govern *how*. They apply
to every English artifact: specs, tracking, docs, comments, commit
messages, and handoffs to other agents.

Classify the passage first, because the limits differ:

- **Procedural** (instructions): imperative, **20 words** per
  sentence, one instruction per sentence.
- **Descriptive** (explanations): simple tenses, **25 words** per
  sentence, one topic per paragraph, six sentences maximum.

Then:

1. **One word, one meaning, one part of speech, whole document.**
   Pick one verb for the check/verify/confirm concept and one noun
   for config/settings. Use no other word for that concept.
2. **Simple tenses only.** No present perfect ("has been updated" →
   "we updated"), no "is to be installed".
3. **No `-ing` verb forms.** They are legal only inside a technical
   noun ("logging"), never as a verb or a trailing clause.
4. **Active voice.** Passive is legal in descriptive text only when
   the agent is unknown.
5. **Modals: `can`, `will`, `must` only.** Never should, would, may,
   might, could. A requirement is "must"; a suggestion is stated as
   fact or deleted. Agents read "should" as optional.
6. **Condition before command**, divided by a comma: "If the build
   fails, read the log." Never the reverse.
7. **No semicolons.** Write two sentences.
8. **Keep articles and "that".** STE is short, not terse.
9. **No phrasal verbs** ("set up" → "install" or "configure").
10. **No Latin abbreviations.** "e.g." → "for example"; "i.e." →
    "that is"; delete "etc." and name the items.

**Untouchable**, even when they break these rules: code, identifiers,
commands, file paths, quoted diagnostics, and product names. Each
counts as one word against the sentence limit.

**Warnings put the command first, the risk second.** "CAUTION: Do not
run this against the device suite. It overwrites the committed
golden."

**Self-check before delivering.** Count the words in the three longest
sentences. Search for `has been`, `have been`, `should`, semicolons,
contractions, and `-ing` after a comma. Check that every `if` and
`when` starts its sentence.

**Out of scope:** this is for technical facts and instructions. It
does not apply to chat with the user (which is Japanese), where the
*spirit* still holds — short sentences, one meaning per word,
condition before command, no hedging.

## Artifacts stand alone

An artifact is anything a reader uses without the conversation that
produced it: code, comments, generated files and their headers,
`lib/*.ts`, `tools/`, `programs/` headers, README and docs. An
artifact reads as an intentional deliverable for its audience. It
does not expose the prompt, the handoff, the review round, the
constraint, the rejected alternative, or the production process.

Classify every sentence before it enters an artifact:

1. **Audience content** — what the reader needs. It enters.
2. **Production guidance** — a constraint or a decision from the
   conversation. It shapes the artifact's design and never appears
   as prose. "The owner forbids X" becomes an artifact that does Y.
3. **Residue** — a trace of how the artifact came to be. It is
   deleted.

Residue phrases, each a defect in an artifact: "as requested", "the
owner wanted", "this was changed to", "unlike the previous version",
"to satisfy the review", "the handoff said", "rewritten after", a
date that records when a sentence was edited, a phase or slice name
of this or any other project, a rule id that does not resolve in
`specs/blocks/`, and a disclaimer about a tool the artifact does not
use.

Specs and tracking are not artifacts under this rule. They are the
decision record. A dated owner decision, a correction with its
evidence, a review finding, and a rule id are audience content there.
The line is the directory: `specs/` records provenance, everything
else hides it.

Review: a reader with no context finds nothing in an artifact that
explains why it was written this way. The phase review checks this
for every artifact the phase touched.

## Rigor is proportional to the cost of being wrong

The gates in this repository exist because a wrong facade signature
corrupts memory, a wrong layout table writes the wrong bytes, and a
wrong ownership rule frees a live handle. Those justify byte-identical
goldens, demonstrated reds, and adversarial review.

**Do not apply that standard to artifacts whose failure is cheap.**
A tutorial sentence that is imprecise costs a reader one look at the
code. A guide is not a gate.

Practical limits:

- **Two rounds.** A prose artifact gets one verification pass and one
  fix pass. If a third pass is needed, the artifact is done and the
  remaining defects are recorded, not fixed. Announce the stop.
- **Gate what is mechanical, accept what is not.** Quoted code and
  produced outcomes are gated because a machine can check them
  cheaply and forever. Causal prose is reviewed once and then trusted.
- **Exit criteria must match the artifact.** When a criterion cannot
  be met by improving the artifact — only by product work — the
  criterion is wrong, not the artifact.

## Core principles

1. **Every public API has a direct test**, shipped in the same commit —
   facade functions, generator entry points, runtime-library classes,
   harness runners alike.
2. **Program-suite-first.** `programs/` entries with committed goldens
   are the library's executable definition. A generator, facade, or
   API-layer decision without a program exercising it is not decided.
3. **Differential testing.** Every suite program runs under both
   subscript tiers (dev JIT and ship C AOT) with byte-identical output
   against the committed golden, headless, on every test run.
4. **WGSL is a golden.** Every kernel's emitted WGSL is committed as a
   `.wgsl` golden beside its program and validated by `naga` in the
   generator's tests. Generated support modules are not committed.
   The differential run compiles them in memory.
5. **Headless-first** (invariant 5).
6. **No panics in library code**; `Result` and `?`. The facade's
   `extern "C"` boundary is the single exception surface and never
   unwinds across it. Failures reach scripts as status codes, never
   silently.
7. **Generated code is never hand-edited.** Committed generated
   artifacts carry byte-identical regeneration gates. Fix the
   generator.
8. **Exit criteria before implementation.** Every phase's spec names,
   in advance, the measurement that would kill or pass it.
9. **Demonstrated red.** A rejection rule counts only after a recorded
   red run against a fixture that violates it.

## Workflow per area

1. Write/extend `specs/blocks/<area>.md` — contract first.
2. Add program-suite entries (accept + reject where applicable) — Red.
3. Implement — Green.
4. Run the tier-differential suite.
5. Log evidence in `specs/tracking/<topic>.md`.

**Every phase ends with a mandatory Phase Review ("Clean Review Then
Fix"):** a fresh no-context subagent reviews the phase's cumulative
diff and emits `CRITICAL`/`MAJOR`/`MINOR` findings, including
residue under "Artifacts stand alone". Findings are fixed in severity
order. A phase cannot be COMPLETE with any open CRITICAL/MAJOR.

**A gate result is evidence only when the citation names which gate
ran and quotes its wall time and test count.**

## Privacy / repo hygiene

- No credentials, signing material, or device-specific secrets
  committed.
- `.gitignore`: `target/`, `node_modules/`, `.claude/`, `.cargo/`,
  local test transcripts.

### No local or sibling paths in committed files

**Nothing committed to this repository may reference a path outside
it.** Applies to every tracked file — docs, specs, comments, code,
tests, CI config, and commit messages.

Forbidden: absolute paths into any developer's filesystem; relative
paths escaping the repository root (including sibling checkouts);
machine- or user-specific names.

Required instead: cite external projects by upstream URL and by their
own repo-relative paths. Pin external sources as git submodules or
fetched artifacts resolved by a tool or an env var with a documented
default. When a claim was verified against a local checkout, record
the finding and the upstream citation, never the local path.

## Tooling — sandbox

Avoid `dangerouslyDisableSandbox: true` whenever possible. Network ops
(`git push`/`pull`, `cargo` fetches, submodule fetches) are invoked by
the user via the `!` prompt, not run by Claude with the sandbox
disabled. The one exception is `tools/live.sh`: Claude runs it with
the sandbox disabled, because a sandboxed shell sees zero adapters
and that reads exactly like a backend defect.
