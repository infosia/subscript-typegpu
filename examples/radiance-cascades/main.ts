// example: radiance-cascades
// Builds a layered radiance field for an interactive signed-distance scene.
// Output and cascade storage commit to 512 square pixels, so resize stretches the final
// field instead of rebuilding it. This drops the upstream overlay and layer slider.
// Its derivative-sized surface edge becomes a fixed 0.002 field-space width because the
// kernel subset has no fwidth. Pointer dragging keeps the nearest-element interaction.
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

const OUTPUT_SIZE: u32 = 512;
const CASCADE_PROBES: u32 = 256;
const CASCADE_DIM: u32 = 512;
const CASCADE_COUNT: u32 = 6;
const WORKGROUP_SIZE: u32 = 16;
const SURFACE_EDGE: f32 = 0.002;

@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

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

@CStruct
class Scene {
  disks: FixedArray<DiskData, 4>;
  boxes: FixedArray<BoxData, 2>;

  constructor(disks: FixedArray<DiskData, 4>, boxes: FixedArray<BoxData, 2>) {
    this.disks = disks;
    this.boxes = boxes;
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
class Varyings {
  position: Vec4f;
  uv: Vec2f;

  constructor(position: Vec4f, uv: Vec2f) {
    this.position = position;
    this.uv = uv;
  }
}

@CStruct
class SceneHit {
  dist: f32;
  color: Vec3f;

  constructor(dist: f32, color: Vec3f) {
    this.dist = dist;
    this.color = color;
  }
}

class CascadeLayout {
  upper!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
  scene!: Uniform<Scene>;
  params!: Uniform<CascadeParams>;
}

class FieldLayout {
  cascade0!: Texture2d<f32>;
  linear!: Sampler;
  target!: StorageTexture2d<Rgba16float>;
}

class RenderLayout {
  field!: Texture2d<f32>;
  linear!: Sampler;
  scene!: Uniform<Scene>;
}

function sceneSdf(scene: Scene, point: Vec2f): SceneHit {
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

// One stored direction owns four actual rays. A surviving ray samples the already-built
// next layer; RGB carries radiance and alpha carries transmittance.
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
    const rayDirection = new Vec2f(
      cosine,
      -sine,
    );
    let radiance = new Vec3f(0.0, 0.0, 0.0);
    let transmittance: f32 = 1.0;
    let distanceAlong: f32 = rayStart;
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
  // A fixed field-space edge replaces the upstream derivative-sized edge.
  const surfaceAlpha: f32 = 1.0 - smoothstep(-SURFACE_EDGE, SURFACE_EDGE, hit.dist);
  const color: Vec3f = fieldColor.mix(surface, surfaceAlpha);
  return new Vec4f(color.x, color.y, color.z, 1.0);
}

export const cascadePass: ComputePipelineSpec = computePipeline<CascadeLayout>(
  cascadeKernel,
  { name: "cascadePass", workgroupSize: [16, 16, 1] },
);

export const fieldBuild: ComputePipelineSpec = computePipeline<FieldLayout>(fieldKernel, {
  name: "fieldBuild",
  workgroupSize: [16, 16, 1],
});

export const radianceRender: RenderPipelineSpec = renderPipelineL<
  RenderLayout,
  Vertex,
  Varyings
>(radianceVertex, radianceFragment, { format: "bgra8unorm" });

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

let activeState: RadianceState | null = null;

function absolute(value: f32): f32 {
  return Math.abs(value as f64) as f32;
}

// The host uses the same signed-distance bodies as the kernels, then retains the chosen
// element until every pointer button is released.
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
  if (format !== radianceRender_TARGET_FORMAT) {
    print(`FAIL format expected=${radianceRender_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const dimensions = cascadeDimensions(OUTPUT_SIZE);
  if (dimensions.cascadeProbes !== CASCADE_PROBES
    || dimensions.cascadeDim !== CASCADE_DIM
    || dimensions.cascadeCount !== CASCADE_COUNT) {
    print("FAIL committed cascade dimensions");
    return;
  }
  const hostDevice = hostOwnedGPUDevice(instance, device);
  const vertices = hostDevice.createBuffer({
    label: "radiance-cascades-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  const sceneBuffer = hostDevice.createBuffer({
    label: "radiance-cascades-scene",
    size: Scene_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
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
  const field = hostDevice.createTexture({
    label: "radiance-field",
    size: { width: OUTPUT_SIZE, height: OUTPUT_SIZE, depthOrArrayLayers: 1 },
    format: "rgba16float",
    usage: cascadeUsage,
  });
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
  const samplerDescriptor: GPUSamplerDescriptor = { minFilter: "linear", magFilter: "linear" };
  const sampler = hostDevice.createSampler(samplerDescriptor);
  const scene = initialScene();
  using queue = hostDevice.queue();
  queue.writeBuffer(vertices, 0, Context.bytesOf<FixedArray<Vertex, 3>>([
    new Vertex(new Vec2f(-1.0, -1.0)),
    new Vertex(new Vec2f(3.0, -1.0)),
    new Vertex(new Vec2f(-1.0, 3.0)),
  ]));
  queue.writeBuffer(sceneBuffer, 0, Context.bytesOf<Scene>(scene));
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

  hostDevice.pushErrorScope("validation");
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
    radianceRender_WGSL,
    radianceRender_VERTEX_ENTRY,
    radianceRender_FRAGMENT_ENTRY,
    [radianceRender_LAYOUT0],
    [radianceRender_VERTEX_LAYOUT0],
    radianceRender,
  );
  const validationError = hostDevice.popErrorScope();
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

  using cascadeLayout = cascadePipeline.bindGroupLayout(0);
  using fieldLayout = fieldPipeline.bindGroupLayout(0);
  using renderLayout = renderPipeline.bindGroupLayout(0);
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
        textureResource(views[(sourceSide * CASCADE_COUNT + upperLayer) as i32]),
        samplerResource(sampler),
        textureResource(views[(side * CASCADE_COUNT + layer) as i32]),
        bufferResource(sceneBuffer),
        bufferResource(paramsBuffers[layer as i32]),
      ],
    ));
    layer += 1;
  }
  const cascade0Side: u32 = cascadeWriteSide(CASCADE_COUNT, 0);
  const fieldGroup = createBindGroupHost(hostDevice, fieldLayout, fieldBuild_LAYOUT0, [
    textureResource(views[(cascade0Side * CASCADE_COUNT) as i32]),
    samplerResource(sampler),
    textureResource(fieldView),
  ]);
  const renderGroup = createBindGroupHost(hostDevice, renderLayout, radianceRender_LAYOUT0, [
    textureResource(fieldView),
    samplerResource(sampler),
    bufferResource(sceneBuffer),
  ]);
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
  if (activeState === null) return;
  const active = activeState;
  if (buttons !== 0 && pointerX >= 0.0 && pointerY >= 0.0) {
    const point = new Vec2f(
      clamp(pointerX / (width as f32), 0.0, 1.0),
      clamp(1.0 - pointerY / (height as f32), 0.0, 1.0),
    );
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

  using queue = active.device.queue();
  using encoder = active.device.createCommandEncoderDefault();
  if (active.dirty) {
    queue.writeBuffer(active.sceneBuffer, 0, Context.bytesOf<Scene>(active.scene));
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
    active.fieldBuild.dispatch(
      encoder,
      [active.fieldGroup],
      OUTPUT_SIZE / WORKGROUP_SIZE,
      OUTPUT_SIZE / WORKGROUP_SIZE,
      1,
    );
    active.dirty = false;
  }

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
  active.render.bind(pass, [active.renderGroup], [active.vertices]);
  pass.draw(3);
  pass.end();
  using command = encoder.finishDefault();
  queue.submit([command]);
}

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
  activeState = null;
}
