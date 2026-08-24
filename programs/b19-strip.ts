// program: b19-strip
// purpose: prove a four-vertex triangle strip reaches the render pipeline descriptor
// exercises: BF1, BF2, PI14, RN1, RN2, RN4, RN5, RN10, RN12, RN20
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
  strip_FRAGMENT_ENTRY,
  strip_TARGET_FORMAT,
  strip_VERTEX_ENTRY,
  strip_VERTEX_LAYOUT0,
  strip_WGSL,
  Vertex_STRIDE,
} from "./b19-strip.typegpu";

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

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0));
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(0.25, 0.5, 0.75, 1.0);
}

export const strip: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    topology: "triangle-strip",
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
    const values: FixedArray<Vertex, 4> = [
      new Vertex(new Vec2f(-0.7, -0.6)),
      new Vertex(new Vec2f(0.6, -0.6)),
      new Vertex(new Vec2f(-0.7, 0.6)),
      new Vertex(new Vec2f(0.6, 0.6)),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      4,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b19-vertices",
    );
    vertices.write(device.queue(), 0, Context.bytesOf<FixedArray<Vertex, 4>>(values));
    using target = device.createTexture({
      label: "b19-target",
      size: { width: 8, height: 8 },
      format: strip_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      strip_WGSL,
      strip_VERTEX_ENTRY,
      strip_FRAGMENT_ENTRY,
      [],
      [strip_VERTEX_LAYOUT0],
      strip,
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
    using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({
      colorAttachments: [{
        view,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass, [], [vertices.handle()]);
    pass.draw(4);
    pass.end();
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print(`strip.topology=${strip.topology}`);
    print(`strip_VERTEX_LAYOUT0.arrayStride=${strip_VERTEX_LAYOUT0.arrayStride}`);
    print(`strip_WGSL_LINES=${strip_WGSL.split("\n").length}`);
    print("render:submitted");
  }
  gpu.dispose();
  print("PASS");
}
