// example: matrix-next
// Compares a naive kernel and a workgroup-tiled kernel against one host result.
// This port drops the upstream strategy switch, the sliders, and the timestamp timing.
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

@CStruct
class Matrix {
  size: Vec2u;
  body: FixedArray<f32, 16>;

  constructor(size: Vec2u, body: FixedArray<f32, 16>) {
    this.size = size;
    this.body = body;
  }
}

class MatrixLayout {
  left!: Storage<Matrix>;
  right!: Storage<Matrix>;
  product!: MutStorage<Matrix>;
}

function naiveKernel(res: MatrixLayout, ctx: ComputeInvocation): void {
  const left: Matrix = res.left.get(0);
  const right: Matrix = res.right.get(0);
  const output: Matrix = res.product.get(0);
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
  res.product.set(0, output);
}

// This port fixes both matrices and the tile to 4-by-4 for one inspectable dispatch.
const leftTile: WorkgroupArray<f32> = workgroupArray<f32>(16);
const rightTile: WorkgroupArray<f32> = workgroupArray<f32>(16);

// TypeGPU loops over sixteen-wide tiles and bounds-checks every load. One tile covers
// the whole matrix here, so each lane loads two values and multiplies four pairs.
function tiledKernel(res: MatrixLayout, ctx: ComputeInvocation): void {
  const left: Matrix = res.left.get(0);
  const right: Matrix = res.right.get(0);
  const lane: u32 = ctx.localId.y * 4 + ctx.localId.x;
  leftTile[lane] = left.body[(ctx.globalId.y * 4 + ctx.localId.x) as i32];
  rightTile[lane] = right.body[(ctx.localId.y * 4 + ctx.globalId.x) as i32];
  workgroupBarrier();
  let total: f32 = 0.0;
  for (let inner: u32 = 0; inner < 4; inner += 1) {
    total += leftTile[ctx.localId.y * 4 + inner]
      * rightTile[inner * 4 + ctx.localId.x];
  }
  res.product[0].body[(ctx.globalId.y * 4 + ctx.globalId.x) as i32] = total;
}

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

function matrixBuffer(device: GPUDevice, usage: u64, label: string): Buffer<Matrix> {
  return createBuffer<Matrix>(device, Matrix_SIZE, 1, usage, label);
}

export async function main(): Promise<void> {
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
  {
    using adapter = adapterResult;
    using device = deviceResult;
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
    using queue = device.queue();
    left.writeOne(queue, 0, Context.bytesOf<Matrix>(leftValue));
    right.writeOne(queue, 0, Context.bytesOf<Matrix>(rightValue));
    naiveOutput.writeOne(queue, 0, Context.bytesOf<Matrix>(zeroMatrix()));
    tiledOutput.writeOne(queue, 0, Context.bytesOf<Matrix>(zeroMatrix()));

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
    const validationError = await device.popErrorScope();
    if (validationError === null) {
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
      using encoder = device.createCommandEncoderDefault();
      naivePipeline.dispatchThreads(encoder, [naiveGroup], 1, 1, 1);
      // `dispatchThreads` takes thread counts and rounds up by the workgroup size.
      // TypeGPU's example computes its workgroup counts with `Math.ceil`.
      tiledPipeline.dispatchThreads(encoder, [tiledGroup], 4, 4, 1);
      using command = encoder.finishDefault();
      queue.submit([command]);

      const host = new MatrixLayout();
      host.left = new Storage<Matrix>([leftValue]);
      host.right = new Storage<Matrix>([rightValue]);
      host.product = new MutStorage<Matrix>([zeroMatrix()]);
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
      const naiveBytes: u8[] = await naiveOutput.readOne(device, 0);
      const tiledBytes: u8[] = await tiledOutput.readOne(device, 0);
      const naiveActual: Matrix = Context.fromBytes<Matrix>(naiveBytes, 0);
      const tiledActual: Matrix = Context.fromBytes<Matrix>(tiledBytes, 0);
      const expected: Matrix = host.product.get(0);
      print(`products:first=${naiveActual.body[0]},${tiledActual.body[0]}`);
      // Noop validates both kernels but leaves both products zeroed.
      naiveState = resultState(naiveActual, expected);
      tiledState = resultState(tiledActual, expected);
    }
  }
  gpu.dispose();
  print(`check:naive ${naiveState}`);
  print(`check:tiled ${tiledState}`);
}
