# The copy-bound @CStruct mutation request (R38)

2026-09-01. Escalated to the subscript owner as R38, written to
subscript's `HANDOFF-R38.md`
(https://github.com/infosia/subscript).

The measured semantics, from a scratch dev-tier probe at pin
`2f9ed28`: a `@CStruct` binds by copy into a function parameter
and into a local bound from another binding. An assignment to a
field of such a copy succeeds silently, and the stored value
never changes. Only a write through the stored field chain
mutates the held value.

The consequence this project measured: the `radiance-cascades`
drag helper edited a parameter copy, compiled clean, and did
nothing. The owner's visual run found it. The record is
`specs/tracking/p16-texture-arrays.md`, the fix is `01fa15e`.

The request: a checker diagnostic on an assignment whose target
chain roots in a copy-bound `@CStruct`. Reads stay legal. The
owner decides between the diagnostic and a larger move to
reference semantics.

## R38 landed (2026-09-01)

subscript closed the request as W004 (compiler contract §81, the
diagnostic at `f993d60`, the review record at `22619be`): a write
through a `@CStruct` value copy that nothing reads is a named
diagnostic, with shadowing, for-of bindings, index roots, and
synthetic locals excluded by review. This tree compiles under the
new pin with no W004 finding, which matches the scan: the one
copy-write site left at `01fa15e`. The request is closed.
