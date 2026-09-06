// example: matrix-multiplication
// Multiplies two committed four-by-four matrices and checks the product on the host.
// The upstream size sliders and random values become fixed data.
// Ported from TypeGPU's matrix-multiplication example (https://github.com/software-mansion/TypeGPU).

import {
  Buffer,
  ComputeInvocation,
  ComputePipelineSpec,
  MutStorage,
  Storage,
  bufferResource,
  computePipeline,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  simulateComputeThreads,
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
  multiply_ENTRY,
  multiply_HOST_RUNNABLE,
  multiply_LAYOUT0,
  multiply_WGSL,
  multiply_WORKGROUP_X,
  multiply_WORKGROUP_Y,
  multiply_WORKGROUP_Z,
} from "./main.typegpu";

// The matrix record: the live size and a fixed 16-value body. TypeGPU sizes its body for a
// six-by-six maximum and drives the live size from sliders.
@CStruct
class Matrix {
  size: Vec2u;
  body: FixedArray<f32, 16>;

  constructor(size: Vec2u, body: FixedArray<f32, 16>) {
    this.size = size;
    this.body = body;
  }
}

// TypeGPU declares the same three bindings with a bind group layout. A resources
// class names them here, and the generator emits `multiply_LAYOUT0` from it.
class MatrixLayout {
  left!: Storage<Matrix>;
  right!: Storage<Matrix>;
  product!: MutStorage<Matrix>;
}

// TypeGPU writes this kernel as a WGSL template and resolves it at run time.
// The kernel is subscript source here, and the generator emits the WGSL.
function multiplyKernel(res: MatrixLayout, ctx: ComputeInvocation): void {
  const left: Matrix = res.left[0];
  const right: Matrix = res.right[0];
  for (let row: u32 = 0; row < left.size.x; row += 1) {
    for (let column: u32 = 0; column < right.size.y; column += 1) {
      let total: f32 = 0.0;
      for (let inner: u32 = 0; inner < left.size.y; inner += 1) {
        total += left.body[(row * left.size.y + inner) as i32]
          * right.body[(inner * right.size.y + column) as i32];
      }
      res.product[0].body[(row * right.size.y + column) as i32] = total;
    }
  }
}

export const multiply: ComputePipelineSpec = computePipeline<MatrixLayout>(
  multiplyKernel,
  {
    name: "multiply",
    // TypeGPU runs one thread per output cell in eight-by-eight workgroups. One thread
    // walks every cell here, so a single dispatch stays inspectable.
    workgroupSize: [1, 1, 1],
  },
);

// The product buffer starts from a zeroed record, so a partial dispatch leaves no stale
// value in the readback.
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

// Three outcomes: `noop` when the device wrote nothing, `pass` on an exact match, and `fail`
// otherwise. The match is exact, because both lanes add in the same order.
function resultState(actual: Matrix, expected: Matrix): string {
  let allZero: boolean = true;
  let hostNonzero: boolean = false;
  let matches: boolean = true;
  for (let i: i32 = 0; i < 16; i += 1) {
    if (actual.body[i] !== 0.0) {
      allZero = false;
    }
    if (expected.body[i] !== 0.0) {
      hostNonzero = true;
    }
    if (actual.body[i] !== expected.body[i]) {
      matches = false;
    }
  }
  if (allZero && hostNonzero) {
    return "noop";
  }
  return matches ? "pass" : "fail";
}

export async function main(): Promise<void> {
  // The API layer polls the future itself, so the script never pumps the event loop. A null
  // adapter reports the failure by value, because the layers carry no exceptions.
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    gpu.dispose();
    print("check:product fail");
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    adapterResult.dispose();
    gpu.dispose();
    print("check:product fail");
    return;
  }
  let state: string = "fail";
  {
    // Every GPU handle lives in this block, and the script releases each one at the end.
    // TypeGPU leaves the same handles to `root.destroy` and the collector.
    using adapter = adapterResult;
    using device = deviceResult;
    // Two committed four-by-four matrices. TypeGPU fills its matrices with random values on
    // every reshuffle, so its result changes between runs.
    const leftValue = new Matrix(
      new Vec2u(4, 4),
      [
        1.0, 2.0, 3.0, 4.0,
        2.0, 0.0, 1.0, 3.0,
        4.0, 1.0, 2.0, 0.0,
        3.0, 2.0, 0.0, 1.0,
      ],
    );
    const rightValue = new Matrix(
      new Vec2u(4, 4),
      [
        2.0, 1.0, 0.0, 3.0,
        1.0, 3.0, 2.0, 0.0,
        4.0, 0.0, 1.0, 2.0,
        0.0, 2.0, 3.0, 1.0,
      ],
    );
    // `Matrix_SIZE` is a generated constant, so each buffer follows the schema layout. The two
    // inputs need STORAGE and COPY_DST, and the product adds COPY_SRC for the readback.
    using left: Buffer<Matrix> = createBuffer<Matrix>(
      device,
      Matrix_SIZE,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "matrix-left",
    );
    using right: Buffer<Matrix> = createBuffer<Matrix>(
      device,
      Matrix_SIZE,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST,
      "matrix-right",
    );
    using product: Buffer<Matrix> = createBuffer<Matrix>(
      device,
      Matrix_SIZE,
      1,
      GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC,
      "matrix-product",
    );
    // The queue keeps its work in the order it receives it, so these writes land before the
    // submitted dispatch reads them.
    left.writeOne(device.queue, 0, Context.bytesOf<Matrix>(leftValue));
    right.writeOne(device.queue, 0, Context.bytesOf<Matrix>(rightValue));
    product.writeOne(device.queue, 0, Context.bytesOf<Matrix>(zeroMatrix()));
    // The generated WGSL, entry name, layout, and workgroup size build the pipeline. The scope
    // catches a validation error from that call.
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      multiply_WGSL,
      multiply_ENTRY,
      [multiply_LAYOUT0],
      [multiply_WORKGROUP_X, multiply_WORKGROUP_Y, multiply_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError === null) {
      // The bind group joins the three buffers to the generated layout. The resource order
      // follows the field order of `MatrixLayout`.
      using nativeLayout = pipeline.bindGroupLayout(0);
      using bindGroup = createBindGroup(
        device,
        nativeLayout,
        multiply_LAYOUT0,
        [
          bufferResource(left.handle()),
          bufferResource(right.handle()),
          bufferResource(product.handle()),
        ],
      );
      using encoder = device.createCommandEncoderDefault();
      // One thread walks every output cell, so the dispatch asks for exactly one thread and
      // the workgroup count stays at one.
      pipeline.dispatchThreads(encoder, [bindGroup], 1, 1, 1);
      // The GPU runs nothing until the queue receives the command buffer.
      using command = encoder.finishDefault();
      device.queue.submit([command]);

      // The host lane holds the same three bindings in wrapper types, so one kernel body
      // serves both lanes.
      const host = new MatrixLayout();
      host.left = new Storage<Matrix>([leftValue]);
      host.right = new Storage<Matrix>([rightValue]);
      host.product = new MutStorage<Matrix>([zeroMatrix()]);
      // The host lane runs the same kernel over host storage, so the example holds no
      // second formula. TypeGPU's example prints the GPU result to the page.
      simulateComputeThreads<MatrixLayout>(
        multiplyKernel,
        host,
        multiply,
        1,
        1,
        1,
        multiply_HOST_RUNNABLE,
      );
      // The read copies into a staging buffer and awaits the map. TypeGPU's `buffer.read`
      // hides the same two steps.
      const bytes: u8[] = await product.readOne(device, 0);
      const actual: Matrix = Context.fromBytes<Matrix>(bytes, 0);
      const expected: Matrix = host.product[0];
      print(`product:first=${actual.body[0]} last=${actual.body[15]}`);
      // Noop validates the pipeline but leaves the product zeroed.
      state = resultState(actual, expected);
    }
  }
  gpu.dispose();
  // One check line reports the outcome, so a reader who runs the example needs no golden.
  print(`check:product ${state}`);
}
