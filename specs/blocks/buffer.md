# Block: buffer (BF-rules)

P1 slice 2 contract. Rev 0, 2026-08-22. Rev 1 (BF9–BF11), 2026-08-23.
Rev 2 (BF9 Rev 1, the R36 pin `ac9436f`), 2026-08-23. Rev 3 (BF2
Rev 1 four-byte alignment), 2026-08-23.
Plan §3 D4 and §4 govern
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
- **BF2 — Bytes cross, values stay typed.** Rev 1, 2026-08-23. The
  host converts with R34 at the call site, because the array length
  is a literal there:
  `buffer.write(queue, 0, Context.bytesOf<FixedArray<Particle, 64>>(particles))`.
  `write(queue, elementIndex: u32, bytes: u8[])` checks that
  `bytes.length` is a multiple of the element size, that
  `elementIndex + bytes.length / elementSize <= count`, and that the
  byte offset `elementIndex * elementSize` and `bytes.length` are
  multiples of 4, and traps with the numbers otherwise. The last
  check is WebGPU's `writeBuffer` rule: a backend rejects a 6-byte
  write and records no error outside an error scope, so the trap is
  the only early signal (measured 2026-08-23 on yawgpu and Dawn with
  a 3-index `u16` buffer: the readback stayed zero). `writeOne` and
  `patch` apply the same three checks. A `u16` index buffer is
  therefore written as a `FixedArray<u16, 4>` or longer even
  multiple. `read`, `readOne`, and `copyTo` apply the same
  multiple-of-4 rule to their byte offset and byte length (WebGPU's
  `copyBufferToBuffer` rule) and trap with `BF9` or `BF8`.
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

## Reading through a staging buffer (P8)

- **BF9 — `read` owns the staging buffer.** Rev 1, 2026-08-23.
  `Buffer<T>` gains `async read(device: GPUDevice, elementIndex:
  u32, elementCount: u32): Promise<u8[]>` (subscript R36 at
  `ac9436f` admits an `async` method on a generic class). The
  method creates a staging buffer of `elementCount * elementSize`
  bytes with `MAP_READ + COPY_DST`, records `copyTo` into a fresh
  command encoder, submits on `device.queue()`, awaits
  `mapAsync(READ)`, copies the bytes out with `readBuffer`, unmaps,
  disposes the staging buffer, and returns the bytes. Before it
  creates the staging buffer, it checks `elementIndex + elementCount
  <= count` and traps with `BF9`, `Buffer.read`, and the three
  numbers. When
  `mapAsync` returns `false`, the method disposes the staging buffer
  and traps with `BF9`, the method, and the three numbers, because a
  failed map is a device-level failure and a nullable array is not
  in the language. `readOne` gains the same form: `async
  readOne(device, elementIndex): Promise<u8[]>`. The caller decodes
  with `Context.fromBytes<T>`. BF3's explicit path stays for a
  caller that owns the readback buffer.
- **BF10 — The usage is known.** `createBuffer<T>` stores the usage
  it receives. `read` traps with `BF10`, the method, and the usage
  value when `COPY_SRC` is absent, before it creates the staging
  buffer. `write`, `writeOne`, and `patch` trap the same way when
  `COPY_DST` is absent.
- **BF11 — `read` is a gate program and the live path.** `b12-readback`
  writes a `FixedArray<Particle, 4>`, reads it back through `read`
  on Noop, decodes with `fromBytes`, and prints `roundtrip:match`
  after a host comparison, plus `readOne` on element 2 by value.
  From P8 on, a new `x` program reads its result through `read`.
  The `x` programs that exist keep their explicit path.

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
