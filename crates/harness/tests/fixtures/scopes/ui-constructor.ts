import { UiRenderer, UiVertex, UiViewport } from "./typegpu-ui";
import { Buffer, RenderPipeline } from "./typegpu";
import { GPUQueue, GPUTexture, GPUTextureView, GPUSampler, GPUBindGroup } from "./webgpu";

export function main(queue: GPUQueue, vertices: Buffer<UiVertex>, indices: Buffer<u16>,
  atlas: GPUTexture, view: GPUTextureView, sampler: GPUSampler, viewport: Buffer<UiViewport>,
  group: GPUBindGroup, pipeline: RenderPipeline): void {
  const renderer = new UiRenderer(queue, false, 1, vertices, indices, atlas, view,
    sampler, viewport, group, pipeline);
}
