struct VectorInput {
  floatA: vec4<f32>,
  floatB: vec4<f32>,
  signedA: vec4<i32>,
  signedB: vec4<i32>,
  unsignedA: vec4<u32>,
  unsignedB: vec4<u32>,
}

struct VectorOutput {
  exactFloat: vec4<f32>,
  transFloat: vec4<f32>,
  signedValue: vec4<i32>,
  unsignedValue: vec4<u32>,
  comparisonBits: vec4<u32>,
  selectedValue: vec4<f32>,
  swizzleFloat: vec4<f32>,
  swizzleSigned: vec4<i32>,
  swizzleUnsigned: vec4<u32>,
  factoryFloat: vec4<f32>,
  factorySigned: vec4<i32>,
  factoryUnsigned: vec4<u32>,
  width2Float: vec2<f32>,
  width2Signed: vec2<i32>,
  width2Unsigned: vec2<u32>,
  width3Float: vec3<f32>,
  width3Signed: vec3<i32>,
  width3Unsigned: vec3<u32>,
  orderStep: vec4<f32>,
  orderSmoothstep: vec4<f32>,
  orderMix: vec4<f32>,
  orderClamp: vec4<f32>,
  orderRefract: vec4<f32>,
  orderFaceForward: vec4<f32>,
  orderSelect: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> input: array<VectorInput>;
@group(0) @binding(1) var<storage, read_write> output: array<VectorOutput>;

fn coverFloat2(a: vec2<f32>, b: vec2<f32>, low: vec2<f32>, high: vec2<f32>) -> vec2<f32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: f32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1.0f;
  } else {
    _g_conditional_0 = 0.0f;
  }
  let marker = _g_conditional_0;
  let distanceValue = distance(a, b);
  return abs(a) + floor(a) + ceil(a) + fract(a) + sqrt(a) + exp(a) + log(a) + sin(a) + cos(a) + tan(a) + sign(a) + min(a, b) + max(a, b) + clamp(a, low, high) + pow(a, b) + mix(a, b, 0.25f) + step(b, a) + smoothstep(low, high, a) + vec2<f32>(distanceValue, distanceValue) + reflect(a, b) + refract(a, b, 0.5f) + faceForward(a, b, low) + select(a, b, inverted) + vec2<f32>(marker, marker);
}

fn coverFloat3(a: vec3<f32>, b: vec3<f32>, low: vec3<f32>, high: vec3<f32>) -> vec3<f32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: f32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1.0f;
  } else {
    _g_conditional_0 = 0.0f;
  }
  let marker = _g_conditional_0;
  let distanceValue = distance(a, b);
  return abs(a) + floor(a) + ceil(a) + fract(a) + sqrt(a) + exp(a) + log(a) + sin(a) + cos(a) + tan(a) + sign(a) + min(a, b) + max(a, b) + clamp(a, low, high) + pow(a, b) + mix(a, b, 0.25f) + step(b, a) + smoothstep(low, high, a) + vec3<f32>(distanceValue, distanceValue, distanceValue) + reflect(a, b) + refract(a, b, 0.5f) + faceForward(a, b, low) + select(a, b, inverted) + vec3<f32>(marker, marker, marker);
}

fn coverSigned2(a: vec2<i32>, b: vec2<i32>) -> vec2<i32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: i32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1i;
  } else {
    _g_conditional_0 = 0i;
  }
  let marker = _g_conditional_0;
  return abs(a) + min(a, b) + max(a, b) + clamp(a, vec2<i32>(-3i, -2i), vec2<i32>(4i, 5i)) + select(a, b, inverted) + vec2<i32>(marker, marker);
}

fn coverSigned3(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: i32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1i;
  } else {
    _g_conditional_0 = 0i;
  }
  let marker = _g_conditional_0;
  return abs(a) + min(a, b) + max(a, b) + clamp(a, vec3<i32>(-3i, -2i, -1i), vec3<i32>(4i, 5i, 6i)) + select(a, b, inverted) + vec3<i32>(marker, marker, marker);
}

fn coverSigned4(a: vec4<i32>, b: vec4<i32>) -> vec4<i32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: i32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1i;
  } else {
    _g_conditional_0 = 0i;
  }
  let marker = _g_conditional_0;
  return abs(a) + min(a, b) + max(a, b) + clamp(a, vec4<i32>(-3i, -2i, -1i, 0i), vec4<i32>(4i, 5i, 6i, 7i)) + select(a, b, inverted) + vec4<i32>(marker, marker, marker, marker);
}

fn coverUnsigned2(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: u32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1u;
  } else {
    _g_conditional_0 = 0u;
  }
  let marker = _g_conditional_0;
  return min(a, b) + max(a, b) + clamp(a, vec2<u32>(1u, 2u), vec2<u32>(8u, 9u)) + select(a, b, inverted) + vec2<u32>(marker, marker);
}

fn coverUnsigned3(a: vec3<u32>, b: vec3<u32>) -> vec3<u32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: u32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1u;
  } else {
    _g_conditional_0 = 0u;
  }
  let marker = _g_conditional_0;
  return min(a, b) + max(a, b) + clamp(a, vec3<u32>(1u, 2u, 3u), vec3<u32>(8u, 9u, 10u)) + select(a, b, inverted) + vec3<u32>(marker, marker, marker);
}

fn coverUnsigned4(a: vec4<u32>, b: vec4<u32>) -> vec4<u32> {
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: u32;
  if (any(lt) || all(le) || any(gt) || all(ge) || any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_0 = 1u;
  } else {
    _g_conditional_0 = 0u;
  }
  let marker = _g_conditional_0;
  return min(a, b) + max(a, b) + clamp(a, vec4<u32>(1u, 2u, 3u, 4u), vec4<u32>(8u, 9u, 10u, 11u)) + select(a, b, inverted) + vec4<u32>(marker, marker, marker, marker);
}

@compute @workgroup_size(1, 1, 1)
fn vectorKernel() {
  var input_ = input[0u];
  var a = input_.floatA;
  var b = input_.floatB;
  var low = vec4<f32>(0.1f, 0.2f, 0.3f, 0.4f);
  var high = vec4<f32>(1.0f, 1.1f, 1.2f, 1.3f);
  var output_ = output[0u];
  output_.exactFloat = abs(a) + floor(a) + ceil(a) + fract(a) + sign(a) + min(a, b) + max(a, b) + clamp(a, low, high) + step(b, a);
  output_.transFloat = sqrt(a) + exp(a) + log(a) + sin(a) + cos(a) + tan(a) + pow(a, b) + mix(a, b, 0.25f) + smoothstep(low, high, a) + vec4<f32>(distance(a, b), distance(a, b), distance(a, b), distance(a, b)) + reflect(a, b) + refract(a, b, 0.5f) + faceForward(a, b, low);
  output_.signedValue = coverSigned4(input_.signedA, input_.signedB);
  output_.unsignedValue = coverUnsigned4(input_.unsignedA, input_.unsignedB);
  var lt = a < b;
  var le = a <= b;
  var gt = a > b;
  var ge = a >= b;
  var eq = a == b;
  var ne = a != b;
  var inverted = !lt;
  var _g_conditional_0: u32;
  if (any(lt)) {
    _g_conditional_0 = 1u;
  } else {
    _g_conditional_0 = 0u;
  }
  var _g_conditional_1: u32;
  if (all(le)) {
    _g_conditional_1 = 1u;
  } else {
    _g_conditional_1 = 0u;
  }
  var _g_conditional_2: u32;
  if (any(gt) || all(ge)) {
    _g_conditional_2 = 1u;
  } else {
    _g_conditional_2 = 0u;
  }
  var _g_conditional_3: u32;
  if (any(eq) || all(ne) || any(inverted) || all(inverted)) {
    _g_conditional_3 = 1u;
  } else {
    _g_conditional_3 = 0u;
  }
  output_.comparisonBits = vec4<u32>(_g_conditional_0, _g_conditional_1, _g_conditional_2, _g_conditional_3);
  output_.selectedValue = select(a, b, inverted);
  output_.width2Float = coverFloat2(vec2<f32>(a.x, a.y), vec2<f32>(b.x, b.y), vec2<f32>(low.x, low.y), vec2<f32>(high.x, high.y));
  output_.width3Float = coverFloat3(vec3<f32>(a.x, a.y, a.z), vec3<f32>(b.x, b.y, b.z), vec3<f32>(low.x, low.y, low.z), vec3<f32>(high.x, high.y, high.z));
  output_.width2Signed = coverSigned2(vec2<i32>(input_.signedA.x, input_.signedA.y), vec2<i32>(input_.signedB.x, input_.signedB.y));
  output_.width3Signed = coverSigned3(vec3<i32>(input_.signedA.x, input_.signedA.y, input_.signedA.z), vec3<i32>(input_.signedB.x, input_.signedB.y, input_.signedB.z));
  output_.width2Unsigned = coverUnsigned2(vec2<u32>(input_.unsignedA.x, input_.unsignedA.y), vec2<u32>(input_.unsignedB.x, input_.unsignedB.y));
  output_.width3Unsigned = coverUnsigned3(vec3<u32>(input_.unsignedA.x, input_.unsignedA.y, input_.unsignedA.z), vec3<u32>(input_.unsignedB.x, input_.unsignedB.y, input_.unsignedB.z));
  output_.orderStep = step(vec4<f32>(0.0f, 0.5f, 1.0f, 3.0f), vec4<f32>(-1.0f, 0.25f, 0.75f, 2.0f));
  output_.orderSmoothstep = smoothstep(vec4<f32>(0.0f, 0.0f, 0.0f, 0.0f), vec4<f32>(1.0f, 1.0f, 1.0f, 1.0f), vec4<f32>(0.25f, 0.5f, 0.75f, 1.0f));
  output_.orderMix = mix(vec4<f32>(1.0f, 2.0f, 3.0f, 4.0f), vec4<f32>(5.0f, 6.0f, 7.0f, 8.0f), 0.25f);
  output_.orderClamp = clamp(vec4<f32>(-2.0f, 0.5f, 2.0f, 8.0f), vec4<f32>(0.0f, 1.0f, 2.0f, 3.0f), vec4<f32>(1.0f, 2.0f, 3.0f, 4.0f));
  output_.orderRefract = refract(vec4<f32>(1.0f, 0.0f, 0.0f, 0.0f), vec4<f32>(0.0f, 1.0f, 0.0f, 0.0f), 0.5f);
  output_.orderFaceForward = faceForward(vec4<f32>(1.0f, 2.0f, 3.0f, 4.0f), vec4<f32>(1.0f, 0.0f, 0.0f, 0.0f), vec4<f32>(-1.0f, 0.0f, 0.0f, 0.0f));
  var orderSelectBase = vec4<f32>(1.0f, 7.0f, 3.0f, 9.0f);
  var orderSelectOther = vec4<f32>(0.0f, 8.0f, 2.0f, 10.0f);
  output_.orderSelect = select(orderSelectBase, orderSelectOther, orderSelectBase < orderSelectOther);
  var f3 = vec3<f32>(a.x, a.y, a.z);
  output_.swizzleFloat = vec4<f32>(f3.xy.x + f3.xz.y + f3.yz.x, a.xy.x + a.xz.y + a.xw.y, a.yz.x + a.yw.y + a.zw.x, a.xyz.x + a.xyw.z + a.xzw.y + a.yzw.z);
  var i3 = vec3<i32>(input_.signedA.x, input_.signedA.y, input_.signedA.z);
  output_.swizzleSigned = vec4<i32>(i3.xy.x + i3.xz.y + i3.yz.x, input_.signedA.xy.x + input_.signedA.xz.y + input_.signedA.xw.y, input_.signedA.yz.x + input_.signedA.yw.y + input_.signedA.zw.x, input_.signedA.xyz.x + input_.signedA.xyw.z + input_.signedA.xzw.y + input_.signedA.yzw.z);
  var u3 = vec3<u32>(input_.unsignedA.x, input_.unsignedA.y, input_.unsignedA.z);
  output_.swizzleUnsigned = vec4<u32>(u3.xy.x + u3.xz.y + u3.yz.x, input_.unsignedA.xy.x + input_.unsignedA.xz.y + input_.unsignedA.xw.y, input_.unsignedA.yz.x + input_.unsignedA.yw.y + input_.unsignedA.zw.x, input_.unsignedA.xyz.x + input_.unsignedA.xyw.z + input_.unsignedA.xzw.y + input_.unsignedA.yzw.z);
  var f2 = vec2<f32>(2.0f);
  var f3From = vec3<f32>(f2, 3.0f);
  output_.factoryFloat = vec4<f32>(f2, 3.0f, 4.0f) + vec4<f32>(f3From, 4.0f) + vec4<f32>(vec3<f32>(1.0f).x, vec3<f32>(1.0f).y, vec3<f32>(1.0f).z, vec4<f32>(1.0f).w);
  var i2 = vec2<i32>(2i);
  var i3From = vec3<i32>(i2, 3i);
  output_.factorySigned = vec4<i32>(i2, 3i, 4i) + vec4<i32>(i3From, 4i) + vec4<i32>(vec3<i32>(1i).x, vec3<i32>(1i).y, vec3<i32>(1i).z, vec4<i32>(1i).w);
  var u2 = vec2<u32>(2u);
  var u3From = vec3<u32>(u2, 3u);
  output_.factoryUnsigned = vec4<u32>(u2, 3u, 4u) + vec4<u32>(u3From, 4u) + vec4<u32>(vec3<u32>(1u).x, vec3<u32>(1u).y, vec3<u32>(1u).z, vec4<u32>(1u).w);
  output[0u] = output_;
}
