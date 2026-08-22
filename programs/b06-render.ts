// program: b06-render
// purpose: prove a render pipeline, vertex layout, offscreen pass, and draw submission
// exercises: RN1-RN13, K1-K16, LY3, BF1-BF5
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
import { Vec2f, Vec3f, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUTextureUsage,
} from "./webgpu";
import {
  tri_FRAGMENT_ENTRY,
  tri_TARGET_FORMAT,
  tri_VERTEX_ENTRY,
  tri_VERTEX_LAYOUT0,
  tri_WGSL,
  Vertex_STRIDE,
} from "./b06-render.typegpu";

@CStruct
class Vertex {
  position: Vec2f;
  color: Vec3f;

  constructor(position: Vec2f, color: Vec3f) {
    this.position = position;
    this.color = color;
  }
}

@CStruct
class Varyings {
  position: Vec4f;
  color: Vec3f;

  constructor(position: Vec4f, color: Vec3f) {
    this.position = position;
    this.color = color;
  }
}

function vert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.color,
  );
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0);
}

export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, {
  format: "rgba8unorm",
});

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
      new Vertex(new Vec2f(-0.6, -0.6), new Vec3f(1.0, 0.0, 0.0)),
      new Vertex(new Vec2f(0.6, -0.6), new Vec3f(0.0, 1.0, 0.0)),
      new Vertex(new Vec2f(0.0, 0.7), new Vec3f(0.0, 0.0, 1.0)),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b06-vertices",
    );
    vertices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Vertex, 3>>(values),
    );
    using target = device.createTexture({
      label: "b06-target",
      size: { width: 64, height: 64 },
      format: tri_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    using pipeline = createRenderPipeline(
      device,
      tri_WGSL,
      tri_VERTEX_ENTRY,
      tri_FRAGMENT_ENTRY,
      [],
      [tri_VERTEX_LAYOUT0],
      tri,
    );
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
    pass.draw(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print("pipeline:created");
    print(`tri_VERTEX_LAYOUT0.arrayStride=${tri_VERTEX_LAYOUT0.arrayStride}`);
    print(`position.offset=${tri_VERTEX_LAYOUT0.attributes[0].offset}`);
    print(`position.format=${tri_VERTEX_LAYOUT0.attributes[0].format}`);
    print(`position.shaderLocation=${tri_VERTEX_LAYOUT0.attributes[0].shaderLocation}`);
    print(`color.offset=${tri_VERTEX_LAYOUT0.attributes[1].offset}`);
    print(`color.format=${tri_VERTEX_LAYOUT0.attributes[1].format}`);
    print(`color.shaderLocation=${tri_VERTEX_LAYOUT0.attributes[1].shaderLocation}`);
    print(`tri_WGSL_LINES=${tri_WGSL.split("\n").length}`);
    print("render:submitted");
  }
  gpu.dispose();
  print("PASS");
}
