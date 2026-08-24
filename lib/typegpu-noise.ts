import {
  Vec3f,
  mix,
} from "./typegpu-types";

// The PRNG hashes each input seed with Wang's 32-bit integer hash. It advances with
// xorshift32 through left 13, right 17, and left 5 shifts.
// `randF32` returns an exact `u32` state and an `f32` value in the range [0, 1).
// The value scales by 2^-32 through `f32`, which keeps 24 significant bits.
// A rounded upper endpoint clamps to the largest `f32` below 1.
@CStruct
export class RandomF32 {
  state: u32;
  value: f32;

  constructor(state: u32, value: f32) {
    this.state = state;
    this.value = value;
  }
}

function xorU32(left: u32, right: u32): u32 {
  return (left | right) & ~(left & right);
}

export function randSeed(seed: u32): u32 {
  let state: u32 = xorU32(xorU32(seed, 61), seed / 65536);
  state *= 9;
  state = xorU32(state, state / 16);
  state *= 668265261;
  return xorU32(state, state / 32768);
}

export function randF32(state: u32): RandomF32 {
  let next: u32 = xorU32(state, state * 8192);
  next = xorU32(next, next / 131072);
  next = xorU32(next, next * 32);
  let value: f32 = (next as f32) * 0.00000000023283064;
  if (value >= 1.0) value = 0.99999994;
  return new RandomF32(next, value);
}

const PERMUTATION: FixedArray<u32, 256> = [
  84, 14, 9, 251, 215, 58, 108, 196, 82, 218, 236, 18, 98, 147, 26, 49,
  29, 74, 64, 83, 141, 38, 87, 103, 86, 232, 197, 28, 37, 186, 15, 43,
  226, 45, 110, 162, 193, 157, 234, 81, 224, 62, 5, 230, 116, 219, 107, 250,
  248, 132, 204, 36, 117, 179, 228, 119, 85, 214, 93, 122, 139, 73, 47, 79,
  39, 69, 151, 135, 164, 146, 118, 115, 225, 194, 51, 91, 100, 183, 17, 235,
  53, 90, 59, 150, 1, 101, 7, 127, 238, 121, 105, 111, 27, 6, 106, 131,
  138, 94, 66, 243, 114, 137, 89, 221, 185, 195, 99, 155, 25, 20, 104, 149,
  65, 198, 63, 154, 213, 30, 2, 212, 71, 190, 97, 211, 163, 161, 67, 233,
  50, 42, 12, 177, 207, 245, 170, 205, 241, 237, 145, 143, 158, 78, 140, 153,
  113, 144, 182, 169, 8, 167, 68, 249, 242, 159, 24, 217, 203, 171, 252, 175,
  130, 220, 156, 60, 31, 181, 46, 231, 19, 201, 165, 160, 95, 126, 173, 244,
  75, 191, 77, 124, 206, 189, 247, 72, 57, 174, 33, 222, 187, 54, 134, 16,
  142, 70, 102, 202, 255, 10, 133, 96, 129, 246, 199, 227, 168, 32, 34, 176,
  11, 210, 223, 40, 123, 178, 61, 184, 188, 56, 21, 152, 44, 0, 180, 239,
  41, 229, 23, 254, 3, 253, 55, 48, 35, 80, 109, 172, 125, 22, 200, 112,
  13, 128, 166, 52, 209, 148, 92, 216, 136, 88, 76, 240, 120, 4, 192, 208,
];

// Returns Perlin's quintic interpolant 6t^5 - 15t^4 + 10t^3. The first and second
// derivatives are zero at 0 and at 1.
export function fade(value: f32): f32 {
  return value * value * value * (value * (value * 6.0 - 15.0) + 10.0);
}

// Returns the dot product of (x, y, z) with one of the twelve improved-noise
// gradient vectors. The low four bits of `hash` select the vector.
export function grad(hash: u32, x: f32, y: f32, z: f32): f32 {
  const code: u32 = hash & 15;
  const first: f32 = code < 8 ? x : y;
  const second: f32 = code < 4 ? y : ((code === 12 || code === 14) ? x : z);
  const signedFirst: f32 = (code & 1) === 0 ? first : -first;
  const signedSecond: f32 = (code & 2) === 0 ? second : -second;
  return signedFirst + signedSecond;
}

function permutation(index: i32): u32 {
  return PERMUTATION[index & 255];
}

function cornerHash(x: i32, y: i32, z: i32): u32 {
  return permutation((permutation((permutation(x) as i32) + y) as i32) + z);
}

// Returns Ken Perlin's improved 3D noise at `p`, from a fixed 256-entry permutation
// table. TypeGPU's perlin3d draws random gradients from a seeded unit-sphere
// sampler, so the two produce different values for the same point.
export function perlin3d(p: Vec3f): f32 {
  const base: Vec3f = p.floor();
  const local: Vec3f = p.sub(base);
  const x: i32 = base.x as i32;
  const y: i32 = base.y as i32;
  const z: i32 = base.z as i32;
  const u: f32 = fade(local.x);
  const v: f32 = fade(local.y);
  const w: f32 = fade(local.z);
  const x00: f32 = mix(
    grad(cornerHash(x, y, z), local.x, local.y, local.z),
    grad(cornerHash(x + 1, y, z), local.x - 1.0, local.y, local.z),
    u,
  );
  const x10: f32 = mix(
    grad(cornerHash(x, y + 1, z), local.x, local.y - 1.0, local.z),
    grad(cornerHash(x + 1, y + 1, z), local.x - 1.0, local.y - 1.0, local.z),
    u,
  );
  const x01: f32 = mix(
    grad(cornerHash(x, y, z + 1), local.x, local.y, local.z - 1.0),
    grad(cornerHash(x + 1, y, z + 1), local.x - 1.0, local.y, local.z - 1.0),
    u,
  );
  const x11: f32 = mix(
    grad(cornerHash(x, y + 1, z + 1), local.x, local.y - 1.0, local.z - 1.0),
    grad(
      cornerHash(x + 1, y + 1, z + 1),
      local.x - 1.0,
      local.y - 1.0,
      local.z - 1.0,
    ),
    u,
  );
  return mix(mix(x00, x10, v), mix(x01, x11, v), w);
}
