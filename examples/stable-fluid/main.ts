// example: stable-fluid
// Advances a stable-fluid velocity and ink field through texture-backed compute passes.
// The upstream photo is reduced to a 512-square host-generated Perlin image, and the
// simulation grid is 256 where upstream runs 512. Both advections sample through the
// linear filter in compute, as upstream does. The upstream sliders for the time step,
// the viscosity, and the Jacobi iterations commit to 0.5, 0.000001, and 10, and the
// pause toggle does not port. Keys 1, 2, and 3 select display modes, and the pointer
// drives the brush.
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

// The committed sizes. The simulation grid is 256 cells square and the background image is
// 512 pixels square. TypeGPU runs a 512-cell grid against a 2048-pixel photo.
const SIM_N: u32 = 256;
const BACKGROUND_N: u32 = 512;
const BACKGROUND_NOISE_N: u32 = 128;
const WORKGROUP_N: u32 = 16;
// The solver constants. `DT` is one frame of simulated time and `VISCOSITY` is the kinematic
// viscosity. TypeGPU exposes the three of them as sliders, and this port fixes them.
const DT: f32 = 0.5;
const VISCOSITY: f32 = 0.000001;
const JACOBI_ITERATIONS: u32 = 10;
const BRUSH_RADIUS: f32 = 16.0; // SIM_N / 16.
const INK_AMOUNT: f32 = 0.02;
const FORCE_SCALE: f32 = 1.0;

// The three display modes. The upstream select control becomes key 1, key 2, and key 3.
const DISPLAY_INK: u32 = 1;
const DISPLAY_VELOCITY: u32 = 2;
const DISPLAY_IMAGE: u32 = 3;

// The index names for the texture list, the pipeline lists, and the bind group lists. Every
// handle lives in an array, so one loop releases each list.
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

// Every source and target pair of the ping-pong owns a bind group, built once in `init`. A
// frame selects a group and never builds one.
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

// One render group per display mode and per current texture.
const RENDER_GROUP_INK_A: i32 = 0;
const RENDER_GROUP_INK_B: i32 = 1;
const RENDER_GROUP_VELOCITY_A: i32 = 2;
const RENDER_GROUP_VELOCITY_B: i32 = 3;
const RENDER_GROUP_IMAGE_A: i32 = 4;
const RENDER_GROUP_IMAGE_B: i32 = 5;

// One clip-space corner of the full-screen triangle. The generator derives the vertex
// attribute layout and the `Vertex_STRIDE` byte stride from this class.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// The brush state of one frame: the pointer cell, the movement since the last frame in
// cells, and 1.0 while a button is down.
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

// The vertex output. The `Vec4f` field named `position` becomes the WGSL builtin position,
// and every other field becomes an interpolated location.
@CStruct
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

// The layout classes replace TypeGPU's run-time bind group layout objects. The field
// order is the binding order, and the generator emits one `_LAYOUT0` spec per class.
class BrushLayout {
  force: StorageTexture2d<Rgba16float>;
  addedInk: StorageTexture2d<Rgba16float>;
  params: Uniform<BrushParams>;

  constructor(
    force: StorageTexture2d<Rgba16float>,
    addedInk: StorageTexture2d<Rgba16float>,
    params: Uniform<BrushParams>,
  ) {
    this.force = force;
    this.addedInk = addedInk;
    this.params = params;
  }
}

// The ink pass and the force pass share one layout. Each reads a field and an addition and
// writes the sum to a third texture.
class AddLayout {
  source: Texture2d<f32>;
  addition: Texture2d<f32>;
  target: StorageTexture2d<Rgba16float>;

  constructor(
    source: Texture2d<f32>,
    addition: Texture2d<f32>,
    target: StorageTexture2d<Rgba16float>,
  ) {
    this.source = source;
    this.addition = addition;
    this.target = target;
  }
}

// The velocity advection writes its result twice: to the next velocity, and to the fixed
// right-hand side of the viscosity solve.
class VelocityAdvectionLayout {
  quantity: Texture2d<f32>;
  velocity: Texture2d<f32>;
  linear: Sampler;
  target: StorageTexture2d<Rgba16float>;
  viscosityRhs: StorageTexture2d<Rgba16float>;

  constructor(
    quantity: Texture2d<f32>,
    velocity: Texture2d<f32>,
    linear: Sampler,
    target: StorageTexture2d<Rgba16float>,
    viscosityRhs: StorageTexture2d<Rgba16float>,
  ) {
    this.quantity = quantity;
    this.velocity = velocity;
    this.linear = linear;
    this.target = target;
    this.viscosityRhs = viscosityRhs;
  }
}

class AdvectionLayout {
  quantity: Texture2d<f32>;
  velocity: Texture2d<f32>;
  linear: Sampler;
  target: StorageTexture2d<Rgba16float>;

  constructor(
    quantity: Texture2d<f32>,
    velocity: Texture2d<f32>,
    linear: Sampler,
    target: StorageTexture2d<Rgba16float>,
  ) {
    this.quantity = quantity;
    this.velocity = velocity;
    this.linear = linear;
    this.target = target;
  }
}

// `rhs` holds the advected velocity and stays fixed through the ten Jacobi steps.
class ViscosityLayout {
  rhs: Texture2d<f32>;
  source: Texture2d<f32>;
  target: StorageTexture2d<Rgba16float>;

  constructor(rhs: Texture2d<f32>, source: Texture2d<f32>, target: StorageTexture2d<Rgba16float>) {
    this.rhs = rhs;
    this.source = source;
    this.target = target;
  }
}

class DivergenceLayout {
  velocity: Texture2d<f32>;
  target: StorageTexture2d<Rgba16float>;

  constructor(velocity: Texture2d<f32>, target: StorageTexture2d<Rgba16float>) {
    this.velocity = velocity;
    this.target = target;
  }
}

class ClearLayout {
  target: StorageTexture2d<Rgba16float>;

  constructor(target: StorageTexture2d<Rgba16float>) {
    this.target = target;
  }
}

class PressureLayout {
  pressure: Texture2d<f32>;
  divergence: Texture2d<f32>;
  target: StorageTexture2d<Rgba16float>;

  constructor(
    pressure: Texture2d<f32>,
    divergence: Texture2d<f32>,
    target: StorageTexture2d<Rgba16float>,
  ) {
    this.pressure = pressure;
    this.divergence = divergence;
    this.target = target;
  }
}

class GradientLayout {
  velocity: Texture2d<f32>;
  pressure: Texture2d<f32>;
  target: StorageTexture2d<Rgba16float>;

  constructor(
    velocity: Texture2d<f32>,
    pressure: Texture2d<f32>,
    target: StorageTexture2d<Rgba16float>,
  ) {
    this.velocity = velocity;
    this.pressure = pressure;
    this.target = target;
  }
}

// The ink view and the velocity view read one texture each, so one layout class serves both
// render pipelines.
class FieldRenderLayout {
  field: Texture2d<f32>;
  linear: Sampler;

  constructor(field: Texture2d<f32>, linear: Sampler) {
    this.field = field;
    this.linear = linear;
  }
}

class ImageRenderLayout {
  ink: Texture2d<f32>;
  background: Texture2d<f32>;
  linear: Sampler;

  constructor(ink: Texture2d<f32>, background: Texture2d<f32>, linear: Sampler) {
    this.ink = ink;
    this.background = background;
    this.linear = linear;
  }
}

// A neighbor read clamps at the border. That clamp is the boundary condition of this solver.
function clampCell(value: i32): i32 {
  let result: i32 = value;
  if (result < 0) result = 0;
  if (result >= (SIM_N as i32)) result = (SIM_N - 1) as i32;
  return result;
}

// While a button is down, a Gaussian of force and ink lands at the pointer.
// An idle brush clears both transient fields. Every cell takes a value here, so neither
// transient texture needs a clear pass.
function brushSplatKernel(res: BrushLayout, ctx: ComputeInvocation): void {
  const params: BrushParams = res.params.$;
  // The brush works in grid cells, so the host converts the pointer position before the write.
  const cell = new Vec2f(ctx.globalId.x as f32, ctx.globalId.y as f32);
  const offset: Vec2f = cell.sub(params.point);
  const distanceSquared: f32 = offset.dot(offset);
  const radiusSquared: f32 = BRUSH_RADIUS * BRUSH_RADIUS;
  // The Gaussian falls to `exp(-0.5)` at one radius and never reaches zero. TypeGPU cuts its
  // stamp off at the radius instead.
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
  // The sample point is this cell center minus one step of its velocity, in cells, and the
  // division normalizes it for the sampler.
  const uv = new Vec2f(
    ((x as f32) + 0.5 - velocity.x * DT) / (SIM_N as f32),
    ((y as f32) + 0.5 - velocity.y * DT) / (SIM_N as f32),
  );
  const advected: Vec4f = res.quantity.sampleLevel(res.linear, uv, 0.0);
  const value = new Vec4f(advected.x, advected.y, 0.0, 0.0);
  // The advected velocity is also the right-hand side of the viscosity solve, so the kernel
  // stores it twice. The second target is the divergence texture, still unused this frame.
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
  // `alpha` is the viscosity times the time step over the squared cell size, and the cell size
  // is 1 over `SIM_N`. TypeGPU normalizes the same step differently.
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

// The oversized triangle covers the clip square, and the uv maps clip space [-1, 1] to the
// field's [0, 1]. The two entries differ only in the layout class each stage shares.
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

// The thirteen declarations are the generator's input. It walks the typed program before the
// run and emits `main.typegpu.ts`: the WGSL text, the entry names, and the layout specs.
// TypeGPU resolves the same shaders from the kernel functions at run time.
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

// The three render pipelines share the vertex entry and differ in the fragment entry.
// `renderPipelineL` adds the layout class, so both stages read the same bindings.
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

// One object holds every handle a frame needs. Scripts own their handles, so `shutdown`
// releases each one. TypeGPU leaves that to garbage collection and `root.destroy`.
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

// `init` fills this binding and `frame` reads it. A failed `init` leaves it `null`, because
// this library reports a failure by value and never by an exception.
let activeState: StableFluidState | null = null;
// The ping-pong state. A true flag names texture A as the field that holds the current
// values, and every pass that writes the pair flips its flag.
let velocityAIsCurrent: boolean = true;
let inkAIsCurrent: boolean = true;
let displayMode: u32 = DISPLAY_IMAGE;
let previousPointerX: f32 = -1.0;
let previousPointerY: f32 = -1.0;
let wasDrawing: boolean = false;

// A new texture holds undefined contents, so every simulation field takes an explicit zero.
function zeroField(): Vec4f[] {
  const pixels: Vec4f[] = [];
  let index: u32 = 0;
  while (index < SIM_N * SIM_N) {
    pixels.push(new Vec4f(0.0, 0.0, 0.0, 0.0));
    index += 1;
  }
  return pixels;
}

// The background comes from host code. An example generates its assets and fetches nothing,
// and TypeGPU loads a photo through `createImageBitmap` instead.
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
  // The noise grid is 128 square and the bilinear step stretches it to 512, because the host
  // computes one noise sample at a time.
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

// Every simulation field is rgba16float with a storage binding, a texture binding, and a
// copy target. One pass writes it, the next pass samples it, and `init` zeroes it.
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

// The pipeline owns the bind group layout, and `using` borrows it for the group creation
// only.
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

// The release helpers give `shutdown` and the `init` failure path one shape.
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
  // The three render pipelines share one generated target format. A surface with another
  // format is a failure here, not a reason to rebuild them.
  if (
    format !== inkRender_TARGET_FORMAT
    || format !== velocityRender_TARGET_FORMAT
    || format !== imageRender_TARGET_FORMAT
  ) {
    print(`FAIL format expected=${imageRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The window host owns the device. The wrapper adds the API-layer methods and has neither
  // `dispose` nor `destroy`.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // One oversized triangle covers the screen. `Vertex_STRIDE` comes from the generator, so
  // the size never restates the layout.
  const vertices = hostDevice.createBuffer({
    label: "stable-fluid-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The brush uniform. Every frame rewrites it before the first dispatch.
  const brushParams = hostDevice.createBuffer({
    label: "stable-fluid-brush-params",
    size: BrushParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });

  // Nine simulation fields and the background. Velocity, ink, and pressure come in pairs,
  // because one dispatch cannot read and write the same texture.
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
  // One view per texture in the same order, so a `TEXTURE_` name indexes both lists.
  const views: GPUTextureView[] = [];
  let textureIndex: i32 = 0;
  while (textureIndex < textures.length) {
    views.push(textures[textureIndex].createView());
    textureIndex += 1;
  }
  // A linear filter with clamp-to-edge. The advection samples between cells, and a backtrace
  // that leaves the grid then reads the border cell.
  const samplerDescriptor: GPUSamplerDescriptor = {
    addressModeU: "clamp-to-edge",
    addressModeV: "clamp-to-edge",
    minFilter: "linear",
    magFilter: "linear",
  };
  const linearSampler = hostDevice.createSampler(samplerDescriptor);

  // The queue handle is borrowed for this block, and `using` releases it at the end of `init`.
  // The resources it fills stay.
  using queue = hostDevice.queue();
  // `Context.bytesOf<T>` produces the exact bytes of the value in the generated layout.
  // TypeGPU converts a JavaScript object to buffer bytes at run time instead.
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
  // The nine simulation fields start at zero. `TEXTURE_BACKGROUND` is the first index past
  // them, so the loop stops before the background.
  const zeros: Vec4f[] = zeroField();
  let fieldIndex: i32 = 0;
  while (fieldIndex < TEXTURE_BACKGROUND) {
    writeTexturePixels(queue, textures[fieldIndex], zeros, SIM_N, SIM_N);
    fieldIndex += 1;
  }
  // `writeTexturePixels` converts each `Vec4f` into the texture's format and uploads the rows.
  // The background never changes after this write.
  writeTexturePixels(
    queue,
    textures[TEXTURE_BACKGROUND],
    backgroundPixels(),
    BACKGROUND_N,
    BACKGROUND_N,
  );

  // The scope catches a backend rejection of the WGSL or the layout. The API layer returns
  // the error as a value, so the code reads the popped result.
  hostDevice.pushErrorScope("validation");
  // Each pipeline takes generated WGSL text, a generated entry name, and a generated layout
  // spec. This file holds no shader text.
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
  // A render pipeline also takes the vertex layout the generator derived from `Vertex`.
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

  // Every bind group is built once, in the order of the `GROUP_` names. A pass that flips the
  // ping-pong gets two groups, one per direction.
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
    // The divergence texture doubles as the viscosity right-hand side before its overwrite.
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
    // Ten pressure iterations are even, so pressure A is current after the loop.
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
  // Each render group adds the linear sampler, so a fragment samples the field and the
  // window size never has to match the grid.
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
  // The failure path releases every handle this function created, newest first. Nothing else
  // frees them, because the state never received them.
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

  // The state takes every handle above. `frame` and `shutdown` reach them through this module
  // binding.
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
  // A failed `init` leaves no state. The host still calls `frame`, so the guard returns.
  if (activeState === null) return;
  const active = activeState;
  // The host reports one key per frame and clears the slot, so a press acts once. 49, 50, and
  // 51 are the Unicode scalars of `1`, `2`, and `3`.
  if (key === 49) displayMode = DISPLAY_INK;
  if (key === 50) displayMode = DISPLAY_VELOCITY;
  if (key === 51) displayMode = DISPLAY_IMAGE;

  // An idle pointer leaves the brush inactive. The splat kernel then writes zeros over the
  // force and the added ink, so no force survives into the next frame.
  let point = new Vec2f(0.0, 0.0);
  let delta = new Vec2f(0.0, 0.0);
  let brushActive: f32 = 0.0;
  const pointerValid: boolean = pointerX >= 0.0 && pointerY >= 0.0
    && width > 0 && height > 0;
  const drawing: boolean = pointerValid && buttons !== 0;
  // The host reports surface pixels with y down. The grid runs from 0 to `SIM_N` with y up,
  // so the y axis flips here.
  if (pointerValid) {
    const currentX: f32 = (pointerX / (width as f32)) * (SIM_N as f32);
    const currentY: f32 = (1.0 - pointerY / (height as f32)) * (SIM_N as f32);
    point = new Vec2f(currentX, currentY);
    if (drawing) {
      brushActive = 1.0;
      // The delta is the pointer movement in cells since the last frame, and it becomes the force
      // direction. A stroke that starts this frame has no delta yet.
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

  // The queue and the encoder live for this frame only.
  using queue = active.device.queue();
  queue.writeBuffer(
    active.brushParams,
    0,
    Context.bytesOf<BrushParams>(new BrushParams(point, delta, brushActive)),
  );
  using encoder = active.device.createCommandEncoderDefault();
  // A dispatch count is a workgroup count. 256 cells over a workgroup of 16 give 16 by 16
  // workgroups per pass.
  const workgroups: u32 = SIM_N / WORKGROUP_N;

  // The dispatch order is the solver order: splat the brush, add the ink and the force, advect
  // the velocity, diffuse it, project it, then advect the ink. Each `dispatch` records its own
  // compute pass, and that pass order is the dependency order.
  active.compute[COMPUTE_BRUSH].dispatch(
    encoder,
    [active.groups[GROUP_BRUSH]],
    workgroups,
    workgroups,
    1,
  );

  // The flag names the texture that holds the current ink. The group reads it and writes the
  // other one, so the flip follows the dispatch.
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

  // The force enters the velocity as an acceleration over one time step.
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

  // The advection moves the velocity through itself, so its two texture bindings name the same
  // current field.
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

  // Ten Jacobi steps solve the implicit viscosity against the fixed right-hand side. The count
  // is host code, so the WGSL holds one step and the loop lives here.
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

  // The divergence pass overwrites the viscosity right-hand side. Every reader of that texture
  // is done by this point.
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

  // TypeGPU keeps the previous frame's pressure and warm starts the solve from it.
  active.compute[COMPUTE_CLEAR_PRESSURE].dispatch(
    encoder,
    [active.groups[GROUP_CLEAR_PRESSURE_A]],
    workgroups,
    workgroups,
    1,
  );
  // The pressure ping-pong lives in the frame, because the solve starts fresh. Ten steps are
  // even, so texture A holds the result and both gradient groups read it.
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

  // The projection subtracts the pressure gradient and leaves a divergence-free velocity.
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

  // The group depends on both flags, so four groups cover the four combinations of current
  // ink and current velocity.
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

  // The display mode selects the pipeline and the group. The ink view and the image view read
  // the current ink, and the velocity view reads the current velocity.
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

  // The host owns the frame's view and presents it. The wrapper adds the API-layer methods
  // and disposes nothing.
  const target = new GPUTextureView(view);
  // The color attachment clears on load, so no separate clear pass exists.
  using renderPass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.015, g: 0.025, b: 0.045, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The grid commits to 256 cells. The viewport stretches the field over the current window
  // instead of a resize of the simulation.
  renderPass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  renderPass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and the vertex buffers on the pass in one call.
  renderPipeline.bind(renderPass, [renderGroup], [active.vertices]);
  renderPass.draw(3);
  renderPass.end();
  // One submit sends every pass of this frame. TypeGPU submits one command buffer per dispatch
  // and per draw.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// Releases every handle in the reverse order of creation: groups, sampler, views, textures,
// buffers, then pipelines. The device and the frame view belong to the host.
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
  // The cleared binding makes a later `frame` call return at its guard, and the ping-pong
  // state starts over.
  activeState = null;
  velocityAIsCurrent = true;
  inkAIsCurrent = true;
  displayMode = DISPLAY_IMAGE;
  previousPointerX = -1.0;
  previousPointerY = -1.0;
  wasDrawing = false;
}
