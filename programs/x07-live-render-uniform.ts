// program: x07-live-render-uniform
// purpose: compare a uniform-offset and storage-tinted triangle with a host rasterizer
// exercises: RN1-RN14, RN17, T4, T15, render bindings and readback
// questions: none

import {
  Buffer,
  createBindGroup,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipelineSpec,
  renderPipelineL,
  Storage,
  Uniform,
  VertexInvocation,
  bufferResource,
} from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  Offset_SIZE,
  shifted_FRAGMENT_ENTRY,
  shifted_LAYOUT0,
  shifted_TARGET_FORMAT,
  shifted_VERTEX_ENTRY,
  shifted_VERTEX_LAYOUT0,
  shifted_WGSL,
  Tint_SIZE,
  Vertex_STRIDE,
} from "./x07-live-render-uniform.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class Offset {
  value: Vec4f;

  constructor(value: Vec4f) {
    this.value = value;
  }
}

@CStruct
class Tint {
  value: Vec4f;

  constructor(value: Vec4f) {
    this.value = value;
  }
}

@CStruct
class Varyings {
  position: Vec4f;

  constructor(position: Vec4f) {
    this.position = position;
  }
}

class RenderLayout {
  params!: Uniform<Offset>;
  tint!: Storage<Tint>;
}

function vert(res: RenderLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  const offset: Offset = res.params.get();
  return new Varyings(
    new Vec4f(
      value.position.x + offset.value.x,
      value.position.y + offset.value.y,
      0.0,
      1.0,
    ),
  );
}

function frag(res: RenderLayout, input: Varyings, ctx: FragmentInvocation): Vec4f {
  const color: Tint = res.tint[0];
  return color.value;
}

export const shifted: RenderPipelineSpec = renderPipelineL<RenderLayout, Vertex, Varyings>(
  vert,
  frag,
  { format: "rgba8unorm" },
);

function edge(a: Vec2f, b: Vec2f, point: Vec2f): f32 {
  return (point.x - a.x) * (b.y - a.y) - (point.y - a.y) * (b.x - a.x);
}

function pixelCenter(x: i32, y: i32): Vec2f {
  // Texel row 0 maps to clip-space y=+1.
  // Each later row lowers the clip-space y coordinate.
  return new Vec2f(
    ((x as f32) + 0.5) * (2.0 / 64.0) - 1.0,
    1.0 - ((y as f32) + 0.5) * (2.0 / 64.0),
  );
}

function insideTriangle(point: Vec2f, a: Vec2f, b: Vec2f, c: Vec2f): boolean {
  const ab: f32 = edge(a, b, point);
  const bc: f32 = edge(b, c, point);
  const ca: f32 = edge(c, a, point);
  return (ab > 0.0 && bc > 0.0 && ca > 0.0)
    || (ab < 0.0 && bc < 0.0 && ca < 0.0);
}

function noCenterOnEdge(a: Vec2f, b: Vec2f, c: Vec2f): boolean {
  let y: i32 = 0;
  while (y < 64) {
    let x: i32 = 0;
    while (x < 64) {
      const point: Vec2f = pixelCenter(x, y);
      if (edge(a, b, point) === 0.0
        || edge(b, c, point) === 0.0
        || edge(c, a, point) === 0.0) {
        return false;
      }
      x = x + 1;
    }
    y = y + 1;
  }
  return true;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  print("adapter:ready");
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  print("device:ready");
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const offset = new Vec2f(0.11, -0.07);
    const a = new Vec2f(-0.49, -0.67);
    const b = new Vec2f(0.71, -0.67);
    const c = new Vec2f(0.11, 0.63);
    if (!noCenterOnEdge(a, b, c)) {
      print("FAIL pixel center on edge");
      return;
    }
    const values: FixedArray<Vertex, 3> = [
      new Vertex(new Vec2f(a.x - offset.x, a.y - offset.y)),
      new Vertex(new Vec2f(b.x - offset.x, b.y - offset.y)),
      new Vertex(new Vec2f(c.x - offset.x, c.y - offset.y)),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x07-vertices",
    );
    using params = device.createBuffer({
      label: "x07-params",
      size: Offset_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    });
    using tint = device.createBuffer({
      label: "x07-tint",
      size: Tint_SIZE as u64,
      usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
    });
    vertices.write(device.queue(), 0, Context.bytesOf<FixedArray<Vertex, 3>>(values));
    device.queue().writeBuffer(
      params,
      0,
      Context.bytesOf<Offset>(new Offset(new Vec4f(offset.x, offset.y, 0.0, 0.0))),
    );
    device.queue().writeBuffer(
      tint,
      0,
      Context.bytesOf<Tint>(new Tint(new Vec4f(0.25, 0.6, 0.75, 1.0))),
    );
    using target = device.createTexture({
      label: "x07-target",
      size: { width: 64, height: 64 },
      format: shifted_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x07-readback",
      size: 16384,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      shifted_WGSL,
      shifted_VERTEX_ENTRY,
      shifted_FRAGMENT_ENTRY,
      [shifted_LAYOUT0],
      [shifted_VERTEX_LAYOUT0],
      shifted,
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(
      device,
      nativeLayout,
      shifted_LAYOUT0,
      [bufferResource(params), bufferResource(tint)],
    );
    print("pipeline:created");
    using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({
      colorAttachments: [{
        view,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass, [bindGroup], [vertices.handle()]);
    pass.draw(3);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: target },
      { buffer: readback, bytesPerRow: 256, rowsPerImage: 64 },
      { width: 64, height: 64 },
    );
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    if (!await device.queue().onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    print("draw:submitted");
    if (!await readback.mapAsync(GPUMapMode.READ, 0, 16384)) {
      print("FAIL map");
      return;
    }
    const pixels: u8[] = readback.readMappedRange(0, 16384);
    print("readback:mapped");
    let y: i32 = 0;
    while (y < 64) {
      let x: i32 = 0;
      while (x < 64) {
        const inside: boolean = insideTriangle(pixelCenter(x, y), a, b, c);
        const expectedR: u8 = inside ? 64 : 0;
        const expectedG: u8 = inside ? 153 : 0;
        const expectedB: u8 = inside ? 191 : 0;
        const expectedA: u8 = 255;
        const byteOffset: i32 = y * 256 + x * 4;
        if (pixels[byteOffset] !== expectedR
          || pixels[byteOffset + 1] !== expectedG
          || pixels[byteOffset + 2] !== expectedB
          || pixels[byteOffset + 3] !== expectedA) {
          print(
            `FAIL x=${x} y=${y} expected=${expectedR},${expectedG},${expectedB},${expectedA} got=${pixels[byteOffset]},${pixels[byteOffset + 1]},${pixels[byteOffset + 2]},${pixels[byteOffset + 3]}`,
          );
          return;
        }
        x = x + 1;
      }
      y = y + 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
