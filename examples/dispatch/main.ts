// example: dispatch
// Counts guarded compute invocations for thread counts that no workgroup size divides.
// This port keeps the four size cases and drops the multiple-dispatch, bind-group, and slot cases.
// Ported from TypeGPU's dispatch example (https://github.com/software-mansion/TypeGPU).

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
} from "./typegpu";
import {
  AtomicU32,
} from "./typegpu-types";
import {
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  gpu,
} from "./webgpu";
import {
  Counter_SIZE,
  count1d_ENTRY,
  count1d_LAYOUT0,
  count1d_WGSL,
  count1d_WORKGROUP_X,
  count1d_WORKGROUP_Y,
  count1d_WORKGROUP_Z,
  count2d_ENTRY,
  count2d_LAYOUT0,
  count2d_WGSL,
  count2d_WORKGROUP_X,
  count2d_WORKGROUP_Y,
  count2d_WORKGROUP_Z,
  count3d_ENTRY,
  count3d_LAYOUT0,
  count3d_WGSL,
  count3d_WORKGROUP_X,
  count3d_WORKGROUP_Y,
  count3d_WORKGROUP_Z,
} from "./main.typegpu";

// One atomic counter. Every invocation adds to the same address, so the final value is the
// number of invocations that ran.
@CStruct
class Counter {
  value: AtomicU32;

  constructor(value: AtomicU32) {
    this.value = value;
  }
}

// TypeGPU's `createGuardedComputePipeline` allocates the mutable and the layout on
// its own. This port declares the binding, and the generator emits `count1d_LAYOUT0`.
class CounterLayout {
  counter!: MutStorage<Counter>;
}

// The kernel body of all three pipelines. The body reads no global id, so the generated
// guard alone decides how many invocations reach the counter.
function countKernel(res: CounterLayout, ctx: ComputeInvocation): void {
  res.counter[0].value.add(1);
}

// 13, 35, and 45 threads. No workgroup size below divides its thread count, so every
// dispatch launches more invocations than the example asks for.
const DISPATCH_1D: FixedArray<u32, 3> = [13, 1, 1];
const DISPATCH_2D: FixedArray<u32, 3> = [7, 5, 1];
const DISPATCH_3D: FixedArray<u32, 3> = [5, 3, 3];

// `guarded: true` makes the generator wrap the kernel body in a global-id bounds
// check. TypeGPU adds the same check inside `createGuardedComputePipeline`.
export const count1d: ComputePipelineSpec = computePipeline<CounterLayout>(countKernel, {
  name: "count1d",
  workgroupSize: [8, 1, 1],
  guarded: true,
});

// The three declarations differ only in the workgroup shape, so one kernel body exercises
// the guard on one axis, on two axes, and on three.
export const count2d: ComputePipelineSpec = computePipeline<CounterLayout>(countKernel, {
  name: "count2d",
  workgroupSize: [4, 4, 1],
  guarded: true,
});

export const count3d: ComputePipelineSpec = computePipeline<CounterLayout>(countKernel, {
  name: "count3d",
  workgroupSize: [4, 2, 2],
  guarded: true,
});

// `Counter_SIZE` is a generated constant. STORAGE binds the counter to the kernel, COPY_DST
// admits the zero write, and COPY_SRC admits the readback copy.
function counterBuffer(device: GPUDevice, label: string): Buffer<Counter> {
  return createBuffer<Counter>(
    device,
    Counter_SIZE,
    1,
    GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
    label,
  );
}

export async function main(): Promise<void> {
  // The API layer polls the future itself, so the script never pumps the event loop. A null
  // adapter reports the failure by value, because the layers carry no exceptions.
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    gpu.dispose();
    print("check:dispatch fail");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    adapterResult.dispose();
    gpu.dispose();
    print("check:dispatch fail");
    return;
  }
  let state: string = "fail";
  {
    // Every GPU handle lives in this block, and the script releases each one at the end.
    // TypeGPU leaves the same handles to `root.destroy` and the collector.
    using adapter = adapterResult;
    using device = deviceResult;
    using first = counterBuffer(device, "dispatch-1d");
    using second = counterBuffer(device, "dispatch-2d");
    using third = counterBuffer(device, "dispatch-3d");
    // The three counters start from the same zero record, so each result counts this dispatch
    // and nothing before it.
    const zero = new Counter(new AtomicU32(0));
    first.writeOne(device.queue, 0, Context.bytesOf<Counter>(zero));
    second.writeOne(device.queue, 0, Context.bytesOf<Counter>(zero));
    third.writeOne(device.queue, 0, Context.bytesOf<Counter>(zero));

    // Each pipeline comes from its own generated constants. One scope covers all three, so a
    // failure in any of them ends the run before the dispatches.
    device.pushErrorScope("validation");
    using firstPipeline = createComputePipeline(
      device,
      count1d_WGSL,
      count1d_ENTRY,
      [count1d_LAYOUT0],
      [count1d_WORKGROUP_X, count1d_WORKGROUP_Y, count1d_WORKGROUP_Z],
    );
    using secondPipeline = createComputePipeline(
      device,
      count2d_WGSL,
      count2d_ENTRY,
      [count2d_LAYOUT0],
      [count2d_WORKGROUP_X, count2d_WORKGROUP_Y, count2d_WORKGROUP_Z],
    );
    using thirdPipeline = createComputePipeline(
      device,
      count3d_WGSL,
      count3d_ENTRY,
      [count3d_LAYOUT0],
      [count3d_WORKGROUP_X, count3d_WORKGROUP_Y, count3d_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError === null) {
      using firstLayout = firstPipeline.bindGroupLayout(0);
      using firstGroup = createBindGroup(
        device,
        firstLayout,
        count1d_LAYOUT0,
        // The guard uniform is a hidden layout entry, so the author's resource list stays
        // unchanged. Each pipeline owns its guard buffer, so one encoder carries all three
        // dispatches.
        [bufferResource(first.handle())],
        firstPipeline.guardBuffer(0),
      );
      using secondLayout = secondPipeline.bindGroupLayout(0);
      using secondGroup = createBindGroup(
        device,
        secondLayout,
        count2d_LAYOUT0,
        [bufferResource(second.handle())],
        secondPipeline.guardBuffer(0),
      );
      using thirdLayout = thirdPipeline.bindGroupLayout(0);
      using thirdGroup = createBindGroup(
        device,
        thirdLayout,
        count3d_LAYOUT0,
        [bufferResource(third.handle())],
        thirdPipeline.guardBuffer(0),
      );
      using encoder = device.createCommandEncoderDefault();
      // No workgroup size divides these counts, so the guard drops the extra invocations.
      // `dispatchThreads` writes the counts into the guard buffer before it records the pass.
      firstPipeline.dispatchThreads(
        encoder,
        [firstGroup],
        DISPATCH_1D[0],
        DISPATCH_1D[1],
        DISPATCH_1D[2],
      );
      secondPipeline.dispatchThreads(
        encoder,
        [secondGroup],
        DISPATCH_2D[0],
        DISPATCH_2D[1],
        DISPATCH_2D[2],
      );
      thirdPipeline.dispatchThreads(
        encoder,
        [thirdGroup],
        DISPATCH_3D[0],
        DISPATCH_3D[1],
        DISPATCH_3D[2],
      );
      // One encoder carries all three dispatches, and the queue runs them in the recorded
      // order. The GPU runs nothing until the queue receives the command buffer.
      using command = encoder.finishDefault();
      device.queue.submit([command]);

      // Each read copies one element into a staging buffer and awaits the map. TypeGPU's
      // `buffer.read` hides the same two steps.
      const firstBytes: u8[] = await first.readOne(device, 0);
      const secondBytes: u8[] = await second.readOne(device, 0);
      const thirdBytes: u8[] = await third.readOne(device, 0);
      // The bytes come back through the same schema layout the kernel wrote, so the atomic
      // load reads the counter the device left.
      const firstValue: Counter = Context.fromBytes<Counter>(firstBytes, 0);
      const secondValue: Counter = Context.fromBytes<Counter>(secondBytes, 0);
      const thirdValue: Counter = Context.fromBytes<Counter>(thirdBytes, 0);
      const a: u32 = firstValue.value.load();
      const b: u32 = secondValue.value.load();
      const c: u32 = thirdValue.value.load();
      print(`counts=${a},${b},${c}`);
      // Noop validates the guarded pipelines but leaves each counter zeroed.
      if (a === 0 && b === 0 && c === 0) {
        state = "noop";
      } else if (
        a === DISPATCH_1D[0] * DISPATCH_1D[1] * DISPATCH_1D[2]
        && b === DISPATCH_2D[0] * DISPATCH_2D[1] * DISPATCH_2D[2]
        && c === DISPATCH_3D[0] * DISPATCH_3D[1] * DISPATCH_3D[2]
      ) {
        state = "pass";
      }
    }
  }
  gpu.dispose();
  // One check line reports the outcome, so a reader who runs the example needs no golden.
  // The upstream page reports the same kind of result in a table.
  print(`check:dispatch ${state}`);
}
