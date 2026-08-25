// program: b22-first-program
// purpose: show one typed counter from buffer creation through GPU and host execution
// exercises: CL4, PI11, PI14
// questions: none

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  createBuffer,
  createComputePipeline,
  computePipeline,
  MutStorage,
  simulateCompute,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";
import {
  CounterLayoutResources,
  createCounterLayoutResources,
  createFirstProgramBindGroup0,
  firstProgram_ENTRY,
  firstProgram_HOST_RUNNABLE,
  firstProgram_LAYOUT0,
  firstProgram_WGSL,
  firstProgram_WORKGROUP_X,
  firstProgram_WORKGROUP_Y,
  firstProgram_WORKGROUP_Z,
  State_OFFSET_incrementBy,
  State_STRIDE,
} from "./b22-first-program.typegpu";

@CStruct
class State {
  counter: u32;
  incrementBy: u32;

  constructor(counter: u32, incrementBy: u32) {
    this.counter = counter;
    this.incrementBy = incrementBy;
  }
}

class CounterLayout {
  state!: MutStorage<State>;
}

function incrementCounter(res: CounterLayout, ctx: ComputeInvocation): void {
  const state: State = res.state[0];
  state.counter += state.incrementBy;
  res.state[0] = state;
}

export const firstProgram: ComputePipelineSpec = computePipeline<CounterLayout>(
  incrementCounter,
  {
    name: "firstProgram",
    workgroupSize: [1, 1, 1],
  },
);

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using stateBuffer: Buffer<State> = createBuffer<State>(
      device,
      State_STRIDE,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC + GPUBufferUsage.COPY_DST,
      "b22-state",
    );

    const initialState = new State(0, 10);
    stateBuffer.write(
      device.queue(),
      0,
      Context.bytesOf<State>(initialState),
    );

    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      firstProgram_WGSL,
      firstProgram_ENTRY,
      [firstProgram_LAYOUT0],
      [
        firstProgram_WORKGROUP_X,
        firstProgram_WORKGROUP_Y,
        firstProgram_WORKGROUP_Z,
      ],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }

    const resources: CounterLayoutResources = createCounterLayoutResources(
      stateBuffer.handle(),
    );
    using bindGroup = createFirstProgramBindGroup0(
      device,
      pipeline,
      resources,
    );

    using firstEncoder = device.createCommandEncoderDefault();
    pipeline.dispatch(firstEncoder, [bindGroup], 1, 1, 1);
    using firstCommand = firstEncoder.finishDefault();
    device.queue().submit([firstCommand]);

    stateBuffer.patch(
      device.queue(),
      0,
      State_OFFSET_incrementBy,
      Context.bytesOf<FixedArray<u32, 1>>([25]),
    );

    using secondEncoder = device.createCommandEncoderDefault();
    pipeline.dispatch(secondEncoder, [bindGroup], 1, 1, 1);
    using secondCommand = secondEncoder.finishDefault();
    device.queue().submit([secondCommand]);

    const readbackBytes: u8[] = await stateBuffer.readOne(device, 0);
    const readback: State = Context.fromBytes<State>(readbackBytes, 0);
    print(`readback:counter=${readback.counter} incrementBy=${readback.incrementBy}`);

    const hostLayout = new CounterLayout();
    hostLayout.state = new MutStorage<State>([new State(0, 10)]);
    simulateCompute<CounterLayout>(
      incrementCounter,
      hostLayout,
      firstProgram,
      [1, 1, 1],
      firstProgram_HOST_RUNNABLE,
    );
    const hostState: State = hostLayout.state[0];
    hostState.incrementBy = 25;
    hostLayout.state[0] = hostState;
    simulateCompute<CounterLayout>(
      incrementCounter,
      hostLayout,
      firstProgram,
      [1, 1, 1],
      firstProgram_HOST_RUNNABLE,
    );
    print(`host:counter=${hostLayout.state[0].counter}`);
  }
  gpu.dispose();
  print("PASS");
}
