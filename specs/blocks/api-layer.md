# Block: API layer (J-rules)

P0 contract. Rev 0, 2026-08-22. These rules come from the proof of
concept at <https://github.com/infosia/subscript-gpu>, restated as
first-party rules. The API generator, the `[api]` policy section, and
`lib/webgpu.ts` cite them by id.

## 1. Lifetime

- **J7 — Disposal replaces collection.** The API layer has no garbage
  collection. Every owned wrapper declares `dispose()` and
  `[Symbol.dispose]()`. An IDL `destroy()` method stays a separate
  method where the IDL has one. The facade C surface keeps its
  release names.

## 2. Coverage

- **J9 — The deviation catalogue.** Every IDL member of the covered
  subset is exactly one of three things: generated, deviation-rowed
  with a reason and the deviating shape, or excluded with a reason.
  The generator enforces the trichotomy. An unpoliced IDL member is a
  generation error. The check runs two-way per F18.

## 3. The host seam

- **J13 — The host handoff seam.** A generated API class takes its
  facade handles through a public constructor. The class holds those
  handles in public fields. A host wraps engine-owned handles through
  this seam. The seam is policy-required and fail-loud. (The proof
  of concept numbered this rule H15. H2, H3, and H5 are adapter rules
  in `facade-generator.md`, so the number moved.)

## 4. What is not a rule here

The proof of concept ran the API layer as slices named E1 through E8.
Those names are schedule labels. E1, E4, and E6 mean the encoder-area
rules of `facade-generator.md` in this repository. A test file that
carried a slice name carries an area name instead:
`api_encoders_red`, `api_device_events_red`,
`api_capability_queries_red`, `api_device_descriptor_red`.

Q32, Q33, R11, R16, R19, and every other Q- or R-id name a subscript
language feature or a subscript change request. They resolve in the
subscript repository.
