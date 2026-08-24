import {
  Vec2f,
  Vec3f,
  clamp,
  mix,
} from "./typegpu-types";

function sdfMin(a: f32, b: f32): f32 {
  return a < b ? a : b;
}

function sdfMax(a: f32, b: f32): f32 {
  return a > b ? a : b;
}

function sdfMax3(a: f32, b: f32, c: f32): f32 {
  return sdfMax(a, sdfMax(b, c));
}

export function sdSphere(p: Vec3f, radius: f32): f32 {
  return p.length() - radius;
}

export function sdBox(p: Vec3f, half: Vec3f): f32 {
  const offset: Vec3f = p.abs().sub(half);
  const outside: Vec3f = offset.max(new Vec3f(0.0, 0.0, 0.0));
  const inside: f32 = sdfMin(sdfMax3(offset.x, offset.y, offset.z), 0.0);
  return outside.length() + inside;
}

export function sdBoxFrame(p: Vec3f, half: Vec3f, edge: f32): f32 {
  const offset: Vec3f = p.abs().sub(half);
  const edgeVector = new Vec3f(edge, edge, edge);
  const inset: Vec3f = offset.add(edgeVector).abs().sub(edgeVector);
  const zero = new Vec3f(0.0, 0.0, 0.0);
  const xEdge = new Vec3f(offset.x, inset.y, inset.z);
  const yEdge = new Vec3f(inset.x, offset.y, inset.z);
  const zEdge = new Vec3f(inset.x, inset.y, offset.z);
  const xDistance: f32 = xEdge.max(zero).length()
    + sdfMin(sdfMax3(xEdge.x, xEdge.y, xEdge.z), 0.0);
  const yDistance: f32 = yEdge.max(zero).length()
    + sdfMin(sdfMax3(yEdge.x, yEdge.y, yEdge.z), 0.0);
  const zDistance: f32 = zEdge.max(zero).length()
    + sdfMin(sdfMax3(zEdge.x, zEdge.y, zEdge.z), 0.0);
  return sdfMin(xDistance, sdfMin(yDistance, zDistance));
}

export function sdPlane(p: Vec3f, normal: Vec3f, height: f32): f32 {
  return p.dot(normal) + height;
}

export function sdLine(p: Vec2f, a: Vec2f, b: Vec2f): f32 {
  const pointOffset: Vec2f = p.sub(a);
  const segment: Vec2f = b.sub(a);
  const amount: f32 = clamp(pointOffset.dot(segment) / segment.dot(segment), 0.0, 1.0);
  return pointOffset.sub(segment.scale(amount)).length();
}

export function opUnion(a: f32, b: f32): f32 {
  return sdfMin(a, b);
}

export function opSmoothUnion(a: f32, b: f32, k: f32): f32 {
  const amount: f32 = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
  return mix(b, a, amount) - k * amount * (1.0 - amount);
}
