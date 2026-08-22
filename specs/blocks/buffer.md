# Block: buffer (BF-rules)

P1 slice 2 contract. Rev 0, 2026-08-22. Plan §3 D4 and §4 govern
this block. The byte path is subscript R34 at `bb9dadc`
(stdlib.md §18 there): `Context.bytesOf<T>`, `bytesInto<T>`,
`fromBytes<T>`, with every padding byte zero.

## The class

- **BF1 — `Buffer<T>` is a library class.** `lib/typegpu.ts` exports
  `createBuffer<T>(device, elementSize: u32, count: u32, usage: u64,
  label: string): Buffer<T>`, where `elementSize` is the schema's
  `X_STRIDE` constant read by name. The class holds the `GPUBuffer`,
  the element size, and the count. `dispose()` and
  `[Symbol.dispose]()` release the `GPUBuffer`. `handle(): GPUBuffer`
  gives the underlying handle for bind groups.
- **BF2 — Bytes cross, values stay typed.** The host converts with
  R34 at the call site, because the array length is a literal there:
  `buffer.write(queue, 0, Context.bytesOf<FixedArray<Particle, 64>>(particles))`.
  `write(queue, elementIndex: u32, bytes: u8[])` checks that
  `bytes.length` is a multiple of the element size and that
  `elementIndex + bytes.length / elementSize <= count`, and traps
  with all three numbers otherwise. `writeOne(queue, elementIndex,
  bytes)` requires `bytes.length == elementSize`.
- **BF3 — Reading is explicit.** `readBuffer<T>(readback: Buffer<T>,
  elementIndex: u32, elementCount: u32): u8[]` (a free function)
  copies from a
  mapped `MAP_READ` buffer the caller mapped (`mapAsync` stays in
  the program, because it awaits). The caller decodes with
  `Context.fromBytes<FixedArray<Particle, 64>>(bytes, 0)`.
  `Buffer<T>` offers `copyTo(encoder, target: Buffer<T>,
  elementIndex, elementCount)` sized in elements.
- **BF4 — Sizes are elements.** Every `Buffer<T>` method counts
  elements. Bytes appear only in the `u8[]` payloads. A size in bytes
  never crosses the method surface.
- **BF5 — The element size is the stride.** `elementSize` equals
  `X_STRIDE` (LY5), so `count` elements occupy `count * X_STRIDE`
  bytes and element `i` starts at `i * X_STRIDE`. A `Buffer<Vec3f>`
  uses `Vec3f_STRIDE` (16), never `Vec3f_SIZE` (12).

## Programs

- **BF6 — The round trip is a gate program.** `b05-buffer` writes a
  `FixedArray<Particle, 4>` through `bytesOf`, copies to a readback
  buffer on Noop (which executes copies), maps, reads, decodes with
  `fromBytes`, and prints the decoded components by value after a
  host comparison: `roundtrip:match`. It prints `bytesOf(...).length`
  by value, which is the stride times the count on both tiers (LY16).
- **BF7 — Live programs use the same path.** From this slice on an
  `x` program encodes with `bytesOf` and decodes with `fromBytes`.
  The `Math.f32ToBits` encoders of `x01` through `x03` are replaced.

## Rejections

- **BF8 — The traps are named.** A write past the end, a byte length
  that is not a multiple of the element size, and a read past the
  end each trap with the rule id, the method, and the three numbers
  (testing.md T7 through a `t`-style fixture run on the dev tier).
