// program: x04-live-control-flow
// purpose: prove loop-condition and nested conditional lowerings on a live backend
// exercises: CL1, CL3, CL4, K9, K14, T15
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  Storage,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  readBuffer,
  simulateComputeThreads,
} from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode } from "./webgpu";
import {
  Item_STRIDE,
  controlFlow_ENTRY,
  controlFlow_HOST_RUNNABLE,
  controlFlow_LAYOUT0,
  controlFlow_WGSL,
  controlFlow_WORKGROUP_X,
  controlFlow_WORKGROUP_Y,
  controlFlow_WORKGROUP_Z,
} from "./x04-live-control-flow.typegpu";

@CStruct
class Item {
  value: f32;
  constructor(value: f32) { this.value = value; }
}

class ControlLayout {
  input!: Storage<Item>;
  output!: MutStorage<Item>;
}

function controlFlowKernel(res: ControlLayout, ctx: ComputeInvocation): void {
  let index: u32 = 0;
  let total: f32 = 0.0;
  while (index < (4 as u32) ? true : false) {
    const source: f32 = res.input[index].value;
    const chosen: f32 = source > 0.0 ? (source > 2.0 ? source : 2.0) : 1.0;
    total += chosen;
    index += 1;
  }
  res.output[0] = new Item(total);
}

export const controlFlow: ComputePipelineSpec = computePipeline<ControlLayout>(controlFlowKernel, {
  name: "controlFlow",
  workgroupSize: [1, 1, 1],
});

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const inputValues: FixedArray<Item, 4> = [new Item(-1.0), new Item(1.0), new Item(3.0), new Item(4.0)];
    const hostLayout = new ControlLayout();
    hostLayout.input = new Storage<Item>([
      new Item(-1.0), new Item(1.0), new Item(3.0), new Item(4.0),
    ]);
    hostLayout.output = new MutStorage<Item>([new Item(0.0)]);
    simulateComputeThreads<ControlLayout>(
      controlFlowKernel,
      hostLayout,
      controlFlow,
      1,
      1,
      1,
      controlFlow_HOST_RUNNABLE,
    );
    using input: Buffer<Item> = createBuffer<Item>(device, Item_STRIDE, 4, GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST, "x04-input");
    using output: Buffer<Item> = createBuffer<Item>(device, Item_STRIDE, 1, GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC, "x04-output");
    using readback: Buffer<Item> = createBuffer<Item>(device, Item_STRIDE, 1, GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST, "x04-readback");
    const queue = device.queue;
    input.write(queue, 0, Context.bytesOf<FixedArray<Item, 4>>(inputValues));
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(device, controlFlow_WGSL, controlFlow_ENTRY, [controlFlow_LAYOUT0], [controlFlow_WORKGROUP_X, controlFlow_WORKGROUP_Y, controlFlow_WORKGROUP_Z]);
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, controlFlow_LAYOUT0, [bufferResource(input.handle()), bufferResource(output.handle())]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], 1, 1, 1);
    output.copyTo(encoder, readback, 0, 1);
    using command = encoder.finishDefault();
    queue.submit([command]);
    if (!await readback.handle().mapAsync(GPUMapMode.READ, 0, Item_STRIDE as u64)) { print("FAIL map"); return; }
    const result: FixedArray<Item, 1> = Context.fromBytes<FixedArray<Item, 1>>(readBuffer<Item>(readback, 0, 1), 0);
    const expected: f32 = hostLayout.output[0].value;
    if (result[0].value !== expected) { print(`FAIL expected=${expected} got=${result[0].value}`); return; }
    readback.handle().unmap();
  }
  gpu.dispose();
  print("PASS");
}
