import fs from "node:fs";
import { registerHooks } from "node:module";

const nodeMajor = Number.parseInt(process.versions.node.split(".", 1)[0], 10);
if (!Number.isInteger(nodeMajor) || nodeMajor < 24) {
  console.error("gen-layout-vectors error: Node.js 24 or newer is required");
  process.exit(1);
}

const upstream = process.env.SUBSCRIPT_TYPEGPU_UPSTREAM_DIR;
if (!upstream) {
  console.error("set SUBSCRIPT_TYPEGPU_UPSTREAM_DIR to a TypeGPU checkout");
  process.exit(1);
}

const packageFile = `${upstream}/packages/typegpu/package.json`;
const { version } = JSON.parse(fs.readFileSync(packageFile, "utf8"));
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === "typegpu/package.json") {
      return {
        url: `data:text/javascript,export const version=${JSON.stringify(version)}`,
        shortCircuit: true,
      };
    }
    return nextResolve(specifier, context);
  },
});

const data = await import(`${upstream}/packages/typegpu/src/data/index.ts`);

function basic(name, schema) {
  return {
    name,
    align: data.alignmentOf(schema),
    size: data.sizeOf(schema),
  };
}

function offset(schema, accessor) {
  return data.memoryLayoutOf(schema, accessor).offset;
}

const scalars = [
  basic("f32", data.f32),
  basic("i32", data.i32),
  basic("u32", data.u32),
  basic("f16", data.f16),
];
const vectors = [
  basic("vec2f", data.vec2f),
  basic("vec3f", data.vec3f),
  basic("vec4f", data.vec4f),
  basic("vec2i", data.vec2i),
  basic("vec3i", data.vec3i),
  basic("vec4i", data.vec4i),
  basic("vec2u", data.vec2u),
  basic("vec3u", data.vec3u),
  basic("vec4u", data.vec4u),
  basic("vec2h", data.vec2h),
  basic("vec3h", data.vec3h),
  basic("vec4h", data.vec4h),
];
const matrices = [
  basic("mat2x2f", data.mat2x2f),
  basic("mat3x3f", data.mat3x3f),
  basic("mat4x4f", data.mat4x4f),
];

const One = data.struct({ a: data.u32, b: data.vec3f });
const OneArray3 = data.arrayOf(One, 3);
const Two = data.struct({ c: OneArray3, d: data.vec4u });
const shapes = [
  {
    ...basic("One", One),
    offsets: { a: offset(One, (value) => value.a), b: offset(One, (value) => value.b) },
  },
  {
    ...basic("OneArray3", OneArray3),
    stride: offset(OneArray3, (value) => value[1]),
    offsets: { element0: offset(OneArray3, (value) => value[0]), element1: offset(OneArray3, (value) => value[1]) },
  },
  {
    ...basic("Two", Two),
    offsets: { c: offset(Two, (value) => value.c), d: offset(Two, (value) => value.d) },
  },
];

const output = { typegpuVersion: version, scalars, vectors, matrices, shapes };
const destination = new URL("../specs/layout-vectors.json", import.meta.url);
fs.writeFileSync(destination, `${JSON.stringify(output, null, 2)}\n`);
