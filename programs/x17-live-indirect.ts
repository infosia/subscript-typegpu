// program: x17-live-indirect
// purpose: compare indirect compute and indirect rendering with host oracles
// exercises: BF9, CL1, PI14, PI16, PI17, RN18
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
  DrawIndirectArgs,
  Vec2f,
  Vec4f,
} from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  indirectCompute_ENTRY,
  indirectCompute_HOST_RUNNABLE,
  indirectCompute_LAYOUT0,
  indirectCompute_WGSL,
  indirectCompute_WORKGROUP_X,
  indirectCompute_WORKGROUP_Y,
  indirectCompute_WORKGROUP_Z,
  indirectRender_FRAGMENT_ENTRY,
  indirectRender_TARGET_FORMAT,
  indirectRender_VERTEX_ENTRY,
  indirectRender_VERTEX_LAYOUT0,
  indirectRender_WGSL,
  Vertex_STRIDE,
} from "./x17-live-indirect.typegpu";

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
    res.output[0] = 23;
  }
}

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
  );
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(0.25, 0.6, 0.75, 1.0);
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
  { format: "rgba8unorm" },
);

function edge(a: Vec2f, b: Vec2f, p: Vec2f): f32 {
  return (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x);
}

function inside(p: Vec2f, a: Vec2f, b: Vec2f, c: Vec2f): boolean {
  const e0 = edge(a, b, p);
  const e1 = edge(b, c, p);
  const e2 = edge(c, a, p);
  return (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0)
    || (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0);
}

function center(x: i32, y: i32): Vec2f {
  return new Vec2f(
    ((x as f32) + 0.5) / 4.0 * 2.0 - 1.0,
    1.0 - ((y as f32) + 0.5) / 4.0 * 2.0,
  );
}

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
      label: "x17-indirect",
      size: 28,
      usage: GPUBufferUsage.INDIRECT + GPUBufferUsage.COPY_DST,
    });
    device.queue().writeBuffer(
      indirect,
      0,
      Context.bytesOf<DispatchIndirectArgs>(new DispatchIndirectArgs(1, 1, 1)),
    );
    device.queue().writeBuffer(
      indirect,
      12,
      Context.bytesOf<DrawIndirectArgs>(new DrawIndirectArgs(3, 1, 0, 0)),
    );
    using output: Buffer<u32> = createBuffer<u32>(
      device,
      4,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_SRC,
      "x17-output",
    );
    const a = new Vec2f(-0.7, -0.6);
    const b = new Vec2f(0.7, -0.6);
    const c = new Vec2f(0.0, 0.7);
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x17-vertices",
    );
    vertices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Vertex, 3>>([
        new Vertex(a),
        new Vertex(b),
        new Vertex(c),
      ]),
    );
    using target = device.createTexture({
      label: "x17-target",
      size: { width: 4, height: 4 },
      format: indirectRender_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x17-readback",
      size: 1024,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
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
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using group = createBindGroup(
      device,
      compute.bindGroupLayout(0),
      indirectCompute_LAYOUT0,
      [bufferResource(output.handle())],
    );
    using encoder = device.createCommandEncoderDefault();
    compute.dispatchIndirect(encoder, [group], indirect, 0);
    using pass = encoder.beginRenderPass({
      colorAttachments: [{
        view,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    render.bind(pass, [], [vertices.handle()]);
    pass.drawIndirect(indirect, 12);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: target },
      { buffer: readback, bytesPerRow: 256, rowsPerImage: 4 },
      { width: 4, height: 4 },
    );
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    if (!await device.queue().onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    const outputBytes: u8[] = await output.read(device, 0, 1);
    const gpuValues: FixedArray<u32, 1> = Context.fromBytes<FixedArray<u32, 1>>(
      outputBytes,
      0,
    );
    const host = new IndirectLayout();
    host.output = new MutStorage<u32>([0]);
    simulateCompute<IndirectLayout>(
      computeStep,
      host,
      indirectCompute,
      [1, 1, 1],
      indirectCompute_HOST_RUNNABLE,
    );
    if (gpuValues[0] !== host.output[0]) {
      print("FAIL compute");
      return;
    }
    if (!await readback.mapAsync(GPUMapMode.READ, 0, 1024)) {
      print("FAIL map");
      return;
    }
    const pixels: u8[] = readback.readMappedRange(0, 1024);
    let y: i32 = 0;
    while (y < 4) {
      let x: i32 = 0;
      while (x < 4) {
        const hit = inside(center(x, y), a, b, c);
        const expectedR: u8 = hit ? 64 : 0;
        const expectedG: u8 = hit ? 153 : 0;
        const expectedB: u8 = hit ? 191 : 0;
        const o = y * 256 + x * 4;
        if (pixels[o] !== expectedR
          || pixels[o + 1] !== expectedG
          || pixels[o + 2] !== expectedB
          || pixels[o + 3] !== 255) {
          print(`FAIL pixel ${x},${y}`);
          return;
        }
        x += 1;
      }
      y += 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
