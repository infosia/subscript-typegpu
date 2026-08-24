// program: x22-live-blend
// purpose: compare two overlapping alpha-blended triangles with the host rasterizer
// exercises: BF1, BF2, PI14, RN1, RN2, RN4, RN5, RN10, RN14, RN21, T4, T15
// questions: none

import {
  Buffer,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  hostBlend,
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
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  blendLive_FRAGMENT_ENTRY,
  blendLive_TARGET_FORMAT,
  blendLive_VERTEX_ENTRY,
  blendLive_VERTEX_LAYOUT0,
  blendLive_WGSL,
  Vertex_STRIDE,
} from "./x22-live-blend.typegpu";

const SIZE: i32 = 64;
const EDGE_MARGIN: f32 = 0.0025;

@CStruct
class Vertex {
  position: Vec2f;
  color: Vec4f;

  constructor(position: Vec2f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

@CStruct
class Varyings {
  position: Vec4f;
  color: Vec4f;

  constructor(position: Vec4f, color: Vec4f) {
    this.position = position;
    this.color = color;
  }
}

function vertexStep(value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    value.color,
  );
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return value.color;
}

export const blendLive: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    blend: {
      color: {
        operation: "add",
        srcFactor: "src-alpha",
        dstFactor: "one-minus-src-alpha",
      },
      alpha: {
        operation: "add",
        srcFactor: "one",
        dstFactor: "one",
      },
    },
  },
);

function edge(a: Vec2f, b: Vec2f, point: Vec2f): f32 {
  return (point.x - a.x) * (b.y - a.y) - (point.y - a.y) * (b.x - a.x);
}

function pixelCenter(x: i32, y: i32): Vec2f {
  // Texel row 0 maps to clip-space y=+1. Increasing texel rows lowers clip-space y.
  return new Vec2f(
    ((x as f32) + 0.5) * (2.0 / (SIZE as f32)) - 1.0,
    1.0 - ((y as f32) + 0.5) * (2.0 / (SIZE as f32)),
  );
}

function insideTriangle(point: Vec2f, a: Vec2f, b: Vec2f, c: Vec2f): boolean {
  const ab: f32 = edge(a, b, point);
  const bc: f32 = edge(b, c, point);
  const ca: f32 = edge(c, a, point);
  return (ab > 0.0 && bc > 0.0 && ca > 0.0)
    || (ab < 0.0 && bc < 0.0 && ca < 0.0);
}

function centersKeepEdgeMargin(a: Vec2f, b: Vec2f, c: Vec2f): boolean {
  let y: i32 = 0;
  while (y < SIZE) {
    let x: i32 = 0;
    while (x < SIZE) {
      const point: Vec2f = pixelCenter(x, y);
      let ab: f32 = edge(a, b, point);
      let bc: f32 = edge(b, c, point);
      let ca: f32 = edge(c, a, point);
      if (ab < 0.0) ab = -ab;
      if (bc < 0.0) bc = -bc;
      if (ca < 0.0) ca = -ca;
      if (ab < EDGE_MARGIN || bc < EDGE_MARGIN || ca < EDGE_MARGIN) {
        return false;
      }
      x += 1;
    }
    y += 1;
  }
  return true;
}

function unorm8(value: f32): u8 {
  return Math.floor((value * 255.0 + 0.5) as f64) as u8;
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
    const firstA = new Vec2f(-0.76, -0.6);
    const firstB = new Vec2f(0.55, -0.6);
    const firstC = new Vec2f(-0.18, 0.85);
    const secondA = new Vec2f(-0.5, -0.66);
    const secondB = new Vec2f(0.75, -0.41);
    const secondC = new Vec2f(0.25, 0.59);
    // Every pixel center stays at least EDGE_MARGIN from each triangle edge.
    if (!centersKeepEdgeMargin(firstA, firstB, firstC)
      || !centersKeepEdgeMargin(secondA, secondB, secondC)) {
      print("FAIL pixel center inside edge margin");
      return;
    }
    const firstColor = new Vec4f(0.8, 0.2, 0.1, 0.4);
    const secondColor = new Vec4f(0.1, 0.3, 0.9, 0.6);
    const values: FixedArray<Vertex, 6> = [
      new Vertex(firstA, firstColor),
      new Vertex(firstB, firstColor),
      new Vertex(firstC, firstColor),
      new Vertex(secondA, secondColor),
      new Vertex(secondB, secondColor),
      new Vertex(secondC, secondColor),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      6,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x22-vertices",
    );
    vertices.write(device.queue(), 0, Context.bytesOf<FixedArray<Vertex, 6>>(values));
    using target = device.createTexture({
      label: "x22-target",
      size: { width: SIZE as u32, height: SIZE as u32 },
      format: blendLive_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x22-readback",
      size: (256 * SIZE) as u64,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      blendLive_WGSL,
      blendLive_VERTEX_ENTRY,
      blendLive_FRAGMENT_ENTRY,
      [],
      [blendLive_VERTEX_LAYOUT0],
      blendLive,
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
        clearValue: { r: 0, g: 0, b: 0, a: 0 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass, [], [vertices.handle()]);
    pass.draw(3);
    pass.draw(3, 1, 3);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: target },
      { buffer: readback, bytesPerRow: 256, rowsPerImage: SIZE as u32 },
      { width: SIZE as u32, height: SIZE as u32, depthOrArrayLayers: 1 },
    );
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    if (!await device.queue().onSubmittedWorkDone()) {
      print("FAIL submit");
      return;
    }
    print("draw:submitted");
    if (!await readback.mapAsync(GPUMapMode.READ, 0, (256 * SIZE) as u64)) {
      print("FAIL map");
      return;
    }
    const pixels: u8[] = readback.readMappedRange(0, (256 * SIZE) as u64);
    print("readback:mapped");
    let y: i32 = 0;
    while (y < SIZE) {
      let x: i32 = 0;
      while (x < SIZE) {
        const point: Vec2f = pixelCenter(x, y);
        let expected = new Vec4f(0.0, 0.0, 0.0, 0.0);
        if (insideTriangle(point, firstA, firstB, firstC)) {
          expected = hostBlend(firstColor, expected, blendLive.blend);
        }
        if (insideTriangle(point, secondA, secondB, secondC)) {
          expected = hostBlend(secondColor, expected, blendLive.blend);
        }
        const expectedR: u8 = unorm8(expected.x);
        const expectedG: u8 = unorm8(expected.y);
        const expectedB: u8 = unorm8(expected.z);
        const expectedA: u8 = unorm8(expected.w);
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
        x += 1;
      }
      y += 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
