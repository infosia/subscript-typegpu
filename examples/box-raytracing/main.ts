// example: box-raytracing
// Accumulates volumetric color through a rotating 7x7x7 field of unit boxes.
// The upstream nested 3D array is flattened to x * 49 + y * 7 + z. Its mat4.aim
// camera reduces to host-computed origin/right/up/forward basis vectors. A zero-alpha
// fragment replaces upstream discard because the kernel subset has no discard.
// The upstream sliders and pointer do not port, and the camera orbits on the frame count.
// Boxes span whole lattice cells from 0 to 7, where upstream centers them with half-box
// bounds, and the quad is a triangle list, not a strip.
// Ported from TypeGPU's box-raytracing example (https://github.com/software-mansion/TypeGPU).

import {
  FragmentInvocation,
  RenderPipeline,
  RenderPipelineSpec,
  Storage,
  Uniform,
  VertexInvocation,
  bufferResource,
  createBindGroupHost,
  createRenderPipelineHost,
  renderPipelineL,
} from "./typegpu";
import {
  Vec2f,
  Vec3f,
  Vec4f,
} from "./typegpu-types";
import {
  linearToSrgb,
  srgbToLinear,
} from "./typegpu-color";
import {
  GPUBindGroup,
  GPUBuffer,
  GPUBufferUsage,
  GPUHostOwnedDevice,
  GPUTextureView,
  hostOwnedGPUDevice,
} from "./webgpu";
import {
  BoxCell_STRIDE,
  Camera_SIZE,
  Vertex_STRIDE,
  boxes_FRAGMENT_ENTRY,
  boxes_LAYOUT0,
  boxes_TARGET_FORMAT,
  boxes_VERTEX_ENTRY,
  boxes_VERTEX_LAYOUT0,
  boxes_WGSL,
} from "./main.typegpu";

// The grid holds 7 cells per axis, so CELL_COUNT is 343. ROTATION_SPEED is radians per
// second, and CAMERA_DISTANCE and BOX_SIZE are world units.
const GRID_SIZE: u32 = 7;
const CELL_COUNT: u32 = 343;
const ROTATION_SPEED: f32 = 1.2;
const CAMERA_DISTANCE: f32 = 16.0;
const BOX_SIZE: f32 = 1.0;
const MATERIAL_DENSITY: f32 = 2.0;

// The vertex record. The generator emits Vertex_STRIDE from this class, so the host code
// never counts bytes.
@CStruct
class Vertex {
  position: Vec2f;

  constructor(position: Vec2f) {
    this.position = position;
  }
}

// One cell of the flattened grid. A schema holds no `boolean`, because WGSL `bool` is not
// host-shareable, so `isActive` is a `u32`.
@CStruct
class BoxCell {
  isActive: u32;
  albedo: Vec3f;

  constructor(isActive: u32, albedo: Vec3f) {
    this.isActive = isActive;
    this.albedo = albedo;
  }
}

// The camera uniform. The host computes the basis vectors, so the fragment needs no matrix.
// Upstream sends an inverse view matrix from `mat4.aim` instead.
@CStruct
class Camera {
  canvasDims: Vec2f;
  origin: Vec3f;
  right: Vec3f;
  up: Vec3f;
  forward: Vec3f;

  constructor(
    canvasDims: Vec2f,
    origin: Vec3f,
    right: Vec3f,
    up: Vec3f,
    forward: Vec3f,
  ) {
    this.canvasDims = canvasDims;
    this.origin = origin;
    this.right = right;
    this.up = up;
    this.forward = forward;
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

// The slab test returns three values at once. `hit` is a `u32` for the same reason as
// `isActive`.
@CStruct
class Intersection {
  hit: u32;
  tMin: f32;
  tMax: f32;

  constructor(hit: u32, tMin: f32, tMax: f32) {
    this.hit = hit;
    this.tMin = tMin;
    this.tMax = tMax;
  }
}

// The bind group layout. `Storage` is read-only, which matches a grid that no pass writes.
class BoxLayout {
  cells: Storage<BoxCell>;
  camera: Uniform<Camera>;

  constructor(cells: Storage<BoxCell>, camera: Uniform<Camera>) {
    this.cells = cells;
    this.camera = camera;
  }
}

function scalarMin(a: f32, b: f32): f32 {
  return a < b ? a : b;
}

function scalarMax(a: f32, b: f32): f32 {
  return a > b ? a : b;
}

// Slab test per axis. A negative tMin clamps to the camera plane.
function intersectBox(
  origin: Vec3f,
  direction: Vec3f,
  minimum: Vec3f,
  maximum: Vec3f,
): Intersection {
  let tMin: f32 = -1000000.0;
  let tMax: f32 = 1000000.0;
  let near: f32 = 0.0;
  let far: f32 = 0.0;
  if (direction.x >= 0.0) {
    near = (minimum.x - origin.x) / direction.x;
    far = (maximum.x - origin.x) / direction.x;
  } else {
    near = (maximum.x - origin.x) / direction.x;
    far = (minimum.x - origin.x) / direction.x;
  }
  tMin = scalarMax(tMin, near);
  tMax = scalarMin(tMax, far);
  if (tMax < tMin) return new Intersection(0, 0.0, 0.0);
  if (direction.y >= 0.0) {
    near = (minimum.y - origin.y) / direction.y;
    far = (maximum.y - origin.y) / direction.y;
  } else {
    near = (maximum.y - origin.y) / direction.y;
    far = (minimum.y - origin.y) / direction.y;
  }
  tMin = scalarMax(tMin, near);
  tMax = scalarMin(tMax, far);
  if (tMax < tMin) return new Intersection(0, 0.0, 0.0);
  if (direction.z >= 0.0) {
    near = (minimum.z - origin.z) / direction.z;
    far = (maximum.z - origin.z) / direction.z;
  } else {
    near = (maximum.z - origin.z) / direction.z;
    far = (minimum.z - origin.z) / direction.z;
  }
  tMin = scalarMax(tMin, near);
  tMax = scalarMin(tMax, far);
  tMin = scalarMax(tMin, 0.0);
  return new Intersection(tMax >= tMin ? 1 : 0, tMin, tMax);
}

// The vertex stage passes the corner through and derives the uv. Upstream computes the ray
// origin here, and this port reads the camera uniform in the fragment stage.
function boxVertex(res: BoxLayout, value: Vertex, ctx: VertexInvocation): Varyings {
  return new Varyings(
    new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec2f((value.position.x + 1.0) * 0.5, (value.position.y + 1.0) * 0.5),
  );
}

// One invocation traces one ray through the whole grid. The direction comes from the camera
// basis, so no matrix multiply runs per pixel.
function boxFragment(
  res: BoxLayout,
  input: Varyings,
  ctx: FragmentInvocation,
): Vec4f {
  const camera: Camera = res.camera.$;
  // The view coordinates divide by the shorter side, so the field of view stays equal on both
  // axes.
  const pixel = input.uv.mul(camera.canvasDims);
  const minimumDimension: f32 = scalarMin(camera.canvasDims.x, camera.canvasDims.y);
  const view = pixel.sub(camera.canvasDims.scale(0.5)).scale(1.0 / minimumDimension);
  const direction: Vec3f = camera.right.scale(view.x)
    .add(camera.up.scale(view.y))
    .add(camera.forward)
    .normalize();
  // One test against the whole grid rejects a ray early, so a miss costs one slab test.
  const bounds: Intersection = intersectBox(
    camera.origin,
    direction,
    new Vec3f(0.0, 0.0, 0.0),
    new Vec3f(7.0 * BOX_SIZE, 7.0 * BOX_SIZE, 7.0 * BOX_SIZE),
  );
  if (bounds.hit === 0) return new Vec4f(0.0, 0.0, 0.0, 0.0);

  let densitySum: f32 = 0.0;
  let inverseColor = new Vec3f(0.0, 0.0, 0.0);
  let hitAny: boolean = false;
  // The flat loop replaces the upstream triple loop over a nested 3D array. The index splits
  // back into x, y, and z, because a flat storage array carries one stride.
  for (let index: u32 = 0; index < CELL_COUNT; index += 1) {
    const cell: BoxCell = res.cells[index];
    if (cell.isActive === 0) continue;
    const x: u32 = index / 49;
    const y: u32 = (index % 49) / 7;
    const z: u32 = index % 7;
    const minimum = new Vec3f(
      (x as f32) * BOX_SIZE,
      (y as f32) * BOX_SIZE,
      (z as f32) * BOX_SIZE,
    );
    const intersection: Intersection = intersectBox(
      camera.origin,
      direction,
      minimum,
      minimum.add(new Vec3f(BOX_SIZE, BOX_SIZE, BOX_SIZE)),
    );
    if (intersection.hit === 0) continue;
    // The traversed length through each box adds density and inverse albedo.
    const boxDensity: f32 = scalarMax(0.0, intersection.tMax - intersection.tMin)
      * MATERIAL_DENSITY * MATERIAL_DENSITY;
    densitySum += boxDensity;
    inverseColor = inverseColor.add(new Vec3f(
      boxDensity / cell.albedo.x,
      boxDensity / cell.albedo.y,
      boxDensity / cell.albedo.z,
    ));
    hitAny = true;
  }
  if (!hitAny) return new Vec4f(0.0, 0.0, 0.0, 0.0);
  // The reciprocal turns the accumulated inverse albedo back into a color.
  const srgb: Vec3f = linearToSrgb(new Vec3f(
    1.0 / inverseColor.x,
    1.0 / inverseColor.y,
    1.0 / inverseColor.z,
  ));
  // The second encode follows the upstream shader's deliberate wash.
  const corrected: Vec3f = srgb.pow(new Vec3f(1.0 / 2.2, 1.0 / 2.2, 1.0 / 2.2));
  // The blend state below expects premultiplied color, so the return scales the color and the
  // alpha by the same factor.
  const alpha: f32 = scalarMin(densitySum, 1.0);
  return new Vec4f(
    scalarMin(corrected.x, 1.0),
    scalarMin(corrected.y, 1.0),
    scalarMin(corrected.z, 1.0),
    1.0,
  ).scale(alpha);
}

// The declaration carries the blend state. `one` and `one-minus-src-alpha` compose the
// premultiplied color the fragment returns over the cleared background.
export const boxes: RenderPipelineSpec = renderPipelineL<BoxLayout, Vertex, Varyings>(
  boxVertex,
  boxFragment,
  {
    format: "bgra8unorm",
    blend: {
      color: {
        operation: "add",
        srcFactor: "one",
        dstFactor: "one-minus-src-alpha",
      },
      alpha: {
        operation: "add",
        srcFactor: "one",
        dstFactor: "one-minus-src-alpha",
      },
    },
  },
);

// The host calls `init`, `frame`, and `shutdown` separately, so every handle that outlives
// `init` lives in module state. A `using` declaration disposes a handle too early here.
let activeDevice: GPUHostOwnedDevice | null = null;
let activePipeline: RenderPipeline | null = null;
let activeVertices: GPUBuffer | null = null;
let activeCells: GPUBuffer | null = null;
let activeCamera: GPUBuffer | null = null;
let activeGroup: GPUBindGroup | null = null;
let frameCount: u32 = 0;

// The host builds the camera basis once per frame. The eye orbits the grid center at
// CAMERA_DISTANCE, and `right`, `up`, and `forward` form the view frame.
function orbitCamera(width: u32, height: u32, time: f32): Camera {
  const center = new Vec3f(3.0, 3.0, 3.0);
  const eye = center.add(new Vec3f(
    (Math.cos(time as f64) as f32) * CAMERA_DISTANCE,
    -5.0,
    (Math.sin(time as f64) as f32) * CAMERA_DISTANCE,
  ));
  const forward: Vec3f = center.sub(eye).normalize();
  const right: Vec3f = forward.cross(new Vec3f(0.0, 1.0, 0.0)).normalize();
  const up: Vec3f = right.cross(forward).normalize();
  return new Camera(new Vec2f(width as f32, height as f32), eye, right, up, forward);
}

// `init` runs once, after the host configures the surface. It creates every long-lived
// resource. The instance and the device stay with the host.
export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  // The generator pins the target format into the pipeline. A mismatch with the host surface
  // fails here, not inside pipeline creation.
  if (format !== boxes_TARGET_FORMAT) {
    print(`FAIL format expected=${boxes_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The wrapper adapts the host handles to the API layer. It carries no `dispose`, because
  // the host owns the device.
  const hostDevice = hostOwnedGPUDevice(instance, device);
  // The vertex buffer holds the three corners of the fullscreen triangle. Vertex_STRIDE keeps
  // the size right when the schema changes.
  const vertices = hostDevice.createBuffer({
    label: "box-raytracing-fullscreen",
    size: (Vertex_STRIDE * 3) as u64,
    usage: GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST,
  });
  // The cell buffer holds 343 records. STORAGE covers the shader read, and COPY_DST covers the
  // host write below.
  const cells = hostDevice.createBuffer({
    label: "box-raytracing-cells",
    size: (BoxCell_STRIDE * CELL_COUNT) as u64,
    usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
  });
  // One uniform buffer holds Camera. COPY_DST admits the queue write that each frame makes.
  const camera = hostDevice.createBuffer({
    label: "box-raytracing-camera",
    size: Camera_SIZE as u64,
    usage: GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST,
  });
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
  // The seed writes one record per cell, at the flat index the fragment reads back. The active
  // test cuts a corner out of the cube, and the albedo maps the cell position to a color.
  for (let x: u32 = 0; x < GRID_SIZE; x += 1) {
    for (let y: u32 = 0; y < GRID_SIZE; y += 1) {
      for (let z: u32 = 0; z < GRID_SIZE; z += 1) {
        const index: u32 = x * 49 + y * 7 + z;
        const active: u32 = 7 - x + y + (7 - z) > 6 ? 1 : 0;
        // The clamp keeps the reciprocal finite.
        const albedo: Vec3f = srgbToLinear(new Vec3f(
          scalarMax(0.004, (x as f32) / 7.0),
          scalarMax(0.004, (y as f32) / 7.0),
          scalarMax(
            0.004,
            ((z as f32) / 7.0) * 0.8 + 0.1 + (((7 - x) as f32) / 7.0) * 0.6,
          ),
        ));
        queue.writeBuffer(
          cells,
          (index as u64) * (BoxCell_STRIDE as u64),
          Context.bytesOf<BoxCell>(new BoxCell(active, albedo)),
        );
      }
    }
  }
  // The first camera write assumes a 1 by 1 surface. The first frame replaces it with the real
  // surface size.
  queue.writeBuffer(camera, 0, Context.bytesOf<Camera>(orbitCamera(1, 1, 0.0)));
  // The error scope catches a validation failure from pipeline creation. The layers return the
  // failure as a value, so a `null` check replaces an exception.
  hostDevice.pushErrorScope("validation");
  // The call takes the generated WGSL text, the two entry names, the bind group layout, and
  // the vertex layout. No shader text is built here.
  const pipeline = createRenderPipelineHost(
    hostDevice,
    boxes_WGSL,
    boxes_VERTEX_ENTRY,
    boxes_FRAGMENT_ENTRY,
    [boxes_LAYOUT0],
    [boxes_VERTEX_LAYOUT0],
    boxes,
  );
  const validationError = hostDevice.popErrorScope();
  // The error path disposes every handle that the failed run already created, because no
  // finalizer runs later.
  if (validationError !== null) {
    pipeline.dispose();
    camera.dispose();
    cells.dispose();
    vertices.dispose();
    print(`FAIL validation ${validationError.message.split("\n")[0]}`);
    return;
  }
  // The native layout comes from the pipeline. The bind group reads it at creation, so `using`
  // releases the handle right after.
  using bindLayout = pipeline.bindGroupLayout(0);
  // The resource order follows the field order of BoxLayout: the cells, then the camera.
  const group = createBindGroupHost(
    hostDevice,
    bindLayout,
    boxes_LAYOUT0,
    [bufferResource(cells), bufferResource(camera)],
  );
  // The state moves to module scope only after every step passes. A failed `init` leaves the
  // fields null, and `frame` returns at once.
  activeDevice = hostDevice;
  activePipeline = pipeline;
  activeVertices = vertices;
  activeCells = cells;
  activeCamera = camera;
  activeGroup = group;
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
  const vertices = activeVertices;
  const camera = activeCamera;
  const group = activeGroup;
  // A null field means `init` failed or never ran. The frame returns, because the layers
  // report failure as a value.
  if (device === null) return;
  if (pipeline === null) return;
  if (vertices === null) return;
  if (camera === null) return;
  if (group === null) return;
  // The frame count is the only clock the window host offers. 60 frames stand for one second,
  // and ROTATION_SPEED turns that into the orbit angle.
  frameCount += 1;
  // The queue applies the camera write below before it runs the submitted command buffer. The
  // fragment then reads the pose of this frame.
  using queue = device.queue();
  queue.writeBuffer(
    camera,
    0,
    Context.bytesOf<Camera>(orbitCamera(
      width,
      height,
      ((frameCount as f32) / 60.0) * ROTATION_SPEED,
    )),
  );
  // The host owns the swapchain view. The wrapper adds no ownership, and `shutdown` never
  // disposes it.
  const target = new GPUTextureView(view);
  // One encoder records the whole frame. `using` disposes it after the submit.
  using encoder = device.createCommandEncoderDefault();
  // The clear paints the background. The blend then composes each ray result over that color.
  using pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: target,
      clearValue: { r: 0.008, g: 0.012, b: 0.025, a: 1.0 },
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
  if (activeCamera !== null) activeCamera.dispose();
  if (activeCells !== null) activeCells.dispose();
  if (activeVertices !== null) activeVertices.dispose();
  if (activePipeline !== null) activePipeline.dispose();
  // The null assignments make a second `shutdown` call safe.
  activeCamera = null;
  activeCells = null;
  activeVertices = null;
  activePipeline = null;
  activeGroup = null;
  activeDevice = null;
  frameCount = 0;
}
