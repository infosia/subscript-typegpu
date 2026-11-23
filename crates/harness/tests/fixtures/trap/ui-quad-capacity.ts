// expected-rule: UIT1
import { UiContext, UiRect, UiRenderer, UiPipelineFacts, UiRenderLayout, UiVertex, UiVarying, uiVertex, uiFragment, uiScissor, UI_BLEND } from "./typegpu-ui";
import { gpu, GPUAdapter, GPUDevice, GPUTextureUsage } from "./webgpu";
import { RenderPipelineSpec, renderPipelineL } from "./typegpu";
import {
  uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY,
  uiPipeline_FRAGMENT_ENTRY, uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0,
} from "./ui-quad-capacity.typegpu";

const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "rgba8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

async function exercise(capacity: u32, spec: RenderPipelineSpec, scale: f32 = 1): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter == null) return;
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device == null) return;
  const facts: UiPipelineFacts = new UiPipelineFacts(
    uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY, uiPipeline_FRAGMENT_ENTRY,
    uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, spec,
  );
  using renderer = UiRenderer.create(device, facts, capacity);
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.drawRect(new UiRect(0, 0, 10, 10), 0xffffffff);
  ui.drawRect(new UiRect(10, 0, 10, 10), 0xffffffff);
  ui.end();
  using target = device.createTexture({
    size: { width: 20, height: 10 }, format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT,
  });
  using view = target.createView();
  using encoder = device.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({ colorAttachments: [{
    view, clearValue: { r: 0, g: 0, b: 0, a: 0 }, loadOp: "clear", storeOp: "store",
  }] });
  renderer.render(ui, pass, 20, 10, scale);
  pass.end();
}

export async function main(): Promise<void> { await exercise(1, uiPipeline); }
export async function zeroCapacity(): Promise<void> { await exercise(0, uiPipeline); }
export async function excessiveCapacity(): Promise<void> { await exercise(16385, uiPipeline); }
export async function zeroScale(): Promise<void> { await exercise(64, uiPipeline, 0); }

export async function nonFiniteScale(): Promise<void> { await exercise(64, uiPipeline, 0.0 / 0.0); }
export function zeroScissorScale(): void {
  uiScissor(new UiRect(0, 0, 10, 10), 20, 10, 0);
}

export async function stripTopology(): Promise<void> {
  uiPipeline.topology = "triangle-strip";
  await exercise(64, uiPipeline);
}
export async function wideIndexFormat(): Promise<void> {
  uiPipeline.indexFormat = "uint32";
  await exercise(64, uiPipeline);
}
