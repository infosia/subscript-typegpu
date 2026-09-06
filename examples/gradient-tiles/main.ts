// example: gradient-tiles
// Quantizes a full-surface gradient into a fixed nine-by-nine grid of flat tiles.
// The two upstream span sliders become one committed tile count.
// Ported from TypeGPU's gradient-tiles example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  createRenderPipelineHost,
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

// The vertex schema. The generator lays it out for the vertex buffer and emits
// `Vertex_STRIDE`, the byte distance between two vertices.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The record that travels from the vertex stage to the fragment stage. The `position`
// field carries the clip-space position builtin, and `uv` takes location 0.
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

// The declaration pairs the two kernels and fixes the target format. The generator reads
// it and emits the WGSL and the constants of `main.typegpu` before the program runs.
export const tiles: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  tilesVertex,
  tilesFragment,
  { format: "bgra8unorm" },
);

// The host calls `init`, `frame`, and `shutdown` at different times, so the handles live
// at module scope. The script owns the pipeline and the buffer, and the host owns the device.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The declaration fixes the target format literally. A surface with another format ends
  // the example before it creates any resource.
  if (format !== tiles_TARGET_FORMAT) {
    print(`FAIL format expected=${tiles_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper adds the API-layer surface and
  // disposes neither.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // One oversized triangle. The two coordinates of 3.0 fall outside the clip volume, so the
  // visible part covers the whole surface with three vertices instead of six.
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ];
  // `Vertex_STRIDE` is a generated constant, so the size needs no hand arithmetic. VERTEX
  // admits the vertex buffer slot, and COPY_DST admits the queue write.
  const vertices = hostDevice.createBuffer({
    label: "gradient-tiles-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // One queue write uploads all three vertices. `Context.bytesOf` produces exactly the bytes
  // the generated vertex layout expects.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
  // The generated entry names, the generated vertex layout, and the declaration build the
  // pipeline. TypeGPU's `root.createRenderPipeline` covers the same step.
  hostDevice.pushErrorScope("validation");
  const createdPipeline = createRenderPipelineHost(
    hostDevice,
    tiles_WGSL,
    tiles_VERTEX_ENTRY,
    tiles_FRAGMENT_ENTRY,
    [],
    [tiles_VERTEX_LAYOUT0],
    tiles,
  );
  // The host-owned device pumps the event loop itself, so the scope result arrives without
  // an await.
  const validationError = hostDevice.popErrorScope();
  // Both handles belong to the script here, so it releases them before it reports the
  // failure and leaves the module state empty.
  if (validationError !== null) {
    createdPipeline.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The module state takes the handles only after validation passes, so `frame` never sees
  // a half-built pipeline.
  activeDevice = hostDevice;
  activeVertices = vertices;
  activePipeline = createdPipeline;
}

export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
  pointerX: f32,
  pointerY: f32,
  buttons: u32,
): void {
  // A failed `init` leaves the module state empty. The layers carry no exceptions, so a null
  // check ends the frame where TypeGPU throws.
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
  // The host acquires and presents the surface texture. The wrapper borrows the view for one
  // frame, and the script disposes neither.
  const target = new GPUTextureView(view);
  // One encoder per frame records the render pass. The triangle covers every pixel, so the
  // clear color never stays on the surface.
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The window size changes between frames, so the pass takes the viewport and the scissor
  // from the size the host reports.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and each vertex buffer at its full size. The
  // tile count is a constant, so this pipeline has no binding and the group list is empty.
  pipeline.bind(pass, [], [vertices]);
  pass.draw(3);
  pass.end();
  // The GPU runs nothing until the queue receives the command buffer. The host presents the
  // surface after this call returns.
  using command = encoder.finishDefault();
  using queue = device.queue();
  queue.submit([command]);
}

// The host calls this one time before it releases the device. TypeGPU frees the same kind of
// resource through `root.destroy`, and this layer disposes each handle by name.
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
