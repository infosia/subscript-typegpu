// program: x05-live-triangle
// purpose: draw one triangle and compare every texel with a host rasterizer
// exercises: RN1-RN14, T4, T15, render copy and map
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
import { Vec2f, Vec3f, Vec4f } from "./typegpu-types";
import {
  gpu,
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  tri_FRAGMENT_ENTRY,
  tri_TARGET_FORMAT,
  tri_VERTEX_ENTRY,
  tri_VERTEX_LAYOUT0,
  tri_WGSL,
  Vertex_STRIDE,
} from "./x05-live-triangle.typegpu";

@CStruct
class Vertex {
  position: Vec2f;
  color: Vec3f;

  constructor(position: Vec2f, color: Vec3f) {
    this.position = position;
    this.color = color;
  }
}

@CStruct
class Varyings {
  position: Vec4f;
  color: Vec3f;

  constructor(position: Vec4f, color: Vec3f) {
    this.position = position;
    this.color = color;
  }
}

function vert(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.color,
  );
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0);
}

export const tri: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(vert, frag, {
  format: "rgba8unorm",
});

function edge(a: Vec2f, b: Vec2f, point: Vec2f): f32 {
  return (point.x - a.x) * (b.y - a.y) - (point.y - a.y) * (b.x - a.x);
}

function pixelCenter(x: i32, y: i32): Vec2f {
  // Texel row 0 maps to clip-space y=+1. Increasing texel rows lowers clip-space y.
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
    const a = new Vec2f(-0.6, -0.6);
    const b = new Vec2f(0.6, -0.6);
    const c = new Vec2f(0.0, 0.7);
    if (!noCenterOnEdge(a, b, c)) {
      print("FAIL pixel center on edge");
      return;
    }
    const color = new Vec3f(0.25, 0.6, 0.75);
    const values: FixedArray<Vertex, 3> = [
      new Vertex(a, color),
      new Vertex(b, color),
      new Vertex(c, color),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x05-vertices",
    );
    vertices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Vertex, 3>>(values),
    );
    using target = device.createTexture({
      label: "x05-target",
      size: { width: 64, height: 64 },
      format: tri_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x05-readback",
      size: 16384,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      tri_WGSL,
      tri_VERTEX_ENTRY,
      tri_FRAGMENT_ENTRY,
      [],
      [tri_VERTEX_LAYOUT0],
      tri,
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
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
    pipeline.bind(pass, [], [vertices.handle()]);
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
        const offset: i32 = y * 256 + x * 4;
        if (pixels[offset] !== expectedR
          || pixels[offset + 1] !== expectedG
          || pixels[offset + 2] !== expectedB
          || pixels[offset + 3] !== expectedA) {
          print(
            `FAIL x=${x} y=${y} expected=${expectedR},${expectedG},${expectedB},${expectedA} got=${pixels[offset]},${pixels[offset + 1]},${pixels[offset + 2]},${pixels[offset + 3]}`,
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
