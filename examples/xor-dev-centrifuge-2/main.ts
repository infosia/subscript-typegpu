// example: xor-dev-centrifuge-2
// Animates a polar tunnel from layered trigonometric bands and the host frame count.
// Ported from TypeGPU's xor-dev-centrifuge-2 example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Uniform,
  VertexInvocation,
  WgslShellSpec,
  renderPipelineL,
  wgslShell,
} from "./typegpu";
import {
  Vec2f,
  Vec3f,
  Vec4f,
} from "./typegpu-types";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPURenderPipeline,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  FrameData_SIZE,
  Vertex_STRIDE,
  tunnel_FRAGMENT_ENTRY,
  tunnel_LAYOUT0,
  tunnel_TARGET_FORMAT,
  tunnel_VERTEX_ENTRY,
  tunnel_VERTEX_LAYOUT0,
  tunnel_WGSL,
} from "./main.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class FrameData {
  values: Vec4f;

  constructor(time: f32, aspect: f32) {
    this.values = new Vec4f(time, aspect, 0.0, 0.0);
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

class TunnelLayout {
  frame!: Uniform<FrameData>;
}

function tunnelBands(point: Vec2f, time: f32): Vec3f {
  const radius: f32 = point.length();
  const pulse: f32 = 0.5 + 0.5 * (Math.sin((radius * 8.0 + time) as f64) as f32);
  return new Vec3f(pulse * 0.25, pulse * 0.08, pulse * 0.55);
}

// The shell uses a compact polar loop because the typed surface has no scalar atan2 mapping.
const tunnelBandsGpu: WgslShellSpec = wgslShell<(point: Vec2f, time: f32) => Vec3f>(
  tunnelBands,
  {
    body: "let radius = length(point); let angle = atan2(point.y, point.x); var color = vec3f(0.0); for (var layer = 0u; layer < 12u; layer = layer + 1u) { let depth = f32(layer) * 0.21 + time * 0.35; let ring = 0.045 / (abs(sin(radius * 11.0 - depth * 3.0)) + 0.07); let spoke = 0.35 + 0.65 * cos(angle * 7.0 + depth * 2.0); let fade = 1.0 / (1.0 + f32(layer) * 0.32); color = color + vec3f(0.22, 0.05, 0.48) * ring * spoke * fade; } return tanh(color * 0.09);",
  },
);

function tunnelVertex(
  res: TunnelLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

function tunnelFragment(
  res: TunnelLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: FrameData = res.frame.get();
  const centered = new Vec2f(
    (input.uv.x * 2.0 - 1.0) * frame.values.y,
    input.uv.y * 2.0 - 1.0,
  );
  const color: Vec3f = tunnelBands(centered, frame.values.x);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const tunnel: RenderPipelineSpec = renderPipelineL<TunnelLayout, Vertex, Varyings>(
  tunnelVertex,
  tunnelFragment,
  { format: "bgra8unorm" },
);

let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== tunnel_TARGET_FORMAT) {
    print(`FAIL format expected=${tunnel_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const values: FixedArray<Vertex, 3> = [
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ];
  const vertices = hostDevice.createBuffer({
    label: "centrifuge-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "centrifuge-frame",
    size: FrameData_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<FrameData>(new FrameData(0.0, 1.0)));
  hostDevice.pushErrorScope("validation");
  using shader = hostDevice.createShaderModule({ code: tunnel_WGSL });
  using bindLayout = hostDevice.createBindGroupLayout({
    entries: [{
      binding: tunnel_LAYOUT0.entries[0].binding,
      visibility: tunnel_LAYOUT0.entries[0].visibility,
      buffer: {
        type: "uniform",
        minBindingSize: tunnel_LAYOUT0.entries[0].minBindingSize,
      },
    }],
  });
  using layout = hostDevice.createPipelineLayout({ bindGroupLayouts: [bindLayout] });
  const nativePipeline: GPURenderPipeline = hostDevice.createRenderPipeline({
    layout,
    vertex: {
      module: shader,
      entryPoint: tunnel_VERTEX_ENTRY,
      buffers: [{
        arrayStride: tunnel_VERTEX_LAYOUT0.arrayStride,
        stepMode: tunnel_VERTEX_LAYOUT0.stepMode,
        attributes: [{
          format: tunnel_VERTEX_LAYOUT0.attributes[0].format,
          offset: tunnel_VERTEX_LAYOUT0.attributes[0].offset,
          shaderLocation: tunnel_VERTEX_LAYOUT0.attributes[0].shaderLocation,
        }],
      }],
    },
    primitive: {
      topology: tunnel.topology,
      cullMode: tunnel.cullMode,
      frontFace: tunnel.frontFace,
    },
    fragment: {
      module: shader,
      entryPoint: tunnel_FRAGMENT_ENTRY,
      targets: [{ format: tunnel_TARGET_FORMAT }],
    },
  });
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    nativePipeline.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  const group = hostDevice.createBindGroup({
    layout: bindLayout,
    entries: [{
      binding: tunnel_LAYOUT0.entries[0].binding,
      buffer: frameBuffer,
      size: FrameData_SIZE as u64,
    }],
  });
  activeDevice = hostDevice;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeGroup = group;
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
  const frameBuffer: GPUBuffer | null = activeFrameBuffer;
  const group: GPUBindGroup | null = activeGroup;
  if (device === null) {
    return;
  }
  if (pipeline === null) {
    return;
  }
  if (vertices === null) {
    return;
  }
  if (frameBuffer === null) {
    return;
  }
  if (group === null) {
    return;
  }
  frameCount += 1;
  const aspect: f32 = (width as f32) / (height as f32);
  using queue = device.queue();
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<FrameData>(new FrameData(frameCount as f32 / 60.0, aspect)),
  );
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
  pipeline.bind(pass, [group], [vertices]);
  pass.draw(3);
  pass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeGroup !== null) {
    activeGroup.dispose();
    activeGroup = null;
  }
  if (activeFrameBuffer !== null) {
    activeFrameBuffer.dispose();
    activeFrameBuffer = null;
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
