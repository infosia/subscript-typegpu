// example: window-triangle
// Renders one triangle on the host-owned surface and changes the clear color on space.
// This example ports no upstream program. It is the smallest windowed program.

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  createRenderPipelineHost,
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
// field carries the clip-space position builtin.
@CStruct
class Varyings {
  position: Vec4f;

  constructor(position: Vec4f) {
    this.position = position;
  }
}

// The vertex kernel. It raises the two-dimensional position into clip space. A w of 1.0
// leaves x and y unchanged after the perspective divide.
function vert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0));
}

// The fragment kernel. One constant color fills the triangle, so this pipeline needs no
// binding and no bind group.
function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(0.95, 0.45, 0.15, 1.0);
}

// The declaration pairs the two kernels and fixes the target format. The generator reads
// it and emits the WGSL and the constants of `main.typegpu` before the program runs.
export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, {
  format: "bgra8unorm",
});

// The host calls `init`, `frame`, and `shutdown` at different times, so the handles live
// at module scope. The script owns the pipeline and the buffer, and the host owns the device.
let ownedDevice: GPUHostOwnedDevice | null = null;
let pipeline: RenderPipeline | null = null;
let vertexBuffer: GPUBuffer | null = null;
let clearIndex: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The declaration fixes the target format literally. A surface with another format ends
  // the example before it creates any resource.
  if (format !== tri_TARGET_FORMAT) {
    print(`FAIL format expected=${tri_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper adds the API-layer surface and
  // disposes neither.
  const deviceWrapper: GPUHostOwnedDevice = hostOwnedGPUDevice(instance, device);
  // Three clip-space corners. The example writes them one time, because the geometry never
  // changes.
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-0.65, -0.55)),
    new Vertex(new Vec2f(0.65, -0.55)),
    new Vertex(new Vec2f(0.0, 0.7)),
  ];
  // `Vertex_STRIDE` is a generated constant, so the size needs no hand arithmetic. VERTEX
  // admits the vertex buffer slot, and COPY_DST admits the queue write.
  const vertices: GPUBuffer = deviceWrapper.createBuffer({
    label: "window-triangle-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // One queue write uploads all three vertices. `Context.bytesOf` produces exactly the bytes
  // the generated vertex layout expects.
  using queue = deviceWrapper.queue();
  queue.writeBuffer(
    vertices,
    0,
    Context.bytesOf<FixedArray<Vertex, 3>>(values),
  );
  // The scope catches a validation error from the pipeline creation. The host-owned device
  // pumps the event loop itself, so the error arrives without an await.
  deviceWrapper.pushErrorScope("validation");
  const createdPipeline = createRenderPipelineHost(
    deviceWrapper,
    tri_WGSL,
    tri_VERTEX_ENTRY,
    tri_FRAGMENT_ENTRY,
    [],
    [tri_VERTEX_LAYOUT0],
    tri,
  );
  const validationError = deviceWrapper.popErrorScope();
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
  ownedDevice = deviceWrapper;
  vertexBuffer = vertices;
  pipeline = createdPipeline;
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
  // The host delivers one key per frame, and 32 is the space scalar. Each press advances the
  // clear color by one step of three.
  if (key === 32) {
    clearIndex = (clearIndex + 1) % 3;
  }
  // The host acquires and presents the surface texture. The wrapper borrows the view for one
  // frame, and the script disposes neither.
  const target = new GPUTextureView(view);
  // One encoder per frame records the render pass. The clear load fills the whole attachment,
  // so the frame needs no separate clear step.
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
  // The window size changes between frames, so the pass takes the viewport and the scissor
  // from the size the host reports.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and each vertex buffer at its full size. This
  // pipeline has no binding, so the group list is empty.
  activePipeline.bind(pass, [], [activeVertices]);
  pass.draw(3);
  pass.end();
  // The GPU runs nothing until the queue receives the command buffer. The host presents the
  // surface after this call returns.
  using command = encoder.finishDefault();
  using queue = activeDevice.queue();
  queue.submit([command]);
}

// The host calls this one time before it releases the device. TypeGPU frees the same kind of
// resource through `root.destroy`, and this layer disposes each handle by name.
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
