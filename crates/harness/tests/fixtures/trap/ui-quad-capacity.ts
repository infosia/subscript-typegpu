// expected-rule: UIT1
import { UiContext, UiRect, UiRenderer, UiPipelineFacts, UiRenderLayout, UiVertex, UiVarying, uiVertex, uiFragment, UI_BLEND } from "./typegpu-ui";
import { gpu, GPUAdapter, GPUDevice } from "./webgpu";
import { RenderPipelineSpec, renderPipelineL } from "./typegpu";
import {
  UiVertex_STRIDE, UiViewport_STRIDE, uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY,
  uiPipeline_FRAGMENT_ENTRY, uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, uiPipeline_TARGET_FORMAT,
} from "./ui-quad-capacity.typegpu";

export const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "rgba8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

async function exercise(capacity: u32): Promise<void> {
  const adapter: GPUAdapter | null = await gpu.requestAdapter();
  if (adapter === null) return;
  const device: GPUDevice | null = await adapter.requestDevice();
  if (device === null) return;
  const facts: UiPipelineFacts = new UiPipelineFacts(
    uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY, uiPipeline_FRAGMENT_ENTRY,
    uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, UiVertex_STRIDE, UiViewport_STRIDE, uiPipeline_TARGET_FORMAT,
  );
  using renderer = new UiRenderer(device, facts, capacity);
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.drawRect(new UiRect(0, 0, 10, 10), 0xffffffff);
  ui.drawRect(new UiRect(10, 0, 10, 10), 0xffffffff);
  ui.end();
  renderer.build(ui);
}

export async function main(): Promise<void> { await exercise(1); }
export async function zeroCapacity(): Promise<void> { await exercise(0); }
export async function excessiveCapacity(): Promise<void> { await exercise(16385); }
