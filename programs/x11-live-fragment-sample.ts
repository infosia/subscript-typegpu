// program: x11-live-fragment-sample
// purpose: compare fragment textureSample over a full-screen quad with the host sampler body
// exercises: TX1-TX7, RN1-RN17, T4, T15
// questions: none

import {
  Buffer,
  FragmentInvocation,
  RenderPipelineSpec,
  Sampler,
  Texture2d,
  VertexInvocation,
  createBindGroup,
  createBuffer,
  createRenderPipeline,
  renderPipelineL,
  samplerResource,
  textureResource,
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
  FragmentVertex_STRIDE,
  fragmentSample_FRAGMENT_ENTRY,
  fragmentSample_LAYOUT0,
  fragmentSample_TARGET_FORMAT,
  fragmentSample_VERTEX_ENTRY,
  fragmentSample_VERTEX_LAYOUT0,
  fragmentSample_WGSL,
} from "./x11-live-fragment-sample.typegpu";

@CStruct
class FragmentVertex {
  position: Vec2f;
  uv: Vec2f;

  constructor(position: Vec2f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

@CStruct
class FragmentVarying {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

class FragmentTextureLayout {
  source!: Texture2d<f32>;
  nearest!: Sampler;
}

function fragmentVertex(
  res: FragmentTextureLayout,
  value: FragmentVertex,
  ctx: VertexInvocation,
): FragmentVarying {
  return new FragmentVarying(new Vec4f(value.position.x, value.position.y, 0.0, 1.0), value.uv);
}

function fragmentColor(
  res: FragmentTextureLayout,
  input: FragmentVarying,
  ctx: FragmentInvocation,
): Vec4f {
  return res.source.sample(res.nearest, input.uv);
}

export const fragmentSample: RenderPipelineSpec = renderPipelineL<
  FragmentTextureLayout,
  FragmentVertex,
  FragmentVarying
>(fragmentVertex, fragmentColor, { format: "rgba8unorm" });

function checkerBytes(): u8[] {
  const values: u8[] = [];
  let y: i32 = 0;
  while (y < 4) {
    let x: i32 = 0;
    while (x < 4) {
      const value: u8 = ((x + y) % 2 === 0) ? 255 : 0;
      values.push(value); values.push(value); values.push(value); values.push(255);
      x = x + 1;
    }
    y = y + 1;
  }
  return values;
}

function checkerFloats(): f32[] {
  const values: f32[] = [];
  let y: i32 = 0;
  while (y < 4) {
    let x: i32 = 0;
    while (x < 4) {
      const value: f32 = ((x + y) % 2 === 0) ? 1.0 : 0.0;
      values.push(value); values.push(value); values.push(value); values.push(1.0);
      x = x + 1;
    }
    y = y + 1;
  }
  return values;
}

function checkerPixels(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let y: i32 = 0;
  while (y < 4) {
    let x: i32 = 0;
    while (x < 4) {
      const value: f32 = ((x + y) % 2 === 0) ? 1.0 : 0.0;
      pixels.push(new Vec4f(value, value, value, 1.0));
      x = x + 1;
    }
    y = y + 1;
  }
  return pixels;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    const vertices: FixedArray<FragmentVertex, 6> = [
      new FragmentVertex(new Vec2f(-1.0, -1.0), new Vec2f(0.0, 1.0)),
      new FragmentVertex(new Vec2f(1.0, -1.0), new Vec2f(1.0, 1.0)),
      new FragmentVertex(new Vec2f(-1.0, 1.0), new Vec2f(0.0, 0.0)),
      new FragmentVertex(new Vec2f(-1.0, 1.0), new Vec2f(0.0, 0.0)),
      new FragmentVertex(new Vec2f(1.0, -1.0), new Vec2f(1.0, 1.0)),
      new FragmentVertex(new Vec2f(1.0, 1.0), new Vec2f(1.0, 0.0)),
    ];
    using vertexBuffer: Buffer<FragmentVertex> = createBuffer<FragmentVertex>(
      device, FragmentVertex_STRIDE, 6,
      GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST, "x11-vertices",
    );
    vertexBuffer.write(device.queue(), 0, Context.bytesOf<FixedArray<FragmentVertex, 6>>(vertices));
    using source = device.createTexture({
      label: "x11-source",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    using target = device.createTexture({
      label: "x11-target",
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: fragmentSample_TARGET_FORMAT,
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using sourceView = source.createView();
    using targetView = target.createView();
    using nearest = device.createSampler({ minFilter: "nearest", magFilter: "nearest" });
    using readback = device.createBuffer({
      label: "x11-readback", size: 1024,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    device.queue().writeTexture(
      { texture: source }, checkerBytes(),
      { offset: 0, bytesPerRow: 16, rowsPerImage: 4 },
      { width: 4, height: 4, depthOrArrayLayers: 1 },
    );
    print("inputs:written");
    using pipeline = createRenderPipeline(
      device, fragmentSample_WGSL, fragmentSample_VERTEX_ENTRY, fragmentSample_FRAGMENT_ENTRY,
      [fragmentSample_LAYOUT0], [fragmentSample_VERTEX_LAYOUT0], fragmentSample,
    );
    print("pipeline:created");
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, fragmentSample_LAYOUT0, [
      textureResource(sourceView), samplerResource(nearest),
    ]);
    using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({
      colorAttachments: [{
        view: targetView,
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: "clear",
        storeOp: "store",
      }],
    });
    pipeline.bind(pass, [bindGroup], [vertexBuffer.handle()]);
    pass.draw(6);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: target },
      { buffer: readback, offset: 0, bytesPerRow: 256, rowsPerImage: 4 },
      { width: 4, height: 4, depthOrArrayLayers: 1 },
    );
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    if (!await device.queue().onSubmittedWorkDone()) { print("FAIL submit"); return; }
    print("draw:submitted");
    if (!await readback.mapAsync(GPUMapMode.READ, 0, 1024)) { print("FAIL map"); return; }
    const pixels: u8[] = readback.readMappedRange(0, 1024);
    print("readback:mapped");
    const hostTexture = new Texture2d<f32>(checkerFloats(), checkerPixels(), 4, 4);
    const hostSampler = new Sampler();
    let y: i32 = 0;
    while (y < 4) {
      let x: i32 = 0;
      while (x < 4) {
        const expected: Vec4f = hostTexture.sample(
          hostSampler,
          new Vec2f(((x as f32) + 0.5) / 4.0, ((y as f32) + 0.5) / 4.0),
        );
        const offset: i32 = y * 256 + x * 4;
        const expectedValue: u8 = (expected.x * 255.0) as u8;
        if (pixels[offset] !== expectedValue || pixels[offset + 1] !== expectedValue
          || pixels[offset + 2] !== expectedValue || pixels[offset + 3] !== 255) {
          print(`FAIL x=${x} y=${y} expected=${expectedValue} got=${pixels[offset]},${pixels[offset + 1]},${pixels[offset + 2]},${pixels[offset + 3]}`);
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
