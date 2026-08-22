# Block: facade generator (F-, S-, and area rules)

P0 contract. Rev 0, 2026-08-22. These rules come from the proof of
concept at <https://github.com/infosia/subscript-gpu>, restated as
first-party rules. The generator, the policy file, and the generated
outputs cite them by id. The ids stay stable for that reason. The
loader, the pump, and the export shape are L-rules in `facade.md`.
The design decisions D1 through D11 are in the plan and never share a
letter with this block, so the pipelines area uses `PL`.

## 1. Names and handles

- **F2 — Names.** Every facade symbol carries the
  `subscript_typegpu_` prefix. The rest of a name matches the
  webgpu.h name. Enum members keep the webgpu.h numeric values. A
  rename needs one policy row with a reason.
- **F4 — One handle per object.** Each webgpu.h object in the subset
  maps to one opaque handle. Creation owns the handle. Each object
  exposes one release export. The facade never exposes AddRef.

## 2. The future protocol

- **F6 — The future triple.** Each async operation returns a future
  id. The script pumps process-events. The script polls an `i32`
  completion status. The script then takes the typed result. The
  facade registers AllowProcessEvents callbacks only.
- **F7 — Callback discipline.** A callback records the outcome into
  the slot table and returns. A callback never calls a webgpu.h
  function. A callback never unwinds. A callback copies every string
  view before it returns.
- **F8 — Slot lifetime.** A slot frees on the first successful take,
  on an explicit future drop, or at instance release. A drop of a
  pending future only marks the slot doomed. A poll of an unknown id
  returns a distinct negative status.

## 3. Strings and records

- **F10 — Strings into C.** A script passes a string as a string-view
  parameter or as a string-view struct field. The call site expands
  the view. A C-filled view field reads back as an owned string.
- **F11 — Records out of C.** Data from C reaches a script as a
  poll-and-copy record. The script never receives a borrowed pointer
  to backend memory.
- **F11 Rev 1 — The fill shape.** A record export takes an
  out-pointer. The export returns true and fills the record when data
  is present. The string bytes behind a filled view stay valid until
  the next fill call on the same parent object.

## 4. Descriptors and out-fills

- **F12 — Chain-free descriptors.** A descriptor crosses as a struct
  pointer with no chain field. A needed extension becomes one
  policy-listed inline field or one exclusion. A count-first pair maps
  to an array. Dictionary defaults belong to the API layer.
- **F13 — Out-fills carry a status.** An out-struct crosses through an
  out-pointer export with an `int32_t` status return. The facade frees
  the backend members internally after the copy. A field value of 0
  means unspecified for the three u64 limits.
- **F20 — Bulk data.** Bulk memory crosses only as a count-first pair.
  The spelling is `size_t <name>Count` followed by the element
  pointer. A fill callee never grows the buffer. The writes are
  visible after the call.

## 5. Device events

- **F14 — The facade owns the device callbacks.** The facade installs
  the uncaptured-error callback and the device-lost callback at device
  creation. A script drains both through F11 record accessors. The
  lost-future entry point stays unused.

## 6. Numerics

- **F15 — No sentinel crosses.** No value at or above 2^53 appears in
  the header. A whole-resource operation gets its own export. A field
  that carried a whole-size sentinel uses a zero rule instead. A
  sentinel below 2^32 crosses verbatim.
- **F16 — Flags and enums.** A flag typedef stays `uint64_t` with bare
  constants. A plain enum stays `i32`. A literal-union alias never
  crosses this boundary. A `bool` is permitted where the semantics are
  two-valued.

## 7. The policy file

- **F18 — The policy file is the record.** `policy.toml` names subset
  membership per area, every exclusion, every rename, every
  extension-field exposure, and every sentinel variant. Each row
  carries a reason. An unknown, dead, duplicate, unpoliced, or
  invalid row fails generation.

## 7. The policy file (continued)

- **F22 — Export exclusion.** Rev 0, 2026-08-23. A `[[export_exclude]]`
  row names one facade export and a reason. The generator then
  emits no Rust export, no header declaration, no mirror
  declaration, no symbol-table row, and resolves no backend symbol
  the excluded export alone needed. The API layer must not reference
  the name (the API join fails otherwise). The row is the only way
  an export leaves, and the coverage list (EG4) is the only reason:
  an export no program reaches and no API-layer member calls. The
  facade's export count at P6: 157, from 163 (six descriptor-less or
  unused variants).

## 7a. The mirror's banner

- **F21 — The mirror is bindgen's output, byte for byte.** The
  banner of `lib/subscript-typegpu.generated.d.ts` is written by
  `subscript bind`. Its citations (`compiler.md §12.2`,
  `collisions.md §2`, "core principle 6") resolve in the subscript
  repository, not here. This is a recorded deviation from "Artifacts
  stand alone": the facade never edits a generated file. A change to
  the banner is a subscript change request.

## 8. Typed data

- **S1 — A typed variant needs a reason.** A typed variant exists only
  for an element type that a script cannot encode by hand. A script
  encodes any integer width with shifts and masks. Only `f32` needs
  the facade. The generator refuses a `u32`, `i32`, or `u16` variant.
- **S3 — Two typed entry points.** The facade gains one typed
  buffer-write export and one typed mapped-read export. Neither has a
  `webgpu.yml` counterpart. Each body calls the same webgpu.h function
  as its byte form. Each needs one policy row with a reason.

## 9. Buffers and queue

- **A3 — Map-async variants.** `map_async` uses the F6 protocol. An
  explicit whole-buffer variant exists beside the offset-and-size
  variant per F15. Map-mode flags follow F16.
- **A5 — Buffer state getters.** The size getter and the usage getter
  return `u64`. The map-state getter returns the pinned enum value
  per F16.

## 10. Textures

- **B1 — Pair spelling.** A count field takes the pointer-field name
  plus `Count` exactly. A pair element is a real C enum, a language
  scalar, or a registered struct. An embedded aggregate stays
  recursively plain.
- **B2 — Texture descriptors.** The texture descriptor, the view
  descriptor, and the sampler descriptor cross chain-free per F12. A
  u32 undefined constant crosses verbatim per F15.
- **B3 — Texture upload.** The texture-write export takes the
  destination pointer, the layout pointer, and the extent pointer.
  The F20 byte pair comes last.
- **B4 — Texture getters.** An enum getter returns the pinned enum
  value per F16. A u32 getter returns `u32`. The usage getter returns
  `u64`.

## 11. Bind groups

- **C1 — Nullable fields and handle pairs.** A nullable handle field
  carries the `_Nullable` marker. A handle-element pair is input-only.
  The header preamble carries the nullability-completeness pragma.
- **C2 — No entry classification.** A bind-group entry keeps the
  buffer field, the sampler field, and the texture-view field all
  nullable. The facade passes null through. The backend validates.
- **C3 — The size-zero rule.** A bind-group entry size of 0 means the
  whole binding. A binding size of 0 is never valid, so the mapping is
  unambiguous. The offset passes verbatim.
- **C4 — Visibility flags.** Shader-stage flag constants cross as bare
  `u64` values per F16. A combination passes through.
- **C5 — Bind-group pair names.** A bind-group pair rename follows the
  exact `<ptr-name>Count` rule. Each rename needs one policy row.

## 12. Shaders and pipelines

- **PL2 — Shader source.** The shader-module descriptor carries a
  label and the code. The generated conversion builds the WGSL source
  chain internally. The chain struct never appears in the header. The
  text crosses per F10.
- **PL3 — Pipeline descriptors.** The compute descriptor carries a
  label, a nullable layout, and the compute state. The render
  descriptor keeps vertex, primitive, and multisample state by value.
  Depth-stencil and fragment cross as nullable struct pointers. A null
  layout means auto-layout.
- **PL4 — Async pipeline creation.** Each async creation uses the F6
  triple. The statuses come from the yml. The negative values stay
  distinct per F8. The messages are copied per F7. The sync variant
  and the async variant are both in scope.
- **PL6 — Pipeline pair names.** A pipeline pair rename follows the
  exact `<ptr-name>Count` rule. Each rename needs one policy row.

## 13. Encoders and passes

- **E1 — The proven encoder shapes.** Three shapes are pinned: a
  parameter-position pair with handle elements, a by-value colour
  aggregate inside an attachment struct, and the full render-pass
  descriptor composite with nested attachments.
- **E4 — Pass lifetime.** An encoder object and a pass object are
  create-owns plus release like every other handle per F4. The end
  call and the finish call pass through. The facade holds no state
  machine. The backend validates.
- **E6 — Encoder pair names.** An encoder pair rename follows the
  exact `<ptr-name>Count` rule. Each rename needs one policy row.

E2 does not exist here. The proof of concept's E2 named the
blend-constant call, and its 17 policy citations described F12
descriptors, F16 enums, and the E1 composite instead. Those rows cite
F12, F16, or E1.

## 14. Device events

- **G1 — The record-fill shape.** A device-event record crosses as an
  out-parameter fill per F11 Rev 1. A returned record pointer is
  rejected at bind time. The pop-scope take uses the same fill shape.
- **G2 — Error scopes.** A push call takes a filter. A pop call
  returns a future id. A take export fills an error record and returns
  true on success. The delivery status and the captured error stay
  separate axes. A null out-pointer returns false and consumes
  nothing.
- **G3 — Uncaptured error drain.** The facade installs the
  immediate-style uncaptured callback at device creation. The callback
  is enqueue-only. The drain export returns records in FIFO order.
  The string bytes stay valid until the next drain on the same device.
- **G4 — Device lost.** An AllowProcessEvents creation callback
  records the loss into a per-device slot. The lost-info export fills
  a reason enum and a message after the script pumps.

## 15. Adapter and limits

- **H2 — Limits.** A limits fill returns an `i32` status. The output
  values pass verbatim, u64 fields included. On input a field value of
  0 means unspecified for the three u64 limits. A u32 limit uses the
  undefined constant verbatim.
- **H3 — Adapter info.** The adapter-info export fills a record. The
  facade copies all four strings. The facade calls the backend
  free-members function internally before it returns. The string
  lifetime follows F11 Rev 1.
- **H5 — Limits goldens.** Limits values and info values are
  backend-reported. A golden prints structural facts only. A golden
  never prints a raw backend value.
