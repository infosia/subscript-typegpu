// program: b17-index-cull
// purpose: expose literal index format, cull mode, and front face by generated name
// exercises: PI14, RN18, RN19
// questions: none

import {
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
  culled_FRAGMENT_ENTRY,
  culled_INDEX_FORMAT,
  culled_TARGET_FORMAT,
  culled_VERTEX_ENTRY,
  culled_VERTEX_LAYOUT0,
  culled_WGSL,
  Vertex_STRIDE,
} from "./b17-index-cull.typegpu";

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
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
  );
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(1.0, 1.0, 1.0, 1.0);
}

export const culled: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    indexFormat: "uint16",
    cullMode: "back",
    frontFace: "ccw",
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
    using vertices = device.createBuffer({
      label: "b17-vertices",
      size: (Vertex_STRIDE * 3) as u64,
      usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
    });
    using indices = device.createBuffer({
      label: "b17-indices",
      size: 8,
      usage: GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(
      vertices,
      0,
      Context.bytesOf<FixedArray<Vertex, 3>>([
        new Vertex(new Vec2f(-0.5, -0.5)),
        new Vertex(new Vec2f(0.5, -0.5)),
        new Vertex(new Vec2f(0.0, 0.5)),
      ]),
    );
    device.queue.writeBuffer(
      indices,
      0,
      Context.bytesOf<FixedArray<u16, 4>>([0, 1, 2, 0]),
    );
    using target = device.createTexture({
      label: "b17-target",
      size: { width: 8, height: 8 },
      format: culled_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      culled_WGSL,
      culled_VERTEX_ENTRY,
      culled_FRAGMENT_ENTRY,
      [],
      [culled_VERTEX_LAYOUT0],
      culled,
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
    pipeline.bind(pass, [], [vertices]);
    pipeline.setIndexBuffer(pass, indices);
    pass.drawIndexed(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    print(`culled_INDEX_FORMAT=${culled_INDEX_FORMAT}`);
    print(`culled.cullMode=${culled.cullMode}`);
    print(`culled.frontFace=${culled.frontFace}`);
    print("render:submitted");
  }
  gpu.dispose();
  print("PASS");
}
