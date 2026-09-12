// example: radiance-cascades-drawing
// Turns painted emissive strokes into a jump-flood SDF and a cascade-lit scene.
// The scene and flood commit to 512 square pixels. Lighting commits to the upstream
// quarter resolution of 128, and resize stretches the result. The light color commits
// to warm orange and the brush radius to 0.03. Key 0 clears. Keys 1 and 2 select the lit
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

// The committed sizes. The stroke and the flood work at 512 pixels, and the light solves at
// 128. `cascadeDimensions(128)` returns the three cascade values, and `init` rejects a
// mismatch.
const SCENE_SIZE: u32 = 512;
const LIGHT_SIZE: u32 = 128;
const CASCADE_PROBES: u32 = 64;
const CASCADE_DIM: u32 = 128;
const CASCADE_COUNT: u32 = 5;
// The jump flood halves its offset from 256 down to 1, so 512 pixels need nine steps. The
// two payload layers carry the seed color and the seed position.
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

// One clip-space corner of the full-screen triangle. The generator derives the vertex
// attribute layout and the `Vertex_STRIDE` byte stride from this class.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// One stroke segment per frame: the previous point, the current point, and the edit mode.
// Both points are in scene units of [0, 1].
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

// The jump-flood offset in pixels. Each step owns a buffer with a constant value, so the
// nine dispatches differ only by their bind group.
@CStruct
class StepParams {
  offset: i32;

  constructor(offset: i32) {
    this.offset = offset;
  }
}

// The per-layer uniform. `init` writes one buffer per layer once, so the five cascade
// dispatches of a frame differ only by their bind group.
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

// The display mode. Two buffers hold the two values, so a key press selects a bind group
// and writes no uniform.
@CStruct
class RenderParams {
  mode: u32;

  constructor(mode: u32) {
    this.mode = mode;
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
class SceneEditLayout {
  scene: StorageTexture2d<Rgba16float>;
  brush: Uniform<BrushParams>;

  constructor(scene: StorageTexture2d<Rgba16float>, brush: Uniform<BrushParams>) {
    this.scene = scene;
    this.brush = brush;
  }
}

// The flood passes bind both payload layers as one 2D-array texture. Layer 0 holds the seed
// color and layer 1 holds the seed position.
class FloodSeedLayout {
  scene: Texture2d<f32>;
  target: WriteStorageTexture2dArray<Rgba16float>;

  constructor(scene: Texture2d<f32>, target: WriteStorageTexture2dArray<Rgba16float>) {
    this.scene = scene;
    this.target = target;
  }
}

// A step reads one payload texture and writes the other, because one dispatch cannot read
// and write the same texture.
class FloodStepLayout {
  source: ReadStorageTexture2dArray<Rgba16float>;
  target: WriteStorageTexture2dArray<Rgba16float>;
  params: Uniform<StepParams>;

  constructor(
    source: ReadStorageTexture2dArray<Rgba16float>,
    target: WriteStorageTexture2dArray<Rgba16float>,
    params: Uniform<StepParams>,
  ) {
    this.source = source;
    this.target = target;
    this.params = params;
  }
}

// The derive pass turns the flood payload into the two textures the light passes read: a
// signed distance and an emissive color.
class FloodDeriveLayout {
  payload: ReadStorageTexture2dArray<Rgba16float>;
  scene: Texture2d<f32>;
  sdf: StorageTexture2d<Rgba16float>;
  colors: StorageTexture2d<Rgba16float>;

  constructor(
    payload: ReadStorageTexture2dArray<Rgba16float>,
    scene: Texture2d<f32>,
    sdf: StorageTexture2d<Rgba16float>,
    colors: StorageTexture2d<Rgba16float>,
  ) {
    this.payload = payload;
    this.scene = scene;
    this.sdf = sdf;
    this.colors = colors;
  }
}

// `upper` reads the layer above and `target` writes this layer. `sdf` and `colors` replace
// the analytic scene function of the first cascade example.
class CascadeLayout {
  upper: Texture2d<f32>;
  sdf: Texture2d<f32>;
  colors: Texture2d<f32>;
  linear: Sampler;
  target: StorageTexture2d<Rgba16float>;
  params: Uniform<CascadeParams>;

  constructor(
    upper: Texture2d<f32>,
    sdf: Texture2d<f32>,
    colors: Texture2d<f32>,
    linear: Sampler,
    target: StorageTexture2d<Rgba16float>,
    params: Uniform<CascadeParams>,
  ) {
    this.upper = upper;
    this.sdf = sdf;
    this.colors = colors;
    this.linear = linear;
    this.target = target;
    this.params = params;
  }
}

// The gather reads cascade 0 through the sampler and writes the light field.
class FieldLayout {
  cascade0: Texture2d<f32>;
  linear: Sampler;
  target: StorageTexture2d<Rgba16float>;

  constructor(cascade0: Texture2d<f32>, linear: Sampler, target: StorageTexture2d<Rgba16float>) {
    this.cascade0 = cascade0;
    this.linear = linear;
    this.target = target;
  }
}

// One layout class serves both render stages, and the mode uniform selects the view.
class RenderLayout {
  field: Texture2d<f32>;
  sdf: Texture2d<f32>;
  colors: Texture2d<f32>;
  linear: Sampler;
  params: Uniform<RenderParams>;

  constructor(
    field: Texture2d<f32>,
    sdf: Texture2d<f32>,
    colors: Texture2d<f32>,
    linear: Sampler,
    params: Uniform<RenderParams>,
  ) {
    this.field = field;
    this.sdf = sdf;
    this.colors = colors;
    this.linear = linear;
    this.params = params;
  }
}

// The edit pass only writes touched cells, so a stroke accumulates in one scene texture.
// A clear dispatch instead writes transparent black over the complete texture. TypeGPU
// clears the same texture with `texture.clear()`.
function sceneEditKernel(res: SceneEditLayout, ctx: ComputeInvocation): void {
  const coords = new Vec2i(ctx.globalId.x as i32, ctx.globalId.y as i32);
  const brush: BrushParams = res.brush.$;
  if (brush.mode === EDIT_CLEAR) {
    res.scene.store(coords, new Vec4f(0.0, 0.0, 0.0, 0.0));
    return;
  }
  // The cell center in scene units of [0, 1], the units the brush points use.
  const point = new Vec2f(
    ((ctx.globalId.x as f32) + 0.5) / (SCENE_SIZE as f32),
    ((ctx.globalId.y as f32) + 0.5) / (SCENE_SIZE as f32),
  );
  // A stroke paints the segment between the two pointer samples, so a fast pointer leaves no
  // gap between frames.
  const segment: Vec2f = brush.current.sub(brush.previous);
  let distance: f32 = point.distance(brush.current);
  if (segment.dot(segment) > 0.00000001) {
    distance = sdLine(point, brush.previous, brush.current);
  }
  // The painted color is the emission the light passes read. Alpha 1.0 marks the cell as a
  // flood seed.
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
  // A painted cell seeds itself. An empty cell stores a negative position, the marker for no
  // seed.
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

// Returns the squared distance to the seed, or a far value when the cell has no seed. The
// comparison needs no square root.
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
  // The cell starts as its own best candidate, so a cell that already owns a nearer seed
  // keeps it.
  let bestColor: Vec4f = res.source.load(coords, 0);
  let bestSeed: Vec4f = res.source.load(coords, 1);
  let bestDistance: f32 = seedDistance(point, bestSeed);

  // Each of the eight neighbors at the step offset can hold a nearer seed. The bounds test
  // keeps the read inside the texture.
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
// TypeGPU floods the inside as well, so its distance inside a stroke is exact.
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
  // The bounds test is code here. TypeGPU's guarded pipeline emits the same test around the
  // kernel and a hidden size uniform to feed it.
  if (x >= CASCADE_DIM || y >= CASCADE_DIM) return;
  const params: CascadeParams = res.params.$;
  const probes: u32 = params.probes;
  const raysStored: u32 = cascadeRaysStored(params.layer);
  // The texture packs one square tile per stored direction. Integer division names the tile,
  // and the remainder names the probe inside it.
  const dirStored = new Vec2u(x / probes, y / probes);
  const probe = new Vec2u(x % probes, y % probes);
  // The probe sits at the center of its cell, in scene units of [0, 1].
  const probePos = new Vec2f(
    ((probe.x as f32) + 0.5) / (probes as f32),
    ((probe.y as f32) + 0.5) / (probes as f32),
  );
  // `interval0` is the cascade-0 ray length, one probe spacing in scene units. Each layer
  // quadruples its segment, so the layers cover the scene without an overlap.
  const interval0: f32 = 1.0 / (params.baseProbes as f32);
  const rayStart: f32 = cascadeIntervalStart(interval0, params.layer);
  const rayEnd: f32 = cascadeIntervalEnd(interval0, params.layer);
  // A hit counts within half a probe spacing, and one march step never falls below a quarter.
  // That floor keeps 64 steps enough to cross the longest segment.
  const eps: f32 = 0.5 / (params.baseProbes as f32);
  const minStep: f32 = 0.25 / (params.baseProbes as f32);
  let accumulated = new Vec4f(0.0, 0.0, 0.0, 0.0);

  // The four rays of one stored direction differ only by their angle. Their mean becomes the
  // texel this invocation writes.
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
    // The march advances by the sampled distance, the sphere-trace step. The fixed 64 steps
    // bound the work per ray.
    for (let step: u32 = 0; step < 64; step += 1) {
      if (distanceAlong > rayEnd) break;
      const point: Vec2f = probePos.add(rayDirection.scale(distanceAlong));
      // A ray that leaves the scene stops. The sampler clamps at the edge, so a sample outside
      // repeats the border distance for the rest of the march.
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
    // A ray that survives its segment merges the layer above. The linear filter between the four
    // upper probes is the bilinear fix, and `cascadeMergeUv` holds it inside the direction tile.
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
  // One texel holds the mean of the four rays. RGB is radiance and alpha is transmittance.
  res.target.store(new Vec2i(x as i32, y as i32), accumulated.scale(0.25));
}

// Gathers cascade 0 into the light field. One invocation averages the four quadrant
// directions that reach its pixel.
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

// The oversized triangle covers the clip square. The uv maps clip space [-1, 1] to the
// scene's [0, 1], and the fragment stage interpolates it.
function drawingVertex(res: RenderLayout, vertex: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(vertex.position.x, vertex.position.y, 0.0, 1.0),
    new Vec2f((vertex.position.x + 1.0) * 0.5, (vertex.position.y + 1.0) * 0.5),
  );
}

function absolute(value: f32): f32 {
  return value < 0.0 ? -value : value;
}

// The ACES filmic curve maps unbounded radiance into [0, 1]. The cascade textures are
// rgba16float, so a value above 1.0 reaches this point.
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

// One fragment entry serves both views. The mode uniform selects the branch, so a key press
// changes the bind group and never the pipeline.
function drawingFragment(
  res: RenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const distance: f32 = res.sdf.sampleLevel(res.linear, input.uv, 0.0).x;
  if (res.params.$.mode === DISPLAY_SDF) {
    // The SDF view paints the outside red and the inside blue. The fade and the bands make the
    // distance value itself visible, and white marks the zero crossing.
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

  // The lit view mixes the tone-mapped light field with the stroke color at the surface.
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

// The seven declarations are the generator's input. It walks the typed program before the
// run and emits `main.typegpu.ts`: the WGSL text, the entry names, and the layout specs.
// TypeGPU resolves the same shaders from the kernel functions at run time.
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

// `renderPipelineL` adds the layout class, so both stages read `RenderLayout`. The target
// format belongs to the declaration, and `init` checks the surface format against it.
export const radianceDrawingRender: RenderPipelineSpec = renderPipelineL<
  RenderLayout,
  Vertex,
  Varyings
>(drawingVertex, drawingFragment, { format: "bgra8unorm" });

// One object holds every handle a frame needs. Scripts own their handles, so `shutdown`
// releases each one. TypeGPU leaves that to garbage collection and `root.destroy`.
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

// `init` fills this binding and `frame` reads it. A failed `init` leaves it `null`, because
// this library reports a failure by value and never by an exception.
let activeState: DrawingState | null = null;
// The stroke state: the previous pointer position, whether the last frame painted, and the
// selected view.
let previousPointer: Vec2f = new Vec2f(0.0, 0.0);
let wasDrawing: boolean = false;
let displayMode: u32 = DISPLAY_LIT;

// Every texture here is rgba16float with a storage binding and a texture binding, so one
// pass writes it and the next pass samples it.
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
  // The generator fixed the color target format. A surface with another format is a failure
  // here, not a reason to rebuild the pipeline.
  if (format !== radianceDrawingRender_TARGET_FORMAT) {
    print(`FAIL format expected=${radianceDrawingRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The kernels read the committed sizes as constants. The host sizing must agree with them,
  // so a mismatch stops `init` before any resource exists.
  const dimensions = cascadeDimensions(LIGHT_SIZE);
  if (dimensions.cascadeProbes !== CASCADE_PROBES
    || dimensions.cascadeDim !== CASCADE_DIM
    || dimensions.cascadeCount !== CASCADE_COUNT) {
    print("FAIL committed cascade dimensions");
    return;
  }
  // The window host owns the device. The wrapper adds the API-layer methods and has neither
  // `dispose` nor `destroy`.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // One oversized triangle covers the screen. `Vertex_STRIDE` comes from the generator, so
  // the size never restates the layout.
  const vertices = hostDevice.createBuffer({
    label: "radiance-drawing-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The brush uniform. A frame that edits the scene rewrites it before the dispatch.
  const brushParams = hostDevice.createBuffer({
    label: "radiance-drawing-brush",
    size: BrushParams_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // One uniform buffer per flood step, from offset 256 down to 1. The values never change,
  // so `init` writes them once.
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
  // One uniform buffer per cascade layer, also written once in `init`.
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
  // One buffer per display mode. The frame then selects a bind group instead of a write.
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

  // The scene, the two flood payloads, the SDF, and the colors work at 512 pixels. The
  // cascades and the light field work at 128, the quarter resolution the lighting needs.
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
  // The flood binds both payload layers as one 2D-array view, so one dispatch moves the color
  // and the position together.
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
  // Each cascade layer gets a single-layer 2D view, because a storage binding writes one
  // layer. Side A comes first, then side B, and the bind groups index the list.
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
  // One list owns every view, so `shutdown` releases them in one loop.
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
  // The linear filter interpolates the SDF along a ray and the upper probes during the merge.
  const samplerDescriptor: GPUSamplerDescriptor = { minFilter: "linear", magFilter: "linear" };
  const sampler = hostDevice.createSampler(samplerDescriptor);

  // The queue handle is borrowed for this block, and `using` releases it at the end of `init`.
  // The buffers it writes stay.
  using queue = hostDevice.queue();
  // `Context.bytesOf<T>` produces the exact bytes of the value in the generated layout.
  // TypeGPU converts a JavaScript object to buffer bytes at run time instead.
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  // Each step halves its offset. The first step jumps 256 pixels and the last jumps 1, so a
  // seed reaches every cell of the 512-pixel scene.
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
  // Each layer halves its probe count and doubles its ray count on one axis, so every layer
  // fills the same texture size.
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

  // The scope catches a backend rejection of the WGSL or the layout. The API layer returns
  // the error as a value, so the code reads the popped result.
  hostDevice.pushErrorScope("validation");
  // Each pipeline takes generated WGSL text, a generated entry name, and a generated layout
  // spec. This file holds no shader text.
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
  // The render pipeline also takes the vertex layout the generator derived from `Vertex`.
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
  // The failure path releases every handle this function created, newest first. Nothing else
  // frees them, because the state never received them.
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

  // The bind group layouts come from the pipelines, and `using` borrows them for the group
  // creation only.
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
  // One bind group per flood step. The source and the target alternate, so step N reads what
  // step N minus 1 wrote.
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
  // One bind group per cascade layer, built once. `cascadeWriteSide` alternates the write
  // texture per layer, so a layer reads one side while it writes the other.
  const cascadeGroups: GPUBindGroup[] = [];
  layer = 0;
  while (layer < CASCADE_COUNT) {
    const side: u32 = cascadeWriteSide(CASCADE_COUNT, layer);
    const sourceSide: u32 = side === 0 ? 1 : 0;
    // The top layer has no layer above it, so it binds its own view. The kernel skips the merge
    // there, and the binding stays valid.
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
  // Layer 0 writes one of the two sides. The gather group reads layer 0 of that side.
  const cascade0Side: u32 = cascadeWriteSide(CASCADE_COUNT, 0);
  const fieldGroup = createBindGroupHost(hostDevice, fieldLayout, fieldBuild_LAYOUT0, [
    textureResource(cascadeViews[(cascade0Side * CASCADE_COUNT) as i32]),
    samplerResource(sampler),
    textureResource(fieldView),
  ]);
  // One render group per display mode. Both read the same three textures and differ only in
  // the mode uniform.
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
  // The state takes every handle above. `frame` and `shutdown` reach them through this module
  // binding.
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
  // A failed `init` leaves no state. The host still calls `frame`, so the guard returns.
  if (activeState === null) return;
  const active = activeState;
  // The host reports one key per frame and clears the slot, so a press acts once. 49 and 50
  // are the Unicode scalars of `1` and `2`.
  if (key === 49) displayMode = DISPLAY_LIT;
  if (key === 50) displayMode = DISPLAY_SDF;
  // The host reports the pointer in surface pixels, and -1 before the pointer first enters
  // the window.
  const pointerValid: boolean = pointerX >= 0.0 && pointerY >= 0.0
    && width > 0 && height > 0;
  const drawing: boolean = pointerValid && (buttons & 1) !== 0;
  let mode: u32 = 0;
  let current = previousPointer;
  // The first frame clears the scene texture, and key 0 clears it again. A clear and a paint
  // never share a frame, because both go through the one edit pass.
  if (!active.initialized || key === 48) {
    mode = EDIT_CLEAR;
    wasDrawing = false;
  } else if (drawing) {
    // Scene units of [0, 1] with y up. The host reports y down, so the y axis flips here.
    current = new Vec2f(
      clamp(pointerX / (width as f32), 0.0, 1.0),
      clamp(1.0 - pointerY / (height as f32), 0.0, 1.0),
    );
    // A new stroke starts at the current point. The first segment then has zero length and the
    // kernel paints a disk.
    if (!wasDrawing) previousPointer = current;
    mode = EDIT_PAINT;
  }

  // The queue and the encoder live for this frame only.
  using queue = active.device.queue();
  using encoder = active.device.createCommandEncoderDefault();
  // The light chain runs only after an edit. A frame with no edit costs one render pass.
  if (mode !== 0) {
    queue.writeBuffer(
      active.brushParams,
      0,
      Context.bytesOf<BrushParams>(new BrushParams(previousPointer, current, mode)),
    );
    // The chain order is the dependency order: edit the scene, seed the flood, jump nine steps,
    // derive the SDF and the colors, light the cascades, then gather the field. Each `dispatch`
    // records its own compute pass.
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
    // The layers run from the top down, because each layer merges the layer above it.
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
    // The gather reads cascade 0 and fills the 128-pixel light field. A dispatch count is a
    // workgroup count, so the size divides by the 16 of the workgroup declaration.
    active.compute[5].dispatch(
      encoder,
      [active.fieldGroup],
      LIGHT_SIZE / CASCADE_WORKGROUP_SIZE,
      LIGHT_SIZE / CASCADE_WORKGROUP_SIZE,
      1,
    );
    active.initialized = true;
  }
  // The stroke advances only after a paint, so a clear frame keeps the previous point.
  if (drawing && mode === EDIT_PAINT) previousPointer = current;
  wasDrawing = drawing && mode === EDIT_PAINT;

  // The host owns the frame's view and presents it. The wrapper adds the API-layer methods
  // and disposes nothing.
  const target = new GPUTextureView(view);
  // The color attachment clears on load, so no separate clear pass exists.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });
  // The scene commits to 512 pixels and the light field to 128. The viewport stretches both
  // over the current window.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and the vertex buffers on the pass in one
  // call. The display mode picks the group.
  active.render.bind(pass, [active.renderGroups[(displayMode - 1) as i32]], [active.vertices]);
  pass.draw(3);
  pass.end();
  // One encoder records every pass of this frame, and one submit sends them. TypeGPU
  // submits a command buffer per dispatch and per draw.
  using command = encoder.finishDefault();
  queue.submit([command]);
}

// Releases every handle in the reverse order of creation: groups, sampler, views, textures,
// buffers, then pipelines. The device and the frame view belong to the host.
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
  // The cleared binding makes a later `frame` call return at its guard, and the next `init`
  // starts from an empty stroke state.
  activeState = null;
  wasDrawing = false;
  displayMode = DISPLAY_LIT;
}
