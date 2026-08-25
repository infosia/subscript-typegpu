// example: clouds
// Layers a Perlin noise texture into drifting cloud cover over a sky gradient.
// TypeGPU exposes one quality select and ray marches a noise volume. This port drops the
// three-octave fbm, the sun light, and the sun glow, and accumulates six fixed noise layers.
// Ported from TypeGPU's clouds example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Sampler,
  Texture2d,
  Uniform,
  VertexInvocation,
  bufferResource,
  createBindGroupHost,
  createRenderPipelineHost,
  renderPipelineL,
  samplerResource,
  textureResource,
  writeTexturePixels,
} from "./typegpu";
import {
  Vec2f,
  Vec3f,
  Vec4f,
} from "./typegpu-types";
import {
  perlin3d,
} from "./typegpu-noise";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUSampler,
  GPUSamplerDescriptor,
  GPUTexture,
  GPUTextureUsage,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  CloudFrame_SIZE,
  Vertex_STRIDE,
  clouds_FRAGMENT_ENTRY,
  clouds_LAYOUT0,
  clouds_TARGET_FORMAT,
  clouds_VERTEX_ENTRY,
  clouds_VERTEX_LAYOUT0,
  clouds_WGSL,
} from "./main.typegpu";

const NOISE_SIZE: u32 = 64;
const NOISE_SCALE: f32 = 2.6;
const CLOUD_DENSITY: f32 = 1.35;
const CLOUD_SPEED: f32 = 0.018;
const CLOUD_THRESHOLD: f32 = 0.43;
const LAYER_COUNT: u32 = 6;
const CLOUD_TIME_PERIOD: u32 = 4096;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class CloudFrame {
  time: f32;
  aspect: f32;

  constructor(time: f32, aspect: f32) {
    this.time = time;
    this.aspect = aspect;
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

class CloudLayout {
  noise!: Texture2d<f32>;
  linear!: Sampler;
  frame!: Uniform<CloudFrame>;
}

// TypeGPU draws one full-screen triangle and picks its three corners from the vertex index.
// This port stores the three corners in a typed vertex buffer.
function cloudVertex(
  res: CloudLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// TypeGPU marches a ray through a noise volume and lights each sample from the sun.
// This port samples one texture at six depths and blends the result into the sky gradient.
function cloudFragment(
  res: CloudLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const frame: CloudFrame = res.frame.$;
  const screen = new Vec2f(
    (input.uv.x - 0.5) * frame.aspect + 0.5,
    input.uv.y,
  );
  let density: f32 = 0.0;
  let visibility: f32 = 1.0;
  for (let layer: u32 = 0; layer < LAYER_COUNT; layer += 1) {
    const depth: f32 = (layer as f32) / ((LAYER_COUNT - 1) as f32);
    const sampleUv = new Vec2f(
      screen.x * NOISE_SCALE + depth * 0.17 + frame.time * CLOUD_SPEED,
      screen.y * NOISE_SCALE * 0.72 - depth * 0.11 + frame.time * CLOUD_SPEED * 0.65,
    );
    const sampled: f32 = res.noise.sampleLevel(res.linear, sampleUv, 0.0).x;
    let layerDensity: f32 = (sampled - CLOUD_THRESHOLD) * CLOUD_DENSITY;
    if (layerDensity < 0.0) layerDensity = 0.0;
    density += layerDensity * visibility * 0.42;
    visibility *= 1.0 - layerDensity * 0.18;
  }
  if (density > 1.0) density = 1.0;
  const skyLow = new Vec3f(0.28, 0.48, 0.72);
  const skyHigh = new Vec3f(0.055, 0.16, 0.34);
  const sky: Vec3f = skyLow.add(skyHigh.sub(skyLow).scale(input.uv.y));
  const cloud = new Vec3f(0.94, 0.96, 1.0);
  const color: Vec3f = sky.add(cloud.sub(sky).scale(density));
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const clouds: RenderPipelineSpec = renderPipelineL<
  CloudLayout,
  Vertex,
  Varyings
>(cloudVertex, cloudFragment, { format: "bgra8unorm" });

function makeNoisePixels(): Vec4f[] {
  const pixels: Vec4f[] = [];
  for (let y: u32 = 0; y < NOISE_SIZE; y += 1) {
    for (let x: u32 = 0; x < NOISE_SIZE; x += 1) {
      const blendX: f32 = (x as f32) / (NOISE_SIZE as f32);
      const blendY: f32 = (y as f32) / (NOISE_SIZE as f32);
      const domainX: f32 = blendX * 4.0;
      const domainY: f32 = blendY * 4.0;
      const nearY: f32 = perlin3d(new Vec3f(domainX, domainY, 1.75))
        * (1.0 - blendX)
        + perlin3d(new Vec3f(domainX - 4.0, domainY, 1.75)) * blendX;
      const farY: f32 = perlin3d(new Vec3f(domainX, domainY - 4.0, 1.75))
        * (1.0 - blendX)
        + perlin3d(new Vec3f(domainX - 4.0, domainY - 4.0, 1.75)) * blendX;
      const value: f32 = (nearY * (1.0 - blendY) + farY * blendY) * 0.5 + 0.5;
      pixels.push(new Vec4f(value, value, value, 1.0));
    }
  }
  return pixels;
}

let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeGroup: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeNoiseTexture: GPUTexture | null = null;
let activeNoiseView: GPUTextureView | null = null;
let activeSampler: GPUSampler | null = null;
let frameCount: u32 = 0;

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== clouds_TARGET_FORMAT) {
    print(`FAIL format expected=${clouds_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "clouds-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const frameBuffer = hostDevice.createBuffer({
    label: "clouds-frame",
    size: CloudFrame_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  const noiseTexture = hostDevice.createTexture({
    label: "clouds-noise",
    size: { width: NOISE_SIZE, height: NOISE_SIZE },
    format: "rgba8unorm",
    usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
  });
  const noiseView = noiseTexture.createView();
  const samplerDescriptor: GPUSamplerDescriptor = {
    addressModeU: "repeat",
    addressModeV: "repeat",
    minFilter: "linear",
    magFilter: "linear",
  };
  const linearSampler = hostDevice.createSampler(samplerDescriptor);
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<CloudFrame>(new CloudFrame(0.0, 1.0)));
  writeTexturePixels(queue, noiseTexture, makeNoisePixels(), NOISE_SIZE, NOISE_SIZE);

  hostDevice.pushErrorScope("validation");
  const pipeline = createRenderPipelineHost(
    hostDevice,
    clouds_WGSL,
    clouds_VERTEX_ENTRY,
    clouds_FRAGMENT_ENTRY,
    [clouds_LAYOUT0],
    [clouds_VERTEX_LAYOUT0],
    clouds,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    pipeline.dispose();
    linearSampler.dispose();
    noiseView.dispose();
    noiseTexture.dispose();
    frameBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  using bindLayout = pipeline.bindGroupLayout(0);
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    clouds_LAYOUT0,
    [
      textureResource(noiseView),
      samplerResource(linearSampler),
      bufferResource(frameBuffer),
    ],
  );
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeGroup = group;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeNoiseTexture = noiseTexture;
  activeNoiseView = noiseView;
  activeSampler = linearSampler;
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
  const device = activeDevice;
  const pipeline = activePipeline;
  const group = activeGroup;
  const vertices = activeVertices;
  const frameBuffer = activeFrameBuffer;
  if (device === null) return;
  if (pipeline === null) return;
  if (group === null) return;
  if (vertices === null) return;
  if (frameBuffer === null) return;
  frameCount = (frameCount + 1) % CLOUD_TIME_PERIOD;
  using queue = device.queue();
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<CloudFrame>(new CloudFrame(
      frameCount as f32,
      (width as f32) / (height as f32),
    )),
  );
  const target = new GPUTextureView(view);
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.055, g: 0.16, b: 0.34, a: 1.0 },
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
  if (activeGroup !== null) activeGroup.dispose();
  if (activeSampler !== null) activeSampler.dispose();
  if (activeNoiseView !== null) activeNoiseView.dispose();
  if (activeNoiseTexture !== null) activeNoiseTexture.dispose();
  if (activeFrameBuffer !== null) activeFrameBuffer.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  activeGroup = null;
  activeSampler = null;
  activeNoiseView = null;
  activeNoiseTexture = null;
  activeFrameBuffer = null;
  activeVertices = null;
  activePipeline = null;
  activeDevice = null;
}
