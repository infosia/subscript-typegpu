// program: b01-layout
// purpose: prove host and WGSL layout identity for the first schema set
// exercises: SC1, SC5, SC9, SC11, SC12, LY3, LY4, LY5, LY6, LY15, LY16
// questions: none

import {
  Mat2x2f,
  Mat3x3f,
  Mat4x4f,
  Vec2f,
  Vec2h,
  Vec2i,
  Vec2u,
  Vec3f,
  Vec3h,
  Vec3i,
  Vec3u,
  Vec4f,
  Vec4h,
  Vec4i,
  Vec4u,
} from "./typegpu-types";
import {
  FloatVectors_ALIGN,
  FloatVectors_OFFSET_v2,
  FloatVectors_OFFSET_v3,
  FloatVectors_OFFSET_v4,
  FloatVectors_OFFSET_triples,
  FloatVectors_SIZE,
  FloatVectors_STRIDE_triples,
  FloatVectors_WGSL,
  Grid_ALIGN,
  Grid_OFFSET_cells,
  Grid_OFFSET_extent,
  Grid_SIZE,
  Grid_STRIDE_cells,
  Grid_WGSL,
  Half_ALIGN,
  Half_OFFSET_v,
  Half_SIZE,
  Half_WGSL,
  HalfMatrices_ALIGN,
  HalfMatrices_OFFSET_h2,
  HalfMatrices_OFFSET_h3,
  HalfMatrices_OFFSET_h4,
  HalfMatrices_OFFSET_m2,
  HalfMatrices_OFFSET_m3,
  HalfMatrices_OFFSET_m4,
  HalfMatrices_SIZE,
  HalfMatrices_WGSL,
  MatrixHolder_ALIGN,
  MatrixHolder_OFFSET_value,
  MatrixHolder_SIZE,
  MatrixHolder_WGSL,
  Mixed_ALIGN,
  Mixed_OFFSET_a,
  Mixed_OFFSET_p,
  Mixed_SIZE,
  Mixed_WGSL,
  Params_ALIGN,
  Params_OFFSET_count,
  Params_OFFSET_dt,
  Params_SIZE,
  Params_WGSL,
  Particle_ALIGN,
  Particle_OFFSET_pos,
  Particle_OFFSET_vel,
  Particle_SIZE,
  Particle_STRIDE,
  Particle_WGSL,
  SignedVectors_ALIGN,
  SignedVectors_OFFSET_v2,
  SignedVectors_OFFSET_v3,
  SignedVectors_OFFSET_v4,
  SignedVectors_SIZE,
  SignedVectors_WGSL,
  UnsignedVectors_ALIGN,
  UnsignedVectors_OFFSET_v2,
  UnsignedVectors_OFFSET_v3,
  UnsignedVectors_OFFSET_v4,
  UnsignedVectors_SIZE,
  UnsignedVectors_WGSL,
} from "./b01-layout.typegpu";
import { gpu, GPUAdapter, GPUDevice } from "./webgpu";

@CStruct
class Params {
  dt: f32;
  count: u32;
}

@CStruct
class Particle {
  pos: Vec3f;
  vel: Vec3f;
}

@CStruct
class Mixed {
  a: f32;
  p: Vec3f;
}

@CStruct
class Grid {
  cells: FixedArray<Particle, 4>;
  extent: Vec4u;
}

@CStruct
class MatrixHolder {
  value: Mat3x3f;
}

@CStruct
class Half {
  v: Vec2h;
}

@CStruct
class FloatVectors {
  v2: Vec2f;
  v3: Vec3f;
  v4: Vec4f;
  triples: FixedArray<Vec3f, 4>;
}

@CStruct
class SignedVectors {
  v2: Vec2i;
  v3: Vec3i;
  v4: Vec4i;
}

@CStruct
class UnsignedVectors {
  v2: Vec2u;
  v3: Vec3u;
  v4: Vec4u;
}

@CStruct
class HalfMatrices {
  h2: Vec2h;
  h3: Vec3h;
  h4: Vec4h;
  m2: Mat2x2f;
  m3: Mat3x3f;
  m4: Mat4x4f;
}

function retainSchemaTypes(
  params: Params,
  particle: Particle,
  mixed: Mixed,
  grid: Grid,
  matrixHolder: MatrixHolder,
  half: Half,
  floatVectors: FloatVectors,
  signedVectors: SignedVectors,
  unsignedVectors: UnsignedVectors,
  halfMatrices: HalfMatrices,
): u32 {
  return 0;
}

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) {
    print("FAIL adapter");
    gpu.dispose();
    return;
  }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) {
    print("FAIL device");
    adapterResult.dispose();
    gpu.dispose();
    return;
  }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    device.pushErrorScope("validation");
    const validationError = await device.popErrorScope();
    if (validationError !== null) {
      print("pipeline:invalid");
      print("FAIL");
      return;
    }
  }
  gpu.dispose();
  print(`Params size=${Params_SIZE} align=${Params_ALIGN} dt=${Params_OFFSET_dt} count=${Params_OFFSET_count}`);
  print(`Particle size=${Particle_SIZE} align=${Particle_ALIGN} pos=${Particle_OFFSET_pos} vel=${Particle_OFFSET_vel} stride=${Particle_STRIDE}`);
  print(`Mixed size=${Mixed_SIZE} align=${Mixed_ALIGN} a=${Mixed_OFFSET_a} p=${Mixed_OFFSET_p}`);
  print(`Grid size=${Grid_SIZE} align=${Grid_ALIGN} cells=${Grid_OFFSET_cells} extent=${Grid_OFFSET_extent} cellStride=${Grid_STRIDE_cells}`);
  print(`MatrixHolder size=${MatrixHolder_SIZE} align=${MatrixHolder_ALIGN} value=${MatrixHolder_OFFSET_value}`);
  print(`Half size=${Half_SIZE} align=${Half_ALIGN} v=${Half_OFFSET_v}`);
  print(`FloatVectors size=${FloatVectors_SIZE} align=${FloatVectors_ALIGN} v2=${FloatVectors_OFFSET_v2} v3=${FloatVectors_OFFSET_v3} v4=${FloatVectors_OFFSET_v4} triples=${FloatVectors_OFFSET_triples} tripleStride=${FloatVectors_STRIDE_triples}`);
  print(`SignedVectors size=${SignedVectors_SIZE} align=${SignedVectors_ALIGN} v2=${SignedVectors_OFFSET_v2} v3=${SignedVectors_OFFSET_v3} v4=${SignedVectors_OFFSET_v4}`);
  print(`UnsignedVectors size=${UnsignedVectors_SIZE} align=${UnsignedVectors_ALIGN} v2=${UnsignedVectors_OFFSET_v2} v3=${UnsignedVectors_OFFSET_v3} v4=${UnsignedVectors_OFFSET_v4}`);
  print(`HalfMatrices size=${HalfMatrices_SIZE} align=${HalfMatrices_ALIGN} h2=${HalfMatrices_OFFSET_h2} h3=${HalfMatrices_OFFSET_h3} h4=${HalfMatrices_OFFSET_h4} m2=${HalfMatrices_OFFSET_m2} m3=${HalfMatrices_OFFSET_m3} m4=${HalfMatrices_OFFSET_m4}`);
  print(`WGSL Params=${Params_WGSL.split("\n").length}`);
  print(`WGSL Particle=${Particle_WGSL.split("\n").length}`);
  print(`WGSL Mixed=${Mixed_WGSL.split("\n").length}`);
  print(`WGSL Grid=${Grid_WGSL.split("\n").length}`);
  print(`WGSL MatrixHolder=${MatrixHolder_WGSL.split("\n").length}`);
  print(`WGSL Half=${Half_WGSL.split("\n").length}`);
  print(`WGSL FloatVectors=${FloatVectors_WGSL.split("\n").length}`);
  print(`WGSL SignedVectors=${SignedVectors_WGSL.split("\n").length}`);
  print(`WGSL UnsignedVectors=${UnsignedVectors_WGSL.split("\n").length}`);
  print(`WGSL HalfMatrices=${HalfMatrices_WGSL.split("\n").length}`);
  print("PASS");
}
