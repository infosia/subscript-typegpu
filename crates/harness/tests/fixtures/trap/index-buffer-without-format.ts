// expected-rule: RN18
// purpose: prove RenderPipeline.setIndexBuffer requires an index format

import { createRenderPipeline, RenderPipelineSpec } from "./typegpu";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUTextureUsage } from "./webgpu";

export async function main(): Promise<void> {
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult; using device = deviceResult;
    const spec: RenderPipelineSpec = { format: "rgba8unorm" };
    const source: string = "@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> { return vec4<f32>(0.0, 0.0, 0.0, 1.0); } @fragment fn fs() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }";
    using pipeline = createRenderPipeline(device, source, "vs", "fs", [], [], spec);
    using indices = device.createBuffer({ label: "indices", size: 6, usage: GPUBufferUsage.INDEX });
    using target = device.createTexture({ label: "target", size: { width: 1, height: 1 }, format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT });
    using view = target.createView(); using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({ colorAttachments: [{ view, clearValue: {r:0,g:0,b:0,a:1}, loadOp:"clear", storeOp:"store" }] });
    pipeline.setIndexBuffer(pass, indices);
  }
  gpu.dispose();
}
