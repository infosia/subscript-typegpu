// expected-rule: UIT1
import { UiContext, UiRect, UiRenderer, UiPipelineFacts, UiRenderLayout, UiVertex, UiVarying, uiVertex, uiFragment, UI_BLEND } from "./typegpu-ui";
import { gpu, GPUAdapter, GPUDevice } from "./webgpu";
import { RenderPipelineSpec, renderPipelineL } from "./typegpu";
import {
  uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY,
  uiPipeline_FRAGMENT_ENTRY, uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0,
} from "./ui-quad-capacity.typegpu";

export const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "rgba8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

async function exercise(capacity: u32, spec: RenderPipelineSpec): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) return;
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) return;
  const facts: UiPipelineFacts = new UiPipelineFacts(
    uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY, uiPipeline_FRAGMENT_ENTRY,
    uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, spec,
  );
  using renderer = new UiRenderer(device, facts, capacity);
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.drawRect(new UiRect(0, 0, 10, 10), 0xffffffff);
  ui.drawRect(new UiRect(10, 0, 10, 10), 0xffffffff);
  ui.end();
  renderer.build(ui);
}

export async function main(): Promise<void> { await exercise(1, uiPipeline); }
export async function zeroCapacity(): Promise<void> { await exercise(0, uiPipeline); }
export async function excessiveCapacity(): Promise<void> { await exercise(16385, uiPipeline); }

export async function stripTopology(): Promise<void> {
  uiPipeline.topology = "triangle-strip";
  await exercise(64, uiPipeline);
}
export async function wideIndexFormat(): Promise<void> {
  uiPipeline.indexFormat = "uint32";
  await exercise(64, uiPipeline);
}
