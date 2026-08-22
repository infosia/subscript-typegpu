// program: b07-draw-variants
// purpose: prove indexed instanced and non-indexed render draws in one program
// exercises: RN1-RN15, K1-K16, LY3, BF1-BF5
// questions: none

import {
  Buffer,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipelineSpec,
  renderPipeline,
  renderPipelineInstanced,
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
  Instance_STRIDE,
  quad_FRAGMENT_ENTRY,
  quad_TARGET_FORMAT,
  quad_VERTEX_ENTRY,
  quad_VERTEX_LAYOUT0,
  quad_VERTEX_LAYOUT1,
  quad_WGSL,
  tri_FRAGMENT_ENTRY,
  tri_TARGET_FORMAT,
  tri_VERTEX_ENTRY,
  tri_VERTEX_LAYOUT0,
  tri_WGSL,
  Vertex_STRIDE,
} from "./b07-draw-variants.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class Instance {
  offset: Vec2f;
  color: Vec3f;

  constructor(offset: Vec2f, color: Vec3f) {
    this.offset = offset;
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

function quadVert(value: Vertex, instance: Instance, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(
      value.position.x + instance.offset.x,
      value.position.y + instance.offset.y,
      0.0,
      1.0,
    ),
    instance.color,
  );
}

function triVert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec3f(1.0, 1.0, 1.0),
  );
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0);
}

export const quad: RenderPipelineSpec = renderPipelineInstanced<Vertex, Instance, Varyings>(
  quadVert,
  frag,
  { format: "rgba8unorm" },
);

export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(triVert, frag, {
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
    const vertexValues: FixedArray<Vertex, 4> = [
      new Vertex(new Vec2f(-0.23, -0.21)),
      new Vertex(new Vec2f(0.21, -0.21)),
      new Vertex(new Vec2f(-0.23, 0.25)),
      new Vertex(new Vec2f(0.21, 0.25)),
    ];
    const instanceValues: FixedArray<Instance, 3> = [
      new Instance(new Vec2f(-0.38, -0.17), new Vec3f(1.0, 0.0, 0.0)),
      new Instance(new Vec2f(0.0, 0.02), new Vec3f(0.0, 1.0, 0.0)),
      new Instance(new Vec2f(0.31, 0.19), new Vec3f(0.0, 0.0, 1.0)),
    ];
    const indices: FixedArray<u16, 6> = [0, 1, 2, 2, 1, 3];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      4,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b07-vertices",
    );
    using instances: Buffer<Instance> = createBuffer<Instance>(
      device,
      Instance_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b07-instances",
    );
    using indexBuffer = device.createBuffer({
      label: "b07-indices",
      size: 12,
      usage: GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST,
    });
    vertices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues),
    );
    instances.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Instance, 3>>(instanceValues),
    );
    device.queue().writeBuffer(
      indexBuffer,
      0,
      Context.bytesOf<FixedArray<u16, 6>>(indices),
    );
    using target = device.createTexture({
      label: "b07-target",
      size: { width: 64, height: 64 },
      format: quad_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    using quadPipeline = createRenderPipeline(
      device,
      quad_WGSL,
      quad_VERTEX_ENTRY,
      quad_FRAGMENT_ENTRY,
      [],
      [quad_VERTEX_LAYOUT0, quad_VERTEX_LAYOUT1],
      quad,
    );
    using triPipeline = createRenderPipeline(
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
    quadPipeline.bind(pass, [], [vertices.handle(), instances.handle()]);
    pass.setIndexBuffer(indexBuffer, "uint16", 0, 12);
    pass.drawIndexed(6, 3);
    triPipeline.bind(pass, [], [vertices.handle()]);
    pass.draw(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print("pipelines:created");
    print(`quad_VERTEX_LAYOUT0.arrayStride=${quad_VERTEX_LAYOUT0.arrayStride}`);
    print(`quad_VERTEX_LAYOUT0.stepMode=${quad_VERTEX_LAYOUT0.stepMode}`);
    print(`position.shaderLocation=${quad_VERTEX_LAYOUT0.attributes[0].shaderLocation}`);
    print(`quad_VERTEX_LAYOUT1.arrayStride=${quad_VERTEX_LAYOUT1.arrayStride}`);
    print(`quad_VERTEX_LAYOUT1.stepMode=${quad_VERTEX_LAYOUT1.stepMode}`);
    print(`offset.offset=${quad_VERTEX_LAYOUT1.attributes[0].offset}`);
    print(`offset.shaderLocation=${quad_VERTEX_LAYOUT1.attributes[0].shaderLocation}`);
    print(`color.offset=${quad_VERTEX_LAYOUT1.attributes[1].offset}`);
    print(`color.shaderLocation=${quad_VERTEX_LAYOUT1.attributes[1].shaderLocation}`);
    print(`tri_VERTEX_LAYOUT0.arrayStride=${tri_VERTEX_LAYOUT0.arrayStride}`);
    print(`tri_TARGET_FORMAT=${tri_TARGET_FORMAT}`);
    print(`quad_WGSL_LINES=${quad_WGSL.split("\n").length}`);
    print(`tri_WGSL_LINES=${tri_WGSL.split("\n").length}`);
    print("drawIndexed:submitted");
    print("draw:submitted");
  }
  gpu.dispose();
  print("PASS");
}
