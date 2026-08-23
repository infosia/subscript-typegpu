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

  mul(other: Vec2f): Vec2f {
    return new Vec2f(this.x * other.x, this.y * other.y);
  }

  scale(s: f32): Vec2f {
    return new Vec2f(this.x * s, this.y * s);
  }

  dot(other: Vec2f): f32 {
    return this.x * other.x + this.y * other.y;
  }

  length(): f32 {
    return Math.sqrt(this.dot(this) as f64) as f32;
  }

  normalize(): Vec2f {
    const magnitude: f32 = this.length();
    if (magnitude === 0.0) {
      return new Vec2f(0.0, 0.0);
    }
    return this.scale(1.0 / magnitude);
  }

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
  reflect(normal: Vec2f): Vec2f { return this.sub(normal.scale(2.0 * this.dot(normal))); }
  refract(normal: Vec2f, eta: f32): Vec2f {
    const product: f32 = this.dot(normal);
    const factor: f32 = 1.0 - eta * eta * (1.0 - product * product);
    if (factor < 0.0) return new Vec2f(0.0, 0.0);
    return this.scale(eta).sub(normal.scale(eta * product + (Math.sqrt(factor as f64) as f32)));
  }
  faceForward(incident: Vec2f, reference: Vec2f): Vec2f { return incident.dot(reference) < 0.0 ? this : this.scale(-1.0); }
  lt(other: Vec2f): Vec2b { return new Vec2b(this.x < other.x, this.y < other.y); }
  le(other: Vec2f): Vec2b { return new Vec2b(this.x <= other.x, this.y <= other.y); }
  gt(other: Vec2f): Vec2b { return new Vec2b(this.x > other.x, this.y > other.y); }
  ge(other: Vec2f): Vec2b { return new Vec2b(this.x >= other.x, this.y >= other.y); }
  eq(other: Vec2f): Vec2b { return new Vec2b(this.x === other.x, this.y === other.y); }
  ne(other: Vec2f): Vec2b { return new Vec2b(this.x !== other.x, this.y !== other.y); }
  select(other: Vec2f, mask: Vec2b): Vec2f { return new Vec2f(mask.x ? other.x : this.x, mask.y ? other.y : this.y); }
}

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
  xy(): Vec2f { return new Vec2f(this.x, this.y); }
  xz(): Vec2f { return new Vec2f(this.x, this.z); }
  yz(): Vec2f { return new Vec2f(this.y, this.z); }
}

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
  xy(): Vec2f { return new Vec2f(this.x, this.y); }
  xz(): Vec2f { return new Vec2f(this.x, this.z); }
  xw(): Vec2f { return new Vec2f(this.x, this.w); }
  yz(): Vec2f { return new Vec2f(this.y, this.z); }
  yw(): Vec2f { return new Vec2f(this.y, this.w); }
  zw(): Vec2f { return new Vec2f(this.z, this.w); }
  xyz(): Vec3f { return new Vec3f(this.x, this.y, this.z); }
  xyw(): Vec3f { return new Vec3f(this.x, this.y, this.w); }
  xzw(): Vec3f { return new Vec3f(this.x, this.z, this.w); }
  yzw(): Vec3f { return new Vec3f(this.y, this.z, this.w); }
}

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
  xy(): Vec2i { return new Vec2i(this.x, this.y); }
  xz(): Vec2i { return new Vec2i(this.x, this.z); }
  yz(): Vec2i { return new Vec2i(this.y, this.z); }
}

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
  xy(): Vec2i { return new Vec2i(this.x, this.y); }
  xz(): Vec2i { return new Vec2i(this.x, this.z); }
  xw(): Vec2i { return new Vec2i(this.x, this.w); }
  yz(): Vec2i { return new Vec2i(this.y, this.z); }
  yw(): Vec2i { return new Vec2i(this.y, this.w); }
  zw(): Vec2i { return new Vec2i(this.z, this.w); }
  xyz(): Vec3i { return new Vec3i(this.x, this.y, this.z); }
  xyw(): Vec3i { return new Vec3i(this.x, this.y, this.w); }
  xzw(): Vec3i { return new Vec3i(this.x, this.z, this.w); }
  yzw(): Vec3i { return new Vec3i(this.y, this.z, this.w); }
}

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
  xy(): Vec2u { return new Vec2u(this.x, this.y); }
  xz(): Vec2u { return new Vec2u(this.x, this.z); }
  yz(): Vec2u { return new Vec2u(this.y, this.z); }
}

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
  xy(): Vec2u { return new Vec2u(this.x, this.y); }
  xz(): Vec2u { return new Vec2u(this.x, this.z); }
  xw(): Vec2u { return new Vec2u(this.x, this.w); }
  yz(): Vec2u { return new Vec2u(this.y, this.z); }
  yw(): Vec2u { return new Vec2u(this.y, this.w); }
  zw(): Vec2u { return new Vec2u(this.z, this.w); }
  xyz(): Vec3u { return new Vec3u(this.x, this.y, this.z); }
  xyw(): Vec3u { return new Vec3u(this.x, this.y, this.w); }
  xzw(): Vec3u { return new Vec3u(this.x, this.z, this.w); }
  yzw(): Vec3u { return new Vec3u(this.y, this.z, this.w); }
}

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

@CStruct({ align: 4 })
export class Vec2h {
  x: f16;
  y: f16;

  constructor(x: f16, y: f16) {
    this.x = x;
    this.y = y;
  }
}

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

@CStruct({ align: 8 })
export class Mat2x2f {
  c0: Vec2f;
  c1: Vec2f;

  constructor(c0: Vec2f, c1: Vec2f) {
    this.c0 = c0;
    this.c1 = c1;
  }

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

export function v2f(x: f32, y: f32): Vec2f { return new Vec2f(x, y); }
export function v3f(x: f32, y: f32, z: f32): Vec3f { return new Vec3f(x, y, z); }
export function v4f(x: f32, y: f32, z: f32, w: f32): Vec4f { return new Vec4f(x, y, z, w); }
export function v2i(x: i32, y: i32): Vec2i { return new Vec2i(x, y); }
export function v3i(x: i32, y: i32, z: i32): Vec3i { return new Vec3i(x, y, z); }
export function v4i(x: i32, y: i32, z: i32, w: i32): Vec4i { return new Vec4i(x, y, z, w); }
export function v2u(x: u32, y: u32): Vec2u { return new Vec2u(x, y); }
export function v3u(x: u32, y: u32, z: u32): Vec3u { return new Vec3u(x, y, z); }
export function v4u(x: u32, y: u32, z: u32, w: u32): Vec4u { return new Vec4u(x, y, z, w); }
export function v3fFrom2(v: Vec2f, z: f32): Vec3f { return new Vec3f(v.x, v.y, z); }
export function v4fFrom2(v: Vec2f, z: f32, w: f32): Vec4f { return new Vec4f(v.x, v.y, z, w); }
export function v4fFrom3(v: Vec3f, w: f32): Vec4f { return new Vec4f(v.x, v.y, v.z, w); }
export function v2fSplat(s: f32): Vec2f { return new Vec2f(s, s); }
export function v3fSplat(s: f32): Vec3f { return new Vec3f(s, s, s); }
export function v4fSplat(s: f32): Vec4f { return new Vec4f(s, s, s, s); }
export function v3iFrom2(v: Vec2i, z: i32): Vec3i { return new Vec3i(v.x, v.y, z); }
export function v4iFrom2(v: Vec2i, z: i32, w: i32): Vec4i { return new Vec4i(v.x, v.y, z, w); }
export function v4iFrom3(v: Vec3i, w: i32): Vec4i { return new Vec4i(v.x, v.y, v.z, w); }
export function v2iSplat(s: i32): Vec2i { return new Vec2i(s, s); }
export function v3iSplat(s: i32): Vec3i { return new Vec3i(s, s, s); }
export function v4iSplat(s: i32): Vec4i { return new Vec4i(s, s, s, s); }
export function v3uFrom2(v: Vec2u, z: u32): Vec3u { return new Vec3u(v.x, v.y, z); }
export function v4uFrom2(v: Vec2u, z: u32, w: u32): Vec4u { return new Vec4u(v.x, v.y, z, w); }
export function v4uFrom3(v: Vec3u, w: u32): Vec4u { return new Vec4u(v.x, v.y, v.z, w); }
export function v2uSplat(s: u32): Vec2u { return new Vec2u(s, s); }
export function v3uSplat(s: u32): Vec3u { return new Vec3u(s, s, s); }
export function v4uSplat(s: u32): Vec4u { return new Vec4u(s, s, s, s); }
export function v2h(x: f16, y: f16): Vec2h { return new Vec2h(x, y); }
export function v3h(x: f16, y: f16, z: f16): Vec3h { return new Vec3h(x, y, z); }
export function v4h(x: f16, y: f16, z: f16, w: f16): Vec4h { return new Vec4h(x, y, z, w); }

export function clamp(value: f32, low: f32, high: f32): f32 {
  if (value < low) {
    return low;
  }
  if (value > high) {
    return high;
  }
  return value;
}

export function mix(left: f32, right: f32, amount: f32): f32 {
  return left + (right - left) * amount;
}

export function step(edge: f32, value: f32): f32 {
  if (value < edge) {
    return 0.0;
  }
  return 1.0;
}

export function smoothstep(low: f32, high: f32, value: f32): f32 {
  const amount: f32 = clamp((value - low) / (high - low), 0.0, 1.0);
  return amount * amount * (3.0 - 2.0 * amount);
}

export function fract(value: f32): f32 {
  return value - (Math.floor(value as f64) as f32);
}

export function sign(value: f32): f32 {
  if (value < 0.0) {
    return -1.0;
  }
  if (value > 0.0) {
    return 1.0;
  }
  return 0.0;
}

export function mat2x2fIdentity(): Mat2x2f {
  return new Mat2x2f(v2f(1.0, 0.0), v2f(0.0, 1.0));
}

export function mat3x3fIdentity(): Mat3x3f {
  return new Mat3x3f(
    v3f(1.0, 0.0, 0.0),
    v3f(0.0, 1.0, 0.0),
    v3f(0.0, 0.0, 1.0),
  );
}

export function mat4x4fIdentity(): Mat4x4f {
  return new Mat4x4f(
    v4f(1.0, 0.0, 0.0, 0.0),
    v4f(0.0, 1.0, 0.0, 0.0),
    v4f(0.0, 0.0, 1.0, 0.0),
    v4f(0.0, 0.0, 0.0, 1.0),
  );
}

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
