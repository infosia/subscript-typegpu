// example: square
// Draws an indexed quad and interpolates four committed corner colors.
// The upstream color pickers become fixed vertex colors.
// Ported from TypeGPU's square example (https://github.com/software-mansion/TypeGPU).

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
  square_FRAGMENT_ENTRY,
  square_TARGET_FORMAT,
  square_VERTEX_ENTRY,
  square_VERTEX_LAYOUT0,
  square_WGSL,
} from "./main.typegpu";

// The vertex schema. The generator lays out both fields for the vertex buffer and emits
// `Vertex_STRIDE`, the byte distance between two vertices.
@CStruct
class Vertex {
  position: Vec2f;
  color: Vec4f;

  constructor(position: Vec2f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

// The record that travels from the vertex stage to the fragment stage. The `position`
// field carries the clip-space position builtin, and `color` takes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  color: Vec4f;

  constructor(position: Vec4f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

// TypeGPU keeps the four positions inside the shader and streams only the colors.
// This port streams position and color together as one `Vertex` record.
function squareVertex(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.color,
  );
}

// The fragment kernel. The rasterizer interpolates the four corner colors over both
// triangles of the quad.
function squareFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return input.color;
}

// The index format belongs to the declaration. TypeGPU attaches the index buffer to
// the pipeline with `withIndexBuffer` instead.
export const square: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  squareVertex,
  squareFragment,
  { format: "bgra8unorm", indexFormat: "uint16" },
);

// The host calls `init`, `frame`, and `shutdown` at different times, so the handles live
// at module scope. The script owns the three handles, and the host owns the device.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeIndices: GPUBuffer | null = null;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The declaration fixes the target format literally. A surface with another format ends
  // the example before it creates any resource.
  if (format !== square_TARGET_FORMAT) {
    print(`FAIL format expected=${square_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper adds the API-layer surface and
  // disposes neither.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The four corners, bottom row first. The index list below names them in this order.
  const values: FixedArray<Vertex, 4> = [
    new Vertex(new Vec2f(-0.68, -0.68), new Vec4f(0.95, 0.15, 0.15, 1.0)),
    new Vertex(new Vec2f(0.68, -0.68), new Vec4f(0.15, 0.9, 0.2, 1.0)),
    new Vertex(new Vec2f(-0.68, 0.68), new Vec4f(0.2, 0.25, 0.95, 1.0)),
    new Vertex(new Vec2f(0.68, 0.68), new Vec4f(0.95, 0.85, 0.15, 1.0)),
  ];
  // The winding follows this port's vertex order. TypeGPU lists `[0, 2, 1, 0, 3, 2]`
  // for its own corner order.
  const indices: FixedArray<u16, 6> = [0, 1, 2, 2, 1, 3];
  // `Vertex_STRIDE` is a generated constant, so the size needs no hand arithmetic. VERTEX
  // admits the vertex buffer slot, and COPY_DST admits the queue write.
  const vertices = hostDevice.createBuffer({
    label: "example-square-vertices",
    size: (Vertex_STRIDE * 4) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // Six `uint16` indices occupy 12 bytes. The declaration fixes that index format, so the
  // element width and the buffer size agree.
  const indexBuffer = hostDevice.createBuffer({
    label: "example-square-indices",
    size: 12,
    usage: GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST,
  });
  // Two queue writes upload the vertices and the indices. `Context.bytesOf` produces exactly
  // the bytes each generated layout expects.
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 4>>(values));
  queue.writeBuffer(indexBuffer, 0, Context.bytesOf<FixedArray<u16, 6>>(indices));
  // The generated entry names, the generated vertex layout, and the declaration build the
  // pipeline. The scope catches a validation error from that call.
  hostDevice.pushErrorScope("validation");
  const createdPipeline = createRenderPipelineHost(
    hostDevice,
    square_WGSL,
    square_VERTEX_ENTRY,
    square_FRAGMENT_ENTRY,
    [],
    [square_VERTEX_LAYOUT0],
    square,
  );
  // The host-owned device pumps the event loop itself, so the scope result arrives without
  // an await.
  const validationError = hostDevice.popErrorScope();
  // All three handles belong to the script here, so it releases them before it reports the
  // failure and leaves the module state empty.
  if (validationError !== null) {
    createdPipeline.dispose();
    indexBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The module state takes the handles only after validation passes, so `frame` never sees
  // a half-built pipeline.
  activeDevice = hostDevice;
  activeVertices = vertices;
  activeIndices = indexBuffer;
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
  // The host acquires and presents the surface texture. The wrapper borrows the view for one
  // frame, and the script disposes neither.
  const target = new GPUTextureView(view);
  // One encoder per frame records the render pass. The clear load fills the whole attachment,
  // so the frame needs no separate clear step.
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.03, g: 0.03, b: 0.045, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The window size changes between frames, so the pass takes the viewport and the scissor
  // from the size the host reports.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and each vertex buffer at its full size. This
  // pipeline has no binding, so the group list is empty.
  pipeline.bind(pass, [], [vertices]);
  // The pipeline supplies the index format that the declaration fixed. It traps when
  // that format is undefined.
  pipeline.setIndexBuffer(pass, indices);
  // Six indices form two triangles, and one instance draws the quad one time.
  pass.drawIndexed(6, 1);
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
