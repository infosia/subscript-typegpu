import {
  Vec2f,
  Vec2u,
  clamp,
} from "./typegpu-types";

// The host sizing of one cascade set. `cascadeProbes` counts the probes on one axis of cascade
// 0, `cascadeDim` is the texture width, and `cascadeCount` is the layer count.
@CStruct
export class CascadeDimensions {
  cascadeProbes: u32;
  cascadeDim: u32;
  cascadeCount: u32;

  constructor(cascadeProbes: u32, cascadeDim: u32, cascadeCount: u32) {
    this.cascadeProbes = cascadeProbes;
    this.cascadeDim = cascadeDim;
    this.cascadeCount = cascadeCount;
  }
}

// The host sizing keeps the nearest power of two explicit. This avoids a dependency on
// logarithm rounding at the exact midpoint between two supported probe counts.
export function cascadeDimensions(outputSize: u32): CascadeDimensions {
  const wanted: f64 = (outputSize as f64) * Math.sqrt(2.0) * 0.35;
  let lower: u32 = 1;
  while (((lower * 2) as f64) <= wanted) lower *= 2;
  const upper: u32 = lower * 2;
  const cascadeProbes: u32 = wanted < (lower as f64) * Math.sqrt(2.0) ? lower : upper;
  const scheduleLimit: f64 = 1.5 * 3.0 * (cascadeProbes as f64) + 1.0;
  let cascadeCount: u32 = 0;
  let scheduleReach: f64 = 1.0;
  while (scheduleReach < scheduleLimit) {
    scheduleReach *= 4.0;
    cascadeCount += 1;
  }
  return new CascadeDimensions(cascadeProbes, cascadeProbes * 2, cascadeCount);
}

// Zero names side A and one names side B. Adjacent layers therefore alternate textures.
export function cascadeWriteSide(cascadeCount: u32, layer: u32): u32 {
  return (cascadeCount - 1 - layer) % 2;
}

function cascadePow2(layer: u32): u32 {
  let value: u32 = 1;
  let index: u32 = 0;
  while (index < layer) {
    value *= 2;
    index += 1;
  }
  return value;
}

// Returns the stored direction count on one axis at `layer`. The count doubles for each layer,
// and one stored direction owns four actual rays.
export function cascadeRaysStored(layer: u32): u32 {
  return 2 * cascadePow2(layer);
}

// Returns the probe count on one axis at `layer`. The probe spacing doubles for each layer, and
// the result never falls below one.
export function cascadeProbesAt(baseProbes: u32, layer: u32): u32 {
  const probes: u32 = baseProbes / cascadePow2(layer);
  return probes > 0 ? probes : 1;
}

// Returns the distance where the layer's ray segment starts, in the units of `interval0`. The
// schedule quadruples each segment, so the start sums the earlier layers.
export function cascadeIntervalStart(interval0: f32, layer: u32): f32 {
  const scale: u32 = cascadePow2(layer);
  const scaleSquared: u32 = scale * scale;
  return interval0 * ((scaleSquared - 1) as f32) / 3.0;
}

// Returns the distance where the layer's ray segment ends. The segment length is `interval0`
// times four to the power of `layer`.
export function cascadeIntervalEnd(interval0: f32, layer: u32): f32 {
  const scale: u32 = cascadePow2(layer);
  const scaleSquared: u32 = scale * scale;
  return cascadeIntervalStart(interval0, layer) + interval0 * (scaleSquared as f32);
}

// Returns the ray angle in radians, in the range [-pi, pi). The direction index runs row by row
// over the square grid, and each ray takes the center of its slice.
export function cascadeRayAngle(dirActual: Vec2u, raysDimActual: u32): f32 {
  const rayIndex: u32 = dirActual.y * raysDimActual + dirActual.x;
  const rayCount: u32 = raysDimActual * raysDimActual;
  return (((rayIndex as f32) + 0.5) / (rayCount as f32))
    * 2.0 * 3.141592653589793 - 3.141592653589793;
}

// Each direction owns one square tile. The half-probe clamp keeps filtered samples
// inside that tile. It preserves bilinear interpolation between upper probes.
export function cascadeMergeUv(
  dirActual: Vec2u,
  probesUpper: u32,
  probePos: Vec2f,
  cascadeDim: f32,
): Vec2f {
  const probes: f32 = probesUpper as f32;
  const probeSample: Vec2f = probePos.scale(probes).clamp(
    new Vec2f(0.5, 0.5),
    new Vec2f(probes - 0.5, probes - 0.5),
  );
  return new Vec2f(
    (dirActual.x as f32) * probes + probeSample.x,
    (dirActual.y as f32) * probes + probeSample.y,
  ).scale(1.0 / cascadeDim);
}

// Returns the cascade-0 texture position that the final gather reads for direction `quadrant`.
// `uv` is the output position in [0, 1], and the result is a normalized texture position.
export function radianceGatherUv(
  quadrant: u32,
  uv: Vec2f,
  cascadeProbes: f32,
  cascadeDim: f32,
): Vec2f {
  const direction = new Vec2f((quadrant % 2) as f32, (quadrant / 2) as f32);
  const probeSample = new Vec2f(
    clamp(uv.x * cascadeProbes, 0.5, cascadeProbes - 0.5),
    clamp(uv.y * cascadeProbes, 0.5, cascadeProbes - 0.5),
  );
  return direction.scale(cascadeProbes).add(probeSample).scale(1.0 / cascadeDim);
}
