// program: b08-render-bindings
// purpose: prove render binding visibility from vertex and fragment kernel reach
// exercises: RN1-RN13, RN17, PI3-PI8, K1-K16
// questions: none

import {
  Buffer,
  createBindGroup,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipelineSpec,
  renderPipelineL,
  Storage,
  Uniform,
  VertexInvocation,
} from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUTextureUsage,
} from "./webgpu";
import {
  Offset_SIZE,
  shifted_FRAGMENT_ENTRY,
  shifted_LAYOUT0,
  shifted_TARGET_FORMAT,
  shifted_VERTEX_ENTRY,
  shifted_VERTEX_LAYOUT0,
  shifted_WGSL,
  Tint_SIZE,
  Vertex_STRIDE,
} from "./b08-render-bindings.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class Offset {
  value: Vec4f;

  constructor(value: Vec4f) {
    this.value = value;
  }
}

@CStruct
class Tint {
  value: Vec4f;

  constructor(value: Vec4f) {
    this.value = value;
  }
}

@CStruct
class Varyings {
  position: Vec4f;

  constructor(position: Vec4f) {
    this.position = position;
  }
}

class RenderLayout {
  params!: Uniform<Offset>;
  tint!: Storage<Tint>;
}

function vert(res: RenderLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  const offset: Offset = res.params.get();
  return new Varyings(
    new Vec4f(
      value.position.x + offset.value.x,
      value.position.y + offset.value.y,
      0.0,
      1.0,
    ),
  );
}

function frag(res: RenderLayout, input: Varyings, ctx: FragmentInvocation): Vec4f {
  const color: Tint = res.tint[0];
  return color.value;
}

export const shifted: RenderPipelineSpec = renderPipelineL<RenderLayout, Vertex, Varyings>(
  vert,
  frag,
  { format: "rgba8unorm" },
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
      new Vertex(new Vec2f(-0.6, -0.6)),
      new Vertex(new Vec2f(0.6, -0.6)),
      new Vertex(new Vec2f(0.0, 0.7)),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "b08-vertices",
    );
    using params = device.createBuffer({
      label: "b08-params",
      size: Offset_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using tint = device.createBuffer({
      label: "b08-tint",
      size: Tint_SIZE as u64,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    vertices.write(device.queue(), 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
    device.queue().writeBuffer(
      params,
      0,
      Context.bytesOf<Offset>(new Offset(new Vec4f(0.1, -0.1, 0.0, 0.0))),
    );
    device.queue().writeBuffer(
      tint,
      0,
      Context.bytesOf<Tint>(new Tint(new Vec4f(0.25, 0.6, 0.75, 1.0))),
    );
    using target = device.createTexture({
      label: "b08-target",
      size: { width: 64, height: 64 },
      format: shifted_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    using pipeline = createRenderPipeline(
      device,
      shifted_WGSL,
      shifted_VERTEX_ENTRY,
      shifted_FRAGMENT_ENTRY,
      [shifted_LAYOUT0],
      [shifted_VERTEX_LAYOUT0],
      shifted,
    );
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      shifted_LAYOUT0,
      [params, tint],
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
    pipeline.bind(pass, [bindGroup], [vertices.handle()]);
    pass.draw(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    print("pipeline:created");
    print(`params.visibility=${shifted_LAYOUT0.entries[0].visibility}`);
    print(`tint.visibility=${shifted_LAYOUT0.entries[1].visibility}`);
    print(`shifted_WGSL_LINES=${shifted_WGSL.split("\n").length}`);
    print("render:submitted");
  }
  gpu.dispose();
  print("PASS");
}
