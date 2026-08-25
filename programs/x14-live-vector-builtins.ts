// program: x14-live-vector-builtins
// purpose: compare the GPU vector surface with simulateCompute through an owned staging buffer
// exercises: BF9, BF11, CL1, CL2, K10, K25, K26, K27, PI14
// questions: none
// tolerance: sqrt, exp, log, sin, cos, tan, pow, mix, smoothstep, distance, reflect, refract, and faceForward use 2^-16 relative tolerance

import {
  Buffer,
  bufferResource,
  ComputeInvocation,
  computePipeline,
  ComputePipelineSpec,
  createBindGroup,
  createBuffer,
  createComputePipeline,
  MutStorage,
  simulateComputeThreads,
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
  vec2fSplat,
  vec2iSplat,
  vec2uSplat,
  vec3fFrom2,
  vec3fSplat,
  vec3iFrom2,
  vec3iSplat,
  vec3uFrom2,
  vec3uSplat,
  vec4fFrom2,
  vec4fFrom3,
  vec4fSplat,
  vec4iFrom2,
  vec4iFrom3,
  vec4iSplat,
  vec4uFrom2,
  vec4uFrom3,
  vec4uSplat,
} from "./typegpu-types";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice } from "./webgpu";
import {
  VectorInput_STRIDE,
  VectorOutput_STRIDE,
  vectorBuiltins_ENTRY,
  vectorBuiltins_HOST_RUNNABLE,
  vectorBuiltins_LAYOUT0,
  vectorBuiltins_WGSL,
  vectorBuiltins_WORKGROUP_X,
  vectorBuiltins_WORKGROUP_Y,
  vectorBuiltins_WORKGROUP_Z,
} from "./x14-live-vector-builtins.typegpu";

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
  width2Float: Vec2f;
  width2Signed: Vec2i;
  width2Unsigned: Vec2u;
  width3Float: Vec3f;
  width3Signed: Vec3i;
  width3Unsigned: Vec3u;
  orderStep: Vec4f;
  orderSmoothstep: Vec4f;
  orderMix: Vec4f;
  orderClamp: Vec4f;
  orderRefract: Vec4f;
  orderFaceForward: Vec4f;
  orderSelect: Vec4f;

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
    this.width2Float = new Vec2f(0.0, 0.0);
    this.width2Signed = new Vec2i(0, 0);
    this.width2Unsigned = new Vec2u(0, 0);
    this.width3Float = new Vec3f(0.0, 0.0, 0.0);
    this.width3Signed = new Vec3i(0, 0, 0);
    this.width3Unsigned = new Vec3u(0, 0, 0);
    this.orderStep = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.orderSmoothstep = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.orderMix = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.orderClamp = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.orderRefract = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.orderFaceForward = new Vec4f(0.0, 0.0, 0.0, 0.0);
    this.orderSelect = new Vec4f(0.0, 0.0, 0.0, 0.0);
  }
}

class VectorLayout {
  input!: Storage<VectorInput>;
  output!: MutStorage<VectorOutput>;
}

function coverFloat2(a: Vec2f, b: Vec2f, low: Vec2f, high: Vec2f): Vec2f {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: f32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1.0 : 0.0;
  const distanceValue: f32 = a.distance(b);
  return a.abs().add(a.floor()).add(a.ceil()).add(a.fract()).add(a.sqrt())
    .add(a.exp()).add(a.log()).add(a.sin()).add(a.cos()).add(a.tan()).add(a.sign())
    .add(a.min(b)).add(a.max(b)).add(a.clamp(low, high)).add(a.pow(b))
    .add(a.mix(b, 0.25)).add(a.step(b)).add(a.smoothstep(low, high))
    .add(new Vec2f(distanceValue, distanceValue)).add(a.reflect(b))
    .add(a.refract(b, 0.5)).add(a.faceForward(b, low))
    .add(a.select(b, inverted)).add(new Vec2f(marker, marker));
}

function coverFloat3(a: Vec3f, b: Vec3f, low: Vec3f, high: Vec3f): Vec3f {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: f32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1.0 : 0.0;
  const distanceValue: f32 = a.distance(b);
  return a.abs().add(a.floor()).add(a.ceil()).add(a.fract()).add(a.sqrt())
    .add(a.exp()).add(a.log()).add(a.sin()).add(a.cos()).add(a.tan()).add(a.sign())
    .add(a.min(b)).add(a.max(b)).add(a.clamp(low, high)).add(a.pow(b))
    .add(a.mix(b, 0.25)).add(a.step(b)).add(a.smoothstep(low, high))
    .add(new Vec3f(distanceValue, distanceValue, distanceValue)).add(a.reflect(b))
    .add(a.refract(b, 0.5)).add(a.faceForward(b, low))
    .add(a.select(b, inverted)).add(new Vec3f(marker, marker, marker));
}

function coverSigned2(a: Vec2i, b: Vec2i): Vec2i {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: i32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0;
  return a.abs().add(a.min(b)).add(a.max(b))
    .add(a.clamp(new Vec2i(-3, -2), new Vec2i(4, 5)))
    .add(a.select(b, inverted)).add(new Vec2i(marker, marker));
}

function coverSigned3(a: Vec3i, b: Vec3i): Vec3i {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: i32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0;
  return a.abs().add(a.min(b)).add(a.max(b))
    .add(a.clamp(new Vec3i(-3, -2, -1), new Vec3i(4, 5, 6)))
    .add(a.select(b, inverted)).add(new Vec3i(marker, marker, marker));
}

function coverSigned4(a: Vec4i, b: Vec4i): Vec4i {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: i32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0;
  return a.abs().add(a.min(b)).add(a.max(b))
    .add(a.clamp(new Vec4i(-3, -2, -1, 0), new Vec4i(4, 5, 6, 7)))
    .add(a.select(b, inverted)).add(new Vec4i(marker, marker, marker, marker));
}

function coverUnsigned2(a: Vec2u, b: Vec2u): Vec2u {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: u32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0;
  return a.min(b).add(a.max(b)).add(a.clamp(new Vec2u(1, 2), new Vec2u(8, 9)))
    .add(a.select(b, inverted)).add(new Vec2u(marker, marker));
}

function coverUnsigned3(a: Vec3u, b: Vec3u): Vec3u {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: u32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0;
  return a.min(b).add(a.max(b)).add(a.clamp(new Vec3u(1, 2, 3), new Vec3u(8, 9, 10)))
    .add(a.select(b, inverted)).add(new Vec3u(marker, marker, marker));
}

function coverUnsigned4(a: Vec4u, b: Vec4u): Vec4u {
  const lt = a.lt(b);
  const le = a.le(b);
  const gt = a.gt(b);
  const ge = a.ge(b);
  const eq = a.eq(b);
  const ne = a.ne(b);
  const inverted = lt.not();
  const marker: u32 = lt.any() || le.all() || gt.any() || ge.all()
    || eq.any() || ne.all() || inverted.any() || inverted.all() ? 1 : 0;
  return a.min(b).add(a.max(b))
    .add(a.clamp(new Vec4u(1, 2, 3, 4), new Vec4u(8, 9, 10, 11)))
    .add(a.select(b, inverted)).add(new Vec4u(marker, marker, marker, marker));
}


function vectorKernel(res: VectorLayout, ctx: ComputeInvocation): void {
  const input: VectorInput = res.input[0];
  const a: Vec4f = input.floatA;
  const b: Vec4f = input.floatB;
  const low: Vec4f = new Vec4f(0.1, 0.2, 0.3, 0.4);
  const high: Vec4f = new Vec4f(1.0, 1.1, 1.2, 1.3);
  let output: VectorOutput = res.output[0];
  output.exactFloat = a.abs().add(a.floor()).add(a.ceil()).add(a.fract()).add(a.sign())
    .add(a.min(b)).add(a.max(b)).add(a.clamp(low, high)).add(a.step(b));
  output.transFloat = a.sqrt().add(a.exp()).add(a.log()).add(a.sin()).add(a.cos()).add(a.tan())
    .add(a.pow(b)).add(a.mix(b, 0.25)).add(a.smoothstep(low, high))
    .add(new Vec4f(a.distance(b), a.distance(b), a.distance(b), a.distance(b)))
    .add(a.reflect(b)).add(a.refract(b, 0.5)).add(a.faceForward(b, low));
  output.signedValue = coverSigned4(input.signedA, input.signedB);
  output.unsignedValue = coverUnsigned4(input.unsignedA, input.unsignedB);
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
  output.width2Float = coverFloat2(
    new Vec2f(a.x, a.y), new Vec2f(b.x, b.y),
    new Vec2f(low.x, low.y), new Vec2f(high.x, high.y),
  );
  output.width3Float = coverFloat3(
    new Vec3f(a.x, a.y, a.z), new Vec3f(b.x, b.y, b.z),
    new Vec3f(low.x, low.y, low.z), new Vec3f(high.x, high.y, high.z),
  );
  output.width2Signed = coverSigned2(
    new Vec2i(input.signedA.x, input.signedA.y), new Vec2i(input.signedB.x, input.signedB.y),
  );
  output.width3Signed = coverSigned3(
    new Vec3i(input.signedA.x, input.signedA.y, input.signedA.z),
    new Vec3i(input.signedB.x, input.signedB.y, input.signedB.z),
  );
  output.width2Unsigned = coverUnsigned2(
    new Vec2u(input.unsignedA.x, input.unsignedA.y), new Vec2u(input.unsignedB.x, input.unsignedB.y),
  );
  output.width3Unsigned = coverUnsigned3(
    new Vec3u(input.unsignedA.x, input.unsignedA.y, input.unsignedA.z),
    new Vec3u(input.unsignedB.x, input.unsignedB.y, input.unsignedB.z),
  );
  output.orderStep = new Vec4f(-1.0, 0.25, 0.75, 2.0).step(new Vec4f(0.0, 0.5, 1.0, 3.0));
  output.orderSmoothstep = new Vec4f(0.25, 0.5, 0.75, 1.0)
    .smoothstep(new Vec4f(0.0, 0.0, 0.0, 0.0), new Vec4f(1.0, 1.0, 1.0, 1.0));
  output.orderMix = new Vec4f(1.0, 2.0, 3.0, 4.0).mix(new Vec4f(5.0, 6.0, 7.0, 8.0), 0.25);
  output.orderClamp = new Vec4f(-2.0, 0.5, 2.0, 8.0)
    .clamp(new Vec4f(0.0, 1.0, 2.0, 3.0), new Vec4f(1.0, 2.0, 3.0, 4.0));
  output.orderRefract = new Vec4f(1.0, 0.0, 0.0, 0.0)
    .refract(new Vec4f(0.0, 1.0, 0.0, 0.0), 0.5);
  output.orderFaceForward = new Vec4f(1.0, 2.0, 3.0, 4.0)
    .faceForward(new Vec4f(1.0, 0.0, 0.0, 0.0), new Vec4f(-1.0, 0.0, 0.0, 0.0));
  const orderSelectBase = new Vec4f(1.0, 7.0, 3.0, 9.0);
  const orderSelectOther = new Vec4f(0.0, 8.0, 2.0, 10.0);
  output.orderSelect = orderSelectBase.select(orderSelectOther, orderSelectBase.lt(orderSelectOther));
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
  const f2: Vec2f = vec2fSplat(2.0);
  const f3From: Vec3f = vec3fFrom2(f2, 3.0);
  output.factoryFloat = vec4fFrom2(f2, 3.0, 4.0).add(vec4fFrom3(f3From, 4.0))
    .add(new Vec4f(vec3fSplat(1.0).x, vec3fSplat(1.0).y, vec3fSplat(1.0).z, vec4fSplat(1.0).w));
  const i2: Vec2i = vec2iSplat(2);
  const i3From: Vec3i = vec3iFrom2(i2, 3);
  output.factorySigned = vec4iFrom2(i2, 3, 4).add(vec4iFrom3(i3From, 4))
    .add(new Vec4i(vec3iSplat(1).x, vec3iSplat(1).y, vec3iSplat(1).z, vec4iSplat(1).w));
  const u2: Vec2u = vec2uSplat(2);
  const u3From: Vec3u = vec3uFrom2(u2, 3);
  output.factoryUnsigned = vec4uFrom2(u2, 3, 4).add(vec4uFrom3(u3From, 4))
    .add(new Vec4u(vec3uSplat(1).x, vec3uSplat(1).y, vec3uSplat(1).z, vec4uSplat(1).w));
  res.output[0] = output;
}

export const vectorBuiltins: ComputePipelineSpec = computePipeline<VectorLayout>(
  vectorKernel,
  { name: "vectorBuiltins", workgroupSize: [1, 1, 1] },
);

function inputValue(): VectorInput {
  return new VectorInput(
    new Vec4f(0.25, 0.5, 0.75, 1.0), new Vec4f(1.0, 1.5, 2.0, 2.5),
    new Vec4i(-4, -2, 3, 6), new Vec4i(1, -3, 5, 2),
    new Vec4u(2, 4, 6, 8), new Vec4u(3, 1, 7, 5),
  );
}

function exactF32(left: f32, right: f32): boolean {
  return Math.f32ToBits(left as f64) === Math.f32ToBits(right as f64);
}

function exactVec4f(left: Vec4f, right: Vec4f): boolean {
  return exactF32(left.x, right.x) && exactF32(left.y, right.y)
    && exactF32(left.z, right.z) && exactF32(left.w, right.w);
}

function exactVec4i(left: Vec4i, right: Vec4i): boolean {
  return left.x === right.x && left.y === right.y && left.z === right.z && left.w === right.w;
}

function exactVec2i(left: Vec2i, right: Vec2i): boolean {
  return left.x === right.x && left.y === right.y;
}

function exactVec3i(left: Vec3i, right: Vec3i): boolean {
  return left.x === right.x && left.y === right.y && left.z === right.z;
}

function exactVec4u(left: Vec4u, right: Vec4u): boolean {
  return left.x === right.x && left.y === right.y && left.z === right.z && left.w === right.w;
}

function exactVec2u(left: Vec2u, right: Vec2u): boolean {
  return left.x === right.x && left.y === right.y;
}

function exactVec3u(left: Vec3u, right: Vec3u): boolean {
  return left.x === right.x && left.y === right.y && left.z === right.z;
}

function near(left: f32, right: f32): boolean {
  const difference: f64 = Math.abs((left as f64) - (right as f64));
  const scale: f64 = Math.max(1.0, Math.abs(right as f64));
  return difference <= scale / 65536.0;
}

function nearVec4f(left: Vec4f, right: Vec4f): boolean {
  return near(left.x, right.x) && near(left.y, right.y) && near(left.z, right.z) && near(left.w, right.w);
}

function nearVec2f(left: Vec2f, right: Vec2f): boolean {
  return near(left.x, right.x) && near(left.y, right.y);
}

function nearVec3f(left: Vec3f, right: Vec3f): boolean {
  return near(left.x, right.x) && near(left.y, right.y) && near(left.z, right.z);
}

function exactOutput(left: VectorOutput, right: VectorOutput): boolean {
  return exactVec4f(left.exactFloat, right.exactFloat)
    && exactVec4i(left.signedValue, right.signedValue)
    && exactVec4u(left.unsignedValue, right.unsignedValue)
    && exactVec4u(left.comparisonBits, right.comparisonBits)
    && exactVec4f(left.selectedValue, right.selectedValue)
    && exactVec4f(left.swizzleFloat, right.swizzleFloat)
    && exactVec4i(left.swizzleSigned, right.swizzleSigned)
    && exactVec4u(left.swizzleUnsigned, right.swizzleUnsigned)
    && exactVec4f(left.factoryFloat, right.factoryFloat)
    && exactVec4i(left.factorySigned, right.factorySigned)
    && exactVec4u(left.factoryUnsigned, right.factoryUnsigned)
    && exactVec2i(left.width2Signed, right.width2Signed)
    && exactVec2u(left.width2Unsigned, right.width2Unsigned)
    && exactVec3i(left.width3Signed, right.width3Signed)
    && exactVec3u(left.width3Unsigned, right.width3Unsigned)
    && exactVec4f(left.orderStep, right.orderStep)
    && exactVec4f(left.orderClamp, right.orderClamp)
    && exactVec4f(left.orderSelect, right.orderSelect);
}

function nearOutput(left: VectorOutput, right: VectorOutput): boolean {
  return nearVec4f(left.transFloat, right.transFloat)
    && nearVec2f(left.width2Float, right.width2Float)
    && nearVec3f(left.width3Float, right.width3Float)
    && nearVec4f(left.orderSmoothstep, right.orderSmoothstep)
    && nearVec4f(left.orderMix, right.orderMix)
    && nearVec4f(left.orderRefract, right.orderRefract)
    && nearVec4f(left.orderFaceForward, right.orderFaceForward);
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using input: Buffer<VectorInput> = createBuffer<VectorInput>(device, VectorInput_STRIDE, 1, GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST, "x14-input");
    using output: Buffer<VectorOutput> = createBuffer<VectorOutput>(device, VectorOutput_STRIDE, 1, GPUBufferUsage.STORAGE + GPUBufferUsage.COPY_DST + GPUBufferUsage.COPY_SRC, "x14-output");
    const source = inputValue();
    input.writeOne(device.queue(), 0, Context.bytesOf<VectorInput>(source));
    output.writeOne(device.queue(), 0, Context.bytesOf<VectorOutput>(new VectorOutput()));
    device.pushErrorScope("validation");
    using pipeline = createComputePipeline(
      device, vectorBuiltins_WGSL, vectorBuiltins_ENTRY, [vectorBuiltins_LAYOUT0],
      [vectorBuiltins_WORKGROUP_X, vectorBuiltins_WORKGROUP_Y, vectorBuiltins_WORKGROUP_Z],
    );
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print(`FAIL validation ${validationError.message.split("\n")[0]}`);
      return;
    }
    using nativeLayout = pipeline.bindGroupLayout(0);
    using bindGroup = createBindGroup(device, nativeLayout, vectorBuiltins_LAYOUT0, [bufferResource(input.handle()), bufferResource(output.handle())]);
    using encoder = device.createCommandEncoderDefault();
    pipeline.dispatchThreads(encoder, [bindGroup], 1, 1, 1);
    using command = encoder.finishDefault();
    device.queue().submit([command]);
    const gpuBytes: u8[] = await output.read(device, 0, 1);
    const gpuOutput: VectorOutput = Context.fromBytes<VectorOutput>(gpuBytes, 0);
    const hostLayout = new VectorLayout();
    hostLayout.input = new Storage<VectorInput>([source]);
    hostLayout.output = new MutStorage<VectorOutput>([new VectorOutput()]);
    simulateComputeThreads<VectorLayout>(vectorKernel, hostLayout, vectorBuiltins, 1, 1, 1, vectorBuiltins_HOST_RUNNABLE);
    const hostOutput = hostLayout.output[0];
    if (!exactOutput(gpuOutput, hostOutput)) {
      print("FAIL exact");
      return;
    }
    if (!nearOutput(gpuOutput, hostOutput)) {
      print("FAIL transcendental");
      return;
    }
  }
  gpu.dispose();
  print("PASS");
}
