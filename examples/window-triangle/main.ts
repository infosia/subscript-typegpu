// program: window-triangle
// purpose: render one host-owned surface triangle and change its clear color with space
// exercises: W1-W8, RN1-RN13, PI14
// questions: none

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  renderPipeline,
  VertexInvocation,
} from "./typegpu";
import {
  Vec2f,
  Vec4f,
} from "./typegpu-types";
import {
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPURenderPipeline,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  tri_FRAGMENT_ENTRY,
  tri_TARGET_FORMAT,
  tri_VERTEX_ENTRY,
  tri_VERTEX_LAYOUT0,
  tri_WGSL,
  Vertex_STRIDE,
} from "./main.typegpu";

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

function vert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0));
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(0.95, 0.45, 0.15, 1.0);
}

export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, {
  format: "bgra8unorm",
});

let ownedDevice: GPUHostOwnedDevice | null = null;
let pipeline: RenderPipeline | null = null;
let vertexBuffer: GPUBuffer | null = null;
let clearIndex: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== tri_TARGET_FORMAT) {
    print(`FAIL format expected=${tri_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const deviceWrapper: GPUHostOwnedDevice = hostOwnedGPUDevice(instance, device);
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-0.65, -0.55)),
    new Vertex(new Vec2f(0.65, -0.55)),
    new Vertex(new Vec2f(0.0, 0.7)),
  ];
  const vertices: GPUBuffer = deviceWrapper.createBuffer({
    label: "window-triangle-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  using queue = deviceWrapper.queue();
  queue.writeBuffer(
    vertices,
    0,
    Context.bytesOf<FixedArray<Vertex, 3>>(values),
  );
  deviceWrapper.pushErrorScope("validation");
  using shader = deviceWrapper.createShaderModule({ code: tri_WGSL });
  using layout = deviceWrapper.createPipelineLayout({ bindGroupLayouts: [] });
  const nativePipeline: GPURenderPipeline = deviceWrapper.createRenderPipeline({
    layout,
    vertex: {
      module: shader,
      entryPoint: tri_VERTEX_ENTRY,
      buffers: [{
        arrayStride: tri_VERTEX_LAYOUT0.arrayStride,
        stepMode: tri_VERTEX_LAYOUT0.stepMode,
        attributes: [{
          format: tri_VERTEX_LAYOUT0.attributes[0].format,
          offset: tri_VERTEX_LAYOUT0.attributes[0].offset,
          shaderLocation: tri_VERTEX_LAYOUT0.attributes[0].shaderLocation,
        }],
      }],
    },
    primitive: {
      topology: tri.topology,
      cullMode: tri.cullMode,
      frontFace: tri.frontFace,
    },
    fragment: {
      module: shader,
      entryPoint: tri_FRAGMENT_ENTRY,
      targets: [{ format: tri_TARGET_FORMAT }],
    },
  });
  const validationError = deviceWrapper.popErrorScope();
  if (validationError !== null) {
    nativePipeline.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  ownedDevice = deviceWrapper;
  vertexBuffer = vertices;
  pipeline = new RenderPipeline(nativePipeline, "undefined");
}

export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
): void {
  const activeDevice: GPUHostOwnedDevice | null = ownedDevice;
  const activePipeline: RenderPipeline | null = pipeline;
  const activeVertices: GPUBuffer | null = vertexBuffer;
  if (activeDevice === null) {
    return;
  }
  if (activePipeline === null) {
    return;
  }
  if (activeVertices === null) {
    return;
  }
  if (key === 32) {
    clearIndex = (clearIndex + 1) % 3;
  }
  const target = new GPUTextureView(view);
  using encoder = activeDevice.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: clearIndex === 0
        ? { r: 0.04, g: 0.06, b: 0.12, a: 1.0 }
        : clearIndex === 1
          ? { r: 0.12, g: 0.04, b: 0.06, a: 1.0 }
          : { r: 0.04, g: 0.12, b: 0.07, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  activePipeline.bind(pass, [], [activeVertices]);
  pass.draw(3);
  pass.end();
  using command = encoder.finishDefault();
  using queue = activeDevice.queue();
  queue.submit([command]);
}

export function shutdown(): void {
  if (vertexBuffer !== null) {
    vertexBuffer.dispose();
    vertexBuffer = null;
  }
  if (pipeline !== null) {
    pipeline.dispose();
    pipeline = null;
  }
  ownedDevice = null;
}
