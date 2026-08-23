// example: square
// Draws an indexed quad with a four-corner color field interpolated through the varyings.
// Ported from TypeGPU's square example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  VertexInvocation,
  renderPipeline,
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
  Vertex_STRIDE,
  square_FRAGMENT_ENTRY,
  square_INDEX_FORMAT,
  square_TARGET_FORMAT,
  square_VERTEX_ENTRY,
  square_VERTEX_LAYOUT0,
  square_WGSL,
} from "./main.typegpu";

@CStruct
class Vertex {
  position: Vec2f;
  color: Vec4f;

  constructor(position: Vec2f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

@CStruct
class Varyings {
  position: Vec4f;
  color: Vec4f;

  constructor(position: Vec4f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

function squareVertex(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.color,
  );
}

function squareFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return input.color;
}

export const square: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  squareVertex,
  squareFragment,
  { format: "bgra8unorm", indexFormat: "uint16" },
);

let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeIndices: GPUBuffer | null = null;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== square_TARGET_FORMAT) {
    print(`FAIL format expected=${square_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const values: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-0.68, -0.68), new Vec4f(0.95, 0.15, 0.15, 1.0)),
    new Vertex(new Vec2f(0.68, -0.68), new Vec4f(0.15, 0.9, 0.2, 1.0)),
    new Vertex(new Vec2f(-0.68, 0.68), new Vec4f(0.2, 0.25, 0.95, 1.0)),
    new Vertex(new Vec2f(0.68, 0.68), new Vec4f(0.95, 0.85, 0.15, 1.0)),
  ];
  const indices: FixedArray<u16, 6> = [0, 1, 2, 2, 1, 3];
  const vertices = hostDevice.createBuffer({
    label: "example-square-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const indexBuffer = hostDevice.createBuffer({
    label: "example-square-indices",
    size: 12,
    usage: GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(values));
  queue.writeBuffer(indexBuffer, 0, Context.bytesOf<FixedArray<u16, 6>>(indices));
  hostDevice.pushErrorScope("validation");
  using shader = hostDevice.createShaderModule({ code: square_WGSL });
  using layout = hostDevice.createPipelineLayout({ bindGroupLayouts: [] });
  const nativePipeline: GPURenderPipeline = hostDevice.createRenderPipeline({
    layout,
    vertex: {
      module: shader,
      entryPoint: square_VERTEX_ENTRY,
      buffers: [{
        arrayStride: square_VERTEX_LAYOUT0.arrayStride,
        stepMode: square_VERTEX_LAYOUT0.stepMode,
        attributes: [
          {
            format: square_VERTEX_LAYOUT0.attributes[0].format,
            offset: square_VERTEX_LAYOUT0.attributes[0].offset,
            shaderLocation: square_VERTEX_LAYOUT0.attributes[0].shaderLocation,
          },
          {
            format: square_VERTEX_LAYOUT0.attributes[1].format,
            offset: square_VERTEX_LAYOUT0.attributes[1].offset,
            shaderLocation: square_VERTEX_LAYOUT0.attributes[1].shaderLocation,
          },
        ],
      }],
    },
    primitive: {
      topology: square.topology,
      cullMode: square.cullMode,
      frontFace: square.frontFace,
    },
    fragment: {
      module: shader,
      entryPoint: square_FRAGMENT_ENTRY,
      targets: [{ format: square_TARGET_FORMAT }],
    },
  });
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    nativePipeline.dispose();
    indexBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  activeDevice = hostDevice;
  activeVertices = vertices;
  activeIndices = indexBuffer;
  activePipeline = new RenderPipeline(nativePipeline, square_INDEX_FORMAT);
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
  const indices: GPUBuffer | null = activeIndices;
  if (device === null) {
    return;
  }
  if (pipeline === null) {
    return;
  }
  if (vertices === null) {
    return;
  }
  if (indices === null) {
    return;
  }
  frameCount += 1;
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.03, g: 0.03, b: 0.045, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  pipeline.bind(pass, [], [vertices]);
  pipeline.setIndexBuffer(pass, indices);
  pass.drawIndexed(6, 1);
  pass.end();
  using command = encoder.finishDefault();
  using queue = device.queue();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeIndices !== null) {
    activeIndices.dispose();
    activeIndices = null;
  }
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
