// program: x20-live-strip
// purpose: compare four-vertex strip expansion and culling with a host rasterizer
// exercises: PI14, RN12, RN19, RN20, T4, T15
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
  GPUBufferUsage,
  GPUDevice,
  GPUMapMode,
  GPUTextureUsage,
} from "./webgpu";
import {
  stripLive_FRAGMENT_ENTRY,
  stripLive_TARGET_FORMAT,
  stripLive_VERTEX_ENTRY,
  stripLive_VERTEX_LAYOUT0,
  stripLive_WGSL,
  Vertex_STRIDE,
} from "./x20-live-strip.typegpu";

const SIZE: i32 = 16;

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
  return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0));
}

function fragmentStep(value: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(0.25, 0.6, 0.75, 1.0);
}

export const stripLive: RenderPipelineSpec = renderPipeline<Vertex, Varyings>(
  vertexStep,
  fragmentStep,
  {
    format: "rgba8unorm",
    topology: "triangle-strip",
    cullMode: "back",
    frontFace: "ccw",
  },
);

function signedArea(a: Vec2f, b: Vec2f, c: Vec2f): f32 {
  return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
}

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

function triangleCovered(point: Vec2f, a: Vec2f, b: Vec2f, c: Vec2f): boolean {
  const area: f32 = signedArea(a, b, c);
  const front: boolean = stripLive.frontFace === "ccw" ? area > 0.0 : area < 0.0;
  if ((stripLive.cullMode === "back" && !front)
    || (stripLive.cullMode === "front" && front)) {
    return false;
  }
  const ab: f32 = edge(a, b, point);
  const bc: f32 = edge(b, c, point);
  const ca: f32 = edge(c, a, point);
  return (ab > 0.0 && bc > 0.0 && ca > 0.0)
    || (ab < 0.0 && bc < 0.0 && ca < 0.0);
}

function stripCovered(point: Vec2f, vertices: FixedArray<Vec2f, 4>): boolean {
  let triangle: i32 = 0;
  while (triangle < vertices.length - 2) {
    // Expand n vertices into n-2 triangles, flipping odd winding before applying RN19 culling.
    const a: Vec2f = triangle % 2 === 0 ? vertices[triangle] : vertices[triangle + 1];
    const b: Vec2f = triangle % 2 === 0 ? vertices[triangle + 1] : vertices[triangle];
    const c: Vec2f = vertices[triangle + 2];
    if (triangleCovered(point, a, b, c)) return true;
    triangle += 1;
  }
  return false;
}

function noCenterOnStripEdge(vertices: FixedArray<Vec2f, 4>): boolean {
  let y: i32 = 0;
  while (y < SIZE) {
    let x: i32 = 0;
    while (x < SIZE) {
      const point: Vec2f = pixelCenter(x, y);
      let triangle: i32 = 0;
      while (triangle < vertices.length - 2) {
        const a: Vec2f = triangle % 2 === 0 ? vertices[triangle] : vertices[triangle + 1];
        const b: Vec2f = triangle % 2 === 0 ? vertices[triangle + 1] : vertices[triangle];
        const c: Vec2f = vertices[triangle + 2];
        if (edge(a, b, point) === 0.0
          || edge(b, c, point) === 0.0
          || edge(c, a, point) === 0.0) {
          return false;
        }
        triangle += 1;
      }
      x += 1;
    }
    y += 1;
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
    const positions: FixedArray<Vec2f, 4> = [
      new Vec2f(-0.72, -0.63),
      new Vec2f(0.61, -0.58),
      new Vec2f(-0.66, 0.57),
      new Vec2f(0.68, 0.64),
    ];
    if (!noCenterOnStripEdge(positions)) {
      print("FAIL pixel center on edge");
      return;
    }
    const values: FixedArray<Vertex, 4> = [
      new Vertex(positions[0]),
      new Vertex(positions[1]),
      new Vertex(positions[2]),
      new Vertex(positions[3]),
    ];
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      4,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x20-vertices",
    );
    vertices.write(device.queue, 0, Context.bytesOf<FixedArray<Vertex, 4>>(values));
    using target = device.createTexture({
      label: "x20-target",
      size: { width: SIZE as u32, height: SIZE as u32 },
      format: stripLive_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x20-readback",
      size: (256 * SIZE) as u64,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      stripLive_WGSL,
      stripLive_VERTEX_ENTRY,
      stripLive_FRAGMENT_ENTRY,
      [],
      [stripLive_VERTEX_LAYOUT0],
      stripLive,
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
    pass.draw(4);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: target },
      { buffer: readback, bytesPerRow: 256, rowsPerImage: SIZE as u32 },
      { width: SIZE as u32, height: SIZE as u32, depthOrArrayLayers: 1 },
    );
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await device.queue.onSubmittedWorkDone()) {
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
        const covered: boolean = stripCovered(pixelCenter(x, y), positions);
        const expectedR: u8 = covered ? 64 : 0;
        const expectedG: u8 = covered ? 153 : 0;
        const expectedB: u8 = covered ? 191 : 0;
        const offset: i32 = y * 256 + x * 4;
        if (pixels[offset] !== expectedR
          || pixels[offset + 1] !== expectedG
          || pixels[offset + 2] !== expectedB
          || pixels[offset + 3] !== 255) {
          print(
            `FAIL x=${x} y=${y} expected=${expectedR},${expectedG},${expectedB},255 got=${pixels[offset]},${pixels[offset + 1]},${pixels[offset + 2]},${pixels[offset + 3]}`,
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
