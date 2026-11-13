// program: b27-sampler-address
// purpose: prove host nearest address modes and descriptor defaults on both axes
// exercises: CL5, TX3
// questions: none

import { Sampler, Texture2d, samplerFromDescriptor } from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import { GPUSamplerDescriptor } from "./webgpu";

export function main(): void {
  const red = new Vec4f(1.0, 0.0, 0.0, 1.0);
  const blue = new Vec4f(0.0, 0.0, 1.0, 1.0);
  const row = new Texture2d<f32>([red, blue], 2, 1);
  const repeatDescriptor: GPUSamplerDescriptor = { addressModeU: "repeat" };
  const repeated = row.sample(samplerFromDescriptor(repeatDescriptor), new Vec2f(1.25, 0.5));
  print(`repeat(1.25,0.5)=${repeated.x},${repeated.y},${repeated.z},${repeated.w}`);
  if (repeated.x != 1.0 || repeated.z != 0.0) { print("FAIL repeat"); return; }
  const texture = new Texture2d<f32>([
    red, blue, new Vec4f(0.0, 1.0, 0.0, 1.0), new Vec4f(1.0, 1.0, 0.0, 1.0),
  ], 2, 2);
  const descriptors: GPUSamplerDescriptor[] = [
    {},
    { addressModeU: "repeat", addressModeV: "repeat" },
    { addressModeU: "mirror-repeat", addressModeV: "mirror-repeat" },
    { addressModeU: "repeat", addressModeV: "mirror-repeat" },
    { addressModeU: "mirror-repeat", addressModeV: "repeat" },
  ];
  const coordinates: f32[] = [-2.25, -1.25, -1.0, -0.75, -0.25, 0.0, 0.25, 0.75, 1.0, 1.25, 1.75, 2.25];
  for (let mode: i32 = 0; mode < descriptors.length; mode += 1) {
    const sampler = samplerFromDescriptor(descriptors[mode]);
    for (let i: i32 = 0; i < coordinates.length; i += 1) {
      const u = texture.sampleLevel(sampler, new Vec2f(coordinates[i], 0.25), 0.0);
      const v = texture.sample(sampler, new Vec2f(0.25, coordinates[i]));
      print(`mode=${mode} uv=${coordinates[i]} u=${u.x},${u.y},${u.z} v=${v.x},${v.y},${v.z}`);
    }
  }
  const direct = texture.sample(new Sampler("nearest"), new Vec2f(-0.25, 1.25));
  print(`default=${direct.x},${direct.y},${direct.z}`);
  print("PASS");
}
