// example: triangle
// Draws one triangle whose interpolated color starts from the three vertex indices.
// Ported from TypeGPU's triangle example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  VertexInvocation,
  renderPipeline,
} from "./typegpu";
import {
  Vec2f,
  Vec3f,
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
  Vertex_STRIDE,
  triangle_FRAGMENT_ENTRY,
  triangle_TARGET_FORMAT,
  triangle_VERTEX_ENTRY,
  triangle_VERTEX_LAYOUT0,
  triangle_WGSL,
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
  color: Vec3f;

  constructor(position: Vec4f, color: Vec3f) {
    this.position = position;
    this.color = color;
  }
}

function triangleVertex(value: Vertex, ctx: VertexInvocation): Varyings {
  const color: Vec3f = ctx.vertexIndex === 0
    ? new Vec3f(1.0, 0.15, 0.1)
    : ctx.vertexIndex === 1
      ? new Vec3f(0.1, 0.9, 0.3)
      : new Vec3f(0.15, 0.3, 1.0);
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    color,
  );
}

function triangleFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0);
}

export const triangle: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  triangleVertex,
  triangleFragment,
  { format: "bgra8unorm" },
);

let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== triangle_TARGET_FORMAT) {
    print(`FAIL format expected=${triangle_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-0.72, -0.58)),
    new Vertex(new Vec2f(0.7, -0.5)),
    new Vertex(new Vec2f(-0.05, 0.76)),
  ];
  const vertices = hostDevice.createBuffer({
    label: "example-triangle-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
  hostDevice.pushErrorScope("validation");
  using shader = hostDevice.createShaderModule({ code: triangle_WGSL });
  using layout = hostDevice.createPipelineLayout({ bindGroupLayouts: [] });
  const nativePipeline: GPURenderPipeline = hostDevice.createRenderPipeline({
    layout,
    vertex: {
      module: shader,
      entryPoint: triangle_VERTEX_ENTRY,
      buffers: [{
        arrayStride: triangle_VERTEX_LAYOUT0.arrayStride,
        stepMode: triangle_VERTEX_LAYOUT0.stepMode,
        attributes: [{
          format: triangle_VERTEX_LAYOUT0.attributes[0].format,
          offset: triangle_VERTEX_LAYOUT0.attributes[0].offset,
          shaderLocation: triangle_VERTEX_LAYOUT0.attributes[0].shaderLocation,
        }],
      }],
    },
    primitive: {
      topology: triangle.topology,
      cullMode: triangle.cullMode,
      frontFace: triangle.frontFace,
    },
    fragment: {
      module: shader,
      entryPoint: triangle_FRAGMENT_ENTRY,
      targets: [{ format: triangle_TARGET_FORMAT }],
    },
  });
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    nativePipeline.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  activeDevice = hostDevice;
  activeVertices = vertices;
  activePipeline = new RenderPipeline(nativePipeline, "undefined");
}

export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
): void {
  const device: GPUHostOwnedDevice | null = activeDevice;
  const pipeline: RenderPipeline | null = activePipeline;
  const vertices: GPUBuffer | null = activeVertices;
  if (device === null) {
    return;
  }
  if (pipeline === null) {
    return;
  }
  if (vertices === null) {
    return;
  }
  frameCount += 1;
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.025, g: 0.035, b: 0.065, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  pipeline.bind(pass, [], [vertices]);
  pass.draw(3);
  pass.end();
  using command = encoder.finishDefault();
  using queue = device.queue();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeVertices !== null) {
    activeVertices.dispose();
    activeVertices = null;
  }
  if (activePipeline !== null) {
    activePipeline.dispose();
    activePipeline = null;
  }
  activeDevice = null;
}
