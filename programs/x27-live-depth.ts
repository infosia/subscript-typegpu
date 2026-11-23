// program: x27-live-depth
// purpose: compare both draw orders of two opaque triangles against a host depth test
// exercises: BF1, BF2, PI14, RN1, RN2, RN7, RN10, RN11, RN14, RN22, T4, T19
// questions: none

import {
  Buffer,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  renderPipeline,
  VertexInvocation,
} from "./typegpu";
import { Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBuffer,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  terrain_FRAGMENT_ENTRY,
  terrain_VERTEX_ENTRY,
  terrain_VERTEX_LAYOUT0,
  terrain_WGSL,
  Vertex_STRIDE,
} from "./x27-live-depth.typegpu";

const SIZE: u32 = 64;
const ROW_BYTES: u32 = 256;

@ValueType
class Vertex {
  constructor(public position: Vec4f, public color: Vec4f) {}
}

@ValueType
class Varyings {
  constructor(public position: Vec4f, public color: Vec4f) {}
}

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(value.position, value.color);
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return value.color;
}

const terrain: RenderPipelineSpec = renderPipeline(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    depthStencil: {
      format: "depth24plus",
      depthWriteEnabled: true,
      depthCompare: "less",
    },
  },
);

function edge(a: Vec4f, b: Vec4f, x: f32, y: f32): f32 {
  return (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x);
}

function checkPixels(pixels: u8[], vertices: FixedArray<Vertex, 6>, first: i32): boolean {
  let y: i32 = 0;
  while (y < (SIZE as i32)) {
    const py: f32 = 1.0 - ((y as f32) + 0.5) * (2.0 / (SIZE as f32));
    let x: i32 = 0;
    while (x < (SIZE as i32)) {
      const px: f32 = ((x as f32) + 0.5) * (2.0 / (SIZE as f32)) - 1.0;
      let depth: f32 = 1.0;
      let color = new Vec4f(0.0, 0.0, 0.0, 0.0);
      let draw: i32 = 0;
      while (draw < 2) {
        const base: i32 = draw == 0 ? first : 3 - first;
        const a = vertices[base].position;
        const b = vertices[base + 1].position;
        const c = vertices[base + 2].position;
        // Both triangles have counterclockwise vertices and constant depth.
        if (edge(a, b, px, py) > 0.0 && edge(b, c, px, py) > 0.0
          && edge(c, a, px, py) > 0.0 && a.z < depth) {
          depth = a.z;
          color = vertices[base].color;
        }
        draw += 1;
      }
      const r: u8 = (color.x * 255.0) as u8;
      const g: u8 = (color.y * 255.0) as u8;
      const b: u8 = (color.z * 255.0) as u8;
      const a: u8 = (color.w * 255.0) as u8;
      const offset = y * (ROW_BYTES as i32) + x * 4;
      if (pixels[offset] != r || pixels[offset + 1] != g
        || pixels[offset + 2] != b || pixels[offset + 3] != a) {
        print(`FAIL x=${x} y=${y} expected=${r},${g},${b},${a} got=${pixels[offset]},${pixels[offset + 1]},${pixels[offset + 2]},${pixels[offset + 3]}`);
        return false;
      }
      x += 1;
    }
    y += 1;
  }
  return true;
}

async function checkOrder(
  device: GPUDevice,
  pipeline: RenderPipeline,
  vertexBuffer: GPUBuffer,
  vertices: FixedArray<Vertex, 6>,
  first: i32,
): Promise<boolean> {
  using color = device.createTexture({
    size: { width: SIZE, height: SIZE },
    format: terrain.format,
    usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
  });
  using depth = device.createTexture({
    size: { width: SIZE, height: SIZE },
    format: "depth24plus",
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
  });
  using colorView = color.createView();
  using depthView = depth.createView();
  const byteCount: u64 = (ROW_BYTES * SIZE) as u64;
  using readback = device.createBuffer({
    size: byteCount,
    usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
  });
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: colorView,
      clearValue: { r: 0, g: 0, b: 0, a: 0 },
      loadOp: "clear",
      storeOp: "store",
    }],
    depthStencilAttachment: {
      view: depthView,
      depthClearValue: 1.0,
      depthLoadOp: "clear",
      depthStoreOp: "store",
    },
  });
  pipeline.bind(pass, [], [vertexBuffer]);
  pass.draw(3, 1, first as u32);
  pass.draw(3, 1, (3 - first) as u32);
  pass.end();
  encoder.copyTextureToBuffer(
    { texture: color },
    { buffer: readback, bytesPerRow: ROW_BYTES, rowsPerImage: SIZE },
    { width: SIZE, height: SIZE },
  );
  using command = encoder.finishDefault();
  device.queue.submit([command]);
  print(`draw:submitted firstVertex=${first}`);
  if (!await readback.mapAsync(GPUMapMode.READ, 0, byteCount)) {
    print("FAIL map");
    return false;
  }
  const pixels: u8[] = readback.readMappedRange(0, byteCount);
  readback.unmap();
  print("readback:complete");
  return checkPixels(pixels, vertices, first);
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  print("adapter:ready");
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  print("device:ready");
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const nearZ: f32 = 0.25;
    const farZ: f32 = 0.75;
    const red = new Vec4f(1.0, 0.0, 0.0, 1.0);
    const blue = new Vec4f(0.0, 0.0, 1.0, 1.0);
    // In texels, the legs follow grid lines. Each diagonal rises one texel per two columns.
    const vertices: FixedArray<Vertex, 6> = [
      new Vertex(new Vec4f(-0.875, -0.625, nearZ, 1.0), red),
      new Vertex(new Vec4f( 0.625, -0.625, nearZ, 1.0), red),
      new Vertex(new Vec4f(-0.875,  0.125, nearZ, 1.0), red),
      new Vertex(new Vec4f(-0.625, -0.375, farZ, 1.0), blue),
      new Vertex(new Vec4f( 0.875, -0.375, farZ, 1.0), blue),
      new Vertex(new Vec4f(-0.625,  0.375, farZ, 1.0), blue),
    ];
    using buffer: Buffer<Vertex> = createBuffer(
      device, Vertex_STRIDE, 6, GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST, "x27-vertices",
    );
    buffer.write(device.queue, 0, Context.bytesOf<FixedArray<Vertex, 6>>(vertices));
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      terrain_WGSL,
      terrain_VERTEX_ENTRY,
      terrain_FRAGMENT_ENTRY,
      [],
      [terrain_VERTEX_LAYOUT0],
      terrain,
    );
    const validationError = await device.popErrorScope();
    if (validationError != null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    print("pipeline:created");
    if (!await checkOrder(device, pipeline, buffer.handle(), vertices, 0)) return;
    if (!await checkOrder(device, pipeline, buffer.handle(), vertices, 3)) return;
  }
  gpu.dispose();
  print("PASS");
}
