// example: radiance-cascades
// Builds a layered radiance field for an interactive signed-distance scene.
// Output and cascade storage commit to 512 square pixels, so resize stretches the final
// field instead of rebuilding it. This drops the upstream overlay and layer slider.
// Its derivative-sized surface edge becomes a fixed 0.002 field-space width because the
// kernel subset has no fwidth. A drag always moves the nearest body, where upstream
// starts a drag only inside a body.
// Ported from TypeGPU's radiance-cascades example (https://github.com/software-mansion/TypeGPU).

import {
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
  sdBox2d,
  sdDisk,
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
  CascadeParams_SIZE,
  Scene_SIZE,
  Vertex_STRIDE,
  cascadePass_ENTRY,
  cascadePass_LAYOUT0,
  cascadePass_WGSL,
  fieldBuild_ENTRY,
  fieldBuild_LAYOUT0,
  fieldBuild_WGSL,
  radianceRender_FRAGMENT_ENTRY,
  radianceRender_LAYOUT0,
  radianceRender_TARGET_FORMAT,
  radianceRender_VERTEX_ENTRY,
  radianceRender_VERTEX_LAYOUT0,
  radianceRender_WGSL,
} from "./main.typegpu";

// The committed sizes. `cascadeDimensions(512)` returns these three cascade values, and
// `init` rejects a mismatch. The cascade texture is twice the probe count on each axis,
// because cascade 0 stores 2 by 2 directions per probe.
const OUTPUT_SIZE: u32 = 512;
const CASCADE_PROBES: u32 = 256;
const CASCADE_DIM: u32 = 512;
const CASCADE_COUNT: u32 = 6;
const WORKGROUP_SIZE: u32 = 16;
const SURFACE_EDGE: f32 = 0.002;

// One clip-space corner of the full-screen triangle. The generator derives the vertex
// attribute layout and the `Vertex_STRIDE` byte stride from this class.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// A scene body. `emissiveColor` is the radiance a ray takes at a hit, so a black body
// blocks light and adds none.
@CStruct
class DiskData {
  pos: Vec2f;
  radius: f32;
  emissiveColor: Vec3f;

  constructor(pos: Vec2f, radius: f32, emissiveColor: Vec3f) {
    this.pos = pos;
    this.radius = radius;
    this.emissiveColor = emissiveColor;
  }
}

@CStruct
class BoxData {
  pos: Vec2f;
  size: Vec2f;
  emissiveColor: Vec3f;

  constructor(pos: Vec2f, size: Vec2f, emissiveColor: Vec3f) {
    this.pos = pos;
    this.size = size;
    this.emissiveColor = emissiveColor;
  }
}

// The complete scene travels in one uniform buffer. `FixedArray` has a compile-time length,
// so the counts 4 and 2 belong to the layout and no run-time array exists.
@CStruct
class Scene {
  disks: FixedArray<DiskData, 4>;
  boxes: FixedArray<BoxData, 2>;

  constructor(disks: FixedArray<DiskData, 4>, boxes: FixedArray<BoxData, 2>) {
    this.disks = disks;
    this.boxes = boxes;
  }
}

// The per-layer uniform. `init` writes one buffer per layer once, so the six dispatches of
// a frame differ only by their bind group.
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

// One scene query returns a distance and a color together, so the ray march reads both
// results from one call.
@CStruct
class SceneHit {
  dist: f32;
  color: Vec3f;

  constructor(dist: f32, color: Vec3f) {
    this.dist = dist;
    this.color = color;
  }
}

// The layout classes replace TypeGPU's run-time bind group layout objects. The field
// order is the binding order, and the generator emits one `_LAYOUT0` spec per class.
class CascadeLayout {
  // `upper` reads the layer above and `target` writes this layer. The two always name views
  // of different textures, because one dispatch cannot sample what it stores.
  upper!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
  scene!: Uniform<Scene>;
  params!: Uniform<CascadeParams>;
}

// The gather reads cascade 0 through the sampler and writes the output field.
class FieldLayout {
  cascade0!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
}

// One layout class serves both render stages. `renderPipelineL` gives the vertex entry and
// the fragment entry the same bindings.
class RenderLayout {
  field!: Texture2d<f32>;
  linear!: Sampler;
  scene!: Uniform<Scene>;
}

// Returns the nearest body and its color. The kernels and the host call this same function,
// so the ray march, the surface overlay, and the pointer pick never disagree.
function sceneSdf(scene: Scene, point: Vec2f): SceneHit {
  // The seed exceeds every distance the scene produces, so the first body always wins.
  let minimum: f32 = 20000000000000000000000000000000.0;
  let color = new Vec3f(0.0, 0.0, 0.0);
  for (let diskIndex: u32 = 0; diskIndex < 4; diskIndex += 1) {
    const disk: DiskData = scene.disks[diskIndex as i32];
    const distance: f32 = sdDisk(point, disk.pos, disk.radius);
    if (distance < minimum) {
      minimum = distance;
      color = disk.emissiveColor;
    }
  }
  for (let boxIndex: u32 = 0; boxIndex < 2; boxIndex += 1) {
    const box: BoxData = scene.boxes[boxIndex as i32];
    const distance: f32 = sdBox2d(point, box.pos, box.size);
    if (distance < minimum) {
      minimum = distance;
      color = box.emissiveColor;
    }
  }
  return new SceneHit(minimum, color);
}

// One stored direction owns four actual rays. A ray that survives its segment samples the
// layer above. RGB carries radiance and alpha carries transmittance.
function cascadeKernel(res: CascadeLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  // The bounds test is code here. TypeGPU's guarded pipeline emits the same test around
  // the kernel and a hidden size uniform to feed it.
  if (x >= CASCADE_DIM || y >= CASCADE_DIM) return;
  const params: CascadeParams = res.params.$;
  const probes: u32 = params.probes;
  const raysStored: u32 = cascadeRaysStored(params.layer);
  // The texture packs one square tile per stored direction. Integer division names the tile,
  // and the remainder names the probe inside it.
  const dirStored = new Vec2u(x / probes, y / probes);
  const probe = new Vec2u(x % probes, y % probes);
  // The probe sits at the center of its cell, in field units of [0, 1].
  const probePos = new Vec2f(
    ((probe.x as f32) + 0.5) / (probes as f32),
    ((probe.y as f32) + 0.5) / (probes as f32),
  );
  // `interval0` is the cascade-0 ray length, one probe spacing in field units. Each layer
  // quadruples its segment, so the layers cover the field without an overlap.
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
    const rayDirection = new Vec2f(
      cosine,
      -sine,
    );
    let radiance = new Vec3f(0.0, 0.0, 0.0);
    let transmittance: f32 = 1.0;
    let distanceAlong: f32 = rayStart;
    // The march advances by the scene distance, the sphere-trace step. The fixed 64 steps
    // bound the work per ray, and the break ends the march at the far end of the segment.
    for (let step: u32 = 0; step < 64; step += 1) {
      if (distanceAlong > rayEnd) break;
      const hit: SceneHit = sceneSdf(
        res.scene.$,
        probePos.add(rayDirection.scale(distanceAlong)),
      );
      if (hit.dist <= eps) {
        radiance = hit.color;
        transmittance = 0.0;
        break;
      }
      const advance: f32 = hit.dist > minStep ? hit.dist : minStep;
      distanceAlong += advance;
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

// Gathers cascade 0 into the output field. One invocation averages the four quadrant
// directions that reach its pixel.
function fieldKernel(res: FieldLayout, ctx: ComputeInvocation): void {
  const x: u32 = ctx.globalId.x;
  const y: u32 = ctx.globalId.y;
  if (x >= OUTPUT_SIZE || y >= OUTPUT_SIZE) return;
  const uv = new Vec2f(
    ((x as f32) + 0.5) / (OUTPUT_SIZE as f32),
    ((y as f32) + 0.5) / (OUTPUT_SIZE as f32),
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
// field's [0, 1], and the fragment stage interpolates it.
function radianceVertex(
  res: RenderLayout,
  vertex: Vertex,
  ctx: VertexInvocation,
): Varyings {
  return new Varyings(
    new Vec4f(vertex.position.x, vertex.position.y, 0.0, 1.0),
    new Vec2f((vertex.position.x + 1.0) * 0.5, (vertex.position.y + 1.0) * 0.5),
  );
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

// Samples the lit field, then paints the scene bodies over it.
function radianceFragment(
  res: RenderLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const fieldSample: Vec4f = res.field.sample(res.linear, input.uv);
  const fieldColor: Vec3f = acesFilm(new Vec3f(
    clamp(fieldSample.x, 0.0, 1.0),
    clamp(fieldSample.y, 0.0, 1.0),
    clamp(fieldSample.z, 0.0, 1.0),
  ));
  const hit: SceneHit = sceneSdf(res.scene.$, input.uv);
  const surface: Vec3f = acesFilm(hit.color);
  // A fixed field-space edge replaces the upstream derivative-sized edge. The 0.002 width
  // is about one texel of the 512-pixel field, the floor the upstream edge never drops below.
  const surfaceAlpha: f32 = 1.0 - smoothstep(-SURFACE_EDGE, SURFACE_EDGE, hit.dist);
  const color: Vec3f = fieldColor.mix(surface, surfaceAlpha);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

// The three declarations are the generator's input. It walks the typed program before the
// run and emits `main.typegpu.ts`: the WGSL text, the entry names, and the layout specs.
// TypeGPU resolves the same shader from the kernel function at run time.
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
export const radianceRender: RenderPipelineSpec = renderPipelineL<
  RenderLayout,
  Vertex,
  Varyings
>(radianceVertex, radianceFragment, { format: "bgra8unorm" });

// Three emissive disks light the scene. One black disk and two black boxes occlude it.
// Positions and sizes are in field units of [0, 1].
function initialScene(): Scene {
  return new Scene(
    [
      new DiskData(new Vec2f(0.2, 0.3), 0.05, new Vec3f(1.0, 0.0, 0.0)),
      new DiskData(new Vec2f(0.5, 0.3), 0.05, new Vec3f(0.0, 1.0, 0.0)),
      new DiskData(new Vec2f(0.8, 0.3), 0.05, new Vec3f(0.0, 0.0, 1.0)),
      new DiskData(new Vec2f(0.5, 0.75), 0.1, new Vec3f(0.0, 0.0, 0.0)),
    ],
    [
      new BoxData(new Vec2f(0.3, 0.5), new Vec2f(0.08, 0.15), new Vec3f(0.0, 0.0, 0.0)),
      new BoxData(new Vec2f(0.7, 0.65), new Vec2f(0.12, 0.08), new Vec3f(0.0, 0.0, 0.0)),
    ],
  );
}

// One object holds every handle a frame needs. Scripts own their handles, so `shutdown`
// releases each one. TypeGPU leaves that to garbage collection and `root.destroy`.
class RadianceState {
  device: GPUHostOwnedDevice;
  cascade: ComputePipeline;
  fieldBuild: ComputePipeline;
  render: RenderPipeline;
  cascadeGroups: GPUBindGroup[];
  fieldGroup: GPUBindGroup;
  renderGroup: GPUBindGroup;
  vertices: GPUBuffer;
  sceneBuffer: GPUBuffer;
  paramsBuffers: GPUBuffer[];
  textures: GPUTexture[];
  views: GPUTextureView[];
  sampler: GPUSampler;
  scene: Scene;
  // `dirty` records a scene change, so the cascades rebuild only after an edit. TypeGPU
  // rebuilds its lighting on a drag too, and never per frame. `draggedElement` keeps the
  // picked body until every pointer button releases.
  dirty: boolean;
  draggedElement: i32;

  constructor(
    device: GPUHostOwnedDevice,
    cascade: ComputePipeline,
    fieldBuild: ComputePipeline,
    render: RenderPipeline,
    cascadeGroups: GPUBindGroup[],
    fieldGroup: GPUBindGroup,
    renderGroup: GPUBindGroup,
    vertices: GPUBuffer,
    sceneBuffer: GPUBuffer,
    paramsBuffers: GPUBuffer[],
    textures: GPUTexture[],
    views: GPUTextureView[],
    sampler: GPUSampler,
    scene: Scene,
  ) {
    this.device = device;
    this.cascade = cascade;
    this.fieldBuild = fieldBuild;
    this.render = render;
    this.cascadeGroups = cascadeGroups;
    this.fieldGroup = fieldGroup;
    this.renderGroup = renderGroup;
    this.vertices = vertices;
    this.sceneBuffer = sceneBuffer;
    this.paramsBuffers = paramsBuffers;
    this.textures = textures;
    this.views = views;
    this.sampler = sampler;
    this.scene = scene;
    this.dirty = true;
    this.draggedElement = -1;
  }
}

// `init` fills this binding and `frame` reads it. A failed `init` leaves it `null`, because
// this library reports a failure by value and never by an exception.
let activeState: RadianceState | null = null;

function absolute(value: f32): f32 {
  return Math.abs(value as f64) as f32;
}

// The host uses the same signed-distance bodies as the kernels, then retains the chosen
// element until every pointer button is released. TypeGPU starts a drag only when the
// pointer lands inside a body, so a miss there moves nothing.
function nearestElement(scene: Scene, point: Vec2f): i32 {
  let selected: i32 = 0;
  let minimum: f32 = absolute(sdDisk(point, scene.disks[0].pos, scene.disks[0].radius));
  let index: i32 = 1;
  while (index < 4) {
    const distance: f32 = absolute(sdDisk(
      point,
      scene.disks[index].pos,
      scene.disks[index].radius,
    ));
    if (distance < minimum) {
      minimum = distance;
      selected = index;
    }
    index += 1;
  }
  index = 0;
  while (index < 2) {
    const distance: f32 = absolute(sdBox2d(
      point,
      scene.boxes[index].pos,
      scene.boxes[index].size,
    ));
    if (distance < minimum) {
      minimum = distance;
      selected = 4 + index;
    }
    index += 1;
  }
  return selected;
}

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The generator fixed the color target format. A surface with another format is a failure
  // here, not a reason to rebuild the pipeline.
  if (format !== radianceRender_TARGET_FORMAT) {
    print(`FAIL format expected=${radianceRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The kernels read the committed sizes as constants. The host sizing must agree with them,
  // so a mismatch stops `init` before any resource exists.
  const dimensions = cascadeDimensions(OUTPUT_SIZE);
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
    label: "radiance-cascades-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The scene uniform. `Scene_SIZE` is the generated C size, and it equals the WGSL size.
  const sceneBuffer = hostDevice.createBuffer({
    label: "radiance-cascades-scene",
    size: Scene_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
  // One uniform buffer per layer. The values never change, so `init` writes them once and
  // each layer's bind group selects its own buffer.
  const paramsBuffers: GPUBuffer[] = [];
  let layer: u32 = 0;
  while (layer < CASCADE_COUNT) {
    paramsBuffers.push(hostDevice.createBuffer({
      label: `radiance-cascades-layer-${layer}`,
      size: CascadeParams_SIZE as u64,
      usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
    }));
    layer += 1;
  }
  // Two array textures of `CASCADE_COUNT` layers hold the cascade set. A layer reads the
  // layer above from the other texture, because one dispatch cannot sample what it stores.
  const cascadeUsage: u64 = GPUTextureUsage.STORAGE_BINDING + GPUTextureUsage.TEXTURE_BINDING;
  const cascadeA = hostDevice.createTexture({
    label: "radiance-cascades-a",
    size: { width: CASCADE_DIM, height: CASCADE_DIM, depthOrArrayLayers: CASCADE_COUNT },
    format: "rgba16float",
    usage: cascadeUsage,
  });
  const cascadeB = hostDevice.createTexture({
    label: "radiance-cascades-b",
    size: { width: CASCADE_DIM, height: CASCADE_DIM, depthOrArrayLayers: CASCADE_COUNT },
    format: "rgba16float",
    usage: cascadeUsage,
  });
  // The gathered output field. The rgba16float format keeps radiance above 1.0 until the
  // fragment stage tone maps it.
  const field = hostDevice.createTexture({
    label: "radiance-field",
    size: { width: OUTPUT_SIZE, height: OUTPUT_SIZE, depthOrArrayLayers: 1 },
    format: "rgba16float",
    usage: cascadeUsage,
  });
  // Each cascade layer gets a single-layer 2D view, because a storage binding writes one
  // layer. The order is side A, then side B, then the field, and the bind groups index it.
  const views: GPUTextureView[] = [];
  layer = 0;
  while (layer < CASCADE_COUNT) {
    views.push(cascadeA.createView({
      dimension: "2d",
      mipLevelCount: 1,
      baseArrayLayer: layer,
      arrayLayerCount: 1,
    }));
    layer += 1;
  }
  layer = 0;
  while (layer < CASCADE_COUNT) {
    views.push(cascadeB.createView({
      dimension: "2d",
      mipLevelCount: 1,
      baseArrayLayer: layer,
      arrayLayerCount: 1,
    }));
    layer += 1;
  }
  const fieldView = field.createView();
  views.push(fieldView);
  // The linear filter is what interpolates between the upper probes during the merge.
  const samplerDescriptor: GPUSamplerDescriptor = { minFilter: "linear", magFilter: "linear" };
  const sampler = hostDevice.createSampler(samplerDescriptor);
  const scene = initialScene();
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
  queue.writeBuffer(sceneBuffer, 0, Context.bytesOf<Scene>(scene));
  // Each layer halves its probe count and doubles its ray count on one axis, so every layer
  // fills the same texture size.
  layer = 0;
  while (layer < CASCADE_COUNT) {
    queue.writeBuffer(
      paramsBuffers[layer as i32],
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

  // The scope catches a backend rejection of the WGSL or the layout. The API layer returns
  // the error as a value, so the code reads the popped result.
  hostDevice.pushErrorScope("validation");
  // Each pipeline takes generated WGSL text, a generated entry name, and a generated layout
  // spec. This file holds no shader text.
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
    radianceRender_WGSL,
    radianceRender_VERTEX_ENTRY,
    radianceRender_FRAGMENT_ENTRY,
    [radianceRender_LAYOUT0],
    [radianceRender_VERTEX_LAYOUT0],
    radianceRender,
  );
  const validationError = hostDevice.popErrorScope();
  // The failure path releases every handle this function created, newest first. Nothing else
  // frees them, because the state never received them.
  if (validationError !== null) {
    renderPipeline.dispose();
    fieldPipeline.dispose();
    cascadePipeline.dispose();
    sampler.dispose();
    let index: i32 = 0;
    while (index < views.length) {
      views[index].dispose();
      index += 1;
    }
    field.dispose();
    cascadeB.dispose();
    cascadeA.dispose();
    index = 0;
    while (index < paramsBuffers.length) {
      paramsBuffers[index].dispose();
      index += 1;
    }
    sceneBuffer.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }

  // The bind group layouts come from the pipelines, and `using` borrows them for the group
  // creation only.
  using cascadeLayout = cascadePipeline.bindGroupLayout(0);
  using fieldLayout = fieldPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
  // One bind group per layer, built once. `cascadeWriteSide` alternates the write texture per
  // layer, so a layer reads one side and writes the other.
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
        textureResource(views[(sourceSide * CASCADE_COUNT + upperLayer) as i32]),
        samplerResource(sampler),
        textureResource(views[(side * CASCADE_COUNT + layer) as i32]),
        bufferResource(sceneBuffer),
        bufferResource(paramsBuffers[layer as i32]),
      ],
    ));
    layer += 1;
  }
  // Layer 0 writes one of the two sides. The gather group reads layer 0 of that side.
  const cascade0Side: u32 = cascadeWriteSide(CASCADE_COUNT, 0);
  const fieldGroup = createBindGroupHost(hostDevice, fieldLayout, fieldBuild_LAYOUT0, [
    textureResource(views[(cascade0Side * CASCADE_COUNT) as i32]),
    samplerResource(sampler),
    textureResource(fieldView),
  ]);
  // The render group reads the field and the same scene uniform the kernels read.
  const renderGroup = createBindGroupHost(hostDevice, renderLayout, radianceRender_LAYOUT0, [
    textureResource(fieldView),
    samplerResource(sampler),
    bufferResource(sceneBuffer),
  ]);
  // The state takes every handle above. `frame` and `shutdown` reach them through this module
  // binding.
  activeState = new RadianceState(
    hostDevice,
    cascadePipeline,
    fieldPipeline,
    renderPipeline,
    cascadeGroups,
    fieldGroup,
    renderGroup,
    vertices,
    sceneBuffer,
    paramsBuffers,
    [cascadeA, cascadeB, field],
    views,
    sampler,
    scene,
  );
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
  // The host gives the pointer in surface pixels with y down. The scene works in field units
  // of [0, 1] with y up, so the y axis flips here.
  if (buttons !== 0 && pointerX >= 0.0 && pointerY >= 0.0) {
    const point = new Vec2f(
      clamp(pointerX / (width as f32), 0.0, 1.0),
      clamp(1.0 - pointerY / (height as f32), 0.0, 1.0),
    );
    // The drag picks one body and keeps it. A fast pointer therefore never jumps to another
    // body in the middle of a stroke.
    if (active.draggedElement < 0) {
      active.draggedElement = nearestElement(active.scene, point);
    }
    // Keep the write on the stored field chain so it lands in the scene the uniform serializes.
    if (active.draggedElement < 4) {
      active.scene.disks[active.draggedElement].pos = point;
    } else {
      active.scene.boxes[active.draggedElement - 4].pos = point;
    }
    active.dirty = true;
  } else {
    active.draggedElement = -1;
  }

  // The queue and the encoder live for this frame only.
  using queue = active.device.queue();
  using encoder = active.device.createCommandEncoderDefault();
  // The cascades rebuild only after a scene change. A still scene costs one render pass.
  if (active.dirty) {
    queue.writeBuffer(active.sceneBuffer, 0, Context.bytesOf<Scene>(active.scene));
    // The layers run from the top down, because each layer merges the layer above it. Each
    // `dispatch` records its own compute pass, and that pass order is the dependency order.
    let layer: i32 = (CASCADE_COUNT as i32) - 1;
    while (layer >= 0) {
      active.cascade.dispatch(
        encoder,
        [active.cascadeGroups[layer]],
        CASCADE_DIM / WORKGROUP_SIZE,
        CASCADE_DIM / WORKGROUP_SIZE,
        1,
      );
      layer -= 1;
    }
    // The gather runs after the whole layer loop and reads cascade 0. A dispatch count is a
    // workgroup count, so the texture size divides by the 16 of the workgroup declaration.
    active.fieldBuild.dispatch(
      encoder,
      [active.fieldGroup],
      OUTPUT_SIZE / WORKGROUP_SIZE,
      OUTPUT_SIZE / WORKGROUP_SIZE,
      1,
    );
    active.dirty = false;
  }

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
  // The field commits to 512 square pixels. The viewport stretches it over the current window
  // instead of a rebuild of the cascades.
  pass.setViewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
  pass.setScissorRect(0, 0, width, height);
  // `bind` sets the pipeline, the bind groups, and the vertex buffers on the pass in one call.
  active.render.bind(pass, [active.renderGroup], [active.vertices]);
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
  active.renderGroup.dispose();
  active.fieldGroup.dispose();
  let index: i32 = 0;
  while (index < active.cascadeGroups.length) {
    active.cascadeGroups[index].dispose();
    index += 1;
  }
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
  while (index < active.paramsBuffers.length) {
    active.paramsBuffers[index].dispose();
    index += 1;
  }
  active.sceneBuffer.dispose();
  active.vertices.dispose();
  active.render.dispose();
  active.fieldBuild.dispose();
  active.cascade.dispose();
  // The cleared binding makes a later `frame` call return at its guard.
  activeState = null;
}
