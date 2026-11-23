// program: b29-depth
// purpose: draw a triangle with the declared depth format, write flag, and comparison
// exercises: BF1, BF2, PI14, RN1, RN2, RN4, RN5, RN10, RN12, RN22
// questions: none

import {
  Buffer,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipelineSpec,
  renderPipeline,
  VertexInvocation,
} from "./typegpu";
import {
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
  terrain_FRAGMENT_ENTRY,
  terrain_TARGET_FORMAT,
  terrain_VERTEX_ENTRY,
  terrain_VERTEX_LAYOUT0,
  terrain_WGSL,
  Vertex_STRIDE,
} from "./b29-depth.typegpu";

@ValueType
class Vertex {
  constructor(public position: Vec2f) {}
}

@ValueType
class Varyings {
  constructor(public position: Vec4f) {}
}

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0));
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(0.8, 0.2, 0.1, 0.4);
}

const terrain: RenderPipelineSpec = renderPipeline(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    depthStencil: {
      format: "depth24plus",
      depthWriteEnabled: true,
      depthCompare: "less",
    },
  },
);

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const values: FixedArray<Vertex, 3> = [
      new Vertex(new Vec2f(-0.7, -0.6)),
      new Vertex(new Vec2f(0.6, -0.6)),
      new Vertex(new Vec2f(0.0, 0.7)),
    ];
    using vertices: Buffer<Vertex> = createBuffer(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b29-vertices",
    );
    vertices.write(device.queue, 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
    using target = device.createTexture({
      label: "b29-target",
      size: { width: 8, height: 8 },
      format: terrain_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    using depth = device.createTexture({
      size: { width: 8, height: 8 },
      format: "depth24plus",
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using depthView = depth.createView();
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      terrain_WGSL,
      terrain_VERTEX_ENTRY,
      terrain_FRAGMENT_ENTRY,
      [],
      [terrain_VERTEX_LAYOUT0],
      terrain,
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({
      colorAttachments: [{
        view,
        clearValue: { r: 0, g: 0, b: 0, a: 0 },
        loadOp: "clear",
        storeOp: "store",
      }],
      depthStencilAttachment: {
        view: depthView,
        depthClearValue: 1.0,
        depthLoadOp: "clear",
        depthStoreOp: "store",
      },
    });
    pipeline.bind(pass, [], [vertices.handle()]);
    pass.draw(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    const state = terrain.depthStencil;
    if (state == null) {
      print("FAIL depth state missing");
      return;
    }
    if (state.depthCompare == undefined) {
      print("FAIL depth comparison missing");
      return;
    }
    print(`depthStencil.format=${state.format}`);
    print(`depthStencil.depthWriteEnabled=${state.depthWriteEnabled}`);
    print(`depthStencil.depthCompare=${state.depthCompare}`);
    print("render:submitted");
  }
  gpu.dispose();
  print("PASS");
}
