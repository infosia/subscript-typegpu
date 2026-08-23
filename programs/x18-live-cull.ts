// program: x18-live-cull
// purpose: prove indexed winding and back-face culling against a signed-area host rasterizer
// exercises: BF9, PI14, RN18, RN19
// questions: none

import {
  Buffer,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipelineSpec,
  renderPipeline,
  VertexInvocation,
} from "./typegpu";
import {
  Vec2f,
  Vec4f,
} from "./typegpu-types";
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
  culled_FRAGMENT_ENTRY,
  culled_TARGET_FORMAT,
  culled_VERTEX_ENTRY,
  culled_VERTEX_LAYOUT0,
  culled_WGSL,
  Vertex_STRIDE,
} from "./x18-live-cull.typegpu";

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

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
  );
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(1.0, 0.0, 0.0, 1.0);
}

export const culled: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    indexFormat: "uint16",
    cullMode: "back",
    frontFace: "ccw",
  },
);

function signedArea(a: Vec2f, b: Vec2f, c: Vec2f): f32 {
  return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
}

function center(x: i32, y: i32): Vec2f {
  return new Vec2f(
    ((x as f32) + 0.5) / 8.0 * 2.0 - 1.0,
    1.0 - ((y as f32) + 0.5) / 8.0 * 2.0,
  );
}

function edge(a: Vec2f, b: Vec2f, p: Vec2f): f32 {
  return (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x);
}

function covered(
  p: Vec2f,
  a: Vec2f,
  b: Vec2f,
  c: Vec2f,
  cullMode: GPUCullMode,
  frontFace: GPUFrontFace,
): boolean {
  const area: f32 = signedArea(a, b, c);
  const front: boolean = frontFace === "ccw" ? area > 0.0 : area < 0.0;
  if ((cullMode === "back" && !front) || (cullMode === "front" && front)) {
    return false;
  }
  const e0 = edge(a, b, p);
  const e1 = edge(b, c, p);
  const e2 = edge(c, a, p);
  return (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0)
    || (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0);
}

async function checkImage(
  readback: GPUBuffer,
  a: Vec2f,
  b: Vec2f,
  c: Vec2f,
  label: string,
): Promise<boolean> {
  if (!await readback.mapAsync(GPUMapMode.READ, 0, 2048)) {
    print(`FAIL ${label} map`);
    return false;
  }
  const pixels: u8[] = readback.readMappedRange(0, 2048);
  let y: i32 = 0;
  while (y < 8) {
    let x: i32 = 0;
    while (x < 8) {
      const hit = covered(
        center(x, y),
        a,
        b,
        c,
        culled.cullMode,
        culled.frontFace,
      );
      const expectedR: u8 = hit ? 255 : 0;
      const o = y * 256 + x * 4;
      if (pixels[o] !== expectedR
        || pixels[o + 1] !== 0
        || pixels[o + 2] !== 0
        || pixels[o + 3] !== 255) {
        print(`FAIL ${label} ${x},${y}`);
        return false;
      }
      x += 1;
    }
    y += 1;
  }
  readback.unmap();
  return true;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const a = new Vec2f(-0.65, -0.55);
    const b = new Vec2f(0.65, -0.55);
    const c = new Vec2f(0.0, 0.65);
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x18-vertices",
    );
    vertices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Vertex, 3>>([
        new Vertex(a),
        new Vertex(b),
        new Vertex(c),
      ]),
    );
    using indices: Buffer<u16> = createBuffer<u16>(
      device,
      2,
      4,
      GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "x18-indices",
    );
    using first = device.createTexture({
      label: "x18-first",
      size: { width: 8, height: 8 },
      format: culled_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using second = device.createTexture({
      label: "x18-second",
      size: { width: 8, height: 8 },
      format: culled_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using firstRead = device.createBuffer({
      label: "x18-first-read",
      size: 2048,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    using secondRead = device.createBuffer({
      label: "x18-second-read",
      size: 2048,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      culled_WGSL,
      culled_VERTEX_ENTRY,
      culled_FRAGMENT_ENTRY,
      [],
      [culled_VERTEX_LAYOUT0],
      culled,
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    indices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<u16, 4>>([0, 1, 2, 0]),
    );
    using firstView = first.createView();
    using encoder1 = device.createCommandEncoderDefault();
    using pass1 = encoder1.beginRenderPass({
      colorAttachments: [{
        view: firstView,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass1, [], [vertices.handle()]);
    pipeline.setIndexBuffer(pass1, indices.handle());
    pass1.drawIndexed(3);
    pass1.end();
    encoder1.copyTextureToBuffer(
      { texture: first },
      { buffer: firstRead, bytesPerRow: 256, rowsPerImage: 8 },
      { width: 8, height: 8 },
    );
    using command1 = encoder1.finishDefault();
    device.queue().submit([command1]);
    if (!await device.queue().onSubmittedWorkDone()) {
      print("FAIL first submit");
      return;
    }
    indices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<u16, 4>>([0, 2, 1, 0]),
    );
    const indexBytes: u8[] = await indices.read(device, 0, 4);
    const order: FixedArray<u16, 4> = Context.fromBytes<FixedArray<u16, 4>>(
      indexBytes,
      0,
    );
    if (order[0] !== 0 || order[1] !== 2 || order[2] !== 1) {
      print("FAIL index readback");
      return;
    }
    using secondView = second.createView();
    using encoder2 = device.createCommandEncoderDefault();
    using pass2 = encoder2.beginRenderPass({
      colorAttachments: [{
        view: secondView,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass2, [], [vertices.handle()]);
    pipeline.setIndexBuffer(pass2, indices.handle());
    pass2.drawIndexed(3);
    pass2.end();
    encoder2.copyTextureToBuffer(
      { texture: second },
      { buffer: secondRead, bytesPerRow: 256, rowsPerImage: 8 },
      { width: 8, height: 8 },
    );
    using command2 = encoder2.finishDefault();
    device.queue().submit([command2]);
    if (!await device.queue().onSubmittedWorkDone()) {
      print("FAIL second submit");
      return;
    }
    // Pixel centres are converted to NDC with Y flipped. The ccw signed area is the host front face.
    if (!await checkImage(firstRead, a, b, c, "front")) {
      return;
    }
    if (!await checkImage(secondRead, a, c, b, "back")) {
      return;
    }
  }
  gpu.dispose();
  print("PASS");
}
