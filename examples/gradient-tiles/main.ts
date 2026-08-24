// example: gradient-tiles
// Quantizes a full-surface gradient into a fixed nine-by-nine grid of flat tiles.
// The two upstream span sliders become one committed tile count.
// Ported from TypeGPU's gradient-tiles example (https://github.com/software-mansion/TypeGPU).

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
  tiles_FRAGMENT_ENTRY,
  tiles_TARGET_FORMAT,
  tiles_VERTEX_ENTRY,
  tiles_VERTEX_LAYOUT0,
  tiles_WGSL,
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
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// TypeGPU holds both spans in a `vec2f` uniform and writes it on every slider move.
// One constant replaces that uniform here, so the fragment binds no resources.
const TILE_COUNT: f32 = 9.0;

// TypeGPU draws its full-surface triangle with `common.fullScreenTriangle`. This port
// declares the same three clip-space vertices and maps them into uv space.
function tilesVertex(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// TypeGPU computes `floor(uv * span) / span`. This port adds a half-tile offset, so
// each tile shows the color of its own center.
function tilesFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  const cell: Vec2f = input.uv.scale(TILE_COUNT).floor();
  const red: f32 = (cell.x + 0.5) / TILE_COUNT;
  const green: f32 = (cell.y + 0.5) / TILE_COUNT;
  return new Vec4f(red, green, 0.32, 1.0);
}

export const tiles: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  tilesVertex,
  tilesFragment,
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
  if (format !== tiles_TARGET_FORMAT) {
    print(`FAIL format expected=${tiles_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ];
  const vertices = hostDevice.createBuffer({
    label: "gradient-tiles-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
  // The host owns the device, so this example builds the pipeline from the generated
  // entry names and the generated vertex layout. TypeGPU's `root.createRenderPipeline`
  // covers the same step.
  hostDevice.pushErrorScope("validation");
  using shader = hostDevice.createShaderModule({ code: tiles_WGSL });
  using layout = hostDevice.createPipelineLayout({ bindGroupLayouts: [] });
  const nativePipeline: GPURenderPipeline = hostDevice.createRenderPipeline({
    layout,
    vertex: {
      module: shader,
      entryPoint: tiles_VERTEX_ENTRY,
      buffers: [{
        arrayStride: tiles_VERTEX_LAYOUT0.arrayStride,
        stepMode: tiles_VERTEX_LAYOUT0.stepMode,
        attributes: [{
          format: tiles_VERTEX_LAYOUT0.attributes[0].format,
          offset: tiles_VERTEX_LAYOUT0.attributes[0].offset,
          shaderLocation: tiles_VERTEX_LAYOUT0.attributes[0].shaderLocation,
        }],
      }],
    },
    primitive: {
      topology: tiles.topology,
      cullMode: tiles.cullMode,
      frontFace: tiles.frontFace,
    },
    fragment: {
      module: shader,
      entryPoint: tiles_FRAGMENT_ENTRY,
      targets: [{ format: tiles_TARGET_FORMAT }],
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
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
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
