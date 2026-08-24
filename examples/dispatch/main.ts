// example: dispatch
// Counts guarded compute invocations for thread counts that no workgroup size divides.
// This port keeps the upstream guard cases and drops its other tests.
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

function count1dKernel(res: CounterLayout, ctx: ComputeInvocation): void {
  res.counter[0].value.add(1);
}

function count2dKernel(res: CounterLayout, ctx: ComputeInvocation): void {
  res.counter[0].value.add(1);
}

function count3dKernel(res: CounterLayout, ctx: ComputeInvocation): void {
  res.counter[0].value.add(1);
}

// `guarded: true` makes the generator wrap the kernel body in a global-id bounds
// check. TypeGPU adds the same check inside `createGuardedComputePipeline`.
export const count1d: ComputePipelineSpec = computePipeline<CounterLayout>(count1dKernel, {
  name: "count1d",
  workgroupSize: [8, 1, 1],
  guarded: true,
});

export const count2d: ComputePipelineSpec = computePipeline<CounterLayout>(count2dKernel, {
  name: "count2d",
  workgroupSize: [4, 4, 1],
  guarded: true,
});

export const count3d: ComputePipelineSpec = computePipeline<CounterLayout>(count3dKernel, {
  name: "count3d",
  workgroupSize: [4, 2, 2],
  guarded: true,
});

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
    using adapter = adapterResult;
    using device = deviceResult;
    using first = counterBuffer(device, "dispatch-1d");
    using second = counterBuffer(device, "dispatch-2d");
    using third = counterBuffer(device, "dispatch-3d");
    const zero = new Counter(new AtomicU32(0));
    using queue = device.queue();
    first.writeOne(queue, 0, Context.bytesOf<Counter>(zero));
    second.writeOne(queue, 0, Context.bytesOf<Counter>(zero));
    third.writeOne(queue, 0, Context.bytesOf<Counter>(zero));

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
      firstPipeline.dispatchThreads(encoder, [firstGroup], 13, 1, 1);
      secondPipeline.dispatchThreads(encoder, [secondGroup], 7, 5, 1);
      thirdPipeline.dispatchThreads(encoder, [thirdGroup], 5, 3, 3);
      using command = encoder.finishDefault();
      queue.submit([command]);

      const firstBytes: u8[] = await first.readOne(device, 0);
      const secondBytes: u8[] = await second.readOne(device, 0);
      const thirdBytes: u8[] = await third.readOne(device, 0);
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
      } else if (a === 13 && b === 35 && c === 45) {
        state = "pass";
      }
    }
  }
  gpu.dispose();
  print(`check:dispatch ${state}`);
}
