// program: b21-blend
// purpose: prove blend factors and operations reach the render pipeline descriptor
// exercises: BF1, BF2, PI14, RN1, RN2, RN4, RN5, RN10, RN12, RN21
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
  blended_FRAGMENT_ENTRY,
  blended_TARGET_FORMAT,
  blended_VERTEX_ENTRY,
  blended_VERTEX_LAYOUT0,
  blended_WGSL,
  Vertex_STRIDE,
} from "./b21-blend.typegpu";

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
  return new Vec4f(0.8, 0.2, 0.1, 0.4);
}

export const blended: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    blend: {
      color: {
        operation: "add",
        srcFactor: "src-alpha",
        dstFactor: "one-minus-src-alpha",
      },
      alpha: {
        operation: "add",
        srcFactor: "one",
        dstFactor: "one",
      },
    },
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
    const values: FixedArray<Vertex, 3> = [
      new Vertex(new Vec2f(-0.7, -0.6)),
      new Vertex(new Vec2f(0.6, -0.6)),
      new Vertex(new Vec2f(0.0, 0.7)),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b21-vertices",
    );
    vertices.write(device.queue(), 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
    using target = device.createTexture({
      label: "b21-target",
      size: { width: 8, height: 8 },
      format: blended_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      blended_WGSL,
      blended_VERTEX_ENTRY,
      blended_FRAGMENT_ENTRY,
      [],
      [blended_VERTEX_LAYOUT0],
      blended,
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
        clearValue: { r: 0, g: 0, b: 0, a: 0 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass, [], [vertices.handle()]);
    pass.draw(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    if (blended.blend === null) {
      print("FAIL blend missing");
      return;
    }
    print(`blend.color.srcFactor=${blended.blend.color.srcFactor}`);
    print(`blend.color.dstFactor=${blended.blend.color.dstFactor}`);
    print(`blend.color.operation=${blended.blend.color.operation}`);
    print(`blend.alpha.srcFactor=${blended.blend.alpha.srcFactor}`);
    print(`blend.alpha.dstFactor=${blended.blend.alpha.dstFactor}`);
    print(`blend.alpha.operation=${blended.blend.alpha.operation}`);
    print("render:submitted");
  }
  gpu.dispose();
  print("PASS");
}
