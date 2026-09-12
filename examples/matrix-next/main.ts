// example: matrix-next
// Compares a naive kernel and a workgroup-tiled kernel against one host result.
// This port runs all three upstream strategies together (the host lane is the oracle) and
// drops the strategy switch, the sliders, and the timestamp timing.
// Ported from TypeGPU's matrix-next example (https://github.com/software-mansion/TypeGPU).

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  Storage,
  WorkgroupArray,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  simulateComputeThreads,
  workgroupArray,
  workgroupBarrier,
} from "./typegpu";
import {
  Vec2u,
} from "./typegpu-types";
import {
  GPUAdapter,
  GPUBufferUsage,
  GPUDevice,
  gpu,
} from "./webgpu";
import {
  Matrix_SIZE,
  naive_ENTRY,
  naive_HOST_RUNNABLE,
  naive_LAYOUT0,
  naive_WGSL,
  naive_WORKGROUP_X,
  naive_WORKGROUP_Y,
  naive_WORKGROUP_Z,
  tiled_ENTRY,
  tiled_LAYOUT0,
  tiled_WGSL,
  tiled_WORKGROUP_X,
  tiled_WORKGROUP_Y,
  tiled_WORKGROUP_Z,
} from "./main.typegpu";

// One 4-by-4 matrix as a single schema value. TypeGPU keeps the dimensions in a separate
// uniform and grows its element array, because its sliders change the matrix size.
@CStruct
class Matrix {
  size: Vec2u;
  body: FixedArray<f32, 16>;

  constructor(size: Vec2u, body: FixedArray<f32, 16>) {
    this.size = size;
    this.body = body;
  }
}

// A bind group layout is a class here, not a runtime object. Field order fixes the binding
// numbers, and both kernels take the same class, so both bind groups share one shape.
class MatrixLayout {
  left: Storage<Matrix>;
  right: Storage<Matrix>;
  product: MutStorage<Matrix>;

  constructor(left: Storage<Matrix>, right: Storage<Matrix>, product: MutStorage<Matrix>) {
    this.left = left;
    this.right = right;
    this.product = product;
  }
}

// One invocation computes the whole product with three nested loops.
// output is a copy of the storage element, so the last statement stores the copy back.
function naiveKernel(res: MatrixLayout, ctx: ComputeInvocation): void {
  const left: Matrix = res.left[0];
  const right: Matrix = res.right[0];
  const output: Matrix = res.product[0];
  for (let row: u32 = 0; row < 4; row += 1) {
    for (let column: u32 = 0; column < 4; column += 1) {
      let total: f32 = 0.0;
      for (let inner: u32 = 0; inner < 4; inner += 1) {
        total += left.body[(row * 4 + inner) as i32]
          * right.body[(inner * 4 + column) as i32];
      }
      output.body[(row * 4 + column) as i32] = total;
    }
  }
  res.product[0] = output;
}

// This port fixes both matrices and the tile to 4-by-4 for one inspectable dispatch.
// Workgroup memory is shared by the lanes of one workgroup and it dies with the dispatch,
// so the tiles never reach the host.
const leftTile: WorkgroupArray<f32> = workgroupArray<f32>(16);
const rightTile: WorkgroupArray<f32> = workgroupArray<f32>(16);

// TypeGPU loops over sixteen-wide tiles and bounds-checks every load. One tile covers
// the whole matrix here, so each lane loads two values and multiplies four pairs.
function tiledKernel(res: MatrixLayout, ctx: ComputeInvocation): void {
  const left: Matrix = res.left[0];
  const right: Matrix = res.right[0];
  const lane: u32 = ctx.localId.y * 4 + ctx.localId.x;
  leftTile[lane] = left.body[(ctx.globalId.y * 4 + ctx.localId.x) as i32];
  rightTile[lane] = right.body[(ctx.localId.y * 4 + ctx.globalId.x) as i32];
  // Every lane must store its two tile values before any lane reads them. The barrier is
  // the only thing that gives the workgroup that order.
  workgroupBarrier();
  let total: f32 = 0.0;
  for (let inner: u32 = 0; inner < 4; inner += 1) {
    total += leftTile[ctx.localId.y * 4 + inner]
      * rightTile[inner * 4 + ctx.localId.x];
  }
  res.product[0].body[(ctx.globalId.y * 4 + ctx.globalId.x) as i32] = total;
}

// The declaration names the kernel and the workgroup size. A size of one thread suits a
// kernel that already loops over every cell.
export const naive: ComputePipelineSpec = computePipeline<MatrixLayout>(naiveKernel, {
  name: "naive",
  workgroupSize: [1, 1, 1],
});

// The workgroup size belongs to the declaration. The generator writes it into the
// WGSL and into `tiled_WORKGROUP_X`, which the pipeline below reads.
export const tiled: ComputePipelineSpec = computePipeline<MatrixLayout>(tiledKernel, {
  name: "tiled",
  workgroupSize: [4, 4, 1],
});

// Both output buffers start at zero, so a kernel that writes nothing leaves a value the
// check below recognizes.
function zeroMatrix(): Matrix {
  return new Matrix(
    new Vec2u(4, 4),
    [
      0.0, 0.0, 0.0, 0.0,
      0.0, 0.0, 0.0, 0.0,
      0.0, 0.0, 0.0, 0.0,
      0.0, 0.0, 0.0, 0.0,
    ],
  );
}

// An all-zero product beside a non-zero host result means the backend ran no arithmetic.
// That case reports noop, which separates an idle backend from a wrong kernel.
function resultState(actual: Matrix, expected: Matrix): string {
  let allZero: boolean = true;
  let hostNonzero: boolean = false;
  let matches: boolean = true;
  for (let i: i32 = 0; i < 16; i += 1) {
    allZero = allZero && actual.body[i] === 0.0;
    hostNonzero = hostNonzero || expected.body[i] !== 0.0;
    matches = matches && actual.body[i] === expected.body[i];
  }
  if (allZero && hostNonzero) {
    return "noop";
  }
  return matches ? "pass" : "fail";
}

// A typed buffer of one Matrix. Matrix_SIZE is the generated element size, so the host
// bytes and the WGSL struct always match.
function matrixBuffer(device: GPUDevice, usage: u64, label: string): Buffer<Matrix> {
  return createBuffer<Matrix>(device, Matrix_SIZE, 1, usage, label);
}

export async function main(): Promise<void> {
  // Adapter and device requests return null instead of a rejected promise. A headless example
  // prints one check line per kernel, so a run with no device still reports a result.
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    gpu.dispose();
    print("check:naive fail");
    print("check:tiled fail");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    adapterResult.dispose();
    gpu.dispose();
    print("check:naive fail");
    print("check:tiled fail");
    return;
  }
  let naiveState: string = "fail";
  let tiledState: string = "fail";
  // Every GPU handle lives inside this block. The block frees all of them before gpu.dispose
  // below, which the instance requires.
  {
    using adapter = adapterResult;
    using device = deviceResult;
    // Small whole numbers keep every product exact in f32, so the comparison needs no epsilon.
    // TypeGPU fills its matrices with random values and shows them in the page.
    const leftValue = new Matrix(
      new Vec2u(4, 4),
      [
        1.0, 4.0, 2.0, 3.0,
        3.0, 1.0, 0.0, 2.0,
        2.0, 2.0, 1.0, 1.0,
        0.0, 3.0, 4.0, 1.0,
      ],
    );
    const rightValue = new Matrix(
      new Vec2u(4, 4),
      [
        2.0, 0.0, 1.0, 3.0,
        1.0, 2.0, 3.0, 0.0,
        0.0, 4.0, 2.0, 1.0,
        3.0, 1.0, 0.0, 2.0,
      ],
    );
    // Four storage buffers: two inputs shared by both kernels, and one output per kernel.
    // An output also carries COPY_SRC, because the readback copies it into a staging buffer.
    using left = matrixBuffer(
      device,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "matrix-next-left",
    );
    using right = matrixBuffer(
      device,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "matrix-next-right",
    );
    using naiveOutput = matrixBuffer(
      device,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "matrix-next-naive",
    );
    using tiledOutput = matrixBuffer(
      device,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "matrix-next-tiled",
    );
    // Context.bytesOf packs each matrix with the layout rules the shader uses.
    // The four writes enter the queue before the dispatch below reads the buffers.
    left.writeOne(device.queue, 0, Context.bytesOf<Matrix>(leftValue));
    right.writeOne(device.queue, 0, Context.bytesOf<Matrix>(rightValue));
    naiveOutput.writeOne(device.queue, 0, Context.bytesOf<Matrix>(zeroMatrix()));
    tiledOutput.writeOne(device.queue, 0, Context.bytesOf<Matrix>(zeroMatrix()));

    // Pipeline creation raises no exception. The error scope covers both pipelines and turns
    // a bad shader or a bad layout into a value the code below tests.
    device.pushErrorScope("validation");
    using naivePipeline = createComputePipeline(
      device,
      naive_WGSL,
      naive_ENTRY,
      [naive_LAYOUT0],
      [naive_WORKGROUP_X, naive_WORKGROUP_Y, naive_WORKGROUP_Z],
    );
    using tiledPipeline = createComputePipeline(
      device,
      tiled_WGSL,
      tiled_ENTRY,
      [tiled_LAYOUT0],
      [tiled_WORKGROUP_X, tiled_WORKGROUP_Y, tiled_WORKGROUP_Z],
    );
    // The scope result arrives asynchronously, so the caller awaits it. Both states stay fail
    // when validation fails, and no dispatch runs.
    const validationError = await device.popErrorScope();
    if (validationError === null) {
      // Each kernel needs its own bind group, because the product buffer differs. The layout
      // comes back from the pipeline, so the shader and the bind group cannot disagree.
      using naiveNativeLayout = naivePipeline.bindGroupLayout(0);
      using naiveGroup = createBindGroup(
        device,
        naiveNativeLayout,
        naive_LAYOUT0,
        [
          bufferResource(left.handle()),
          bufferResource(right.handle()),
          bufferResource(naiveOutput.handle()),
        ],
      );
      using tiledNativeLayout = tiledPipeline.bindGroupLayout(0);
      using tiledGroup = createBindGroup(
        device,
        tiledNativeLayout,
        tiled_LAYOUT0,
        [
          bufferResource(left.handle()),
          bufferResource(right.handle()),
          bufferResource(tiledOutput.handle()),
        ],
      );
      // One encoder records both dispatches. They read the same inputs and write different
      // outputs, so the queue can run them in any order.
      using encoder = device.createCommandEncoderDefault();
      naivePipeline.dispatchThreads(encoder, [naiveGroup], 1, 1, 1);
      // `dispatchThreads` takes thread counts and rounds up by the workgroup size.
      // TypeGPU's example computes its workgroup counts with `Math.ceil`.
      tiledPipeline.dispatchThreads(encoder, [tiledGroup], 4, 4, 1);
      // The encoder must finish before submit. The readback below awaits the same queue, so the
      // dispatches complete before the copy starts.
      using command = encoder.finishDefault();
      device.queue.submit([command]);

      // The binding wrappers carry real bodies over plain arrays, so the same kernel source runs
      // on the host. TypeGPU keeps a separate function for its host result.
      const host = new MatrixLayout(
        new Storage<Matrix>([leftValue]),
        new Storage<Matrix>([rightValue]),
        new MutStorage<Matrix>([zeroMatrix()]),
      );
      // The naive kernel runs on the host and gives the oracle for both GPU results. The
      // tiled kernel reaches a barrier, so it has no host lane.
      simulateComputeThreads<MatrixLayout>(
        naiveKernel,
        host,
        naive,
        1,
        1,
        1,
        naive_HOST_RUNNABLE,
      );
      // readOne creates a mappable staging buffer, copies into it, submits, and maps it.
      // Context.fromBytes reads the same layout the shader wrote.
      const naiveBytes: u8[] = await naiveOutput.readOne(device, 0);
      const tiledBytes: u8[] = await tiledOutput.readOne(device, 0);
      const naiveActual: Matrix = Context.fromBytes<Matrix>(naiveBytes, 0);
      const tiledActual: Matrix = Context.fromBytes<Matrix>(tiledBytes, 0);
      const expected: Matrix = host.product[0];
      print(`products:first=${naiveActual.body[0]},${tiledActual.body[0]}`);
      // The headless backend validates every command and computes nothing, so both products
      // stay zero. resultState reports that case as noop instead of fail.
      naiveState = resultState(naiveActual, expected);
      tiledState = resultState(tiledActual, expected);
    }
  }
  // The instance closes last. The two check lines state the invariant this example proves:
  // each kernel reproduces the host product.
  gpu.dispose();
  print(`check:naive ${naiveState}`);
  print(`check:tiled ${tiledState}`);
}
