// program: b26-fewer-parameters
// purpose: prove named entries with absent trailing parameters and host execution
// exercises: K1, PI2, RN2, RN6, CL1, CL4
// questions: none

import {
  ComputePipelineSpec, MutStorage, Storage, RenderPipelineSpec,
  computePipeline, computePipeline2, renderPipeline,
  createComputePipeline, createRenderPipeline, createBindGroup, bufferResource,
  simulateCompute, simulateComputeThreads, simulateCompute2,
} from "./typegpu";
import { Vec2f, Vec4f } from "./typegpu-types";
import { gpu, GPUAdapter, GPUDevice, GPUBufferUsage, GPUTextureUsage } from "./webgpu";
import {
  fewer_ENTRY, fewer_HOST_RUNNABLE, fewer_LAYOUT0, fewer_WGSL,
  prefix_ENTRY, prefix_HOST_RUNNABLE, prefix_LAYOUT0, prefix_LAYOUT1, prefix_WGSL,
  triangle_VERTEX_ENTRY, triangle_FRAGMENT_ENTRY, triangle_VERTEX_LAYOUT0, triangle_WGSL,
} from "./b26-fewer-parameters.typegpu";

class OutputLayout {
  constructor(public output: MutStorage<u32>) {}
}
class SpareLayout {
  constructor(public spare: Storage<u32>) {}
}

function fewerKernel(res: OutputLayout): void {
  res.output[0] = res.output[0] + 2;
}
function prefixKernel(res0: OutputLayout): void {
  res0.output[0] = res0.output[0] + 3;
}
const fewer: ComputePipelineSpec = computePipeline(fewerKernel, {
  name: "fewer", workgroupSize: [1, 1, 1],
});
const prefix: ComputePipelineSpec = computePipeline2<OutputLayout, SpareLayout>(prefixKernel, {
  name: "prefix", workgroupSize: [1, 1, 1],
});

@ValueType
class Vertex {
  constructor(public position: Vec2f) {}
}
@ValueType
class Varyings {
  constructor(public position: Vec4f, public color: Vec4f) {}
}
function vert(value: Vertex): Varyings {
  return new Varyings(new Vec4f(value.position.x, value.position.y, 0.0, 1.0),
    new Vec4f(1.0, 0.0, 0.0, 1.0));
}
function frag(input: Varyings): Vec4f { return input.color; }
const triangle: RenderPipelineSpec = renderPipeline(vert, frag, { format: "rgba8unorm" });

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult == null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult == null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    using output = device.createBuffer({ size: 4, usage: GPUBufferUsage.STORAGE });
    using spare = device.createBuffer({ size: 4, usage: GPUBufferUsage.STORAGE });
    using vertices = device.createBuffer({ size: 24, usage: GPUBufferUsage.VERTEX });
    using target = device.createTexture({
      size: { width: 4, height: 4 }, format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    device.pushErrorScope("validation");
    using one = createComputePipeline(device, fewer_WGSL, fewer_ENTRY, [fewer_LAYOUT0], [1, 1, 1]);
    using two = createComputePipeline(device, prefix_WGSL, prefix_ENTRY, [prefix_LAYOUT0, prefix_LAYOUT1], [1, 1, 1]);
    using render = createRenderPipeline(device, triangle_WGSL, triangle_VERTEX_ENTRY,
      triangle_FRAGMENT_ENTRY, [], [triangle_VERTEX_LAYOUT0], triangle);
    const validationError = await device.popErrorScope();
    if (validationError != null) { print("FAIL pipeline"); return; }
    using oneLayout = one.bindGroupLayout(0);
    using oneGroup = createBindGroup(device, oneLayout, fewer_LAYOUT0, [bufferResource(output)]);
    using twoLayout0 = two.bindGroupLayout(0);
    using twoLayout1 = two.bindGroupLayout(1);
    using twoGroup0 = createBindGroup(device, twoLayout0, prefix_LAYOUT0, [bufferResource(output)]);
    using twoGroup1 = createBindGroup(device, twoLayout1, prefix_LAYOUT1, [bufferResource(spare)]);
    using encoder = device.createCommandEncoderDefault();
    one.dispatch(encoder, [oneGroup], 1, 1, 1);
    two.dispatch(encoder, [twoGroup0, twoGroup1], 1, 1, 1);
    using pass = encoder.beginRenderPass({ colorAttachments: [{
      view, clearValue: { r: 0, g: 0, b: 0, a: 1 }, loadOp: "clear", storeOp: "store",
    }] });
    render.bind(pass, [], [vertices]);
    pass.draw(3);
    pass.end();
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    print("pipelines:created");
    print(`prefix_LAYOUT1.entries=${prefix_LAYOUT1.entries.length}`);
    print(`triangle_VERTEX_LAYOUT0.arrayStride=${triangle_VERTEX_LAYOUT0.arrayStride}`);
    print(`fewer_WGSL_LINES=${fewer_WGSL.split("\n").length}`);
    print(`prefix_WGSL_LINES=${prefix_WGSL.split("\n").length}`);
    print(`triangle_WGSL_LINES=${triangle_WGSL.split("\n").length}`);
    print("commands:submitted");
    const host = new OutputLayout(new MutStorage<u32>([0]));
    const unused = new SpareLayout(new Storage<u32>([99]));
    simulateCompute(fewerKernel, host, fewer, [1, 1, 1], fewer_HOST_RUNNABLE);
    const workgroupsResult: u32 = host.output[0];
    simulateComputeThreads(fewerKernel, host, fewer, 2, 1, 1, fewer_HOST_RUNNABLE);
    print(`host:fewer workgroups=${workgroupsResult} threads=${host.output[0]}`);
    simulateCompute2<OutputLayout, SpareLayout>(prefixKernel, host, unused, prefix, [2, 1, 1], prefix_HOST_RUNNABLE);
    print(`host:two-layouts=${host.output[0]} spare=${unused.spare[0]}`);
    if (host.output[0] != 12 || unused.spare[0] != 99) { print("FAIL host"); return; }
  }
  gpu.dispose();
  print("PASS");
}
