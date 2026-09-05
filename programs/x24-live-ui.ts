// program: x24-live-ui
// purpose: compare opaque UI title, body, and button pixels with exact packed default colors
// exercises: UI14, UI15, UI16, BF1, BF2, RN11, RN18, RN21, TX9, T4, T15
// questions: none

import { UiContext, UiRect, UiRenderer, UiPipelineFacts, UiRenderLayout, UiVertex, UiVarying, uiVertex, uiFragment, UI_BLEND } from "./typegpu-ui";
import { gpu, GPUAdapter, GPUBufferUsage, GPUDevice, GPUMapMode, GPUTextureUsage } from "./webgpu";

import { RenderPipelineSpec, renderPipelineL } from "./typegpu";
import {
  uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY,
  uiPipeline_FRAGMENT_ENTRY, uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0,
} from "./x24-live-ui.typegpu";

export const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "rgba8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

export async function main(): Promise<void> {
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.beginWindow("UI", new UiRect(16, 16, 224, 224));
  ui.layoutRow([100], 24);
  ui.button("Button");
  ui.endWindow();
  ui.end();
  const adapterResult: GPUAdapter | null = await gpu.requestAdapter();
  if (adapterResult === null) { print("FAIL adapter"); gpu.dispose(); return; }
  const deviceResult: GPUDevice | null = await adapterResult.requestDevice();
  if (deviceResult === null) { print("FAIL device"); adapterResult.dispose(); gpu.dispose(); return; }
  {
    using adapter = adapterResult;
    using device = deviceResult;
    device.pushErrorScope("validation");
    const facts: UiPipelineFacts = new UiPipelineFacts(
      uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY, uiPipeline_FRAGMENT_ENTRY,
      uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, uiPipeline,
    );
    using renderer = UiRenderer.create(device, facts, 64);
    const error = await device.popErrorScope();
    if (error !== null) { print(`FAIL validation ${error.message.split("\n")[0]}`); return; }
    if (uiPipeline_WGSL.length === 0) { print("FAIL shader"); return; }
    using target = device.createTexture({
      label: "x24-target", size: { width: 256, height: 256 }, format: "rgba8unorm",
      usage: GPUTextureUsage.RENDER_ATTACHMENT + GPUTextureUsage.COPY_SRC,
    });
    using view = target.createView();
    using readback = device.createBuffer({
      label: "x24-readback", size: 262144,
      usage: GPUBufferUsage.MAP_READ + GPUBufferUsage.COPY_DST,
    });
    using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({ colorAttachments: [{
      view, clearValue: { r: 0, g: 0, b: 0, a: 0 }, loadOp: "clear", storeOp: "store",
    }] });
    renderer.render(ui, pass, 256, 256);
    pass.end();
    encoder.copyTextureToBuffer(
      { texture: target }, { buffer: readback, bytesPerRow: 1024, rowsPerImage: 256 },
      { width: 256, height: 256, depthOrArrayLayers: 1 },
    );
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await device.queue.onSubmittedWorkDone()) { print("FAIL submit"); return; }
    if (!await readback.mapAsync(GPUMapMode.READ, 0, 262144)) { print("FAIL map"); return; }
    const pixels: u8[] = readback.readMappedRange(0, 262144);
    // These patches exclude glyphs, borders, and the close icon.
    const patches: UiRect[] = [new UiRect(80, 20, 32, 12), new UiRect(140, 100, 32, 32), new UiRect(100, 49, 16, 16)];
    const colors: u32[] = [0xff191919, 0xff323232, 0xff4b4b4b];
    let compared: u32 = 0;
    for (let i: i32 = 0; i < patches.length; i += 1) {
      const patch: UiRect = patches[i];
      for (let y: i32 = patch.y; y < patch.y + patch.h; y += 1) {
        for (let x: i32 = patch.x; x < patch.x + patch.w; x += 1) {
          const offset: i32 = y * 1024 + x * 4;
          const got: u32 = (pixels[offset] as u32) + (pixels[offset + 1] as u32) * 256
            + (pixels[offset + 2] as u32) * 65536 + (pixels[offset + 3] as u32) * 16777216;
          if (got !== colors[i]) {
            print(`FAIL x=${x} y=${y} expected=${colors[i]} got=${got}`);
            readback.unmap();
            return;
          }
          compared += 1;
        }
      }
    }
    readback.unmap();
    print(`pixels:compared=${compared}`);
  }
  gpu.dispose();
  print("PASS");
}
