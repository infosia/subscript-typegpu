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
