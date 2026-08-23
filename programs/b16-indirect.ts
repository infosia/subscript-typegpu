// program: b16-indirect
// purpose: record compute, draw, and indexed-draw indirect commands from one argument buffer
// exercises: CL1, PI14, PI16, PI17, RN18
// questions: none

import {
  bufferResource,
  ComputeInvocation,
  ComputePipelineSpec,
  computePipeline,
  createBindGroup,
  createComputePipeline,
  createRenderPipeline,
  FragmentInvocation,
  MutStorage,
  RenderPipelineSpec,
  renderPipeline,
  simulateCompute,
  VertexInvocation,
} from "./typegpu";
import {
  DispatchIndirectArgs,
  DrawIndexedIndirectArgs,
  DrawIndirectArgs,
  Vec2f,
  Vec4f,
} from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUTextureUsage,
} from "./webgpu";
import {
  DispatchIndirectArgs_SIZE,
  DrawIndexedIndirectArgs_SIZE,
  DrawIndirectArgs_SIZE,
  indirectCompute_ENTRY,
  indirectCompute_HOST_RUNNABLE,
  indirectCompute_LAYOUT0,
  indirectCompute_WGSL,
  indirectCompute_WORKGROUP_X,
  indirectCompute_WORKGROUP_Y,
  indirectCompute_WORKGROUP_Z,
  indirectRender_FRAGMENT_ENTRY,
  indirectRender_INDEX_FORMAT,
  indirectRender_TARGET_FORMAT,
  indirectRender_VERTEX_ENTRY,
  indirectRender_VERTEX_LAYOUT0,
  indirectRender_WGSL,
  Vertex_STRIDE,
} from "./b16-indirect.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class Varyings {
  position: Vec4f;

  constructor(position: Vec4f) {
    this.position = position;
  }
}

class IndirectLayout {
  output!: MutStorage<u32>;
}

function computeStep(res: IndirectLayout, ctx: ComputeInvocation): void {
  if (ctx.globalId.x === 0) {
    res.output.set(0, 1);
  }
}

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
  );
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(1.0, 0.5, 0.0, 1.0);
}

export const indirectCompute: ComputePipelineSpec = computePipeline<IndirectLayout>(
  computeStep,
  {
    name: "indirectCompute",
    workgroupSize: [1, 1, 1],
  },
);

export const indirectRender: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    indexFormat: "uint16",
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
    using indirect = device.createBuffer({
      label: "b16-indirect",
      size: 48,
      usage: GPUBufferUsage.INDIRECT + GPUBufferUsage.COPY_DST,
    });
    using output = device.createBuffer({
      label: "b16-output",
      size: 4,
      usage: GPUBufferUsage.STORAGE,
    });
    using vertices = device.createBuffer({
      label: "b16-vertices",
      size: (Vertex_STRIDE * 3) as u64,
      usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
    });
    using indices = device.createBuffer({
      label: "b16-indices",
      size: 8,
      usage: GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST,
    });
    const dispatchOffset: u64 = 0;
    const drawOffset: u64 = 12;
    const drawIndexedOffset: u64 = 28;
    device.queue().writeBuffer(
      indirect,
      dispatchOffset,
      Context.bytesOf<DispatchIndirectArgs>(new DispatchIndirectArgs(1, 1, 1)),
    );
    device.queue().writeBuffer(
      indirect,
      drawOffset,
      Context.bytesOf<DrawIndirectArgs>(new DrawIndirectArgs(3, 1, 0, 0)),
    );
    device.queue().writeBuffer(
      indirect,
      drawIndexedOffset,
      Context.bytesOf<DrawIndexedIndirectArgs>(new DrawIndexedIndirectArgs(3, 1, 0, 0, 0)),
    );
    device.queue().writeBuffer(
      vertices,
      0,
      Context.bytesOf<FixedArray<Vertex, 3>>([
        new Vertex(new Vec2f(-0.5, -0.5)),
        new Vertex(new Vec2f(0.5, -0.5)),
        new Vertex(new Vec2f(0.0, 0.5)),
      ]),
    );
    device.queue().writeBuffer(
      indices,
      0,
      Context.bytesOf<FixedArray<u16, 4>>([0, 1, 2, 0]),
    );
    using target = device.createTexture({
      label: "b16-target",
      size: { width: 8, height: 8 },
      format: indirectRender_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    device.pushErrorScope("validation");
    using compute = createComputePipeline(
      device,
      indirectCompute_WGSL,
      indirectCompute_ENTRY,
      [indirectCompute_LAYOUT0],
      [
        indirectCompute_WORKGROUP_X,
        indirectCompute_WORKGROUP_Y,
        indirectCompute_WORKGROUP_Z,
      ],
    );
    using render = createRenderPipeline(
      device,
      indirectRender_WGSL,
      indirectRender_VERTEX_ENTRY,
      indirectRender_FRAGMENT_ENTRY,
      [],
      [indirectRender_VERTEX_LAYOUT0],
      indirectRender,
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using computeGroup = createBindGroup(
      device,
      compute.bindGroupLayout(0),
      indirectCompute_LAYOUT0,
      [bufferResource(output)],
    );
    using encoder = device.createCommandEncoderDefault();
    compute.dispatchIndirect(encoder, [computeGroup], indirect, dispatchOffset);
    using pass = encoder.beginRenderPass({
      colorAttachments: [{
        view,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    render.bind(pass, [], [vertices]);
    pass.drawIndirect(indirect, drawOffset);
    render.setIndexBuffer(pass, indices);
    pass.drawIndexedIndirect(indirect, drawIndexedOffset);
    pass.end();
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const host = new IndirectLayout();
    host.output = new MutStorage<u32>([0]);
    simulateCompute<IndirectLayout>(
      computeStep,
      host,
      indirectCompute,
      [1, 1, 1],
      indirectCompute_HOST_RUNNABLE,
    );
    print(`DispatchIndirectArgs_SIZE=${DispatchIndirectArgs_SIZE}`);
    print(`DrawIndirectArgs_SIZE=${DrawIndirectArgs_SIZE}`);
    print(`DrawIndexedIndirectArgs_SIZE=${DrawIndexedIndirectArgs_SIZE}`);
    print(`offsets=${dispatchOffset},${drawOffset},${drawIndexedOffset}`);
    print(`indirectRender_INDEX_FORMAT=${indirectRender_INDEX_FORMAT}`);
    print(`host:out=${host.output.get(0)}`);
    print("indirect:submitted");
  }
  gpu.dispose();
  print("PASS");
}
