// example: radiance-cascades-drawing
// Turns painted emissive strokes into a jump-flood SDF and a cascade-lit scene.
// The scene and flood commit to 512 square pixels. Lighting commits to the upstream
// quarter resolution of 128, and resize stretches the result. The light color commits
// to warm orange and the brush radius to 0.03. Key 0 clears; keys 1 and 2 select the lit
// and SDF views.
// The upstream color pickers, animated color, and brush-size slider are dropped.
// Ported from TypeGPU's radiance-cascades-drawing example (https://github.com/software-mansion/TypeGPU).

import {
  ComputeInvocation,
  ComputePipeline,
  ComputePipelineSpec,
  FragmentInvocation,
  ReadStorageTexture2dArray,
  RenderPipeline,
  RenderPipelineSpec,
  Rgba16float,
  Sampler,
  StorageTexture2d,
  Texture2d,
  Uniform,
  VertexInvocation,
  WriteStorageTexture2dArray,
  bufferResource,
  computePipeline,
  createBindGroupHost,
  createComputePipelineHost,
  createRenderPipelineHost,
  renderPipelineL,
  samplerResource,
  textureResource,
} from "./typegpu";
import {
  Vec2f,
  Vec2i,
  Vec2u,
  Vec3f,
  Vec4f,
  clamp,
  smoothstep,
} from "./typegpu-types";
import {
  cascadeDimensions,
  cascadeIntervalEnd,
  cascadeIntervalStart,
  cascadeMergeUv,
  cascadeProbesAt,
  cascadeRayAngle,
  cascadeRaysStored,
  cascadeWriteSide,
  radianceGatherUv,
} from "./typegpu-radiance-cascades";
import {
  sdLine,
} from "./typegpu-sdf";
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
  CascadeParams_SIZE,
  RenderParams_SIZE,
  StepParams_SIZE,
  Vertex_STRIDE,
  cascadePass_ENTRY,
  cascadePass_LAYOUT0,
  cascadePass_WGSL,
  fieldBuild_ENTRY,
  fieldBuild_LAYOUT0,
  fieldBuild_WGSL,
  floodDerive_ENTRY,
  floodDerive_LAYOUT0,
  floodDerive_WGSL,
  floodSeed_ENTRY,
  floodSeed_LAYOUT0,
  floodSeed_WGSL,
  floodStep_ENTRY,
  floodStep_LAYOUT0,
  floodStep_WGSL,
  radianceDrawingRender_FRAGMENT_ENTRY,
  radianceDrawingRender_LAYOUT0,
  radianceDrawingRender_TARGET_FORMAT,
  radianceDrawingRender_VERTEX_ENTRY,
  radianceDrawingRender_VERTEX_LAYOUT0,
  radianceDrawingRender_WGSL,
  sceneEdit_ENTRY,
  sceneEdit_LAYOUT0,
  sceneEdit_WGSL,
} from "./main.typegpu";

const SCENE_SIZE: u32 = 512;
const LIGHT_SIZE: u32 = 128;
const CASCADE_PROBES: u32 = 64;
const CASCADE_DIM: u32 = 128;
const CASCADE_COUNT: u32 = 5;
const FLOOD_LAYERS: u32 = 2;
const FLOOD_STEPS: u32 = 9;
const WORKGROUP_SIZE: u32 = 8;
const CASCADE_WORKGROUP_SIZE: u32 = 16;
const BRUSH_RADIUS: f32 = 0.03;
const SURFACE_EDGE: f32 = 0.002;
const EDIT_CLEAR: u32 = 1;
const EDIT_PAINT: u32 = 2;
const DISPLAY_LIT: u32 = 1;
const DISPLAY_SDF: u32 = 2;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

@CStruct
class BrushParams {
  previous: Vec2f;
  current: Vec2f;
  mode: u32;

  constructor(previous: Vec2f, current: Vec2f, mode: u32) {
    this.previous = previous;
    this.current = current;
    this.mode = mode;
  }
}

@CStruct
class StepParams {
  offset: i32;

  constructor(offset: i32) {
    this.offset = offset;
  }
}

@CStruct
class CascadeParams {
  layer: u32;
  probes: u32;
  cascadeCount: u32;
  baseProbes: u32;

  constructor(layer: u32, probes: u32, cascadeCount: u32, baseProbes: u32) {
    this.layer = layer;
    this.probes = probes;
    this.cascadeCount = cascadeCount;
    this.baseProbes = baseProbes;
  }
}

@CStruct
class RenderParams {
  mode: u32;

  constructor(mode: u32) {
    this.mode = mode;
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

class SceneEditLayout {
  scene!: StorageTexture2d<Rgba16float>;
  brush!: Uniform<BrushParams>;
}

class FloodSeedLayout {
  scene!: Texture2d<f32>;
  target!: WriteStorageTexture2dArray<Rgba16float>;
}

class FloodStepLayout {
  source!: ReadStorageTexture2dArray<Rgba16float>;
  target!: WriteStorageTexture2dArray<Rgba16float>;
  params!: Uniform<StepParams>;
}

class FloodDeriveLayout {
  payload!: ReadStorageTexture2dArray<Rgba16float>;
  scene!: Texture2d<f32>;
  sdf!: StorageTexture2d<Rgba16float>;
  colors!: StorageTexture2d<Rgba16float>;
}

class CascadeLayout {
  upper!: Texture2d<f32>;
  sdf!: Texture2d<f32>;
  colors!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
  params!: Uniform<CascadeParams>;
}

class FieldLayout {
  cascade0!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
}

class RenderLayout {
  field!: Texture2d<f32>;
  sdf!: Texture2d<f32>;
  colors!: Texture2d<f32>;
  linear!: Sampler;
  params!: Uniform<RenderParams>;
}

// The edit pass only writes touched cells, so a stroke accumulates in one scene texture.
// A clear dispatch instead writes transparent black over the complete texture.
function sceneEditKernel(res: SceneEditLayout, ctx: ComputeInvocation): void {
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const brush: BrushParams = res.brush.$;
  if (brush.mode === EDIT_CLEAR) {
    res.scene.store(coords, new Vec4f(0.0, 0.0, 0.0, 0.0));
    return;
  }
  const point = new Vec2f(
    ((ctx.globalId.x as f32) + 0.5) / (SCENE_SIZE as f32),
    ((ctx.globalId.y as f32) + 0.5) / (SCENE_SIZE as f32),
  );
  const segment: Vec2f = brush.current.sub(brush.previous);
  let distance: f32 = point.distance(brush.current);
  if (segment.dot(segment) > 0.00000001) {
    distance = sdLine(point, brush.previous, brush.current);
  }
  if (distance <= BRUSH_RADIUS) {
    res.scene.store(coords, new Vec4f(1.0, 0.28, 0.06, 1.0));
  }
}

// Alpha classifies painted cells as seeds. The two layers keep seed color and its
// normalized coordinate together throughout the jump flood.
function floodSeedKernel(res: FloodSeedLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const coords = new Vec2i(x as i32, y as i32);
  const scene: Vec4f = res.scene.load(coords, 0);
  if (scene.w > 0.0) {
    res.target.store(coords, 0, scene);
    res.target.store(coords, 1, new Vec4f(
      ((x as f32) + 0.5) / (SCENE_SIZE as f32),
      ((y as f32) + 0.5) / (SCENE_SIZE as f32),
      0.0,
      0.0,
    ));
  } else {
    res.target.store(coords, 0, new Vec4f(0.0, 0.0, 0.0, 0.0));
    res.target.store(coords, 1, new Vec4f(-1.0, -1.0, 0.0, 0.0));
  }
}

function seedDistance(point: Vec2f, seed: Vec4f): f32 {
  if (seed.x < 0.0) return 100000000000000000000.0;
  const delta: Vec2f = point.sub(new Vec2f(seed.x, seed.y));
  return delta.dot(delta);
}

// The committed 3 by 3 shape mirrors the voronoi pass and moves both payload layers
// whenever a nearer in-bounds seed is found.
function floodStepKernel(res: FloodStepLayout, ctx: ComputeInvocation): void {
  const x: i32 = ctx.globalId.x as i32;
  const y: i32 = ctx.globalId.y as i32;
  const offset: i32 = res.params.$.offset;
  const coords = new Vec2i(x, y);
  const point = new Vec2f(
    ((x as f32) + 0.5) / (SCENE_SIZE as f32),
    ((y as f32) + 0.5) / (SCENE_SIZE as f32),
  );
  let bestColor: Vec4f = res.source.load(coords, 0);
  let bestSeed: Vec4f = res.source.load(coords, 1);
  let bestDistance: f32 = seedDistance(point, bestSeed);

  const nw = new Vec2i(x - offset, y - offset);
  if (nw.x >= 0 && nw.y >= 0) {
    const seed: Vec4f = res.source.load(nw, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(nw, 0);
    }
  }
  const north = new Vec2i(x, y - offset);
  if (north.y >= 0) {
    const seed: Vec4f = res.source.load(north, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(north, 0);
    }
  }
  const ne = new Vec2i(x + offset, y - offset);
  if (ne.x < (SCENE_SIZE as i32) && ne.y >= 0) {
    const seed: Vec4f = res.source.load(ne, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(ne, 0);
    }
  }
  const west = new Vec2i(x - offset, y);
  if (west.x >= 0) {
    const seed: Vec4f = res.source.load(west, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(west, 0);
    }
  }
  const east = new Vec2i(x + offset, y);
  if (east.x < (SCENE_SIZE as i32)) {
    const seed: Vec4f = res.source.load(east, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(east, 0);
    }
  }
  const sw = new Vec2i(x - offset, y + offset);
  if (sw.x >= 0 && sw.y < (SCENE_SIZE as i32)) {
    const seed: Vec4f = res.source.load(sw, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(sw, 0);
    }
  }
  const south = new Vec2i(x, y + offset);
  if (south.y < (SCENE_SIZE as i32)) {
    const seed: Vec4f = res.source.load(south, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(south, 0);
    }
  }
  const se = new Vec2i(x + offset, y + offset);
  if (se.x < (SCENE_SIZE as i32) && se.y < (SCENE_SIZE as i32)) {
    const seed: Vec4f = res.source.load(se, 1);
    const distance: f32 = seedDistance(point, seed);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestSeed = seed;
      bestColor = res.source.load(se, 0);
    }
  }
  res.target.store(coords, 0, bestColor);
  res.target.store(coords, 1, bestSeed);
}

// Painted cells receive a half-cell negative distance. Empty cells carry their true
// distance to the nearest painted cell, while an empty scene uses a safe far distance.
function floodDeriveKernel(res: FloodDeriveLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  const coords = new Vec2i(x as i32, y as i32);
  const seed: Vec4f = res.payload.load(coords, 1);
  const color: Vec4f = res.payload.load(coords, 0);
  let distance: f32 = 2.0;
  if (seed.x >= 0.0) {
    const point = new Vec2f(
      ((x as f32) + 0.5) / (SCENE_SIZE as f32),
      ((y as f32) + 0.5) / (SCENE_SIZE as f32),
    );
    distance = point.distance(new Vec2f(seed.x, seed.y));
  }
  if (res.scene.load(coords, 0).w > 0.0) {
    distance = -0.5 / (SCENE_SIZE as f32);
  }
  res.sdf.store(coords, new Vec4f(distance, 0.0, 0.0, 1.0));
  res.colors.store(coords, color);
}

// One stored direction owns four actual rays. SDF and emission now come from the
// sampled jump-flood outputs instead of an analytic scene function.
function cascadeKernel(res: CascadeLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  if (x >= CASCADE_DIM || y >= CASCADE_DIM) return;
  const params: CascadeParams = res.params.$;
  const probes: u32 = params.probes;
  const raysStored: u32 = cascadeRaysStored(params.layer);
  const dirStored = new Vec2u(x / probes, y / probes);
  const probe = new Vec2u(x % probes, y % probes);
  const probePos = new Vec2f(
    ((probe.x as f32) + 0.5) / (probes as f32),
    ((probe.y as f32) + 0.5) / (probes as f32),
  );
  const interval0: f32 = 1.0 / (params.baseProbes as f32);
  const rayStart: f32 = cascadeIntervalStart(interval0, params.layer);
  const rayEnd: f32 = cascadeIntervalEnd(interval0, params.layer);
  const eps: f32 = 0.5 / (params.baseProbes as f32);
  const minStep: f32 = 0.25 / (params.baseProbes as f32);
  let accumulated = new Vec4f(0.0, 0.0, 0.0, 0.0);

  for (let quadrant: u32 = 0; quadrant < 4; quadrant += 1) {
    const dirActual = new Vec2u(
      dirStored.x * 2 + quadrant % 2,
      dirStored.y * 2 + quadrant / 2,
    );
    const angle: f32 = cascadeRayAngle(dirActual, raysStored * 2);
    // The shared vector carries the scalar sine and cosine in the kernel subset's form.
    const cosine: f32 = new Vec2f(angle, angle).cos().x;
    const sine: f32 = new Vec2f(angle, angle).sin().x;
    const rayDirection = new Vec2f(cosine, -sine);
    let radiance = new Vec3f(0.0, 0.0, 0.0);
    let transmittance: f32 = 1.0;
    let distanceAlong: f32 = rayStart;
    for (let step: u32 = 0; step < 64; step += 1) {
      if (distanceAlong > rayEnd) break;
      const point: Vec2f = probePos.add(rayDirection.scale(distanceAlong));
      if (point.x < 0.0 || point.y < 0.0 || point.x > 1.0 || point.y > 1.0) break;
      const distance: f32 = res.sdf.sampleLevel(res.linear, point, 0.0).x;
      if (distance <= eps) {
        const hitColor: Vec4f = res.colors.sampleLevel(res.linear, point, 0.0);
        radiance = new Vec3f(hitColor.x, hitColor.y, hitColor.z);
        transmittance = 0.0;
        break;
      }
      distanceAlong += distance > minStep ? distance : minStep;
    }
    if (params.layer + 1 < params.cascadeCount && transmittance > 0.01) {
      const probesUpper: u32 = cascadeProbesAt(params.baseProbes, params.layer + 1);
      const upperUv: Vec2f = cascadeMergeUv(
        dirActual,
        probesUpper,
        probePos,
        CASCADE_DIM as f32,
      );
      const upper: Vec4f = res.upper.sampleLevel(res.linear, upperUv, 0.0);
      radiance = radiance.add(new Vec3f(upper.x, upper.y, upper.z).scale(transmittance));
      transmittance *= upper.w;
    }
    accumulated = accumulated.add(new Vec4f(
      radiance.x,
      radiance.y,
      radiance.z,
      transmittance,
    ));
  }
  res.target.store(new Vec2i(x as i32, y as i32), accumulated.scale(0.25));
}

function fieldKernel(res: FieldLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  if (x >= LIGHT_SIZE || y >= LIGHT_SIZE) return;
  const uv = new Vec2f(
    ((x as f32) + 0.5) / (LIGHT_SIZE as f32),
    ((y as f32) + 0.5) / (LIGHT_SIZE as f32),
  );
  let sum = new Vec3f(0.0, 0.0, 0.0);
  for (let quadrant: u32 = 0; quadrant < 4; quadrant += 1) {
    const sampleUv: Vec2f = radianceGatherUv(
      quadrant,
      uv,
      CASCADE_PROBES as f32,
      CASCADE_DIM as f32,
    );
    const sample: Vec4f = res.cascade0.sampleLevel(res.linear, sampleUv, 0.0);
    sum = sum.add(new Vec3f(sample.x, sample.y, sample.z));
  }
  const average: Vec3f = sum.scale(0.25);
  res.target.store(
    new Vec2i(x as i32, y as i32),
    new Vec4f(average.x, average.y, average.z, 1.0),
  );
}

function drawingVertex(res: RenderLayout, vertex: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(vertex.position.x, vertex.position.y, 0.0, 1.0),
    new Vec2f((vertex.position.x + 1.0) * 0.5, (vertex.position.y + 1.0) * 0.5),
  );
}

function absolute(value: f32): f32 {
  return value < 0.0 ? -value : value;
}

function acesChannel(value: f32): f32 {
  return clamp(
    (value * (value * 2.51 + 0.03)) / (value * (value * 2.43 + 0.59) + 0.14),
    0.0,
    1.0,
  );
}

function acesFilm(color: Vec3f): Vec3f {
  return new Vec3f(
    acesChannel(color.x),
    acesChannel(color.y),
    acesChannel(color.z),
  );
}

function drawingFragment(
  res: RenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const distance: f32 = res.sdf.sampleLevel(res.linear, input.uv, 0.0).x;
  if (res.params.$.mode === DISPLAY_SDF) {
    let color = distance >= 0.0
      ? new Vec3f(1.0, 0.2, 0.15)
      : new Vec3f(0.15, 0.35, 1.0);
    const fade: f32 = 1.0 - new Vec2f(
      -80.0 * absolute(distance),
      -80.0 * absolute(distance),
    ).exp().x;
    // The shared vector carries the scalar cosine in the kernel subset's form.
    const bands: f32 = 0.8 + 0.2 * new Vec2f(
      150.0 * distance,
      150.0 * distance,
    ).cos().x;
    color = color.scale(fade * bands);
    const edge: f32 = 1.0 - smoothstep(0.0, SURFACE_EDGE, absolute(distance));
    color = color.mix(new Vec3f(1.0, 1.0, 1.0), edge);
    return new Vec4f(color.x, color.y, color.z, 1.0);
  }

  const field: Vec4f = res.field.sampleLevel(res.linear, input.uv, 0.0);
  const lit: Vec3f = acesFilm(new Vec3f(
    clamp(field.x, 0.0, 1.0),
    clamp(field.y, 0.0, 1.0),
    clamp(field.z, 0.0, 1.0),
  ));
  const strokeSample: Vec4f = res.colors.sampleLevel(res.linear, input.uv, 0.0);
  const stroke: Vec3f = acesFilm(new Vec3f(strokeSample.x, strokeSample.y, strokeSample.z));
  const surfaceAlpha: f32 = 1.0 - smoothstep(-SURFACE_EDGE, SURFACE_EDGE, distance);
  const color: Vec3f = lit.mix(stroke, surfaceAlpha);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const sceneEdit: ComputePipelineSpec = computePipeline<SceneEditLayout>(
  sceneEditKernel,
  { name: "sceneEdit", workgroupSize: [8, 8, 1] },
);

export const floodSeed: ComputePipelineSpec = computePipeline<FloodSeedLayout>(
  floodSeedKernel,
  { name: "floodSeed", workgroupSize: [8, 8, 1] },
);

export const floodStep: ComputePipelineSpec = computePipeline<FloodStepLayout>(
  floodStepKernel,
  { name: "floodStep", workgroupSize: [8, 8, 1] },
);

export const floodDerive: ComputePipelineSpec = computePipeline<FloodDeriveLayout>(
  floodDeriveKernel,
  { name: "floodDerive", workgroupSize: [8, 8, 1] },
);

export const cascadePass: ComputePipelineSpec = computePipeline<CascadeLayout>(
  cascadeKernel,
  { name: "cascadePass", workgroupSize: [16, 16, 1] },
);

export const fieldBuild: ComputePipelineSpec = computePipeline<FieldLayout>(fieldKernel, {
  name: "fieldBuild",
  workgroupSize: [16, 16, 1],
});

export const radianceDrawingRender: RenderPipelineSpec = renderPipelineL<
  RenderLayout,
  Vertex,
  Varyings
>(drawingVertex, drawingFragment, { format: "bgra8unorm" });

class DrawingState {
  device: GPUHostOwnedDevice;
  compute: ComputePipeline[];
  render: RenderPipeline;
  editGroup: GPUBindGroup;
  seedGroup: GPUBindGroup;
  stepGroups: GPUBindGroup[];
  deriveGroup: GPUBindGroup;
  cascadeGroups: GPUBindGroup[];
  fieldGroup: GPUBindGroup;
  renderGroups: GPUBindGroup[];
  vertices: GPUBuffer;
  brushParams: GPUBuffer;
  stepParams: GPUBuffer[];
  cascadeParams: GPUBuffer[];
  renderParams: GPUBuffer[];
  textures: GPUTexture[];
  views: GPUTextureView[];
  sampler: GPUSampler;
  initialized: boolean;

  constructor(
    device: GPUHostOwnedDevice,
    compute: ComputePipeline[],
    render: RenderPipeline,
    editGroup: GPUBindGroup,
    seedGroup: GPUBindGroup,
    stepGroups: GPUBindGroup[],
    deriveGroup: GPUBindGroup,
    cascadeGroups: GPUBindGroup[],
    fieldGroup: GPUBindGroup,
    renderGroups: GPUBindGroup[],
    vertices: GPUBuffer,
    brushParams: GPUBuffer,
    stepParams: GPUBuffer[],
    cascadeParams: GPUBuffer[],
    renderParams: GPUBuffer[],
    textures: GPUTexture[],
    views: GPUTextureView[],
    sampler: GPUSampler,
  ) {
    this.device = device;
    this.compute = compute;
    this.render = render;
    this.editGroup = editGroup;
    this.seedGroup = seedGroup;
    this.stepGroups = stepGroups;
    this.deriveGroup = deriveGroup;
    this.cascadeGroups = cascadeGroups;
    this.fieldGroup = fieldGroup;
    this.renderGroups = renderGroups;
    this.vertices = vertices;
    this.brushParams = brushParams;
    this.stepParams = stepParams;
    this.cascadeParams = cascadeParams;
    this.renderParams = renderParams;
    this.textures = textures;
    this.views = views;
    this.sampler = sampler;
    this.initialized = false;
  }
}

let activeState: DrawingState | null = null;
let previousPointer: Vec2f = new Vec2f(0.0, 0.0);
let wasDrawing: boolean = false;
let displayMode: u32 = DISPLAY_LIT;

function makeTexture(
  device: GPUHostOwnedDevice,
  label: string,
  width: u32,
  height: u32,
  layers: u32,
): GPUTexture {
  return device.createTexture({
    label,
    size: { width, height, depthOrArrayLayers: layers },
    format: "rgba16float",
    usage: GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.TEXTURE_BINDING,
  });
}

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== radianceDrawingRender_TARGET_FORMAT) {
    print(`FAIL format expected=${radianceDrawingRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const dimensions = cascadeDimensions(LIGHT_SIZE);
  if (dimensions.cascadeProbes !== CASCADE_PROBES
    || dimensions.cascadeDim !== CASCADE_DIM
    || dimensions.cascadeCount !== CASCADE_COUNT) {
    print("FAIL committed cascade dimensions");
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "radiance-drawing-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const brushParams = hostDevice.createBuffer({
    label: "radiance-drawing-brush",
    size: BrushParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  const stepParams: GPUBuffer[] = [];
  let offset: u32 = SCENE_SIZE / 2;
  while (offset >= 1) {
    stepParams.push(hostDevice.createBuffer({
      label: `radiance-drawing-flood-${offset}`,
      size: StepParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    }));
    offset /= 2;
  }
  const cascadeParams: GPUBuffer[] = [];
  let layer: u32 = 0;
  while (layer < CASCADE_COUNT) {
    cascadeParams.push(hostDevice.createBuffer({
      label: `radiance-drawing-cascade-${layer}`,
      size: CascadeParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    }));
    layer += 1;
  }
  const renderParams: GPUBuffer[] = [
    hostDevice.createBuffer({
      label: "radiance-drawing-lit-mode",
      size: RenderParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    }),
    hostDevice.createBuffer({
      label: "radiance-drawing-sdf-mode",
      size: RenderParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    }),
  ];

  const scene = makeTexture(hostDevice, "radiance-drawing-scene", SCENE_SIZE, SCENE_SIZE, 1);
  const floodA = makeTexture(hostDevice, "radiance-drawing-flood-a", SCENE_SIZE, SCENE_SIZE, 2);
  const floodB = makeTexture(hostDevice, "radiance-drawing-flood-b", SCENE_SIZE, SCENE_SIZE, 2);
  const sdf = makeTexture(hostDevice, "radiance-drawing-sdf", SCENE_SIZE, SCENE_SIZE, 1);
  const colors = makeTexture(hostDevice, "radiance-drawing-colors", SCENE_SIZE, SCENE_SIZE, 1);
  const cascadeA = makeTexture(
    hostDevice,
    "radiance-drawing-cascade-a",
    CASCADE_DIM,
    CASCADE_DIM,
    CASCADE_COUNT,
  );
  const cascadeB = makeTexture(
    hostDevice,
    "radiance-drawing-cascade-b",
    CASCADE_DIM,
    CASCADE_DIM,
    CASCADE_COUNT,
  );
  const field = makeTexture(hostDevice, "radiance-drawing-field", LIGHT_SIZE, LIGHT_SIZE, 1);
  const sceneView = scene.createView();
  const floodArrayA = floodA.createView({
    dimension: "2d-array",
    mipLevelCount: 1,
    arrayLayerCount: FLOOD_LAYERS,
  });
  const floodArrayB = floodB.createView({
    dimension: "2d-array",
    mipLevelCount: 1,
    arrayLayerCount: FLOOD_LAYERS,
  });
  const sdfView = sdf.createView();
  const colorView = colors.createView();
  const cascadeViews: GPUTextureView[] = [];
  layer = 0;
  while (layer < CASCADE_COUNT) {
    cascadeViews.push(cascadeA.createView({
      dimension: "2d",
      mipLevelCount: 1,
      baseArrayLayer: layer,
      arrayLayerCount: 1,
    }));
    layer += 1;
  }
  layer = 0;
  while (layer < CASCADE_COUNT) {
    cascadeViews.push(cascadeB.createView({
      dimension: "2d",
      mipLevelCount: 1,
      baseArrayLayer: layer,
      arrayLayerCount: 1,
    }));
    layer += 1;
  }
  const fieldView = field.createView();
  const views: GPUTextureView[] = [
    sceneView,
    floodArrayA,
    floodArrayB,
    sdfView,
    colorView,
  ];
  layer = 0;
  while (layer < (cascadeViews.length as u32)) {
    views.push(cascadeViews[layer as i32]);
    layer += 1;
  }
  views.push(fieldView);
  const samplerDescriptor: GPUSamplerDescriptor = { minFilter: "linear", magFilter: "linear" };
  const sampler = hostDevice.createSampler(samplerDescriptor);

  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  offset = SCENE_SIZE / 2;
  let index: i32 = 0;
  while (index < stepParams.length) {
    queue.writeBuffer(
      stepParams[index],
      0,
      Context.bytesOf<StepParams>(new StepParams(offset as i32)),
    );
    offset /= 2;
    index += 1;
  }
  layer = 0;
  while (layer < CASCADE_COUNT) {
    queue.writeBuffer(
      cascadeParams[layer as i32],
      0,
      Context.bytesOf<CascadeParams>(new CascadeParams(
        layer,
        cascadeProbesAt(CASCADE_PROBES, layer),
        CASCADE_COUNT,
        CASCADE_PROBES,
      )),
    );
    layer += 1;
  }
  queue.writeBuffer(
    renderParams[0],
    0,
    Context.bytesOf<RenderParams>(new RenderParams(DISPLAY_LIT)),
  );
  queue.writeBuffer(
    renderParams[1],
    0,
    Context.bytesOf<RenderParams>(new RenderParams(DISPLAY_SDF)),
  );

  hostDevice.pushErrorScope("validation");
  const editPipeline = createComputePipelineHost(
    hostDevice,
    sceneEdit_WGSL,
    sceneEdit_ENTRY,
    [sceneEdit_LAYOUT0],
    [8, 8, 1],
  );
  const seedPipeline = createComputePipelineHost(
    hostDevice,
    floodSeed_WGSL,
    floodSeed_ENTRY,
    [floodSeed_LAYOUT0],
    [8, 8, 1],
  );
  const stepPipeline = createComputePipelineHost(
    hostDevice,
    floodStep_WGSL,
    floodStep_ENTRY,
    [floodStep_LAYOUT0],
    [8, 8, 1],
  );
  const derivePipeline = createComputePipelineHost(
    hostDevice,
    floodDerive_WGSL,
    floodDerive_ENTRY,
    [floodDerive_LAYOUT0],
    [8, 8, 1],
  );
  const cascadePipeline = createComputePipelineHost(
    hostDevice,
    cascadePass_WGSL,
    cascadePass_ENTRY,
    [cascadePass_LAYOUT0],
    [16, 16, 1],
  );
  const fieldPipeline = createComputePipelineHost(
    hostDevice,
    fieldBuild_WGSL,
    fieldBuild_ENTRY,
    [fieldBuild_LAYOUT0],
    [16, 16, 1],
  );
  const renderPipeline = createRenderPipelineHost(
    hostDevice,
    radianceDrawingRender_WGSL,
    radianceDrawingRender_VERTEX_ENTRY,
    radianceDrawingRender_FRAGMENT_ENTRY,
    [radianceDrawingRender_LAYOUT0],
    [radianceDrawingRender_VERTEX_LAYOUT0],
    radianceDrawingRender,
  );
  const validationError = hostDevice.popErrorScope();
  if (validationError !== null) {
    renderPipeline.dispose();
    fieldPipeline.dispose();
    cascadePipeline.dispose();
    derivePipeline.dispose();
    stepPipeline.dispose();
    seedPipeline.dispose();
    editPipeline.dispose();
    sampler.dispose();
    index = 0;
    while (index < views.length) {
      views[index].dispose();
      index += 1;
    }
    field.dispose();
    cascadeB.dispose();
    cascadeA.dispose();
    colors.dispose();
    sdf.dispose();
    floodB.dispose();
    floodA.dispose();
    scene.dispose();
    index = 0;
    while (index < renderParams.length) {
      renderParams[index].dispose();
      index += 1;
    }
    index = 0;
    while (index < cascadeParams.length) {
      cascadeParams[index].dispose();
      index += 1;
    }
    index = 0;
    while (index < stepParams.length) {
      stepParams[index].dispose();
      index += 1;
    }
    brushParams.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  using editLayout = editPipeline.bindGroupLayout(0);
  using seedLayout = seedPipeline.bindGroupLayout(0);
  using stepLayout = stepPipeline.bindGroupLayout(0);
  using deriveLayout = derivePipeline.bindGroupLayout(0);
  using cascadeLayout = cascadePipeline.bindGroupLayout(0);
  using fieldLayout = fieldPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  const editGroup = createBindGroupHost(hostDevice, editLayout, sceneEdit_LAYOUT0, [
    textureResource(sceneView),
    bufferResource(brushParams),
  ]);
  const seedGroup = createBindGroupHost(hostDevice, seedLayout, floodSeed_LAYOUT0, [
    textureResource(sceneView),
    textureResource(floodArrayA),
  ]);
  const stepGroups: GPUBindGroup[] = [];
  index = 0;
  while (index < stepParams.length) {
    const source: GPUTextureView = index % 2 === 0 ? floodArrayA : floodArrayB;
    const target: GPUTextureView = index % 2 === 0 ? floodArrayB : floodArrayA;
    stepGroups.push(createBindGroupHost(hostDevice, stepLayout, floodStep_LAYOUT0, [
      textureResource(source),
      textureResource(target),
      bufferResource(stepParams[index]),
    ]));
    index += 1;
  }
  // FLOOD_STEPS is odd, so the A-to-B first step leaves the final payload on side B.
  const deriveGroup = createBindGroupHost(hostDevice, deriveLayout, floodDerive_LAYOUT0, [
    textureResource(floodArrayB),
    textureResource(sceneView),
    textureResource(sdfView),
    textureResource(colorView),
  ]);
  const cascadeGroups: GPUBindGroup[] = [];
  layer = 0;
  while (layer < CASCADE_COUNT) {
    const side: u32 = cascadeWriteSide(CASCADE_COUNT, layer);
    const sourceSide: u32 = side === 0 ? 1 : 0;
    const upperLayer: u32 = layer + 1 < CASCADE_COUNT ? layer + 1 : layer;
    cascadeGroups.push(createBindGroupHost(
      hostDevice,
      cascadeLayout,
      cascadePass_LAYOUT0,
      [
        textureResource(cascadeViews[(sourceSide * CASCADE_COUNT + upperLayer) as i32]),
        textureResource(sdfView),
        textureResource(colorView),
        samplerResource(sampler),
        textureResource(cascadeViews[(side * CASCADE_COUNT + layer) as i32]),
        bufferResource(cascadeParams[layer as i32]),
      ],
    ));
    layer += 1;
  }
  const cascade0Side: u32 = cascadeWriteSide(CASCADE_COUNT, 0);
  const fieldGroup = createBindGroupHost(hostDevice, fieldLayout, fieldBuild_LAYOUT0, [
    textureResource(cascadeViews[(cascade0Side * CASCADE_COUNT) as i32]),
    samplerResource(sampler),
    textureResource(fieldView),
  ]);
  const renderGroups: GPUBindGroup[] = [];
  index = 0;
  while (index < renderParams.length) {
    renderGroups.push(createBindGroupHost(
      hostDevice,
      renderLayout,
      radianceDrawingRender_LAYOUT0,
      [
        textureResource(fieldView),
        textureResource(sdfView),
        textureResource(colorView),
        samplerResource(sampler),
        bufferResource(renderParams[index]),
      ],
    ));
    index += 1;
  }
  activeState = new DrawingState(
    hostDevice,
    [editPipeline, seedPipeline, stepPipeline, derivePipeline, cascadePipeline, fieldPipeline],
    renderPipeline,
    editGroup,
    seedGroup,
    stepGroups,
    deriveGroup,
    cascadeGroups,
    fieldGroup,
    renderGroups,
    vertices,
    brushParams,
    stepParams,
    cascadeParams,
    renderParams,
    [scene, floodA, floodB, sdf, colors, cascadeA, cascadeB, field],
    views,
    sampler,
  );
  previousPointer = new Vec2f(0.0, 0.0);
  wasDrawing = false;
  displayMode = DISPLAY_LIT;
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
  if (key === 49) displayMode = DISPLAY_LIT;
  if (key === 50) displayMode = DISPLAY_SDF;
  const pointerValid: boolean = pointerX >= 0.0 && pointerY >= 0.0
    && width > 0 && height > 0;
  const drawing: boolean = pointerValid && (buttons & 1) !== 0;
  let mode: u32 = 0;
  let current = previousPointer;
  if (!active.initialized || key === 48) {
    mode = EDIT_CLEAR;
    wasDrawing = false;
  } else if (drawing) {
    current = new Vec2f(
      clamp(pointerX / (width as f32), 0.0, 1.0),
      clamp(1.0 - pointerY / (height as f32), 0.0, 1.0),
    );
    if (!wasDrawing) previousPointer = current;
    mode = EDIT_PAINT;
  }

  using queue = active.device.queue();
  using encoder = active.device.createCommandEncoderDefault();
  if (mode !== 0) {
    queue.writeBuffer(
      active.brushParams,
      0,
      Context.bytesOf<BrushParams>(new BrushParams(previousPointer, current, mode)),
    );
    active.compute[0].dispatch(
      encoder,
      [active.editGroup],
      SCENE_SIZE / WORKGROUP_SIZE,
      SCENE_SIZE / WORKGROUP_SIZE,
      1,
    );
    active.compute[1].dispatch(
      encoder,
      [active.seedGroup],
      SCENE_SIZE / WORKGROUP_SIZE,
      SCENE_SIZE / WORKGROUP_SIZE,
      1,
    );
    let index: i32 = 0;
    while (index < (FLOOD_STEPS as i32)) {
      active.compute[2].dispatch(
        encoder,
        [active.stepGroups[index]],
        SCENE_SIZE / WORKGROUP_SIZE,
        SCENE_SIZE / WORKGROUP_SIZE,
        1,
      );
      index += 1;
    }
    active.compute[3].dispatch(
      encoder,
      [active.deriveGroup],
      SCENE_SIZE / WORKGROUP_SIZE,
      SCENE_SIZE / WORKGROUP_SIZE,
      1,
    );
    let layer: i32 = (CASCADE_COUNT as i32) - 1;
    while (layer >= 0) {
      active.compute[4].dispatch(
        encoder,
        [active.cascadeGroups[layer]],
        CASCADE_DIM / CASCADE_WORKGROUP_SIZE,
        CASCADE_DIM / CASCADE_WORKGROUP_SIZE,
        1,
      );
      layer -= 1;
    }
    active.compute[5].dispatch(
      encoder,
      [active.fieldGroup],
      LIGHT_SIZE / CASCADE_WORKGROUP_SIZE,
      LIGHT_SIZE / CASCADE_WORKGROUP_SIZE,
      1,
    );
    active.initialized = true;
  }
  if (drawing && mode === EDIT_PAINT) previousPointer = current;
  wasDrawing = drawing && mode === EDIT_PAINT;

  const target = new GPUTextureView(view);
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  active.render.bind(pass, [active.renderGroups[(displayMode - 1) as i32]], [active.vertices]);
  pass.draw(3);
  pass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

export function shutdown(): void {
  if (activeState === null) return;
  const active = activeState;
  let index: i32 = 0;
  while (index < active.renderGroups.length) {
    active.renderGroups[index].dispose();
    index += 1;
  }
  active.fieldGroup.dispose();
  index = 0;
  while (index < active.cascadeGroups.length) {
    active.cascadeGroups[index].dispose();
    index += 1;
  }
  active.deriveGroup.dispose();
  index = 0;
  while (index < active.stepGroups.length) {
    active.stepGroups[index].dispose();
    index += 1;
  }
  active.seedGroup.dispose();
  active.editGroup.dispose();
  active.sampler.dispose();
  index = 0;
  while (index < active.views.length) {
    active.views[index].dispose();
    index += 1;
  }
  index = 0;
  while (index < active.textures.length) {
    active.textures[index].dispose();
    index += 1;
  }
  index = 0;
  while (index < active.renderParams.length) {
    active.renderParams[index].dispose();
    index += 1;
  }
  index = 0;
  while (index < active.cascadeParams.length) {
    active.cascadeParams[index].dispose();
    index += 1;
  }
  index = 0;
  while (index < active.stepParams.length) {
    active.stepParams[index].dispose();
    index += 1;
  }
  active.brushParams.dispose();
  active.vertices.dispose();
  active.render.dispose();
  index = 0;
  while (index < active.compute.length) {
    active.compute[index].dispose();
    index += 1;
  }
  activeState = null;
  wasDrawing = false;
  displayMode = DISPLAY_LIT;
}
