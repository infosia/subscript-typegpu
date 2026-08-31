// example: stable-fluid
// Advances a stable-fluid velocity and ink field through texture-backed compute passes.
// The upstream photo is reduced to a 512-square host-generated Perlin image (EX6).
// The Noop and Metal probes accept compute-stage textureSampleLevel on filtered rgba16float,
// so both semi-Lagrangian advection passes keep native linear sampling without a reduction.
// Ported from TypeGPU's stable-fluid example (https://github.com/software-mansion/TypeGPU).

import {
  BindGroupLayoutSpec,
  BindingResource,
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Rgba16float,
  Sampler,
  StorageTexture2d,
  Texture2d,
  Uniform,
  VertexInvocation,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
  samplerResource,
  textureResource,
  writeTexturePixels,
} from "./typegpu";
import {
  Vec2f,
  Vec2i,
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
  BrushParams_SIZE,
  Vertex_STRIDE,
  advectInk_ENTRY,
  advectInk_LAYOUT0,
  advectInk_WGSL,
  advectVelocity_ENTRY,
  advectVelocity_LAYOUT0,
  advectVelocity_WGSL,
  brushSplat_ENTRY,
  brushSplat_LAYOUT0,
  brushSplat_WGSL,
  clearPressure_ENTRY,
  clearPressure_LAYOUT0,
  clearPressure_WGSL,
  divergence_ENTRY,
  divergence_LAYOUT0,
  divergence_WGSL,
  forceAdd_ENTRY,
  forceAdd_LAYOUT0,
  forceAdd_WGSL,
  gradientSubtract_ENTRY,
  gradientSubtract_LAYOUT0,
  gradientSubtract_WGSL,
  imageRender_FRAGMENT_ENTRY,
  imageRender_LAYOUT0,
  imageRender_TARGET_FORMAT,
  imageRender_VERTEX_ENTRY,
  imageRender_VERTEX_LAYOUT0,
  imageRender_WGSL,
  inkAdd_ENTRY,
  inkAdd_LAYOUT0,
  inkAdd_WGSL,
  inkRender_FRAGMENT_ENTRY,
  inkRender_LAYOUT0,
  inkRender_TARGET_FORMAT,
  inkRender_VERTEX_ENTRY,
  inkRender_VERTEX_LAYOUT0,
  inkRender_WGSL,
  pressureJacobi_ENTRY,
  pressureJacobi_LAYOUT0,
  pressureJacobi_WGSL,
  velocityRender_FRAGMENT_ENTRY,
  velocityRender_LAYOUT0,
  velocityRender_TARGET_FORMAT,
  velocityRender_VERTEX_ENTRY,
  velocityRender_VERTEX_LAYOUT0,
  velocityRender_WGSL,
  viscosityJacobi_ENTRY,
  viscosityJacobi_LAYOUT0,
  viscosityJacobi_WGSL,
} from "./main.typegpu";

const SIM_N: u32 = 256;
const BACKGROUND_N: u32 = 512;
const BACKGROUND_NOISE_N: u32 = 128;
const WORKGROUP_N: u32 = 16;
const DT: f32 = 0.5;
const VISCOSITY: f32 = 0.000001;
const JACOBI_ITERATIONS: u32 = 10;
const BRUSH_RADIUS: f32 = 16.0; // SIM_N / 16.
const INK_AMOUNT: f32 = 0.02;
const FORCE_SCALE: f32 = 1.0;

const DISPLAY_INK: u32 = 1;
const DISPLAY_VELOCITY: u32 = 2;
const DISPLAY_IMAGE: u32 = 3;

const TEXTURE_VELOCITY_A: i32 = 0;
const TEXTURE_VELOCITY_B: i32 = 1;
const TEXTURE_INK_A: i32 = 2;
const TEXTURE_INK_B: i32 = 3;
const TEXTURE_PRESSURE_A: i32 = 4;
const TEXTURE_PRESSURE_B: i32 = 5;
const TEXTURE_FORCE: i32 = 6;
const TEXTURE_ADDED_INK: i32 = 7;
const TEXTURE_DIVERGENCE: i32 = 8;
const TEXTURE_BACKGROUND: i32 = 9;

const COMPUTE_BRUSH: i32 = 0;
const COMPUTE_INK_ADD: i32 = 1;
const COMPUTE_FORCE_ADD: i32 = 2;
const COMPUTE_ADVECT_VELOCITY: i32 = 3;
const COMPUTE_VISCOSITY: i32 = 4;
const COMPUTE_DIVERGENCE: i32 = 5;
const COMPUTE_CLEAR_PRESSURE: i32 = 6;
const COMPUTE_PRESSURE: i32 = 7;
const COMPUTE_GRADIENT: i32 = 8;
const COMPUTE_ADVECT_INK: i32 = 9;

const RENDER_INK: i32 = 0;
const RENDER_VELOCITY: i32 = 1;
const RENDER_IMAGE: i32 = 2;

const GROUP_BRUSH: i32 = 0;
const GROUP_INK_AB: i32 = 1;
const GROUP_INK_BA: i32 = 2;
const GROUP_FORCE_AB: i32 = 3;
const GROUP_FORCE_BA: i32 = 4;
const GROUP_ADVECT_VELOCITY_AB: i32 = 5;
const GROUP_ADVECT_VELOCITY_BA: i32 = 6;
const GROUP_VISCOSITY_AB: i32 = 7;
const GROUP_VISCOSITY_BA: i32 = 8;
const GROUP_DIVERGENCE_A: i32 = 9;
const GROUP_DIVERGENCE_B: i32 = 10;
const GROUP_CLEAR_PRESSURE_A: i32 = 11;
const GROUP_PRESSURE_AB: i32 = 12;
const GROUP_PRESSURE_BA: i32 = 13;
const GROUP_GRADIENT_AB: i32 = 14;
const GROUP_GRADIENT_BA: i32 = 15;
const GROUP_ADVECT_INK_IA_VA: i32 = 16;
const GROUP_ADVECT_INK_IA_VB: i32 = 17;
const GROUP_ADVECT_INK_IB_VA: i32 = 18;
const GROUP_ADVECT_INK_IB_VB: i32 = 19;

const RENDER_GROUP_INK_A: i32 = 0;
const RENDER_GROUP_INK_B: i32 = 1;
const RENDER_GROUP_VELOCITY_A: i32 = 2;
const RENDER_GROUP_VELOCITY_B: i32 = 3;
const RENDER_GROUP_IMAGE_A: i32 = 4;
const RENDER_GROUP_IMAGE_B: i32 = 5;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class BrushParams {
  point: Vec2f;
  delta: Vec2f;
  active: f32;

  constructor(point: Vec2f, delta: Vec2f, active: f32) {
    this.point = point;
    this.delta = delta;
    this.active = active;
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

class BrushLayout {
  force!: StorageTexture2d<Rgba16float>;
  addedInk!: StorageTexture2d<Rgba16float>;
  params!: Uniform<BrushParams>;
}

class AddLayout {
  source!: Texture2d<f32>;
  addition!: Texture2d<f32>;
  target!: StorageTexture2d<Rgba16float>;
}

class VelocityAdvectionLayout {
  quantity!: Texture2d<f32>;
  velocity!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
  viscosityRhs!: StorageTexture2d<Rgba16float>;
}

class AdvectionLayout {
  quantity!: Texture2d<f32>;
  velocity!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
}

class ViscosityLayout {
  rhs!: Texture2d<f32>;
  source!: Texture2d<f32>;
  target!: StorageTexture2d<Rgba16float>;
}

class DivergenceLayout {
  velocity!: Texture2d<f32>;
  target!: StorageTexture2d<Rgba16float>;
}

class ClearLayout {
  target!: StorageTexture2d<Rgba16float>;
}

class PressureLayout {
  pressure!: Texture2d<f32>;
  divergence!: Texture2d<f32>;
  target!: StorageTexture2d<Rgba16float>;
}

class GradientLayout {
  velocity!: Texture2d<f32>;
  pressure!: Texture2d<f32>;
  target!: StorageTexture2d<Rgba16float>;
}

class FieldRenderLayout {
  field!: Texture2d<f32>;
  linear!: Sampler;
}

class ImageRenderLayout {
  ink!: Texture2d<f32>;
  background!: Texture2d<f32>;
  linear!: Sampler;
}

function clampCell(value: i32): i32 {
  let result: i32 = value;
  if (result < 0) result = 0;
  if (result >= (SIM_N as i32)) result = (SIM_N - 1) as i32;
  return result;
}

// While a button is down, a Gaussian of force and ink lands at the pointer.
// An idle brush clears both transient fields.
function brushSplatKernel(res: BrushLayout, ctx: ComputeInvocation): void {
  const params: BrushParams = res.params.$;
  const cell = new Vec2f(ctx.globalId.x as f32, ctx.globalId.y as f32);
  const offset: Vec2f = cell.sub(params.point);
  const distanceSquared: f32 = offset.dot(offset);
  const radiusSquared: f32 = BRUSH_RADIUS * BRUSH_RADIUS;
  const exponent: f32 = -distanceSquared / (2.0 * radiusSquared);
  const gaussian: f32 = new Vec2f(exponent, exponent).exp().x * params.active;
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  res.force.store(
    coords,
    new Vec4f(
      params.delta.x * gaussian * FORCE_SCALE,
      params.delta.y * gaussian * FORCE_SCALE,
      0.0,
      0.0,
    ),
  );
  res.addedInk.store(
    coords,
    new Vec4f(gaussian * INK_AMOUNT, 0.0, 0.0, 0.0),
  );
}

// Adds the transient ink into the current field.
function inkAddKernel(res: AddLayout, ctx: ComputeInvocation): void {
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const ink: Vec4f = res.source.load(coords, 0);
  const added: Vec4f = res.addition.load(coords, 0);
  res.target.store(coords, ink.add(added));
}

// Applies the transient force to the velocity.
function forceAddKernel(res: AddLayout, ctx: ComputeInvocation): void {
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const velocity: Vec4f = res.source.load(coords, 0);
  const force: Vec4f = res.addition.load(coords, 0);
  res.target.store(
    coords,
    new Vec4f(
      velocity.x + force.x * DT,
      velocity.y + force.y * DT,
      0.0,
      0.0,
    ),
  );
}

// Semi-Lagrangian backtrace through the linear sampler. Borders stay zero.
function advectVelocityKernel(res: VelocityAdvectionLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const coords = new Vec2i(x as i32, y as i32);
  if (x === 0 || y === 0 || x + 1 === SIM_N || y + 1 === SIM_N) {
    res.target.store(coords, new Vec4f(0.0, 0.0, 0.0, 0.0));
    res.viscosityRhs.store(coords, new Vec4f(0.0, 0.0, 0.0, 0.0));
    return;
  }
  const velocity: Vec4f = res.velocity.load(coords, 0);
  const uv = new Vec2f(
    ((x as f32) + 0.5 - velocity.x * DT) / (SIM_N as f32),
    ((y as f32) + 0.5 - velocity.y * DT) / (SIM_N as f32),
  );
  const advected: Vec4f = res.quantity.sampleLevel(res.linear, uv, 0.0);
  const value = new Vec4f(advected.x, advected.y, 0.0, 0.0);
  res.target.store(coords, value);
  res.viscosityRhs.store(coords, value);
}

// One Jacobi step against the advected field as the fixed right-hand side.
function viscosityJacobiKernel(res: ViscosityLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const left: Vec4f = res.source.load(new Vec2i(clampCell(x - 1), y), 0);
  const right: Vec4f = res.source.load(new Vec2i(clampCell(x + 1), y), 0);
  const down: Vec4f = res.source.load(new Vec2i(x, clampCell(y - 1)), 0);
  const up: Vec4f = res.source.load(new Vec2i(x, clampCell(y + 1)), 0);
  const rhs: Vec4f = res.rhs.load(new Vec2i(x, y), 0);
  const alpha: f32 = VISCOSITY * DT * (SIM_N as f32) * (SIM_N as f32);
  const denominator: f32 = 1.0 + 4.0 * alpha;
  res.target.store(
    new Vec2i(x, y),
    new Vec4f(
      (rhs.x + alpha * (left.x + right.x + down.x + up.x)) / denominator,
      (rhs.y + alpha * (left.y + right.y + down.y + up.y)) / denominator,
      0.0,
      0.0,
    ),
  );
}

// Centered divergence with edge-clamped neighbors.
function divergenceKernel(res: DivergenceLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const left: Vec4f = res.velocity.load(new Vec2i(clampCell(x - 1), y), 0);
  const right: Vec4f = res.velocity.load(new Vec2i(clampCell(x + 1), y), 0);
  const down: Vec4f = res.velocity.load(new Vec2i(x, clampCell(y - 1)), 0);
  const up: Vec4f = res.velocity.load(new Vec2i(x, clampCell(y + 1)), 0);
  const value: f32 = 0.5 * ((right.x - left.x) + (up.y - down.y));
  res.target.store(new Vec2i(x, y), new Vec4f(value, 0.0, 0.0, 0.0));
}

// The Poisson solve starts from zero pressure every frame.
function clearPressureKernel(res: ClearLayout, ctx: ComputeInvocation): void {
  res.target.store(
    new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32),
    new Vec4f(0.0, 0.0, 0.0, 0.0),
  );
}

// One Jacobi step of the pressure Poisson solve.
function pressureJacobiKernel(res: PressureLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const left: f32 = res.pressure.load(new Vec2i(clampCell(x - 1), y), 0).x;
  const right: f32 = res.pressure.load(new Vec2i(clampCell(x + 1), y), 0).x;
  const down: f32 = res.pressure.load(new Vec2i(x, clampCell(y - 1)), 0).x;
  const up: f32 = res.pressure.load(new Vec2i(x, clampCell(y + 1)), 0).x;
  const divergenceValue: f32 = res.divergence.load(new Vec2i(x, y), 0).x;
  const pressure: f32 = (left + right + down + up - divergenceValue) * 0.25;
  res.target.store(new Vec2i(x, y), new Vec4f(pressure, 0.0, 0.0, 0.0));
}

// Removes the pressure gradient, the projection to a divergence-free field.
function gradientSubtractKernel(res: GradientLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const left: f32 = res.pressure.load(new Vec2i(clampCell(x - 1), y), 0).x;
  const right: f32 = res.pressure.load(new Vec2i(clampCell(x + 1), y), 0).x;
  const down: f32 = res.pressure.load(new Vec2i(x, clampCell(y - 1)), 0).x;
  const up: f32 = res.pressure.load(new Vec2i(x, clampCell(y + 1)), 0).x;
  const velocity: Vec4f = res.velocity.load(new Vec2i(x, y), 0);
  res.target.store(
    new Vec2i(x, y),
    new Vec4f(
      velocity.x - 0.5 * (right - left),
      velocity.y - 0.5 * (up - down),
      0.0,
      0.0,
    ),
  );
}

// Carries the ink through the projected velocity.
function advectInkKernel(res: AdvectionLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const coords = new Vec2i(x as i32, y as i32);
  const velocity: Vec4f = res.velocity.load(coords, 0);
  const uv = new Vec2f(
    ((x as f32) + 0.5 - velocity.x * DT) / (SIM_N as f32),
    ((y as f32) + 0.5 - velocity.y * DT) / (SIM_N as f32),
  );
  const ink: Vec4f = res.quantity.sampleLevel(res.linear, uv, 0.0);
  res.target.store(coords, ink);
}

function fieldVertex(
  res: FieldRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

function imageVertex(
  res: ImageRenderLayout,
  value: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// Key 1: the ink density tints the output.
function inkFragment(
  res: FieldRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  let density: f32 = res.field.sampleLevel(res.linear, input.uv, 0.0).x;
  if (density < 0.0) density = 0.0;
  if (density > 1.0) density = 1.0;
  return new Vec4f(
    0.015 + density * 0.12,
    0.025 + density * 0.58,
    0.045 + density * 0.92,
    1.0,
  );
}

// Key 2: velocity direction and magnitude map to color.
function velocityFragment(
  res: FieldRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const velocity: Vec4f = res.field.sampleLevel(res.linear, input.uv, 0.0);
  const speed: f32 = new Vec2f(velocity.x, velocity.y).length();
  let directionX: f32 = 0.0;
  let directionY: f32 = 0.0;
  if (speed > 0.00001) {
    directionX = velocity.x / speed;
    directionY = velocity.y / speed;
  }
  let magnitude: f32 = speed * 0.08;
  if (magnitude > 1.0) magnitude = 1.0;
  return new Vec4f(
    0.04 + magnitude * (0.5 + 0.5 * directionX),
    0.04 + magnitude * (0.5 + 0.5 * directionY),
    0.07 + magnitude * (0.5 - 0.25 * directionX - 0.25 * directionY),
    1.0,
  );
}

// Key 3: the ink gradient warps the background lookup, the upstream refraction.
function imageFragment(
  res: ImageRenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const texel: f32 = 1.0 / (SIM_N as f32);
  const left: f32 = res.ink.sampleLevel(
    res.linear,
    new Vec2f(input.uv.x - texel, input.uv.y),
    0.0,
  ).x;
  const right: f32 = res.ink.sampleLevel(
    res.linear,
    new Vec2f(input.uv.x + texel, input.uv.y),
    0.0,
  ).x;
  const down: f32 = res.ink.sampleLevel(
    res.linear,
    new Vec2f(input.uv.x, input.uv.y - texel),
    0.0,
  ).x;
  const up: f32 = res.ink.sampleLevel(
    res.linear,
    new Vec2f(input.uv.x, input.uv.y + texel),
    0.0,
  ).x;
  const warp = new Vec2f(right - left, up - down).scale(0.035);
  const background: Vec4f = res.background.sampleLevel(
    res.linear,
    input.uv.add(warp),
    0.0,
  );
  let density: f32 = res.ink.sampleLevel(res.linear, input.uv, 0.0).x;
  if (density < 0.0) density = 0.0;
  if (density > 1.0) density = 1.0;
  return new Vec4f(
    background.x + density * 0.05,
    background.y + density * 0.12,
    background.z + density * 0.18,
    1.0,
  );
}

export const brushSplat: ComputePipelineSpec = computePipeline<BrushLayout>(
  brushSplatKernel,
  { name: "brushSplat", workgroupSize: [16, 16, 1] },
);

export const inkAdd: ComputePipelineSpec = computePipeline<AddLayout>(
  inkAddKernel,
  { name: "inkAdd", workgroupSize: [16, 16, 1] },
);

export const forceAdd: ComputePipelineSpec = computePipeline<AddLayout>(
  forceAddKernel,
  { name: "forceAdd", workgroupSize: [16, 16, 1] },
);

export const advectVelocity: ComputePipelineSpec = computePipeline<VelocityAdvectionLayout>(
  advectVelocityKernel,
  { name: "advectVelocity", workgroupSize: [16, 16, 1] },
);

export const viscosityJacobi: ComputePipelineSpec = computePipeline<ViscosityLayout>(
  viscosityJacobiKernel,
  { name: "viscosityJacobi", workgroupSize: [16, 16, 1] },
);

export const divergence: ComputePipelineSpec = computePipeline<DivergenceLayout>(
  divergenceKernel,
  { name: "divergence", workgroupSize: [16, 16, 1] },
);

export const clearPressure: ComputePipelineSpec = computePipeline<ClearLayout>(
  clearPressureKernel,
  { name: "clearPressure", workgroupSize: [16, 16, 1] },
);

export const pressureJacobi: ComputePipelineSpec = computePipeline<PressureLayout>(
  pressureJacobiKernel,
  { name: "pressureJacobi", workgroupSize: [16, 16, 1] },
);

export const gradientSubtract: ComputePipelineSpec = computePipeline<GradientLayout>(
  gradientSubtractKernel,
  { name: "gradientSubtract", workgroupSize: [16, 16, 1] },
);

export const advectInk: ComputePipelineSpec = computePipeline<AdvectionLayout>(
  advectInkKernel,
  { name: "advectInk", workgroupSize: [16, 16, 1] },
);

export const inkRender: RenderPipelineSpec = renderPipelineL<
  FieldRenderLayout,
  Vertex,
  Varyings
>(fieldVertex, inkFragment, { format: "bgra8unorm" });

export const velocityRender: RenderPipelineSpec = renderPipelineL<
  FieldRenderLayout,
  Vertex,
  Varyings
>(fieldVertex, velocityFragment, { format: "bgra8unorm" });

export const imageRender: RenderPipelineSpec = renderPipelineL<
  ImageRenderLayout,
  Vertex,
  Varyings
>(imageVertex, imageFragment, { format: "bgra8unorm" });

class StableFluidState {
  device: GPUHostOwnedDevice;
  compute: ComputePipeline[];
  render: RenderPipeline[];
  groups: GPUBindGroup[];
  renderGroups: GPUBindGroup[];
  vertices: GPUBuffer;
  brushParams: GPUBuffer;
  textures: GPUTexture[];
  views: GPUTextureView[];
  linearSampler: GPUSampler;

  constructor(
    device: GPUHostOwnedDevice,
    compute: ComputePipeline[],
    render: RenderPipeline[],
    groups: GPUBindGroup[],
    renderGroups: GPUBindGroup[],
    vertices: GPUBuffer,
    brushParams: GPUBuffer,
    textures: GPUTexture[],
    views: GPUTextureView[],
    linearSampler: GPUSampler,
  ) {
    this.device = device;
    this.compute = compute;
    this.render = render;
    this.groups = groups;
    this.renderGroups = renderGroups;
    this.vertices = vertices;
    this.brushParams = brushParams;
    this.textures = textures;
    this.views = views;
    this.linearSampler = linearSampler;
  }
}

let activeState: StableFluidState | null = null;
let velocityAIsCurrent: boolean = true;
let inkAIsCurrent: boolean = true;
let displayMode: u32 = DISPLAY_IMAGE;
let previousPointerX: f32 = -1.0;
let previousPointerY: f32 = -1.0;
let wasDrawing: boolean = false;

function zeroField(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < SIM_N * SIM_N) {
    pixels.push(new Vec4f(0.0, 0.0, 0.0, 0.0));
    index += 1;
  }
  return pixels;
}

function backgroundPixels(): Vec4f[] {
  const noiseSamples: f32[] = [];
  let sampleY: u32 = 0;
  while (sampleY < BACKGROUND_NOISE_N) {
    let sampleX: u32 = 0;
    while (sampleX < BACKGROUND_NOISE_N) {
      const sampleU: f32 = (sampleX as f32) / ((BACKGROUND_NOISE_N - 1) as f32);
      const sampleV: f32 = (sampleY as f32) / ((BACKGROUND_NOISE_N - 1) as f32);
      noiseSamples.push(
        perlin3d(new Vec3f(sampleU * 5.0, sampleV * 5.0, 0.75)) * 0.5 + 0.5,
      );
      sampleX += 1;
    }
    sampleY += 1;
  }
  const pixels: Vec4f[] = [];
  let y: u32 = 0;
  while (y < BACKGROUND_N) {
    let x: u32 = 0;
    while (x < BACKGROUND_N) {
      const u: f32 = (x as f32) / ((BACKGROUND_N - 1) as f32);
      const v: f32 = (y as f32) / ((BACKGROUND_N - 1) as f32);
      const scaledX: f32 = u * ((BACKGROUND_NOISE_N - 1) as f32);
      const scaledY: f32 = v * ((BACKGROUND_NOISE_N - 1) as f32);
      const x0: u32 = Math.floor(scaledX as f64) as u32;
      const y0: u32 = Math.floor(scaledY as f64) as u32;
      const x1: u32 = x0 + 1 < BACKGROUND_NOISE_N ? x0 + 1 : x0;
      const y1: u32 = y0 + 1 < BACKGROUND_NOISE_N ? y0 + 1 : y0;
      const blendX: f32 = scaledX - (x0 as f32);
      const blendY: f32 = scaledY - (y0 as f32);
      const nearNoise: f32 = noiseSamples[(y0 * BACKGROUND_NOISE_N + x0) as i32]
        * (1.0 - blendX)
        + noiseSamples[(y0 * BACKGROUND_NOISE_N + x1) as i32] * blendX;
      const farNoise: f32 = noiseSamples[(y1 * BACKGROUND_NOISE_N + x0) as i32]
        * (1.0 - blendX)
        + noiseSamples[(y1 * BACKGROUND_NOISE_N + x1) as i32] * blendX;
      const noise: f32 = nearNoise * (1.0 - blendY) + farNoise * blendY;
      pixels.push(new Vec4f(
        0.08 + 0.34 * u + 0.12 * noise,
        0.12 + 0.28 * v + 0.16 * noise,
        0.20 + 0.28 * (1.0 - u) + 0.22 * noise,
        1.0,
      ));
      x += 1;
    }
    y += 1;
  }
  return pixels;
}

function createFieldTexture(device: GPUHostOwnedDevice, label: string): GPUTexture {
  return device.createTexture({
    label,
    size: { width: SIM_N, height: SIM_N },
    format: "rgba16float",
    usage: GPUTextureUsage.STORAGE_BINDING
      + GPUTextureUsage.TEXTURE_BINDING
      + GPUTextureUsage.COPY_DST,
  });
}

function bindCompute(
  device: GPUHostOwnedDevice,
  pipeline: ComputePipeline,
  spec: BindGroupLayoutSpec,
  resources: BindingResource[],
): GPUBindGroup {
  using layout = pipeline.bindGroupLayout(0);
  return createBindGroupHost(device, layout, spec, resources);
}

function bindRender(
  device: GPUHostOwnedDevice,
  pipeline: RenderPipeline,
  spec: BindGroupLayoutSpec,
  resources: BindingResource[],
): GPUBindGroup {
  using layout = pipeline.bindGroupLayout(0);
  return createBindGroupHost(device, layout, spec, resources);
}

function disposeGroups(groups: GPUBindGroup[]): void {
  let index: i32 = 0;
  while (index < groups.length) {
    groups[index].dispose();
    index += 1;
  }
}

function disposeCompute(pipelines: ComputePipeline[]): void {
  let index: i32 = 0;
  while (index < pipelines.length) {
    pipelines[index].dispose();
    index += 1;
  }
}

function disposeRender(pipelines: RenderPipeline[]): void {
  let index: i32 = 0;
  while (index < pipelines.length) {
    pipelines[index].dispose();
    index += 1;
  }
}

function disposeViews(views: GPUTextureView[]): void {
  let index: i32 = 0;
  while (index < views.length) {
    views[index].dispose();
    index += 1;
  }
}

function disposeTextures(textures: GPUTexture[]): void {
  let index: i32 = 0;
  while (index < textures.length) {
    textures[index].dispose();
    index += 1;
  }
}

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (
    format !== inkRender_TARGET_FORMAT
    || format !== velocityRender_TARGET_FORMAT
    || format !== imageRender_TARGET_FORMAT
  ) {
    print(`FAIL format expected=${imageRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "stable-fluid-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const brushParams = hostDevice.createBuffer({
    label: "stable-fluid-brush-params",
    size: BrushParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });

  const textures: GPUTexture[] = [
    createFieldTexture(hostDevice, "stable-fluid-velocity-a"),
    createFieldTexture(hostDevice, "stable-fluid-velocity-b"),
    createFieldTexture(hostDevice, "stable-fluid-ink-a"),
    createFieldTexture(hostDevice, "stable-fluid-ink-b"),
    createFieldTexture(hostDevice, "stable-fluid-pressure-a"),
    createFieldTexture(hostDevice, "stable-fluid-pressure-b"),
    createFieldTexture(hostDevice, "stable-fluid-force"),
    createFieldTexture(hostDevice, "stable-fluid-added-ink"),
    createFieldTexture(hostDevice, "stable-fluid-divergence"),
    hostDevice.createTexture({
      label: "stable-fluid-background",
      size: { width: BACKGROUND_N, height: BACKGROUND_N },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    }),
  ];
  const views: GPUTextureView[] = [];
  let textureIndex: i32 = 0;
  while (textureIndex < textures.length) {
    views.push(textures[textureIndex].createView());
    textureIndex += 1;
  }
  const samplerDescriptor: GPUSamplerDescriptor = {
    addressModeU: "clamp-to-edge",
    addressModeV: "clamp-to-edge",
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
  queue.writeBuffer(
    brushParams,
    0,
    Context.bytesOf<BrushParams>(
      new BrushParams(new Vec2f(0.0, 0.0), new Vec2f(0.0, 0.0), 0.0),
    ),
  );
  const zeros: Vec4f[] = zeroField();
  let fieldIndex: i32 = 0;
  while (fieldIndex < TEXTURE_BACKGROUND) {
    writeTexturePixels(queue, textures[fieldIndex], zeros, SIM_N, SIM_N);
    fieldIndex += 1;
  }
  writeTexturePixels(
    queue,
    textures[TEXTURE_BACKGROUND],
    backgroundPixels(),
    BACKGROUND_N,
    BACKGROUND_N,
  );

  hostDevice.pushErrorScope("validation");
  const compute: ComputePipeline[] = [
    createComputePipelineHost(
      hostDevice,
      brushSplat_WGSL,
      brushSplat_ENTRY,
      [brushSplat_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      inkAdd_WGSL,
      inkAdd_ENTRY,
      [inkAdd_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      forceAdd_WGSL,
      forceAdd_ENTRY,
      [forceAdd_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      advectVelocity_WGSL,
      advectVelocity_ENTRY,
      [advectVelocity_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      viscosityJacobi_WGSL,
      viscosityJacobi_ENTRY,
      [viscosityJacobi_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      divergence_WGSL,
      divergence_ENTRY,
      [divergence_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      clearPressure_WGSL,
      clearPressure_ENTRY,
      [clearPressure_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      pressureJacobi_WGSL,
      pressureJacobi_ENTRY,
      [pressureJacobi_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      gradientSubtract_WGSL,
      gradientSubtract_ENTRY,
      [gradientSubtract_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
    createComputePipelineHost(
      hostDevice,
      advectInk_WGSL,
      advectInk_ENTRY,
      [advectInk_LAYOUT0],
      [WORKGROUP_N, WORKGROUP_N, 1],
    ),
  ];
  const render: RenderPipeline[] = [
    createRenderPipelineHost(
      hostDevice,
      inkRender_WGSL,
      inkRender_VERTEX_ENTRY,
      inkRender_FRAGMENT_ENTRY,
      [inkRender_LAYOUT0],
      [inkRender_VERTEX_LAYOUT0],
      inkRender,
    ),
    createRenderPipelineHost(
      hostDevice,
      velocityRender_WGSL,
      velocityRender_VERTEX_ENTRY,
      velocityRender_FRAGMENT_ENTRY,
      [velocityRender_LAYOUT0],
      [velocityRender_VERTEX_LAYOUT0],
      velocityRender,
    ),
    createRenderPipelineHost(
      hostDevice,
      imageRender_WGSL,
      imageRender_VERTEX_ENTRY,
      imageRender_FRAGMENT_ENTRY,
      [imageRender_LAYOUT0],
      [imageRender_VERTEX_LAYOUT0],
      imageRender,
    ),
  ];

  const groups: GPUBindGroup[] = [
    bindCompute(hostDevice, compute[COMPUTE_BRUSH], brushSplat_LAYOUT0, [
      textureResource(views[TEXTURE_FORCE]),
      textureResource(views[TEXTURE_ADDED_INK]),
      bufferResource(brushParams),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_INK_ADD], inkAdd_LAYOUT0, [
      textureResource(views[TEXTURE_INK_A]),
      textureResource(views[TEXTURE_ADDED_INK]),
      textureResource(views[TEXTURE_INK_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_INK_ADD], inkAdd_LAYOUT0, [
      textureResource(views[TEXTURE_INK_B]),
      textureResource(views[TEXTURE_ADDED_INK]),
      textureResource(views[TEXTURE_INK_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_FORCE_ADD], forceAdd_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_A]),
      textureResource(views[TEXTURE_FORCE]),
      textureResource(views[TEXTURE_VELOCITY_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_FORCE_ADD], forceAdd_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_B]),
      textureResource(views[TEXTURE_FORCE]),
      textureResource(views[TEXTURE_VELOCITY_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_ADVECT_VELOCITY], advectVelocity_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_A]),
      textureResource(views[TEXTURE_VELOCITY_A]),
      samplerResource(linearSampler),
      textureResource(views[TEXTURE_VELOCITY_B]),
      textureResource(views[TEXTURE_DIVERGENCE]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_ADVECT_VELOCITY], advectVelocity_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_B]),
      textureResource(views[TEXTURE_VELOCITY_B]),
      samplerResource(linearSampler),
      textureResource(views[TEXTURE_VELOCITY_A]),
      textureResource(views[TEXTURE_DIVERGENCE]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_VISCOSITY], viscosityJacobi_LAYOUT0, [
      textureResource(views[TEXTURE_DIVERGENCE]),
      textureResource(views[TEXTURE_VELOCITY_A]),
      textureResource(views[TEXTURE_VELOCITY_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_VISCOSITY], viscosityJacobi_LAYOUT0, [
      textureResource(views[TEXTURE_DIVERGENCE]),
      textureResource(views[TEXTURE_VELOCITY_B]),
      textureResource(views[TEXTURE_VELOCITY_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_DIVERGENCE], divergence_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_A]),
      textureResource(views[TEXTURE_DIVERGENCE]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_DIVERGENCE], divergence_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_B]),
      textureResource(views[TEXTURE_DIVERGENCE]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_CLEAR_PRESSURE], clearPressure_LAYOUT0, [
      textureResource(views[TEXTURE_PRESSURE_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_PRESSURE], pressureJacobi_LAYOUT0, [
      textureResource(views[TEXTURE_PRESSURE_A]),
      textureResource(views[TEXTURE_DIVERGENCE]),
      textureResource(views[TEXTURE_PRESSURE_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_PRESSURE], pressureJacobi_LAYOUT0, [
      textureResource(views[TEXTURE_PRESSURE_B]),
      textureResource(views[TEXTURE_DIVERGENCE]),
      textureResource(views[TEXTURE_PRESSURE_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_GRADIENT], gradientSubtract_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_A]),
      textureResource(views[TEXTURE_PRESSURE_A]),
      textureResource(views[TEXTURE_VELOCITY_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_GRADIENT], gradientSubtract_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_B]),
      textureResource(views[TEXTURE_PRESSURE_A]),
      textureResource(views[TEXTURE_VELOCITY_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_ADVECT_INK], advectInk_LAYOUT0, [
      textureResource(views[TEXTURE_INK_A]),
      textureResource(views[TEXTURE_VELOCITY_A]),
      samplerResource(linearSampler),
      textureResource(views[TEXTURE_INK_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_ADVECT_INK], advectInk_LAYOUT0, [
      textureResource(views[TEXTURE_INK_A]),
      textureResource(views[TEXTURE_VELOCITY_B]),
      samplerResource(linearSampler),
      textureResource(views[TEXTURE_INK_B]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_ADVECT_INK], advectInk_LAYOUT0, [
      textureResource(views[TEXTURE_INK_B]),
      textureResource(views[TEXTURE_VELOCITY_A]),
      samplerResource(linearSampler),
      textureResource(views[TEXTURE_INK_A]),
    ]),
    bindCompute(hostDevice, compute[COMPUTE_ADVECT_INK], advectInk_LAYOUT0, [
      textureResource(views[TEXTURE_INK_B]),
      textureResource(views[TEXTURE_VELOCITY_B]),
      samplerResource(linearSampler),
      textureResource(views[TEXTURE_INK_A]),
    ]),
  ];
  const renderGroups: GPUBindGroup[] = [
    bindRender(hostDevice, render[RENDER_INK], inkRender_LAYOUT0, [
      textureResource(views[TEXTURE_INK_A]),
      samplerResource(linearSampler),
    ]),
    bindRender(hostDevice, render[RENDER_INK], inkRender_LAYOUT0, [
      textureResource(views[TEXTURE_INK_B]),
      samplerResource(linearSampler),
    ]),
    bindRender(hostDevice, render[RENDER_VELOCITY], velocityRender_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_A]),
      samplerResource(linearSampler),
    ]),
    bindRender(hostDevice, render[RENDER_VELOCITY], velocityRender_LAYOUT0, [
      textureResource(views[TEXTURE_VELOCITY_B]),
      samplerResource(linearSampler),
    ]),
    bindRender(hostDevice, render[RENDER_IMAGE], imageRender_LAYOUT0, [
      textureResource(views[TEXTURE_INK_A]),
      textureResource(views[TEXTURE_BACKGROUND]),
      samplerResource(linearSampler),
    ]),
    bindRender(hostDevice, render[RENDER_IMAGE], imageRender_LAYOUT0, [
      textureResource(views[TEXTURE_INK_B]),
      textureResource(views[TEXTURE_BACKGROUND]),
      samplerResource(linearSampler),
    ]),
  ];
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    disposeGroups(renderGroups);
    disposeGroups(groups);
    disposeRender(render);
    disposeCompute(compute);
    linearSampler.dispose();
    disposeViews(views);
    disposeTextures(textures);
    brushParams.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  activeState = new StableFluidState(
    hostDevice,
    compute,
    render,
    groups,
    renderGroups,
    vertices,
    brushParams,
    textures,
    views,
    linearSampler,
  );
  velocityAIsCurrent = true;
  inkAIsCurrent = true;
  displayMode = DISPLAY_IMAGE;
  previousPointerX = -1.0;
  previousPointerY = -1.0;
  wasDrawing = false;
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
  if (activeState === null) return;
  const active = activeState;
  if (key === 49) displayMode = DISPLAY_INK;
  if (key === 50) displayMode = DISPLAY_VELOCITY;
  if (key === 51) displayMode = DISPLAY_IMAGE;

  let point = new Vec2f(0.0, 0.0);
  let delta = new Vec2f(0.0, 0.0);
  let brushActive: f32 = 0.0;
  const pointerValid: boolean = pointerX >= 0.0 && pointerY >= 0.0
    && width > 0 && height > 0;
  const drawing: boolean = pointerValid && buttons !== 0;
  if (pointerValid) {
    const currentX: f32 = (pointerX / (width as f32)) * (SIM_N as f32);
    const currentY: f32 = (1.0 - pointerY / (height as f32)) * (SIM_N as f32);
    point = new Vec2f(currentX, currentY);
    if (drawing) {
      brushActive = 1.0;
      if (wasDrawing) {
        delta = new Vec2f(
          currentX - previousPointerX,
          currentY - previousPointerY,
        );
      }
    }
    previousPointerX = currentX;
    previousPointerY = currentY;
  }
  wasDrawing = drawing;

  using queue = active.device.queue();
  queue.writeBuffer(
    active.brushParams,
    0,
    Context.bytesOf<BrushParams>(new BrushParams(point, delta, brushActive)),
  );
  using encoder = active.device.createCommandEncoderDefault();
  const workgroups: u32 = SIM_N / WORKGROUP_N;

  active.compute[COMPUTE_BRUSH].dispatch(
    encoder,
    [active.groups[GROUP_BRUSH]],
    workgroups,
    workgroups,
    1,
  );

  const inkAddGroup: GPUBindGroup = inkAIsCurrent
    ? active.groups[GROUP_INK_AB]
    : active.groups[GROUP_INK_BA];
  active.compute[COMPUTE_INK_ADD].dispatch(
    encoder,
    [inkAddGroup],
    workgroups,
    workgroups,
    1,
  );
  inkAIsCurrent = !inkAIsCurrent;

  const forceGroup: GPUBindGroup = velocityAIsCurrent
    ? active.groups[GROUP_FORCE_AB]
    : active.groups[GROUP_FORCE_BA];
  active.compute[COMPUTE_FORCE_ADD].dispatch(
    encoder,
    [forceGroup],
    workgroups,
    workgroups,
    1,
  );
  velocityAIsCurrent = !velocityAIsCurrent;

  const velocityAdvectionGroup: GPUBindGroup = velocityAIsCurrent
    ? active.groups[GROUP_ADVECT_VELOCITY_AB]
    : active.groups[GROUP_ADVECT_VELOCITY_BA];
  active.compute[COMPUTE_ADVECT_VELOCITY].dispatch(
    encoder,
    [velocityAdvectionGroup],
    workgroups,
    workgroups,
    1,
  );
  velocityAIsCurrent = !velocityAIsCurrent;

  let viscosityIteration: u32 = 0;
  while (viscosityIteration < JACOBI_ITERATIONS) {
    const viscosityGroup: GPUBindGroup = velocityAIsCurrent
      ? active.groups[GROUP_VISCOSITY_AB]
      : active.groups[GROUP_VISCOSITY_BA];
    active.compute[COMPUTE_VISCOSITY].dispatch(
      encoder,
      [viscosityGroup],
      workgroups,
      workgroups,
      1,
    );
    velocityAIsCurrent = !velocityAIsCurrent;
    viscosityIteration += 1;
  }

  const divergenceGroup: GPUBindGroup = velocityAIsCurrent
    ? active.groups[GROUP_DIVERGENCE_A]
    : active.groups[GROUP_DIVERGENCE_B];
  active.compute[COMPUTE_DIVERGENCE].dispatch(
    encoder,
    [divergenceGroup],
    workgroups,
    workgroups,
    1,
  );

  active.compute[COMPUTE_CLEAR_PRESSURE].dispatch(
    encoder,
    [active.groups[GROUP_CLEAR_PRESSURE_A]],
    workgroups,
    workgroups,
    1,
  );
  let pressureAIsCurrent: boolean = true;
  let pressureIteration: u32 = 0;
  while (pressureIteration < JACOBI_ITERATIONS) {
    const pressureGroup: GPUBindGroup = pressureAIsCurrent
      ? active.groups[GROUP_PRESSURE_AB]
      : active.groups[GROUP_PRESSURE_BA];
    active.compute[COMPUTE_PRESSURE].dispatch(
      encoder,
      [pressureGroup],
      workgroups,
      workgroups,
      1,
    );
    pressureAIsCurrent = !pressureAIsCurrent;
    pressureIteration += 1;
  }

  const gradientGroup: GPUBindGroup = velocityAIsCurrent
    ? active.groups[GROUP_GRADIENT_AB]
    : active.groups[GROUP_GRADIENT_BA];
  active.compute[COMPUTE_GRADIENT].dispatch(
    encoder,
    [gradientGroup],
    workgroups,
    workgroups,
    1,
  );
  velocityAIsCurrent = !velocityAIsCurrent;

  let inkAdvectionGroup: GPUBindGroup = active.groups[GROUP_ADVECT_INK_IA_VA];
  if (inkAIsCurrent && !velocityAIsCurrent) {
    inkAdvectionGroup = active.groups[GROUP_ADVECT_INK_IA_VB];
  } else if (!inkAIsCurrent && velocityAIsCurrent) {
    inkAdvectionGroup = active.groups[GROUP_ADVECT_INK_IB_VA];
  } else if (!inkAIsCurrent && !velocityAIsCurrent) {
    inkAdvectionGroup = active.groups[GROUP_ADVECT_INK_IB_VB];
  }
  active.compute[COMPUTE_ADVECT_INK].dispatch(
    encoder,
    [inkAdvectionGroup],
    workgroups,
    workgroups,
    1,
  );
  inkAIsCurrent = !inkAIsCurrent;

  let renderPipeline: RenderPipeline = active.render[RENDER_IMAGE];
  let renderGroup: GPUBindGroup = inkAIsCurrent
    ? active.renderGroups[RENDER_GROUP_IMAGE_A]
    : active.renderGroups[RENDER_GROUP_IMAGE_B];
  if (displayMode === DISPLAY_INK) {
    renderPipeline = active.render[RENDER_INK];
    renderGroup = inkAIsCurrent
      ? active.renderGroups[RENDER_GROUP_INK_A]
      : active.renderGroups[RENDER_GROUP_INK_B];
  } else if (displayMode === DISPLAY_VELOCITY) {
    renderPipeline = active.render[RENDER_VELOCITY];
    renderGroup = velocityAIsCurrent
      ? active.renderGroups[RENDER_GROUP_VELOCITY_A]
      : active.renderGroups[RENDER_GROUP_VELOCITY_B];
  }

  const target = new GPUTextureView(view);
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.025, b: 0.045, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  renderPipeline.bind(renderPass, [renderGroup], [active.vertices]);
  renderPass.draw(3);
  renderPass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeState === null) return;
  const active = activeState;
  disposeGroups(active.renderGroups);
  disposeGroups(active.groups);
  active.linearSampler.dispose();
  disposeViews(active.views);
  disposeTextures(active.textures);
  active.brushParams.dispose();
  active.vertices.dispose();
  disposeRender(active.render);
  disposeCompute(active.compute);
  activeState = null;
  velocityAIsCurrent = true;
  inkAIsCurrent = true;
  displayMode = DISPLAY_IMAGE;
  previousPointerX = -1.0;
  previousPointerY = -1.0;
  wasDrawing = false;
}
