// program: x15-live-shell
// purpose: compare a typed WGSL shell on the GPU with its host implementation
// exercises: BF9, CL1, K29, K30, K31, PI14
// questions: none

import {
  Buffer,
  bufferResource,
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  MutStorage,
  simulateCompute,
  wgslDeclarations,
  WgslShellSpec,
  wgslShell,
} from "./typegpu";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
} from "./webgpu";
import {
  shellPipeline_ENTRY,
  shellPipeline_HOST_RUNNABLE,
  shellPipeline_LAYOUT0,
  shellPipeline_WGSL,
  shellPipeline_WORKGROUP_X,
  shellPipeline_WORKGROUP_Y,
  shellPipeline_WORKGROUP_Z,
} from "./x15-live-shell.typegpu";

wgslDeclarations("const SHELL_BIAS: u32 = 7u;");

function addBias(value: u32): u32 {
  return value + 7;
}

const addBiasShell: WgslShellSpec = wgslShell<(value: u32) => u32>(
  addBias,
  {
    body: "return value + SHELL_BIAS;",
  },
);

class ShellLayout {
  output!: MutStorage<u32>;
}

function shellKernel(res: ShellLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x === 0) {
    res.output[0] = addBias(5);
  }
}

export const shellPipeline: ComputePipelineSpec = computePipeline<ShellLayout>(
  shellKernel,
  {
    name: "shellPipeline",
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
    using output: Buffer<u32> = createBuffer<u32>(
      device,
      4,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x15-output",
    );
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      shellPipeline_WGSL,
      shellPipeline_ENTRY,
      [shellPipeline_LAYOUT0],
      [
        shellPipeline_WORKGROUP_X,
        shellPipeline_WORKGROUP_Y,
        shellPipeline_WORKGROUP_Z,
      ],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using bindGroup = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      shellPipeline_LAYOUT0,
      [bufferResource(output.handle())],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const bytes: u8[] = await output.read(device, 0, 1);
    const gpuValues: FixedArray<u32, 1> = Context.fromBytes<FixedArray<u32, 1>>(bytes, 0);
    const gpuValue: u32 = gpuValues[0];
    const host = new ShellLayout();
    host.output = new MutStorage<u32>([0]);
    simulateCompute<ShellLayout>(
      shellKernel,
      host,
      shellPipeline,
      [1, 1, 1],
      shellPipeline_HOST_RUNNABLE,
    );
    if (gpuValue !== host.output[0]) {
      print(`FAIL gpu=${gpuValue} host=${host.output[0]}`);
      return;
    }
  }
  gpu.dispose();
  print("PASS");
}
