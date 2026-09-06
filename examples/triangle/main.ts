// example: triangle
// Draws one triangle and blends three corner colors that the vertex index selects.
// Ported from TypeGPU's triangle example (https://github.com/software-mansion/TypeGPU).

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
  Vec3f,
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
  triangle_FRAGMENT_ENTRY,
  triangle_TARGET_FORMAT,
  triangle_VERTEX_ENTRY,
  triangle_VERTEX_LAYOUT0,
  triangle_WGSL,
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
// field carries the clip-space position builtin, and `color` takes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  color: Vec3f;

  constructor(position: Vec4f, color: Vec3f) {
    this.position = position;
    this.color = color;
  }
}

// TypeGPU reads three positions from a constant array and mixes two colors over
// the uv. This port streams the positions through a vertex buffer and picks one
// color per corner.
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

// The fragment kernel. The rasterizer interpolates `color` across the triangle, so three
// corner colors produce the gradient.
function triangleFragment(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0);
}

// The declaration pairs the two entry points and fixes the target format. The
// generator walks it and emits the WGSL and the constants of `main.typegpu`.
export const triangle: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  triangleVertex,
  triangleFragment,
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
  if (format !== triangle_TARGET_FORMAT) {
    print(`FAIL format expected=${triangle_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper adds the API-layer surface and
  // disposes neither.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // Three clip-space corners. The vertex index the kernel reads follows this order.
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-0.72, -0.58)),
    new Vertex(new Vec2f(0.7, -0.5)),
    new Vertex(new Vec2f(-0.05, 0.76)),
  ];
  const vertices = hostDevice.createBuffer({
    label: "example-triangle-vertices",
    // `Vertex_STRIDE` is a generated constant. TypeGPU computes the same number at run
    // time from the schema object.
    size: (Vertex_STRIDE * 3) as u64,
    // VERTEX admits the vertex buffer slot, and COPY_DST admits the queue write below.
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
    triangle_WGSL,
    triangle_VERTEX_ENTRY,
    triangle_FRAGMENT_ENTRY,
    [],
    [triangle_VERTEX_LAYOUT0],
    triangle,
  );
  // The host-owned device pumps the event loop itself, so the scope result arrives without
  // an await. The device lane awaits the same call.
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

// The window host owns the surface and the loop, and it calls this once per frame.
// TypeGPU's example configures a canvas context and draws one time.
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
  // One encoder per frame records the render pass. The clear load fills the whole attachment,
  // so the frame needs no separate clear step.
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.025, g: 0.035, b: 0.065, a: 1.0 },
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
