// program: b13-vector-builtins
// purpose: execute every vector method family through one host-runnable kernel
// exercises: CL1, CL2, K10, K25, K26, K27, K28, PI14
// questions: none

import {
  bufferResource,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  createBindGroup,
  createComputePipeline,
  MutStorage,
  simulateCompute,
  Storage,
} from "./typegpu";
import {
  Vec2f,
  Vec2i,
  Vec2u,
  Vec3f,
  Vec3i,
  Vec3u,
  Vec4f,
  Vec4i,
  Vec4u,
  v2fSplat,
  v2iSplat,
  v2uSplat,
  v3fFrom2,
  v3fSplat,
  v3iFrom2,
  v3iSplat,
  v3uFrom2,
  v3uSplat,
  v4fFrom2,
  v4fFrom3,
  v4fSplat,
  v4iFrom2,
  v4iFrom3,
  v4iSplat,
  v4uFrom2,
  v4uFrom3,
  v4uSplat,
} from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  VectorInput_SIZE,
  VectorOutput_SIZE,
  vectorBuiltins_ENTRY,
  vectorBuiltins_HOST_RUNNABLE,
  vectorBuiltins_LAYOUT0,
  vectorBuiltins_WGSL,
  vectorBuiltins_WORKGROUP_X,
  vectorBuiltins_WORKGROUP_Y,
  vectorBuiltins_WORKGROUP_Z,
} from "./b13-vector-builtins.typegpu";

@CStruct
class VectorInput {
  floatA: Vec4f;
  floatB: Vec4f;
  signedA: Vec4i;
  signedB: Vec4i;
  unsignedA: Vec4u;
  unsignedB: Vec4u;

  constructor(floatA: Vec4f, floatB: Vec4f, signedA: Vec4i, signedB: Vec4i, unsignedA: Vec4u, unsignedB: Vec4u) {
    this.floatA = floatA;
    this.floatB = floatB;
    this.signedA = signedA;
    this.signedB = signedB;
    this.unsignedA = unsignedA;
    this.unsignedB = unsignedB;
  }
}

@CStruct
class VectorOutput {
  exactFloat: Vec4f;
  transFloat: Vec4f;
  signedValue: Vec4i;
  unsignedValue: Vec4u;
  comparisonBits: Vec4u;
  selectedValue: Vec4f;
  swizzleFloat: Vec4f;
  swizzleSigned: Vec4i;
  swizzleUnsigned: Vec4u;
  factoryFloat: Vec4f;
  factorySigned: Vec4i;
  factoryUnsigned: Vec4u;

  constructor() {
    this.exactFloat = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.transFloat = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.signedValue = new Vec4i(0, 0, 0, 0);
    this.unsignedValue = new Vec4u(0, 0, 0, 0);
    this.comparisonBits = new Vec4u(0, 0, 0, 0);
    this.selectedValue = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.swizzleFloat = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.swizzleSigned = new Vec4i(0, 0, 0, 0);
    this.swizzleUnsigned = new Vec4u(0, 0, 0, 0);
    this.factoryFloat = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.factorySigned = new Vec4i(0, 0, 0, 0);
    this.factoryUnsigned = new Vec4u(0, 0, 0, 0);
  }
}

class VectorLayout {
  input!: Storage<VectorInput>;
  output!: MutStorage<VectorOutput>;
}

function vectorKernel(res: VectorLayout, ctx: ComputeInvocation): void {
  const input: VectorInput = res.input.get(0);
  const a: Vec4f = input.floatA;
  const b: Vec4f = input.floatB;
  const low: Vec4f = new Vec4f(0.1, 0.2, 0.3, 0.4);
  const high: Vec4f = new Vec4f(1.0, 1.1, 1.2, 1.3);
  let output: VectorOutput = res.output.get(0);

  output.exactFloat = a.abs()
    .add(a.floor())
    .add(a.ceil())
    .add(a.fract())
    .add(a.sign())
    .add(a.min(b))
    .add(a.max(b))
    .add(a.clamp(low, high))
    .add(a.step(b));
  output.transFloat = a.sqrt()
    .add(a.exp())
    .add(a.log())
    .add(a.sin())
    .add(a.cos())
    .add(a.tan())
    .add(a.pow(b))
    .add(a.mix(b, 0.25))
    .add(a.smoothstep(low, high))
    .add(new Vec4f(a.distance(b), a.distance(b), a.distance(b), a.distance(b)))
    .add(a.reflect(b))
    .add(a.refract(b, 0.5))
    .add(a.faceForward(b, low));

  const signedMask = input.signedA.lt(input.signedB).not();
  output.signedValue = input.signedA.abs()
    .add(input.signedA.min(input.signedB))
    .add(input.signedA.max(input.signedB))
    .add(input.signedA.clamp(new Vec4i(-3, -2, -1, 0), new Vec4i(4, 5, 6, 7)))
    .add(input.signedA.select(input.signedB, signedMask));
  const unsignedMask = input.unsignedA.ge(input.unsignedB);
  output.unsignedValue = input.unsignedA.min(input.unsignedB)
    .add(input.unsignedA.max(input.unsignedB))
    .add(input.unsignedA.clamp(new Vec4u(1, 2, 3, 4), new Vec4u(8, 9, 10, 11)))
    .add(input.unsignedA.select(input.unsignedB, unsignedMask));

  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  output.comparisonBits = new Vec4u(
    lt.any() ? 1 : 0,
    le.all() ? 1 : 0,
    gt.any() || ge.all() ? 1 : 0,
    eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0,
  );
  output.selectedValue = a.select(b, inverted);

  const f3 = new Vec3f(a.x, a.y, a.z);
  output.swizzleFloat = new Vec4f(
    f3.xy().x + f3.xz().y + f3.yz().x,
    a.xy().x + a.xz().y + a.xw().y,
    a.yz().x + a.yw().y + a.zw().x,
    a.xyz().x + a.xyw().z + a.xzw().y + a.yzw().z,
  );
  const i3 = new Vec3i(input.signedA.x, input.signedA.y, input.signedA.z);
  output.swizzleSigned = new Vec4i(
    i3.xy().x + i3.xz().y + i3.yz().x,
    input.signedA.xy().x + input.signedA.xz().y + input.signedA.xw().y,
    input.signedA.yz().x + input.signedA.yw().y + input.signedA.zw().x,
    input.signedA.xyz().x + input.signedA.xyw().z + input.signedA.xzw().y + input.signedA.yzw().z,
  );
  const u3 = new Vec3u(input.unsignedA.x, input.unsignedA.y, input.unsignedA.z);
  output.swizzleUnsigned = new Vec4u(
    u3.xy().x + u3.xz().y + u3.yz().x,
    input.unsignedA.xy().x + input.unsignedA.xz().y + input.unsignedA.xw().y,
    input.unsignedA.yz().x + input.unsignedA.yw().y + input.unsignedA.zw().x,
    input.unsignedA.xyz().x + input.unsignedA.xyw().z + input.unsignedA.xzw().y + input.unsignedA.yzw().z,
  );

  const f2: Vec2f = v2fSplat(2.0);
  const f3From: Vec3f = v3fFrom2(f2, 3.0);
  output.factoryFloat = v4fFrom2(f2, 3.0, 4.0)
    .add(v4fFrom3(f3From, 4.0))
    .add(new Vec4f(v3fSplat(1.0).x, v3fSplat(1.0).y, v3fSplat(1.0).z, v4fSplat(1.0).w));
  const i2: Vec2i = v2iSplat(2);
  const i3From: Vec3i = v3iFrom2(i2, 3);
  output.factorySigned = v4iFrom2(i2, 3, 4)
    .add(v4iFrom3(i3From, 4))
    .add(new Vec4i(v3iSplat(1).x, v3iSplat(1).y, v3iSplat(1).z, v4iSplat(1).w));
  const u2: Vec2u = v2uSplat(2);
  const u3From: Vec3u = v3uFrom2(u2, 3);
  output.factoryUnsigned = v4uFrom2(u2, 3, 4)
    .add(v4uFrom3(u3From, 4))
    .add(new Vec4u(v3uSplat(1).x, v3uSplat(1).y, v3uSplat(1).z, v4uSplat(1).w));
  res.output.set(0, output);
}

export const vectorBuiltins: ComputePipelineSpec = computePipeline<VectorLayout>(
  vectorKernel,
  { name: "vectorBuiltins", workgroupSize: [1, 1, 1] },
);

function inputValue(): VectorInput {
  return new VectorInput(
    new Vec4f(0.25, 0.5, 0.75, 1.0),
    new Vec4f(1.0, 1.5, 2.0, 2.5),
    new Vec4i(-4, -2, 3, 6),
    new Vec4i(1, -3, 5, 2),
    new Vec4u(2, 4, 6, 8),
    new Vec4u(3, 1, 7, 5),
  );
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using input = device.createBuffer({ label: "b13-input", size: VectorInput_SIZE as u64, usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST });
    using output = device.createBuffer({ label: "b13-output", size: VectorOutput_SIZE as u64, usage: GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST });
    const source = inputValue();
    device.queue().writeBuffer(input, 0, Context.bytesOf<VectorInput>(source));
    device.queue().writeBuffer(output, 0, Context.bytesOf<VectorOutput>(new VectorOutput()));
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device,
      vectorBuiltins_WGSL,
      vectorBuiltins_ENTRY,
      [vectorBuiltins_LAYOUT0],
      [vectorBuiltins_WORKGROUP_X, vectorBuiltins_WORKGROUP_Y, vectorBuiltins_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) { print("pipeline:invalid"); print("FAIL"); return; }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, vectorBuiltins_LAYOUT0, [bufferResource(input), bufferResource(output)]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], 1, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);

    const hostLayout = new VectorLayout();
    hostLayout.input = new Storage<VectorInput>([source]);
    hostLayout.output = new MutStorage<VectorOutput>([new VectorOutput()]);
    simulateCompute<VectorLayout>(vectorKernel, hostLayout, vectorBuiltins, [1, 1, 1], vectorBuiltins_HOST_RUNNABLE);
    const host = hostLayout.output.get(0);
    print(`host:k25-float=${host.exactFloat.x},${host.exactFloat.y},${host.exactFloat.z},${host.exactFloat.w};${host.transFloat.x},${host.transFloat.y},${host.transFloat.z},${host.transFloat.w}`);
    print(`host:k25-integer=${host.signedValue.x},${host.signedValue.y},${host.signedValue.z},${host.signedValue.w};${host.unsignedValue.x},${host.unsignedValue.y},${host.unsignedValue.z},${host.unsignedValue.w}`);
    print(`host:k26=${host.comparisonBits.x},${host.comparisonBits.y},${host.comparisonBits.z},${host.comparisonBits.w};${host.selectedValue.x},${host.selectedValue.y},${host.selectedValue.z},${host.selectedValue.w}`);
    print(`host:k27-swizzles=${host.swizzleFloat.x},${host.swizzleFloat.y},${host.swizzleFloat.z},${host.swizzleFloat.w};${host.swizzleSigned.x},${host.swizzleSigned.y},${host.swizzleSigned.z},${host.swizzleSigned.w};${host.swizzleUnsigned.x},${host.swizzleUnsigned.y},${host.swizzleUnsigned.z},${host.swizzleUnsigned.w}`);
    print(`host:k27-factories=${host.factoryFloat.x},${host.factoryFloat.y},${host.factoryFloat.z},${host.factoryFloat.w};${host.factorySigned.x},${host.factorySigned.y},${host.factorySigned.z},${host.factorySigned.w};${host.factoryUnsigned.x},${host.factoryUnsigned.y},${host.factoryUnsigned.z},${host.factoryUnsigned.w}`);
    print(`vectorBuiltins_WGSL_LINES=${vectorBuiltins_WGSL.split("\n").length}`);
  }
  gpu.dispose();
  print("PASS");
}
