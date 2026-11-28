// program: b31-call-shapes
// purpose: copy nested batches and evaluate default arguments across two binding groups
// exercises: K14, K34, PI6, RN9, CL2
// questions: none

import {
  ComputePipelineSpec, MutStorage, RenderPipelineSpec, Storage,
  computePipeline, computePipeline2, renderPipelineL, simulateCompute, simulateCompute2,
} from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import { batches_HOST_RUNNABLE, pixels_HOST_RUNNABLE } from "./b31-call-shapes.typegpu";

@ValueType
class Item {
  constructor(public value: u32) {}
}
@ValueType
class Batch {
  constructor(public slots: FixedArray<Item, 2>, public scale: f32) {}
}
class Batches {
  constructor(public data: MutStorage<Batch>) {}
}
function copyBatch(res: Batches): void {
  res.data[1] = res.data[0];
}
const batches: ComputePipelineSpec = computePipeline(copyBatch, { name: "batches", workgroupSize: [1, 1, 1] });

class A {
  constructor(public pixels: Storage<u32>) {}
}
class B {
  constructor(public pixels: MutStorage<u32>) {}
}
function pair(value: u32, bias: u32 = 7): u32 { return value + bias; }
function copyPixels(a: A, b: B): void {
  b.pixels[0] = pair(a.pixels[0]);
  b.pixels[1] = pair(a.pixels[1], 0);
}
const pixels: ComputePipelineSpec = computePipeline2<A, B>(copyPixels, { name: "pixels", workgroupSize: [1, 1, 1] });

@ValueType
class Vertex {
  constructor(public xy: Vec2f) {}
}
@ValueType
class Varyings {
  constructor(public position: Vec4f) {}
}
class Palette {
  constructor(public data: Storage<Batch>) {}
}
function vert(res: Palette, vertex: Vertex): Varyings {
  return new Varyings(new Vec4f(vertex.xy.x, vertex.xy.y, 0.0, 1.0));
}
function frag(res: Palette, input: Varyings): Vec4f {
  const scale = res.data[0].scale;
  return new Vec4f(scale, 0.0, 0.0, 1.0);
}
const color: RenderPipelineSpec = renderPipelineL(vert, frag, { format: "rgba8unorm" });

export function main(): void {
  const slots: FixedArray<Item, 2> = [new Item(11), new Item(22)];
  const empty: FixedArray<Item, 2> = [new Item(0), new Item(0)];
  const values = new Batches(new MutStorage<Batch>([new Batch(slots, 0.5), new Batch(empty, 0.0)]));
  simulateCompute(copyBatch, values, batches, [1, 1, 1], batches_HOST_RUNNABLE);
  const result = values.data[1];
  print(`host:batch=${result.slots[0].value},${result.slots[1].value},${result.scale}`);
  const a = new A(new Storage<u32>([1, 2]));
  const b = new B(new MutStorage<u32>([0, 0]));
  simulateCompute2<A, B>(copyPixels, a, b, pixels, [1, 1, 1], pixels_HOST_RUNNABLE);
  print(`host:pixels=${b.pixels[0]},${b.pixels[1]}`);
  const palette = new Palette(new Storage<Batch>([result]));
  const sample = frag(palette, vert(palette, new Vertex(new Vec2f(0.0, 0.0))));
  print(`fragment=${sample.x}`);
}
