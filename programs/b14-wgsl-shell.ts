// program: b14-wgsl-shell
// purpose: prove typed WGSL shells use literal GPU bodies and ordinary host bodies
// exercises: CL1, K29, K30, K31, PI14
// questions: none

import {
  bufferResource,
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createBindGroup,
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
} from "./b14-wgsl-shell.typegpu";

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
    res.output.set(0, addBias(5));
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
    using output = device.createBuffer({
      label: "b14-output",
      size: 4,
      usage: GPUBufferUsage.STORAGE,
    });
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
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using bindGroup = createBindGroup(
      device,
      pipeline.bindGroupLayout(0),
      shellPipeline_LAYOUT0,
      [bufferResource(output)],
    );
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatch(encoder, [bindGroup], 1, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const host = new ShellLayout();
    host.output = new MutStorage<u32>([0]);
    simulateCompute<ShellLayout>(
      shellKernel,
      host,
      shellPipeline,
      [1, 1, 1],
      shellPipeline_HOST_RUNNABLE,
    );
    print(`shellPipeline_WGSL_LINES=${shellPipeline_WGSL.split("\n").length}`);
    print(`host:out=${host.output.get(0)}`);
  }
  gpu.dispose();
  print("PASS");
}
