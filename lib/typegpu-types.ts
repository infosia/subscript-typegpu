// A two-component `f32` vector, WGSL `vec2<f32>`. A value class copies on assignment, on an
// argument pass, and on a return. A kernel never runs the host bodies below: the generator
// maps each method to a WGSL operator or builtin.
@CStruct({ align: 8 })
export class Vec2f {
  x: f32;
  y: f32;

  constructor(x: f32, y: f32) {
    this.x = x;
    this.y = y;
  }

  add(other: Vec2f): Vec2f {
    return new Vec2f(this.x + other.x, this.y + other.y);
  }

  sub(other: Vec2f): Vec2f {
    return new Vec2f(this.x - other.x, this.y - other.y);
  }

  // The componentwise product, WGSL `a * b`. `scale` is the scalar form, because subscript has
  // no overloads.
  mul(other: Vec2f): Vec2f {
    return new Vec2f(this.x * other.x, this.y * other.y);
  }

  // The scalar multiple, WGSL `v * s`. WGSL spells `mul` and `scale` alike, and the argument
  // type separates them here.
  scale(s: f32): Vec2f {
    return new Vec2f(this.x * s, this.y * s);
  }

  dot(other: Vec2f): f32 {
    return this.x * other.x + this.y * other.y;
  }

  length(): f32 {
    return Math.sqrt(this.dot(this) as f64) as f32;
  }

  // Returns the zero vector when the length is zero. WGSL `normalize` divides by the length and
  // carries no such guard, so a zero input makes the CPU lane and the GPU disagree.
  normalize(): Vec2f {
    const magnitude: f32 = this.length();
    if (magnitude === 0.0) {
      return new Vec2f(0.0, 0.0);
    }
    return this.scale(1.0 / magnitude);
  }

  // The componentwise builtins carry the WGSL name of the same spelling, and the receiver is the
  // builtin's value argument. `step` and `smoothstep` take their edges first, as WGSL does.
  // Every other method puts the receiver first.
  abs(): Vec2f { return new Vec2f(Math.abs(this.x as f64) as f32, Math.abs(this.y as f64) as f32); }
  floor(): Vec2f { return new Vec2f(Math.floor(this.x as f64) as f32, Math.floor(this.y as f64) as f32); }
  ceil(): Vec2f { return new Vec2f(Math.ceil(this.x as f64) as f32, Math.ceil(this.y as f64) as f32); }
  fract(): Vec2f { return new Vec2f(fract(this.x), fract(this.y)); }
  sqrt(): Vec2f { return new Vec2f(Math.sqrt(this.x as f64) as f32, Math.sqrt(this.y as f64) as f32); }
  exp(): Vec2f { return new Vec2f(Math.exp(this.x as f64) as f32, Math.exp(this.y as f64) as f32); }
  log(): Vec2f { return new Vec2f(Math.log(this.x as f64) as f32, Math.log(this.y as f64) as f32); }
  sin(): Vec2f { return new Vec2f(Math.sin(this.x as f64) as f32, Math.sin(this.y as f64) as f32); }
  cos(): Vec2f { return new Vec2f(Math.cos(this.x as f64) as f32, Math.cos(this.y as f64) as f32); }
  tan(): Vec2f { return new Vec2f(Math.tan(this.x as f64) as f32, Math.tan(this.y as f64) as f32); }
  sign(): Vec2f { return new Vec2f(sign(this.x), sign(this.y)); }
  min(other: Vec2f): Vec2f { return new Vec2f(Math.min(this.x as f64, other.x as f64) as f32, Math.min(this.y as f64, other.y as f64) as f32); }
  max(other: Vec2f): Vec2f { return new Vec2f(Math.max(this.x as f64, other.x as f64) as f32, Math.max(this.y as f64, other.y as f64) as f32); }
  clamp(low: Vec2f, high: Vec2f): Vec2f { return new Vec2f(clamp(this.x, low.x, high.x), clamp(this.y, low.y, high.y)); }
  pow(other: Vec2f): Vec2f { return new Vec2f(Math.pow(this.x as f64, other.x as f64) as f32, Math.pow(this.y as f64, other.y as f64) as f32); }
  mix(other: Vec2f, amount: f32): Vec2f { return new Vec2f(mix(this.x, other.x, amount), mix(this.y, other.y, amount)); }
  step(edge: Vec2f): Vec2f { return new Vec2f(step(edge.x, this.x), step(edge.y, this.y)); }
  smoothstep(low: Vec2f, high: Vec2f): Vec2f { return new Vec2f(smoothstep(low.x, high.x, this.x), smoothstep(low.y, high.y, this.y)); }
  distance(other: Vec2f): f32 { return this.sub(other).length(); }
  // WGSL `reflect(v, normal)`. `normal` must be a unit vector, or the result is not a reflection.
  reflect(normal: Vec2f): Vec2f { return this.sub(normal.scale(2.0 * this.dot(normal))); }
  // WGSL `refract(v, normal, eta)`. `eta` is the source refractive index divided by the
  // destination index, and `normal` must be a unit vector.
  refract(normal: Vec2f, eta: f32): Vec2f {
    const product: f32 = this.dot(normal);
    const factor: f32 = 1.0 - eta * eta * (1.0 - product * product);
    if (factor < 0.0) return new Vec2f(0.0, 0.0);
    return this.scale(eta).sub(normal.scale(eta * product + (Math.sqrt(factor as f64) as f32)));
  }
  faceForward(incident: Vec2f, reference: Vec2f): Vec2f { return incident.dot(reference) < 0.0 ? this : this.scale(-1.0); }
  // The six comparisons return a `Vec2b` mask, WGSL `a < b` and family. A mask reaches a value
  // again through `select`, `any`, or `all`.
  lt(other: Vec2f): Vec2b { return new Vec2b(this.x < other.x, this.y < other.y); }
  le(other: Vec2f): Vec2b { return new Vec2b(this.x <= other.x, this.y <= other.y); }
  gt(other: Vec2f): Vec2b { return new Vec2b(this.x > other.x, this.y > other.y); }
  ge(other: Vec2f): Vec2b { return new Vec2b(this.x >= other.x, this.y >= other.y); }
  eq(other: Vec2f): Vec2b { return new Vec2b(this.x === other.x, this.y === other.y); }
  ne(other: Vec2f): Vec2b { return new Vec2b(this.x !== other.x, this.y !== other.y); }
  // Takes `other` where the mask component is `true` and the receiver where it is `false`.
  // WGSL `select(v, other, mask)`.
  select(other: Vec2f, mask: Vec2b): Vec2f { return new Vec2f(mask.x ? other.x : this.x, mask.y ? other.y : this.y); }
}

// A three-component `f32` vector, WGSL `vec3<f32>`. The C size is 16 and the WGSL size is 12.
// A schema that puts a scalar field after a `Vec3f` field fails the layout check.
@CStruct({ align: 16 })
export class Vec3f {
  x: f32;
  y: f32;
  z: f32;

  constructor(x: f32, y: f32, z: f32) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  add(other: Vec3f): Vec3f {
    return new Vec3f(this.x + other.x, this.y + other.y, this.z + other.z);
  }

  sub(other: Vec3f): Vec3f {
    return new Vec3f(this.x - other.x, this.y - other.y, this.z - other.z);
  }

  mul(other: Vec3f): Vec3f {
    return new Vec3f(this.x * other.x, this.y * other.y, this.z * other.z);
  }

  scale(s: f32): Vec3f {
    return new Vec3f(this.x * s, this.y * s, this.z * s);
  }

  dot(other: Vec3f): f32 {
    return this.x * other.x + this.y * other.y + this.z * other.z;
  }

  // WGSL `cross(a, b)`. WGSL defines the cross product for three components only, so no other
  // vector class carries it.
  cross(other: Vec3f): Vec3f {
    return new Vec3f(
      this.y * other.z - this.z * other.y,
      this.z * other.x - this.x * other.z,
      this.x * other.y - this.y * other.x,
    );
  }

  length(): f32 {
    return Math.sqrt(this.dot(this) as f64) as f32;
  }

  normalize(): Vec3f {
    const magnitude: f32 = this.length();
    if (magnitude === 0.0) {
      return new Vec3f(0.0, 0.0, 0.0);
    }
    return this.scale(1.0 / magnitude);
  }

  abs(): Vec3f { return new Vec3f(Math.abs(this.x as f64) as f32, Math.abs(this.y as f64) as f32, Math.abs(this.z as f64) as f32); }
  floor(): Vec3f { return new Vec3f(Math.floor(this.x as f64) as f32, Math.floor(this.y as f64) as f32, Math.floor(this.z as f64) as f32); }
  ceil(): Vec3f { return new Vec3f(Math.ceil(this.x as f64) as f32, Math.ceil(this.y as f64) as f32, Math.ceil(this.z as f64) as f32); }
  fract(): Vec3f { return new Vec3f(fract(this.x), fract(this.y), fract(this.z)); }
  sqrt(): Vec3f { return new Vec3f(Math.sqrt(this.x as f64) as f32, Math.sqrt(this.y as f64) as f32, Math.sqrt(this.z as f64) as f32); }
  exp(): Vec3f { return new Vec3f(Math.exp(this.x as f64) as f32, Math.exp(this.y as f64) as f32, Math.exp(this.z as f64) as f32); }
  log(): Vec3f { return new Vec3f(Math.log(this.x as f64) as f32, Math.log(this.y as f64) as f32, Math.log(this.z as f64) as f32); }
  sin(): Vec3f { return new Vec3f(Math.sin(this.x as f64) as f32, Math.sin(this.y as f64) as f32, Math.sin(this.z as f64) as f32); }
  cos(): Vec3f { return new Vec3f(Math.cos(this.x as f64) as f32, Math.cos(this.y as f64) as f32, Math.cos(this.z as f64) as f32); }
  tan(): Vec3f { return new Vec3f(Math.tan(this.x as f64) as f32, Math.tan(this.y as f64) as f32, Math.tan(this.z as f64) as f32); }
  sign(): Vec3f { return new Vec3f(sign(this.x), sign(this.y), sign(this.z)); }
  min(other: Vec3f): Vec3f { return new Vec3f(Math.min(this.x as f64, other.x as f64) as f32, Math.min(this.y as f64, other.y as f64) as f32, Math.min(this.z as f64, other.z as f64) as f32); }
  max(other: Vec3f): Vec3f { return new Vec3f(Math.max(this.x as f64, other.x as f64) as f32, Math.max(this.y as f64, other.y as f64) as f32, Math.max(this.z as f64, other.z as f64) as f32); }
  clamp(low: Vec3f, high: Vec3f): Vec3f { return new Vec3f(clamp(this.x, low.x, high.x), clamp(this.y, low.y, high.y), clamp(this.z, low.z, high.z)); }
  pow(other: Vec3f): Vec3f { return new Vec3f(Math.pow(this.x as f64, other.x as f64) as f32, Math.pow(this.y as f64, other.y as f64) as f32, Math.pow(this.z as f64, other.z as f64) as f32); }
  mix(other: Vec3f, amount: f32): Vec3f { return new Vec3f(mix(this.x, other.x, amount), mix(this.y, other.y, amount), mix(this.z, other.z, amount)); }
  step(edge: Vec3f): Vec3f { return new Vec3f(step(edge.x, this.x), step(edge.y, this.y), step(edge.z, this.z)); }
  smoothstep(low: Vec3f, high: Vec3f): Vec3f { return new Vec3f(smoothstep(low.x, high.x, this.x), smoothstep(low.y, high.y, this.y), smoothstep(low.z, high.z, this.z)); }
  distance(other: Vec3f): f32 { return this.sub(other).length(); }
  reflect(normal: Vec3f): Vec3f { return this.sub(normal.scale(2.0 * this.dot(normal))); }
  refract(normal: Vec3f, eta: f32): Vec3f {
    const product: f32 = this.dot(normal);
    const factor: f32 = 1.0 - eta * eta * (1.0 - product * product);
    if (factor < 0.0) return new Vec3f(0.0, 0.0, 0.0);
    return this.scale(eta).sub(normal.scale(eta * product + (Math.sqrt(factor as f64) as f32)));
  }
  faceForward(incident: Vec3f, reference: Vec3f): Vec3f { return incident.dot(reference) < 0.0 ? this : this.scale(-1.0); }
  lt(other: Vec3f): Vec3b { return new Vec3b(this.x < other.x, this.y < other.y, this.z < other.z); }
  le(other: Vec3f): Vec3b { return new Vec3b(this.x <= other.x, this.y <= other.y, this.z <= other.z); }
  gt(other: Vec3f): Vec3b { return new Vec3b(this.x > other.x, this.y > other.y, this.z > other.z); }
  ge(other: Vec3f): Vec3b { return new Vec3b(this.x >= other.x, this.y >= other.y, this.z >= other.z); }
  eq(other: Vec3f): Vec3b { return new Vec3b(this.x === other.x, this.y === other.y, this.z === other.z); }
  ne(other: Vec3f): Vec3b { return new Vec3b(this.x !== other.x, this.y !== other.y, this.z !== other.z); }
  select(other: Vec3f, mask: Vec3b): Vec3f { return new Vec3f(mask.x ? other.x : this.x, mask.y ? other.y : this.y, mask.z ? other.z : this.z); }
  // The swizzles are read accessors, WGSL `v.xy`. A swizzle is never an assignment target, so a
  // kernel writes the component fields instead.
  get xy(): Vec2f { return new Vec2f(this.x, this.y); }
  get xz(): Vec2f { return new Vec2f(this.x, this.z); }
  get yz(): Vec2f { return new Vec2f(this.y, this.z); }
}

// A four-component `f32` vector, WGSL `vec4<f32>`. The C size and the WGSL size are both 16, so
// a `Vec4f` field never moves the field after it.
@CStruct({ align: 16 })
export class Vec4f {
  x: f32;
  y: f32;
  z: f32;
  w: f32;

  constructor(x: f32, y: f32, z: f32, w: f32) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
  }

  add(other: Vec4f): Vec4f {
    return new Vec4f(this.x + other.x, this.y + other.y, this.z + other.z, this.w + other.w);
  }

  sub(other: Vec4f): Vec4f {
    return new Vec4f(this.x - other.x, this.y - other.y, this.z - other.z, this.w - other.w);
  }

  mul(other: Vec4f): Vec4f {
    return new Vec4f(this.x * other.x, this.y * other.y, this.z * other.z, this.w * other.w);
  }

  scale(s: f32): Vec4f {
    return new Vec4f(this.x * s, this.y * s, this.z * s, this.w * s);
  }

  dot(other: Vec4f): f32 {
    return this.x * other.x + this.y * other.y + this.z * other.z + this.w * other.w;
  }

  length(): f32 {
    return Math.sqrt(this.dot(this) as f64) as f32;
  }

  normalize(): Vec4f {
    const magnitude: f32 = this.length();
    if (magnitude === 0.0) {
      return new Vec4f(0.0, 0.0, 0.0, 0.0);
    }
    return this.scale(1.0 / magnitude);
  }

  abs(): Vec4f { return new Vec4f(Math.abs(this.x as f64) as f32, Math.abs(this.y as f64) as f32, Math.abs(this.z as f64) as f32, Math.abs(this.w as f64) as f32); }
  floor(): Vec4f { return new Vec4f(Math.floor(this.x as f64) as f32, Math.floor(this.y as f64) as f32, Math.floor(this.z as f64) as f32, Math.floor(this.w as f64) as f32); }
  ceil(): Vec4f { return new Vec4f(Math.ceil(this.x as f64) as f32, Math.ceil(this.y as f64) as f32, Math.ceil(this.z as f64) as f32, Math.ceil(this.w as f64) as f32); }
  fract(): Vec4f { return new Vec4f(fract(this.x), fract(this.y), fract(this.z), fract(this.w)); }
  sqrt(): Vec4f { return new Vec4f(Math.sqrt(this.x as f64) as f32, Math.sqrt(this.y as f64) as f32, Math.sqrt(this.z as f64) as f32, Math.sqrt(this.w as f64) as f32); }
  exp(): Vec4f { return new Vec4f(Math.exp(this.x as f64) as f32, Math.exp(this.y as f64) as f32, Math.exp(this.z as f64) as f32, Math.exp(this.w as f64) as f32); }
  log(): Vec4f { return new Vec4f(Math.log(this.x as f64) as f32, Math.log(this.y as f64) as f32, Math.log(this.z as f64) as f32, Math.log(this.w as f64) as f32); }
  sin(): Vec4f { return new Vec4f(Math.sin(this.x as f64) as f32, Math.sin(this.y as f64) as f32, Math.sin(this.z as f64) as f32, Math.sin(this.w as f64) as f32); }
  cos(): Vec4f { return new Vec4f(Math.cos(this.x as f64) as f32, Math.cos(this.y as f64) as f32, Math.cos(this.z as f64) as f32, Math.cos(this.w as f64) as f32); }
  tan(): Vec4f { return new Vec4f(Math.tan(this.x as f64) as f32, Math.tan(this.y as f64) as f32, Math.tan(this.z as f64) as f32, Math.tan(this.w as f64) as f32); }
  sign(): Vec4f { return new Vec4f(sign(this.x), sign(this.y), sign(this.z), sign(this.w)); }
  min(other: Vec4f): Vec4f { return new Vec4f(Math.min(this.x as f64, other.x as f64) as f32, Math.min(this.y as f64, other.y as f64) as f32, Math.min(this.z as f64, other.z as f64) as f32, Math.min(this.w as f64, other.w as f64) as f32); }
  max(other: Vec4f): Vec4f { return new Vec4f(Math.max(this.x as f64, other.x as f64) as f32, Math.max(this.y as f64, other.y as f64) as f32, Math.max(this.z as f64, other.z as f64) as f32, Math.max(this.w as f64, other.w as f64) as f32); }
  clamp(low: Vec4f, high: Vec4f): Vec4f { return new Vec4f(clamp(this.x, low.x, high.x), clamp(this.y, low.y, high.y), clamp(this.z, low.z, high.z), clamp(this.w, low.w, high.w)); }
  pow(other: Vec4f): Vec4f { return new Vec4f(Math.pow(this.x as f64, other.x as f64) as f32, Math.pow(this.y as f64, other.y as f64) as f32, Math.pow(this.z as f64, other.z as f64) as f32, Math.pow(this.w as f64, other.w as f64) as f32); }
  mix(other: Vec4f, amount: f32): Vec4f { return new Vec4f(mix(this.x, other.x, amount), mix(this.y, other.y, amount), mix(this.z, other.z, amount), mix(this.w, other.w, amount)); }
  step(edge: Vec4f): Vec4f { return new Vec4f(step(edge.x, this.x), step(edge.y, this.y), step(edge.z, this.z), step(edge.w, this.w)); }
  smoothstep(low: Vec4f, high: Vec4f): Vec4f { return new Vec4f(smoothstep(low.x, high.x, this.x), smoothstep(low.y, high.y, this.y), smoothstep(low.z, high.z, this.z), smoothstep(low.w, high.w, this.w)); }
  distance(other: Vec4f): f32 { return this.sub(other).length(); }
  reflect(normal: Vec4f): Vec4f { return this.sub(normal.scale(2.0 * this.dot(normal))); }
  refract(normal: Vec4f, eta: f32): Vec4f {
    const product: f32 = this.dot(normal);
    const factor: f32 = 1.0 - eta * eta * (1.0 - product * product);
    if (factor < 0.0) return new Vec4f(0.0, 0.0, 0.0, 0.0);
    return this.scale(eta).sub(normal.scale(eta * product + (Math.sqrt(factor as f64) as f32)));
  }
  faceForward(incident: Vec4f, reference: Vec4f): Vec4f { return incident.dot(reference) < 0.0 ? this : this.scale(-1.0); }
  lt(other: Vec4f): Vec4b { return new Vec4b(this.x < other.x, this.y < other.y, this.z < other.z, this.w < other.w); }
  le(other: Vec4f): Vec4b { return new Vec4b(this.x <= other.x, this.y <= other.y, this.z <= other.z, this.w <= other.w); }
  gt(other: Vec4f): Vec4b { return new Vec4b(this.x > other.x, this.y > other.y, this.z > other.z, this.w > other.w); }
  ge(other: Vec4f): Vec4b { return new Vec4b(this.x >= other.x, this.y >= other.y, this.z >= other.z, this.w >= other.w); }
  eq(other: Vec4f): Vec4b { return new Vec4b(this.x === other.x, this.y === other.y, this.z === other.z, this.w === other.w); }
  ne(other: Vec4f): Vec4b { return new Vec4b(this.x !== other.x, this.y !== other.y, this.z !== other.z, this.w !== other.w); }
  select(other: Vec4f, mask: Vec4b): Vec4f { return new Vec4f(mask.x ? other.x : this.x, mask.y ? other.y : this.y, mask.z ? other.z : this.z, mask.w ? other.w : this.w); }
  get xy(): Vec2f { return new Vec2f(this.x, this.y); }
  get xz(): Vec2f { return new Vec2f(this.x, this.z); }
  get xw(): Vec2f { return new Vec2f(this.x, this.w); }
  get yz(): Vec2f { return new Vec2f(this.y, this.z); }
  get yw(): Vec2f { return new Vec2f(this.y, this.w); }
  get zw(): Vec2f { return new Vec2f(this.z, this.w); }
  get xyz(): Vec3f { return new Vec3f(this.x, this.y, this.z); }
  get xyw(): Vec3f { return new Vec3f(this.x, this.y, this.w); }
  get xzw(): Vec3f { return new Vec3f(this.x, this.z, this.w); }
  get yzw(): Vec3f { return new Vec3f(this.y, this.z, this.w); }
}

// A two-component `i32` vector, WGSL `vec2<i32>`. The integer vectors carry no `length`,
// `normalize`, or transcendental method, because WGSL defines those for float types only.
@CStruct({ align: 8 })
export class Vec2i {
  x: i32;
  y: i32;

  constructor(x: i32, y: i32) {
    this.x = x;
    this.y = y;
  }

  add(other: Vec2i): Vec2i { return new Vec2i(this.x + other.x, this.y + other.y); }
  sub(other: Vec2i): Vec2i { return new Vec2i(this.x - other.x, this.y - other.y); }
  mul(other: Vec2i): Vec2i { return new Vec2i(this.x * other.x, this.y * other.y); }
  scale(s: i32): Vec2i { return new Vec2i(this.x * s, this.y * s); }
  dot(other: Vec2i): i32 { return this.x * other.x + this.y * other.y; }
  // WGSL `abs(v)`. The `i32` minimum is outside the domain: the host returns a value that `i32`
  // cannot hold, and WGSL returns the minimum itself.
  abs(): Vec2i { return new Vec2i(Math.abs(this.x as f64) as i32, Math.abs(this.y as f64) as i32); }
  min(other: Vec2i): Vec2i { return new Vec2i(Math.min(this.x as f64, other.x as f64) as i32, Math.min(this.y as f64, other.y as f64) as i32); }
  max(other: Vec2i): Vec2i { return new Vec2i(Math.max(this.x as f64, other.x as f64) as i32, Math.max(this.y as f64, other.y as f64) as i32); }
  clamp(low: Vec2i, high: Vec2i): Vec2i { return this.max(low).min(high); }
  lt(other: Vec2i): Vec2b { return new Vec2b(this.x < other.x, this.y < other.y); }
  le(other: Vec2i): Vec2b { return new Vec2b(this.x <= other.x, this.y <= other.y); }
  gt(other: Vec2i): Vec2b { return new Vec2b(this.x > other.x, this.y > other.y); }
  ge(other: Vec2i): Vec2b { return new Vec2b(this.x >= other.x, this.y >= other.y); }
  eq(other: Vec2i): Vec2b { return new Vec2b(this.x === other.x, this.y === other.y); }
  ne(other: Vec2i): Vec2b { return new Vec2b(this.x !== other.x, this.y !== other.y); }
  select(other: Vec2i, mask: Vec2b): Vec2i { return new Vec2i(mask.x ? other.x : this.x, mask.y ? other.y : this.y); }
}

// A three-component `i32` vector, WGSL `vec3<i32>`. The C size is 16 and the WGSL size is 12, as
// with `Vec3f`.
@CStruct({ align: 16 })
export class Vec3i {
  x: i32;
  y: i32;
  z: i32;

  constructor(x: i32, y: i32, z: i32) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  add(other: Vec3i): Vec3i { return new Vec3i(this.x + other.x, this.y + other.y, this.z + other.z); }
  sub(other: Vec3i): Vec3i { return new Vec3i(this.x - other.x, this.y - other.y, this.z - other.z); }
  mul(other: Vec3i): Vec3i { return new Vec3i(this.x * other.x, this.y * other.y, this.z * other.z); }
  scale(s: i32): Vec3i { return new Vec3i(this.x * s, this.y * s, this.z * s); }
  dot(other: Vec3i): i32 { return this.x * other.x + this.y * other.y + this.z * other.z; }
  abs(): Vec3i { return new Vec3i(Math.abs(this.x as f64) as i32, Math.abs(this.y as f64) as i32, Math.abs(this.z as f64) as i32); }
  min(other: Vec3i): Vec3i { return new Vec3i(Math.min(this.x as f64, other.x as f64) as i32, Math.min(this.y as f64, other.y as f64) as i32, Math.min(this.z as f64, other.z as f64) as i32); }
  max(other: Vec3i): Vec3i { return new Vec3i(Math.max(this.x as f64, other.x as f64) as i32, Math.max(this.y as f64, other.y as f64) as i32, Math.max(this.z as f64, other.z as f64) as i32); }
  clamp(low: Vec3i, high: Vec3i): Vec3i { return this.max(low).min(high); }
  lt(other: Vec3i): Vec3b { return new Vec3b(this.x < other.x, this.y < other.y, this.z < other.z); }
  le(other: Vec3i): Vec3b { return new Vec3b(this.x <= other.x, this.y <= other.y, this.z <= other.z); }
  gt(other: Vec3i): Vec3b { return new Vec3b(this.x > other.x, this.y > other.y, this.z > other.z); }
  ge(other: Vec3i): Vec3b { return new Vec3b(this.x >= other.x, this.y >= other.y, this.z >= other.z); }
  eq(other: Vec3i): Vec3b { return new Vec3b(this.x === other.x, this.y === other.y, this.z === other.z); }
  ne(other: Vec3i): Vec3b { return new Vec3b(this.x !== other.x, this.y !== other.y, this.z !== other.z); }
  select(other: Vec3i, mask: Vec3b): Vec3i { return new Vec3i(mask.x ? other.x : this.x, mask.y ? other.y : this.y, mask.z ? other.z : this.z); }
  get xy(): Vec2i { return new Vec2i(this.x, this.y); }
  get xz(): Vec2i { return new Vec2i(this.x, this.z); }
  get yz(): Vec2i { return new Vec2i(this.y, this.z); }
}

// A four-component `i32` vector, WGSL `vec4<i32>`.
@CStruct({ align: 16 })
export class Vec4i {
  x: i32;
  y: i32;
  z: i32;
  w: i32;

  constructor(x: i32, y: i32, z: i32, w: i32) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
  }

  add(other: Vec4i): Vec4i { return new Vec4i(this.x + other.x, this.y + other.y, this.z + other.z, this.w + other.w); }
  sub(other: Vec4i): Vec4i { return new Vec4i(this.x - other.x, this.y - other.y, this.z - other.z, this.w - other.w); }
  mul(other: Vec4i): Vec4i { return new Vec4i(this.x * other.x, this.y * other.y, this.z * other.z, this.w * other.w); }
  scale(s: i32): Vec4i { return new Vec4i(this.x * s, this.y * s, this.z * s, this.w * s); }
  dot(other: Vec4i): i32 { return this.x * other.x + this.y * other.y + this.z * other.z + this.w * other.w; }
  abs(): Vec4i { return new Vec4i(Math.abs(this.x as f64) as i32, Math.abs(this.y as f64) as i32, Math.abs(this.z as f64) as i32, Math.abs(this.w as f64) as i32); }
  min(other: Vec4i): Vec4i { return new Vec4i(Math.min(this.x as f64, other.x as f64) as i32, Math.min(this.y as f64, other.y as f64) as i32, Math.min(this.z as f64, other.z as f64) as i32, Math.min(this.w as f64, other.w as f64) as i32); }
  max(other: Vec4i): Vec4i { return new Vec4i(Math.max(this.x as f64, other.x as f64) as i32, Math.max(this.y as f64, other.y as f64) as i32, Math.max(this.z as f64, other.z as f64) as i32, Math.max(this.w as f64, other.w as f64) as i32); }
  clamp(low: Vec4i, high: Vec4i): Vec4i { return this.max(low).min(high); }
  lt(other: Vec4i): Vec4b { return new Vec4b(this.x < other.x, this.y < other.y, this.z < other.z, this.w < other.w); }
  le(other: Vec4i): Vec4b { return new Vec4b(this.x <= other.x, this.y <= other.y, this.z <= other.z, this.w <= other.w); }
  gt(other: Vec4i): Vec4b { return new Vec4b(this.x > other.x, this.y > other.y, this.z > other.z, this.w > other.w); }
  ge(other: Vec4i): Vec4b { return new Vec4b(this.x >= other.x, this.y >= other.y, this.z >= other.z, this.w >= other.w); }
  eq(other: Vec4i): Vec4b { return new Vec4b(this.x === other.x, this.y === other.y, this.z === other.z, this.w === other.w); }
  ne(other: Vec4i): Vec4b { return new Vec4b(this.x !== other.x, this.y !== other.y, this.z !== other.z, this.w !== other.w); }
  select(other: Vec4i, mask: Vec4b): Vec4i { return new Vec4i(mask.x ? other.x : this.x, mask.y ? other.y : this.y, mask.z ? other.z : this.z, mask.w ? other.w : this.w); }
  get xy(): Vec2i { return new Vec2i(this.x, this.y); }
  get xz(): Vec2i { return new Vec2i(this.x, this.z); }
  get xw(): Vec2i { return new Vec2i(this.x, this.w); }
  get yz(): Vec2i { return new Vec2i(this.y, this.z); }
  get yw(): Vec2i { return new Vec2i(this.y, this.w); }
  get zw(): Vec2i { return new Vec2i(this.z, this.w); }
  get xyz(): Vec3i { return new Vec3i(this.x, this.y, this.z); }
  get xyw(): Vec3i { return new Vec3i(this.x, this.y, this.w); }
  get xzw(): Vec3i { return new Vec3i(this.x, this.z, this.w); }
  get yzw(): Vec3i { return new Vec3i(this.y, this.z, this.w); }
}

// A two-component `u32` vector, WGSL `vec2<u32>`.
@CStruct({ align: 8 })
export class Vec2u {
  x: u32;
  y: u32;

  constructor(x: u32, y: u32) {
    this.x = x;
    this.y = y;
  }

  add(other: Vec2u): Vec2u { return new Vec2u(this.x + other.x, this.y + other.y); }
  sub(other: Vec2u): Vec2u { return new Vec2u(this.x - other.x, this.y - other.y); }
  mul(other: Vec2u): Vec2u { return new Vec2u(this.x * other.x, this.y * other.y); }
  scale(s: u32): Vec2u { return new Vec2u(this.x * s, this.y * s); }
  dot(other: Vec2u): u32 { return this.x * other.x + this.y * other.y; }
  min(other: Vec2u): Vec2u { return new Vec2u(Math.min(this.x as f64, other.x as f64) as u32, Math.min(this.y as f64, other.y as f64) as u32); }
  max(other: Vec2u): Vec2u { return new Vec2u(Math.max(this.x as f64, other.x as f64) as u32, Math.max(this.y as f64, other.y as f64) as u32); }
  clamp(low: Vec2u, high: Vec2u): Vec2u { return this.max(low).min(high); }
  lt(other: Vec2u): Vec2b { return new Vec2b(this.x < other.x, this.y < other.y); }
  le(other: Vec2u): Vec2b { return new Vec2b(this.x <= other.x, this.y <= other.y); }
  gt(other: Vec2u): Vec2b { return new Vec2b(this.x > other.x, this.y > other.y); }
  ge(other: Vec2u): Vec2b { return new Vec2b(this.x >= other.x, this.y >= other.y); }
  eq(other: Vec2u): Vec2b { return new Vec2b(this.x === other.x, this.y === other.y); }
  ne(other: Vec2u): Vec2b { return new Vec2b(this.x !== other.x, this.y !== other.y); }
  select(other: Vec2u, mask: Vec2b): Vec2u { return new Vec2u(mask.x ? other.x : this.x, mask.y ? other.y : this.y); }
}

// A three-component `u32` vector, WGSL `vec3<u32>`. The C size is 16 and the WGSL size is 12, as
// with `Vec3f`.
@CStruct({ align: 16 })
export class Vec3u {
  x: u32;
  y: u32;
  z: u32;

  constructor(x: u32, y: u32, z: u32) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  add(other: Vec3u): Vec3u { return new Vec3u(this.x + other.x, this.y + other.y, this.z + other.z); }
  sub(other: Vec3u): Vec3u { return new Vec3u(this.x - other.x, this.y - other.y, this.z - other.z); }
  mul(other: Vec3u): Vec3u { return new Vec3u(this.x * other.x, this.y * other.y, this.z * other.z); }
  scale(s: u32): Vec3u { return new Vec3u(this.x * s, this.y * s, this.z * s); }
  dot(other: Vec3u): u32 { return this.x * other.x + this.y * other.y + this.z * other.z; }
  min(other: Vec3u): Vec3u { return new Vec3u(Math.min(this.x as f64, other.x as f64) as u32, Math.min(this.y as f64, other.y as f64) as u32, Math.min(this.z as f64, other.z as f64) as u32); }
  max(other: Vec3u): Vec3u { return new Vec3u(Math.max(this.x as f64, other.x as f64) as u32, Math.max(this.y as f64, other.y as f64) as u32, Math.max(this.z as f64, other.z as f64) as u32); }
  clamp(low: Vec3u, high: Vec3u): Vec3u { return this.max(low).min(high); }
  lt(other: Vec3u): Vec3b { return new Vec3b(this.x < other.x, this.y < other.y, this.z < other.z); }
  le(other: Vec3u): Vec3b { return new Vec3b(this.x <= other.x, this.y <= other.y, this.z <= other.z); }
  gt(other: Vec3u): Vec3b { return new Vec3b(this.x > other.x, this.y > other.y, this.z > other.z); }
  ge(other: Vec3u): Vec3b { return new Vec3b(this.x >= other.x, this.y >= other.y, this.z >= other.z); }
  eq(other: Vec3u): Vec3b { return new Vec3b(this.x === other.x, this.y === other.y, this.z === other.z); }
  ne(other: Vec3u): Vec3b { return new Vec3b(this.x !== other.x, this.y !== other.y, this.z !== other.z); }
  select(other: Vec3u, mask: Vec3b): Vec3u { return new Vec3u(mask.x ? other.x : this.x, mask.y ? other.y : this.y, mask.z ? other.z : this.z); }
  get xy(): Vec2u { return new Vec2u(this.x, this.y); }
  get xz(): Vec2u { return new Vec2u(this.x, this.z); }
  get yz(): Vec2u { return new Vec2u(this.y, this.z); }
}

// A four-component `u32` vector, WGSL `vec4<u32>`.
@CStruct({ align: 16 })
export class Vec4u {
  x: u32;
  y: u32;
  z: u32;
  w: u32;

  constructor(x: u32, y: u32, z: u32, w: u32) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
  }

  add(other: Vec4u): Vec4u { return new Vec4u(this.x + other.x, this.y + other.y, this.z + other.z, this.w + other.w); }
  sub(other: Vec4u): Vec4u { return new Vec4u(this.x - other.x, this.y - other.y, this.z - other.z, this.w - other.w); }
  mul(other: Vec4u): Vec4u { return new Vec4u(this.x * other.x, this.y * other.y, this.z * other.z, this.w * other.w); }
  scale(s: u32): Vec4u { return new Vec4u(this.x * s, this.y * s, this.z * s, this.w * s); }
  dot(other: Vec4u): u32 { return this.x * other.x + this.y * other.y + this.z * other.z + this.w * other.w; }
  min(other: Vec4u): Vec4u { return new Vec4u(Math.min(this.x as f64, other.x as f64) as u32, Math.min(this.y as f64, other.y as f64) as u32, Math.min(this.z as f64, other.z as f64) as u32, Math.min(this.w as f64, other.w as f64) as u32); }
  max(other: Vec4u): Vec4u { return new Vec4u(Math.max(this.x as f64, other.x as f64) as u32, Math.max(this.y as f64, other.y as f64) as u32, Math.max(this.z as f64, other.z as f64) as u32, Math.max(this.w as f64, other.w as f64) as u32); }
  clamp(low: Vec4u, high: Vec4u): Vec4u { return this.max(low).min(high); }
  lt(other: Vec4u): Vec4b { return new Vec4b(this.x < other.x, this.y < other.y, this.z < other.z, this.w < other.w); }
  le(other: Vec4u): Vec4b { return new Vec4b(this.x <= other.x, this.y <= other.y, this.z <= other.z, this.w <= other.w); }
  gt(other: Vec4u): Vec4b { return new Vec4b(this.x > other.x, this.y > other.y, this.z > other.z, this.w > other.w); }
  ge(other: Vec4u): Vec4b { return new Vec4b(this.x >= other.x, this.y >= other.y, this.z >= other.z, this.w >= other.w); }
  eq(other: Vec4u): Vec4b { return new Vec4b(this.x === other.x, this.y === other.y, this.z === other.z, this.w === other.w); }
  ne(other: Vec4u): Vec4b { return new Vec4b(this.x !== other.x, this.y !== other.y, this.z !== other.z, this.w !== other.w); }
  select(other: Vec4u, mask: Vec4b): Vec4u { return new Vec4u(mask.x ? other.x : this.x, mask.y ? other.y : this.y, mask.z ? other.z : this.z, mask.w ? other.w : this.w); }
  get xy(): Vec2u { return new Vec2u(this.x, this.y); }
  get xz(): Vec2u { return new Vec2u(this.x, this.z); }
  get xw(): Vec2u { return new Vec2u(this.x, this.w); }
  get yz(): Vec2u { return new Vec2u(this.y, this.z); }
  get yw(): Vec2u { return new Vec2u(this.y, this.w); }
  get zw(): Vec2u { return new Vec2u(this.z, this.w); }
  get xyz(): Vec3u { return new Vec3u(this.x, this.y, this.z); }
  get xyw(): Vec3u { return new Vec3u(this.x, this.y, this.w); }
  get xzw(): Vec3u { return new Vec3u(this.x, this.z, this.w); }
  get yzw(): Vec3u { return new Vec3u(this.y, this.z, this.w); }
}

// A two-component boolean mask, WGSL `vec2<bool>`. A comparison method builds one, and `select`
// reads it. A `Vec2b` field in a schema is a diagnostic, because WGSL `bool` has no
// host-shareable layout.
@CStruct
export class Vec2b {
  x: boolean;
  y: boolean;

  constructor(x: boolean, y: boolean) {
    this.x = x;
    this.y = y;
  }

  any(): boolean { return this.x || this.y; }
  all(): boolean { return this.x && this.y; }
  not(): Vec2b { return new Vec2b(!this.x, !this.y); }
}

// A three-component boolean mask, WGSL `vec3<bool>`.
@CStruct
export class Vec3b {
  x: boolean;
  y: boolean;
  z: boolean;

  constructor(x: boolean, y: boolean, z: boolean) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  any(): boolean { return this.x || this.y || this.z; }
  all(): boolean { return this.x && this.y && this.z; }
  not(): Vec3b { return new Vec3b(!this.x, !this.y, !this.z); }
}

// A four-component boolean mask, WGSL `vec4<bool>`.
@CStruct
export class Vec4b {
  x: boolean;
  y: boolean;
  z: boolean;
  w: boolean;

  constructor(x: boolean, y: boolean, z: boolean, w: boolean) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
  }

  any(): boolean { return this.x || this.y || this.z || this.w; }
  all(): boolean { return this.x && this.y && this.z && this.w; }
  not(): Vec4b { return new Vec4b(!this.x, !this.y, !this.z, !this.w); }
}

// A two-component `f16` vector, WGSL `vec2<f16>`. The `f16` vectors declare no arithmetic,
// because subscript treats `f16` as storage only. A module that names an `f16` type opens with
// `enable f16;`.
@CStruct({ align: 4 })
export class Vec2h {
  x: f16;
  y: f16;

  constructor(x: f16, y: f16) {
    this.x = x;
    this.y = y;
  }
}

// A three-component `f16` vector, WGSL `vec3<f16>`. The C size is 8 and the WGSL size is 6.
@CStruct({ align: 8 })
export class Vec3h {
  x: f16;
  y: f16;
  z: f16;

  constructor(x: f16, y: f16, z: f16) {
    this.x = x;
    this.y = y;
    this.z = z;
  }
}

// A four-component `f16` vector, WGSL `vec4<f16>`.
@CStruct({ align: 8 })
export class Vec4h {
  x: f16;
  y: f16;
  z: f16;
  w: f16;

  constructor(x: f16, y: f16, z: f16, w: f16) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
  }
}

// A `u32` cell for atomic access, WGSL `atomic<u32>`. `add`, `sub`, `min`, `max`, and `exchange`
// return the value from before the update. The receiver must be a place in a storage binding or
// a workgroup variable.
@CStruct({ align: 4 })
export class AtomicU32 {
  value: u32;

  constructor(value: u32) {
    this.value = value;
  }

  load(): u32 {
    return this.value;
  }

  store(value: u32): void {
    this.value = value;
  }

  add(value: u32): u32 {
    const old: u32 = this.value;
    this.value += value;
    return old;
  }

  sub(value: u32): u32 {
    const old: u32 = this.value;
    this.value -= value;
    return old;
  }

  min(value: u32): u32 {
    const old: u32 = this.value;
    if (value < this.value) {
      this.value = value;
    }
    return old;
  }

  max(value: u32): u32 {
    const old: u32 = this.value;
    if (value > this.value) {
      this.value = value;
    }
    return old;
  }

  exchange(value: u32): u32 {
    const old: u32 = this.value;
    this.value = value;
    return old;
  }
}

// An `i32` cell for atomic access, WGSL `atomic<i32>`. A kernel cannot copy a schema that holds
// an atomic into a local, and cannot write one as a whole value.
@CStruct({ align: 4 })
export class AtomicI32 {
  value: i32;

  constructor(value: i32) {
    this.value = value;
  }

  load(): i32 {
    return this.value;
  }

  store(value: i32): void {
    this.value = value;
  }

  add(value: i32): i32 {
    const old: i32 = this.value;
    this.value += value;
    return old;
  }

  sub(value: i32): i32 {
    const old: i32 = this.value;
    this.value -= value;
    return old;
  }

  min(value: i32): i32 {
    const old: i32 = this.value;
    if (value < this.value) {
      this.value = value;
    }
    return old;
  }

  max(value: i32): i32 {
    const old: i32 = this.value;
    if (value > this.value) {
      this.value = value;
    }
    return old;
  }

  exchange(value: i32): i32 {
    const old: i32 = this.value;
    this.value = value;
    return old;
  }
}

// A 2x2 `f32` matrix, WGSL `mat2x2<f32>`. Each field is one column, because WGSL matrices are
// column-major.
@CStruct({ align: 8 })
export class Mat2x2f {
  c0: Vec2f;
  c1: Vec2f;

  constructor(c0: Vec2f, c1: Vec2f) {
    this.c0 = c0;
    this.c1 = c1;
  }

  // The matrix-vector product, WGSL `m * v`. `mul` is the matrix-matrix form, and both emit the
  // same WGSL operator.
  mulVec(value: Vec2f): Vec2f {
    return new Vec2f(
      this.c0.x * value.x + this.c1.x * value.y,
      this.c0.y * value.x + this.c1.y * value.y,
    );
  }

  mul(other: Mat2x2f): Mat2x2f {
    return new Mat2x2f(this.mulVec(other.c0), this.mulVec(other.c1));
  }

  transpose(): Mat2x2f {
    return new Mat2x2f(
      new Vec2f(this.c0.x, this.c1.x),
      new Vec2f(this.c0.y, this.c1.y),
    );
  }
}

// A 3x3 `f32` matrix, WGSL `mat3x3<f32>`. Each column is a `Vec3f` with a 4-byte tail, so the
// size is 48 bytes, not 36.
@CStruct({ align: 16 })
export class Mat3x3f {
  c0: Vec3f;
  c1: Vec3f;
  c2: Vec3f;

  constructor(c0: Vec3f, c1: Vec3f, c2: Vec3f) {
    this.c0 = c0;
    this.c1 = c1;
    this.c2 = c2;
  }

  mulVec(value: Vec3f): Vec3f {
    return new Vec3f(
      this.c0.x * value.x + this.c1.x * value.y + this.c2.x * value.z,
      this.c0.y * value.x + this.c1.y * value.y + this.c2.y * value.z,
      this.c0.z * value.x + this.c1.z * value.y + this.c2.z * value.z,
    );
  }

  mul(other: Mat3x3f): Mat3x3f {
    return new Mat3x3f(
      this.mulVec(other.c0),
      this.mulVec(other.c1),
      this.mulVec(other.c2),
    );
  }

  transpose(): Mat3x3f {
    return new Mat3x3f(
      new Vec3f(this.c0.x, this.c1.x, this.c2.x),
      new Vec3f(this.c0.y, this.c1.y, this.c2.y),
      new Vec3f(this.c0.z, this.c1.z, this.c2.z),
    );
  }
}

// A 4x4 `f32` matrix, WGSL `mat4x4<f32>`. The size is 64 bytes.
@CStruct({ align: 16 })
export class Mat4x4f {
  c0: Vec4f;
  c1: Vec4f;
  c2: Vec4f;
  c3: Vec4f;

  constructor(c0: Vec4f, c1: Vec4f, c2: Vec4f, c3: Vec4f) {
    this.c0 = c0;
    this.c1 = c1;
    this.c2 = c2;
    this.c3 = c3;
  }

  mulVec(value: Vec4f): Vec4f {
    return new Vec4f(
      this.c0.x * value.x + this.c1.x * value.y + this.c2.x * value.z + this.c3.x * value.w,
      this.c0.y * value.x + this.c1.y * value.y + this.c2.y * value.z + this.c3.y * value.w,
      this.c0.z * value.x + this.c1.z * value.y + this.c2.z * value.z + this.c3.z * value.w,
      this.c0.w * value.x + this.c1.w * value.y + this.c2.w * value.z + this.c3.w * value.w,
    );
  }

  mul(other: Mat4x4f): Mat4x4f {
    return new Mat4x4f(
      this.mulVec(other.c0),
      this.mulVec(other.c1),
      this.mulVec(other.c2),
      this.mulVec(other.c3),
    );
  }

  transpose(): Mat4x4f {
    return new Mat4x4f(
      new Vec4f(this.c0.x, this.c1.x, this.c2.x, this.c3.x),
      new Vec4f(this.c0.y, this.c1.y, this.c2.y, this.c3.y),
      new Vec4f(this.c0.z, this.c1.z, this.c2.z, this.c3.z),
      new Vec4f(this.c0.w, this.c1.w, this.c2.w, this.c3.w),
    );
  }
}

// The component factories take the components in order, WGSL `vec2<f32>(x, y)` for `vec2f`.
// `new Vec2f(x, y)` builds the same value, and a kernel accepts both spellings.
export function vec2f(x: f32, y: f32): Vec2f { return new Vec2f(x, y); }
export function vec3f(x: f32, y: f32, z: f32): Vec3f { return new Vec3f(x, y, z); }
export function vec4f(x: f32, y: f32, z: f32, w: f32): Vec4f { return new Vec4f(x, y, z, w); }
export function vec2i(x: i32, y: i32): Vec2i { return new Vec2i(x, y); }
export function vec3i(x: i32, y: i32, z: i32): Vec3i { return new Vec3i(x, y, z); }
export function vec4i(x: i32, y: i32, z: i32, w: i32): Vec4i { return new Vec4i(x, y, z, w); }
export function vec2u(x: u32, y: u32): Vec2u { return new Vec2u(x, y); }
export function vec3u(x: u32, y: u32, z: u32): Vec3u { return new Vec3u(x, y, z); }
export function vec4u(x: u32, y: u32, z: u32, w: u32): Vec4u { return new Vec4u(x, y, z, w); }
// The `From` factories widen a shorter vector, WGSL `vec3<f32>(v, z)`. The source vector fills
// the low components, and the arguments fill the rest.
export function vec3fFrom2(v: Vec2f, z: f32): Vec3f { return new Vec3f(v.x, v.y, z); }
export function vec4fFrom2(v: Vec2f, z: f32, w: f32): Vec4f { return new Vec4f(v.x, v.y, z, w); }
export function vec4fFrom3(v: Vec3f, w: f32): Vec4f { return new Vec4f(v.x, v.y, v.z, w); }
// The splat factories repeat one scalar in every component, WGSL `vec2<f32>(s)`.
export function vec2fSplat(s: f32): Vec2f { return new Vec2f(s, s); }
export function vec3fSplat(s: f32): Vec3f { return new Vec3f(s, s, s); }
export function vec4fSplat(s: f32): Vec4f { return new Vec4f(s, s, s, s); }
export function vec3iFrom2(v: Vec2i, z: i32): Vec3i { return new Vec3i(v.x, v.y, z); }
export function vec4iFrom2(v: Vec2i, z: i32, w: i32): Vec4i { return new Vec4i(v.x, v.y, z, w); }
export function vec4iFrom3(v: Vec3i, w: i32): Vec4i { return new Vec4i(v.x, v.y, v.z, w); }
export function vec2iSplat(s: i32): Vec2i { return new Vec2i(s, s); }
export function vec3iSplat(s: i32): Vec3i { return new Vec3i(s, s, s); }
export function vec4iSplat(s: i32): Vec4i { return new Vec4i(s, s, s, s); }
export function vec3uFrom2(v: Vec2u, z: u32): Vec3u { return new Vec3u(v.x, v.y, z); }
export function vec4uFrom2(v: Vec2u, z: u32, w: u32): Vec4u { return new Vec4u(v.x, v.y, z, w); }
export function vec4uFrom3(v: Vec3u, w: u32): Vec4u { return new Vec4u(v.x, v.y, v.z, w); }
export function vec2uSplat(s: u32): Vec2u { return new Vec2u(s, s); }
export function vec3uSplat(s: u32): Vec3u { return new Vec3u(s, s, s); }
export function vec4uSplat(s: u32): Vec4u { return new Vec4u(s, s, s, s); }
// The `f16` factories build a value for a buffer write. A kernel that computes with an `f16`
// value fails generation.
export function vec2h(x: f16, y: f16): Vec2h { return new Vec2h(x, y); }
export function vec3h(x: f16, y: f16, z: f16): Vec3h { return new Vec3h(x, y, z); }
export function vec4h(x: f16, y: f16, z: f16, w: f16): Vec4h { return new Vec4h(x, y, z, w); }

// The scalar `clamp`, WGSL `clamp(value, low, high)`. `Math` carries no member for it, so the
// library owns the host body.
export function clamp(value: f32, low: f32, high: f32): f32 {
  if (value < low) {
    return low;
  }
  if (value > high) {
    return high;
  }
  return value;
}

// The scalar linear blend, WGSL `mix(left, right, amount)`. An `amount` outside [0, 1]
// extrapolates, because the formula carries no clamp.
export function mix(left: f32, right: f32, amount: f32): f32 {
  return left + (right - left) * amount;
}

// The scalar `step`, WGSL `step(edge, value)`. The result is 1.0 when `value` equals `edge`.
export function step(edge: f32, value: f32): f32 {
  if (value < edge) {
    return 0.0;
  }
  return 1.0;
}

// The scalar `smoothstep`, WGSL `smoothstep(low, high, value)`. The result stays in [0, 1], and
// its first derivative is zero at both edges.
export function smoothstep(low: f32, high: f32, value: f32): f32 {
  const amount: f32 = clamp((value - low) / (high - low), 0.0, 1.0);
  return amount * amount * (3.0 - 2.0 * amount);
}

// The scalar fraction, WGSL `fract(value)`. A negative input still gives a result in [0, 1),
// because the floor rounds toward minus infinity.
export function fract(value: f32): f32 {
  return value - (Math.floor(value as f64) as f32);
}

// The scalar `sign`, WGSL `sign(value)`. The result is an `f32`, and zero maps to 0.0.
export function sign(value: f32): f32 {
  if (value < 0.0) {
    return -1.0;
  }
  if (value > 0.0) {
    return 1.0;
  }
  return 0.0;
}

// Returns the 2x2 identity, the unit of `Mat2x2f.mul`.
export function mat2x2fIdentity(): Mat2x2f {
  return new Mat2x2f(vec2f(1.0, 0.0), vec2f(0.0, 1.0));
}

// Returns the 3x3 identity, the unit of `Mat3x3f.mul`.
export function mat3x3fIdentity(): Mat3x3f {
  return new Mat3x3f(
    vec3f(1.0, 0.0, 0.0),
    vec3f(0.0, 1.0, 0.0),
    vec3f(0.0, 0.0, 1.0),
  );
}

// Returns the 4x4 identity, the unit of `Mat4x4f.mul`.
export function mat4x4fIdentity(): Mat4x4f {
  return new Mat4x4f(
    vec4f(1.0, 0.0, 0.0, 0.0),
    vec4f(0.0, 1.0, 0.0, 0.0),
    vec4f(0.0, 0.0, 1.0, 0.0),
    vec4f(0.0, 0.0, 0.0, 1.0),
  );
}

// The argument block of an indirect dispatch, 12 bytes in the order WebGPU fixes. A program
// writes it with `Context.bytesOf` into a buffer that carries `GPUBufferUsage.INDIRECT`.
@CStruct
export class DispatchIndirectArgs {
  x: u32;
  y: u32;
  z: u32;

  constructor(x: u32, y: u32, z: u32) {
    this.x = x;
    this.y = y;
    this.z = z;
  }
}

// The argument block of an indirect draw, 16 bytes in the order WebGPU fixes.
@CStruct
export class DrawIndirectArgs {
  vertexCount: u32;
  instanceCount: u32;
  firstVertex: u32;
  firstInstance: u32;

  constructor(vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
    this.vertexCount = vertexCount;
    this.instanceCount = instanceCount;
    this.firstVertex = firstVertex;
    this.firstInstance = firstInstance;
  }
}

// The argument block of an indirect indexed draw, 20 bytes. `baseVertex` is signed, and it
// shifts every index the draw reads.
@CStruct
export class DrawIndexedIndirectArgs {
  indexCount: u32;
  instanceCount: u32;
  firstIndex: u32;
  baseVertex: i32;
  firstInstance: u32;

  constructor(
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32,
  ) {
    this.indexCount = indexCount;
    this.instanceCount = instanceCount;
    this.firstIndex = firstIndex;
    this.baseVertex = baseVertex;
    this.firstInstance = firstInstance;
  }
}
