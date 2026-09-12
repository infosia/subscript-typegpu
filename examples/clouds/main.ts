// example: clouds
// Layers a Perlin noise texture into drifting cloud cover over a sky gradient.
// TypeGPU exposes one quality select and ray marches a noise volume. This port drops the
// three-octave fbm, the sun light, and the sun glow, and accumulates six fixed noise layers.
// A 64-square Perlin field computed in code replaces the 256-square random noise texture.
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

// The noise texture holds 64 by 64 pixels. NOISE_SCALE sets how many noise tiles fill the
// screen. CLOUD_TIME_PERIOD wraps the frame counter, so the drift stays exact in f32.
const NOISE_SIZE: u32 = 64;
const NOISE_SCALE: f32 = 2.6;
const CLOUD_DENSITY: f32 = 1.35;
const CLOUD_SPEED: f32 = 0.018;
const CLOUD_THRESHOLD: f32 = 0.43;
const LAYER_COUNT: u32 = 6;
const CLOUD_TIME_PERIOD: u32 = 4096;

// The vertex record. The generator emits Vertex_STRIDE from this class, so the host code
// never counts bytes.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The per-frame uniform. `time` carries the wrapped frame count, not seconds. `aspect`
// carries the surface width divided by the surface height.
@CStruct
class CloudFrame {
  time: f32;
  aspect: f32;

  constructor(time: f32, aspect: f32) {
    this.time = time;
    this.aspect = aspect;
  }
}

// The inter-stage record. The field named `position` becomes the clip-space builtin, and
// `uv` becomes location 0.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The bind group layout. TypeGPU builds a layout object at run time. Here the layout is a
// class, and the field order fixes binding 0, 1, and 2 of group 0.
class CloudLayout {
  noise: Texture2d<f32>;
  linear: Sampler;
  frame: Uniform<CloudFrame>;

  constructor(noise: Texture2d<f32>, linear: Sampler, frame: Uniform<CloudFrame>) {
    this.noise = noise;
    this.linear = linear;
    this.frame = frame;
  }
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
  // The aspect correction stretches the sample coordinates, so a cloud keeps its shape on a
  // wide surface.
  const screen = new Vec2f(
    (input.uv.x - 0.5) * frame.aspect + 0.5,
    input.uv.y,
  );
  let density: f32 = 0.0;
  let visibility: f32 = 1.0;
  // Each layer samples the same texture at its own offset and drift rate. `visibility` holds
  // the light left after the nearer layers, so a near layer hides a far one.
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
  // The sky is a vertical gradient from horizon to zenith. The accumulated density mixes the
  // cloud color over it.
  const skyLow = new Vec3f(0.28, 0.48, 0.72);
  const skyHigh = new Vec3f(0.055, 0.16, 0.34);
  const sky: Vec3f = skyLow.add(skyHigh.sub(skyLow).scale(input.uv.y));
  const cloud = new Vec3f(0.94, 0.96, 1.0);
  const color: Vec3f = sky.add(cloud.sub(sky).scale(density));
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The pipeline declaration joins the layout class, the vertex schema, and the varyings.
// `subscript-typegpu-gen` reads it and emits the WGSL before the run, so no shader text is built here.
export const clouds: RenderPipelineSpec = renderPipelineL<
  CloudLayout,
  Vertex,
  Varyings
>(cloudVertex, cloudFragment, { format: "bgra8unorm" });

// The noise is generated, never fetched. The blend across the 4-unit domain makes the
// texture wrap, so the repeat address mode shows no seam.
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

// The host calls `init`, `frame`, and `shutdown` separately, so every handle that outlives
// `init` lives in module state. A `using` declaration disposes a handle too early here.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeGroup: GPUBindGroup | null = null;
let activeVertices: GPUBuffer | null = null;
let activeFrameBuffer: GPUBuffer | null = null;
let activeNoiseTexture: GPUTexture | null = null;
let activeNoiseView: GPUTextureView | null = null;
let activeSampler: GPUSampler | null = null;
let frameCount: u32 = 0;

// `init` runs once, after the host configures the surface. It creates every long-lived
// resource. The instance and the device stay with the host.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The generator pins the target format into the pipeline. A mismatch with the host surface
  // fails here, not inside pipeline creation.
  if (format !== clouds_TARGET_FORMAT) {
    print(`FAIL format expected=${clouds_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The wrapper adapts the host handles to the API layer. It carries no `dispose`, because
  // the host owns the device.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds the three corners of the fullscreen triangle. Vertex_STRIDE keeps
  // the size right when the schema changes.
  const vertices = hostDevice.createBuffer({
    label: "clouds-vertices",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // One uniform buffer holds CloudFrame. COPY_DST admits the queue write that each frame makes.
  const frameBuffer = hostDevice.createBuffer({
    label: "clouds-frame",
    size: CloudFrame_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // The noise lives in a texture, so the sampler gives the fragment bilinear filtering for free.
  // TEXTURE_BINDING covers the shader read, and COPY_DST covers the host write.
  const noiseTexture = hostDevice.createTexture({
    label: "clouds-noise",
    size: { width: NOISE_SIZE, height: NOISE_SIZE },
    format: "rgba8unorm",
    usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
  });
  // The view is a handle of its own. The bind group holds the view, and `shutdown` disposes
  // both the view and the texture.
  const noiseView = noiseTexture.createView();
  // The repeat address mode lets the drift scroll past the texture edge forever. Linear
  // filtering hides the 64-pixel grid.
  const samplerDescriptor: GPUSamplerDescriptor = {
    addressModeU: "repeat",
    addressModeV: "repeat",
    minFilter: "linear",
    magFilter: "linear",
  };
  const linearSampler = hostDevice.createSampler(samplerDescriptor);
  // The queue wrapper is a handle. `using` disposes it at the end of `init`, and `frame`
  // takes a fresh one.
  using queue = hostDevice.queue();
  // `Context.bytesOf` lays out the values with the generated C layout, so the bytes match the
  // WGSL that the generator emits.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  // The first uniform write gives the buffer a defined value before the first frame runs.
  queue.writeBuffer(frameBuffer, 0, Context.bytesOf<CloudFrame>(new CloudFrame(0.0, 1.0)));
  // The helper pads each row to the 256-byte row alignment that a texture write needs.
  writeTexturePixels(queue, noiseTexture, makeNoisePixels(), NOISE_SIZE, NOISE_SIZE);

  // The error scope catches a validation failure from pipeline creation. The layers return the
  // failure as a value, so a `null` check replaces an exception.
  hostDevice.pushErrorScope("validation");
  // The call takes the generated WGSL text, the two entry names, the bind group layout, and
  // the vertex layout. No shader text is built here.
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
  // The error path disposes every handle that the failed run already created, because no
  // finalizer runs later.
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
  // The native layout comes from the pipeline. The bind group reads it at creation, so `using`
  // releases the handle right after.
  using bindLayout = pipeline.bindGroupLayout(0);
  // The resource order follows the field order of CloudLayout: texture, sampler, uniform.
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
  // The state moves to module scope only after every step passes. A failed `init` leaves the
  // fields null, and `frame` returns at once.
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeGroup = group;
  activeVertices = vertices;
  activeFrameBuffer = frameBuffer;
  activeNoiseTexture = noiseTexture;
  activeNoiseView = noiseView;
  activeSampler = linearSampler;
}

// The host calls `frame` once per presented frame. `view` is the swapchain view the host
// owns, and `width` and `height` are surface pixels.
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
  // A null field means `init` failed or never ran. The frame returns, because the layers
  // report failure as a value.
  if (device === null) return;
  if (pipeline === null) return;
  if (group === null) return;
  if (vertices === null) return;
  if (frameBuffer === null) return;
  // The wrap keeps the uniform time small. The cloud drift repeats every CLOUD_TIME_PERIOD frames.
  frameCount = (frameCount + 1) % CLOUD_TIME_PERIOD;
  using queue = device.queue();
  // The queue applies this write before it runs the command buffer that the submit below sends,
  // so the fragment reads the values of this frame.
  queue.writeBuffer(
    frameBuffer,
    0,
    Context.bytesOf<CloudFrame>(new CloudFrame(
      frameCount as f32,
      (width as f32) / (height as f32),
    )),
  );
  // The host owns the swapchain view. The wrapper adds no ownership, and `shutdown` never
  // disposes it.
  const target = new GPUTextureView(view);
  // One encoder records the whole frame. `using` disposes it after the submit.
  using encoder = device.createCommandEncoderDefault();
  // The clear value repeats the zenith color, so any pixel that the triangle misses still
  // shows sky.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.055, g: 0.16, b: 0.34, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The viewport and the scissor follow the current surface size, because the host resizes the
  // swapchain without a new pipeline.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and the vertex buffers in one call. TypeGPU
  // spells the same step as `.with(bindGroup)`.
  pipeline.bind(pass, [group], [vertices]);
  // The three vertices reach past the surface, and the rasterizer clips the excess.
  pass.draw(3);
  pass.end();
  // `finishDefault` closes the encoder, and `submit` hands the command buffer to the queue.
  // The host presents the surface after `frame` returns.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// The host calls `shutdown` once, before it releases the device. The bind group goes first,
// because it names the other handles.
export function shutdown(): void {
  if (activeGroup !== null) activeGroup.dispose();
  if (activeSampler !== null) activeSampler.dispose();
  if (activeNoiseView !== null) activeNoiseView.dispose();
  if (activeNoiseTexture !== null) activeNoiseTexture.dispose();
  if (activeFrameBuffer !== null) activeFrameBuffer.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  // The null assignments make a second `shutdown` call safe.
  activeGroup = null;
  activeSampler = null;
  activeNoiseView = null;
  activeNoiseTexture = null;
  activeFrameBuffer = null;
  activeVertices = null;
  activePipeline = null;
  activeDevice = null;
}
