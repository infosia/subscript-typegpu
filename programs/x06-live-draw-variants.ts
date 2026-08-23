// program: x06-live-draw-variants
// purpose: compare three indexed instanced quads with a host rasterizer
// exercises: RN1-RN15, T4, T15, render copy and map
// questions: none

import {
  Buffer,
  createBuffer,
  createRenderPipeline,
  FragmentInvocation,
  RenderPipelineSpec,
  renderPipelineInstanced,
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
  Instance_STRIDE,
  quad_FRAGMENT_ENTRY,
  quad_TARGET_FORMAT,
  quad_VERTEX_ENTRY,
  quad_VERTEX_LAYOUT0,
  quad_VERTEX_LAYOUT1,
  quad_WGSL,
  Vertex_STRIDE,
} from "./x06-live-draw-variants.typegpu";

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class Instance {
  offset: Vec2f;
  color: Vec3f;

  constructor(offset: Vec2f, color: Vec3f) {
    this.offset = offset;
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

function quadVert(value: Vertex, instance: Instance, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(
      value.position.x + instance.offset.x,
      value.position.y + instance.offset.y,
      0.0,
      1.0,
    ),
    instance.color,
  );
}

function frag(input: Varyings, ctx: FragmentInvocation): Vec4f {
  return new Vec4f(input.color.x, input.color.y, input.color.z, 1.0);
}

export const quad: RenderPipelineSpec = renderPipelineInstanced<Vertex, Instance, Varyings>(
  quadVert,
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

function translated(value: Vec2f, offset: Vec2f): Vec2f {
  return new Vec2f(value.x + offset.x, value.y + offset.y);
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

function instanceHasNoCenterOnEdge(
  vertices: FixedArray<Vertex, 4>,
  instance: Instance,
): boolean {
  const a: Vec2f = translated(vertices[0].position, instance.offset);
  const b: Vec2f = translated(vertices[1].position, instance.offset);
  const c: Vec2f = translated(vertices[2].position, instance.offset);
  const d: Vec2f = translated(vertices[3].position, instance.offset);
  return noCenterOnEdge(a, b, c) && noCenterOnEdge(c, b, d);
}

function covers(
  point: Vec2f,
  vertices: FixedArray<Vertex, 4>,
  instance: Instance,
): boolean {
  const a: Vec2f = translated(vertices[0].position, instance.offset);
  const b: Vec2f = translated(vertices[1].position, instance.offset);
  const c: Vec2f = translated(vertices[2].position, instance.offset);
  const d: Vec2f = translated(vertices[3].position, instance.offset);
  return insideTriangle(point, a, b, c) || insideTriangle(point, c, b, d);
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
    const vertexValues: FixedArray<Vertex, 4> = [
      new Vertex(new Vec2f(-0.23, -0.21)),
      new Vertex(new Vec2f(0.21, -0.21)),
      new Vertex(new Vec2f(-0.23, 0.25)),
      new Vertex(new Vec2f(0.21, 0.25)),
    ];
    const instanceValues: FixedArray<Instance, 3> = [
      new Instance(new Vec2f(-0.38, -0.17), new Vec3f(1.0, 0.0, 0.0)),
      new Instance(new Vec2f(0.0, 0.02), new Vec3f(0.0, 1.0, 0.0)),
      new Instance(new Vec2f(0.31, 0.19), new Vec3f(0.0, 0.0, 1.0)),
    ];
    const indices: FixedArray<u16, 6> = [0, 1, 2, 2, 1, 3];
    let instanceIndex: i32 = 0;
    while (instanceIndex < 3) {
      if (!instanceHasNoCenterOnEdge(vertexValues, instanceValues[instanceIndex])) {
        print(`FAIL pixel center on instance ${instanceIndex} edge`);
        return;
      }
      instanceIndex = instanceIndex + 1;
    }
    using vertices: Buffer<Vertex> = createBuffer<Vertex>(
      device,
      Vertex_STRIDE,
      4,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x06-vertices",
    );
    using instances: Buffer<Instance> = createBuffer<Instance>(
      device,
      Instance_STRIDE,
      3,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
      "x06-instances",
    );
    using indexBuffer = device.createBuffer({
      label: "x06-indices",
      size: 12,
      usage: GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST,
    });
    vertices.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Vertex, 4>>(vertexValues),
    );
    instances.write(
      device.queue(),
      0,
      Context.bytesOf<FixedArray<Instance, 3>>(instanceValues),
    );
    device.queue().writeBuffer(
      indexBuffer,
      0,
      Context.bytesOf<FixedArray<u16, 6>>(indices),
    );
    using target = device.createTexture({
      label: "x06-target",
      size: { width: 64, height: 64 },
      format: quad_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x06-readback",
      size: 16384,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    print("inputs:written");
    device.pushErrorScope("validation");
    using pipeline = createRenderPipeline(
      device,
      quad_WGSL,
      quad_VERTEX_ENTRY,
      quad_FRAGMENT_ENTRY,
      [],
      [quad_VERTEX_LAYOUT0, quad_VERTEX_LAYOUT1],
      quad,
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
    pipeline.bind(pass, [], [vertices.handle(), instances.handle()]);
    pass.setIndexBuffer(indexBuffer, "uint16", 0, 12);
    pass.drawIndexed(6, 3);
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
        const point: Vec2f = pixelCenter(x, y);
        let expectedR: u8 = 0;
        let expectedG: u8 = 0;
        let expectedB: u8 = 0;
        instanceIndex = 0;
        // Instances rasterize in order, so a later covered instance owns the pixel.
        while (instanceIndex < 3) {
          if (covers(point, vertexValues, instanceValues[instanceIndex])) {
            if (instanceIndex === 0) {
              expectedR = 255;
              expectedG = 0;
              expectedB = 0;
            } else if (instanceIndex === 1) {
              expectedR = 0;
              expectedG = 255;
              expectedB = 0;
            } else {
              expectedR = 0;
              expectedG = 0;
              expectedB = 255;
            }
          }
          instanceIndex = instanceIndex + 1;
        }
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
        x = x + 1;
      }
      y = y + 1;
    }
    readback.unmap();
  }
  gpu.dispose();
  print("PASS");
}
