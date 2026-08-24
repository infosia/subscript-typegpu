import {
  Vec2f,
  Vec3f,
  mix,
} from "./typegpu-types";

// The PRNG is a 16-bit linear congruential generator. `randF32` returns the next
// state exactly in x and its zero-to-one value in y; callers cast x back to u32.
export function randSeed(seed: u32): u32 {
  return seed % 65536;
}

export function randF32(state: u32): Vec2f {
  const next: u32 = (state * 25173 + 13849) % 65536;
  return new Vec2f(next as f32, (next as f32) / 65536.0);
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

export function fade(value: f32): f32 {
  return value * value * value * (value * (value * 6.0 - 15.0) + 10.0);
}

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
