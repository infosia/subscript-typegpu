# Retrospective review

Run 2026-08-23 at `82175df` by a fresh reviewer on the top-level
model, after every phase closed. Scope: cross-phase contradictions,
drift, vacuous gates, the invariant audit over the cumulative tree,
the seams between layers, and the decision record.

Findings: CRITICAL 0, MAJOR 2, MINOR 12. MAJOR 1: render kernels
received no module globals, so a vertex or fragment kernel that
read a module constant got a generator-owned K15 diagnostic (RN9
now names the constants, the code follows). MAJOR 2: PI8 Rev 2 and
PI9 Rev 1 were cited by tracking entries but absent from
`pipeline.md` (now written; plan §10 C3 records the rule). MINOR:
two per-program name lists in Rust, `coverage.md` overwritten
instead of compared, schema intent decided by name case, host
texture bodies returning zero for a non-zero level, the T6 scratch
root seeded with outputs, the pairing check matching a global by
name, three backend-value sets, stale plan text, the rule-id
scanner's `SC1a` gap. The review also recorded D4's tail-packing
limit (plan §10 C2).

Verified correct by the review: every design invariant and
build-time rule on the cumulative tree, the dependency closure per
crate, the submodule pins, the loud failure paths of the loader,
every rejection fixture with one diagnostic and an owner, every
`.wgsl` golden byte-identical and naga-valid, the K14 Rev 4
reservation, the K22 fixtures, the CL6 tests, the invocation order
and builtin formulas of `simulateCompute`, the coverage thunks' ABI
by compilation, the R34 byte path gated through the stride check,
the C proof for every `b` program, hygiene, no residue, no local
path.

The code findings landed at `c0ebfed`: render kernels reach module
constants and private variables (`b06`'s fragment alpha is a module
constant), the tests carry no per-program name lists, `coverage.md`
is compared, schema intent comes from the declarations, the host
texture bodies trap for a non-zero level, regeneration from absence
is gated, the pairing check reads the constant's origin, the
rule-id scanner covers `docs/` and `README.md`, one export-name
source.
