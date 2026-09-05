// program: b24-ui-render
// purpose: prove UI vertex bytes, clip ranges, and an indexed offscreen render submission
// exercises: UI14, UI15, UI19, BF1, BF2, RN11, RN18, RN21, TX9
// questions: none

import { UiContext, UiRoot, UiRect, UiRenderer, UiPipelineFacts, UiRenderLayout, UiVertex, UiVarying, uiVertex, uiFragment, UI_BLEND } from "./typegpu-ui";
import { gpu, GPUAdapter, GPUDevice, GPUTextureUsage } from "./webgpu";

import { Vec2f } from "./typegpu-types";
import { RenderPipelineSpec, renderPipelineL } from "./typegpu";
import {
  uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY,
  uiPipeline_FRAGMENT_ENTRY, uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0,
} from "./b24-ui-render.typegpu";

export const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "rgba8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

function check(name: string, value: boolean): void { print(`${name} ${value}`); }

function checkColorClip(renderer: UiRenderer, name: string, color: u32, x: i32, y: i32): void {
  const stride: u32 = Context.bytesOf<UiVertex>(new UiVertex(new Vec2f(0, 0), new Vec2f(0, 0), 0)).length as u32;
  for (let q: u32 = 0; q < renderer.quadCount; q += 1) {
    const vertex: UiVertex = Context.fromBytes<UiVertex>(renderer.vertexBytes, q * 4 * stride);
    if (vertex.color !== color) continue;
    for (let i: i32 = 0; i < renderer.rangeCount; i += 1) {
      const range = renderer.ranges[i];
      if (q * 6 < range.first || q * 6 >= range.first + range.count) continue;
      check(name, x >= range.clip.x && x < range.clip.x + range.clip.w
        && y >= range.clip.y && y < range.clip.y + range.clip.h);
      return;
    }
  }
  unreachable();
}

function checkFrames(renderer: UiRenderer): void {
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.pushClip(new UiRect(0, 0, 10, 10));
  ui.drawRect(new UiRect(0, 0, 20, 20), 0xff000001);
  ui.popClip();
  ui.drawRect(new UiRect(20, 0, 10, 10), 0xff000002);
  ui.end();
  check("clip commands", ui.commands[0].kind === 1 && ui.commands[0].w === 10
    && ui.commands[2].kind === 1 && ui.commands[2].w === 16777216);
  renderer.build(ui);
  check("clip exit counts", renderer.quadCount === 2 && renderer.indexCount === 12);
  check("clip exit ranges", renderer.rangeCount === 2 && renderer.ranges[0].clip.w === 10);
  checkColorClip(renderer, "clip exit", 0xff000002, 25, 5);

  ui.begin();
  ui.drawRect(new UiRect(0, 0, 1, 1), 0xff000009);
  ui.beginRoot(new UiRoot(1, new UiRect(0, 0, 100, 100), 2));
  ui.pushClip(new UiRect(0, 0, 10, 10));
  ui.drawRect(new UiRect(0, 0, 20, 20), 0xff000001);
  ui.popClip();
  ui.beginRoot(new UiRoot(2, new UiRect(100, 0, 100, 100), 1));
  ui.drawRect(new UiRect(120, 0, 10, 10), 0xff000002);
  ui.endRoot();
  ui.drawRect(new UiRect(20, 0, 10, 10), 0xff000003);
  ui.endRoot();
  ui.end();
  renderer.build(ui);
  check("nested roots and orphan commands", renderer.quadCount === 3);
  const first: UiVertex = Context.fromBytes<UiVertex>(renderer.vertexBytes, 0);
  check("root order", first.color === 0xff000002);
  checkColorClip(renderer, "nested parent exit", 0xff000003, 25, 5);
  checkColorClip(renderer, "nested child", 0xff000002, 125, 5);

  ui.begin();
  ui.beginWindow("panel test", new UiRect(0, 0, 200, 200));
  ui.layoutRow([100], 40);
  ui.beginPanel("panel");
  ui.drawRect(new UiRect(0, 20, 200, 200), 0xff000001);
  ui.endPanel();
  ui.drawRect(new UiRect(150, 100, 10, 10), 0xff000002);
  ui.endWindow();
  ui.beginWindow("second", new UiRect(220, 0, 200, 200));
  ui.drawRect(new UiRect(250, 100, 10, 10), 0xff000003);
  ui.endWindow();
  ui.end();
  renderer.build(ui);
  checkColorClip(renderer, "panel exit", 0xff000002, 155, 105);
  checkColorClip(renderer, "second window", 0xff000003, 255, 105);

  const bytes = renderer.vertexBytes;
  const ranges = renderer.ranges;
  const firstRange = renderer.ranges[0];
  const byteCapacity: i32 = bytes.length;
  const rangeCapacity: i32 = ranges.length;
  ui.begin(); ui.end();
  renderer.build(ui);
  check("empty frame counts", renderer.quadCount === 0 && renderer.indexCount === 0);
  check("empty frame storage", renderer.vertexBytes.length === byteCapacity && renderer.ranges.length === rangeCapacity
    && renderer.rangeCount === 0);
  ui.begin(); ui.drawRect(new UiRect(2, 3, 4, 5), 0xff000007); ui.end();
  renderer.build(ui);
  const reused: UiVertex = Context.fromBytes<UiVertex>(bytes, 0);
  check("frame storage reuse", renderer.vertexBytes.length === byteCapacity
    && ranges.length === rangeCapacity && renderer.ranges[0] === firstRange && renderer.rangeCount === 1
    && renderer.ranges[0].first === 0 && renderer.ranges[0].count === 6
    && renderer.quadCount === 1 && renderer.indexCount === 6
    && reused.position.x === 2.0 && reused.color === 0xff000007);
}

export async function main(): Promise<void> {
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.beginWindow("UI", new UiRect(8, 8, 180, 160));
  ui.layoutRow([100], 20);
  ui.button("Button");
  ui.label("Label");
  ui.layoutRow([100], 40);
  ui.beginPanel("panel");
  ui.drawRect(new UiRect(4, 85, 120, 35), 0xff804020);
  ui.endPanel();
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
    if (error !== null) { print("pipeline:invalid"); print("FAIL"); return; }
    if (uiPipeline_WGSL.length === 0) { print("FAIL shader"); return; }
    using target = device.createTexture({
      label: "b24-target", size: { width: 256, height: 256 }, format: "rgba8unorm",
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
    using view = target.createView();
    using encoder = device.createCommandEncoderDefault();
    using pass = encoder.beginRenderPass({ colorAttachments: [{
      view, clearValue: { r: 0, g: 0, b: 0, a: 0 }, loadOp: "clear", storeOp: "store",
    }] });
    renderer.render(ui, pass, 256, 256);
    print(`quads ${renderer.quadCount}`);
    print(`indices ${renderer.indexCount}`);
    for (let i: i32 = 0; i < renderer.rangeCount; i += 1) {
      const range = renderer.ranges[i];
      print(`range ${range.first} ${range.count} clip ${range.clip.x} ${range.clip.y} ${range.clip.w} ${range.clip.h}`);
    }
    let checksum: u32 = 2166136261;
    const stride: u32 = Context.bytesOf<UiVertex>(new UiVertex(new Vec2f(0, 0), new Vec2f(0, 0), 0)).length as u32;
    const byteCount: i32 = (renderer.quadCount * 4 * stride) as i32;
    for (let i: i32 = 0; i < byteCount; i += 1) {
      checksum = (checksum ^ (renderer.vertexBytes[i] as u32)) * 16777619;
    }
    print(`vertices fnv1a ${checksum}`);
    ui.begin(); ui.end();
    renderer.render(ui, pass, 256, 256);
    check("empty render", renderer.quadCount === 0 && renderer.indexCount === 0 && renderer.rangeCount === 0);
    pass.end();
    using command = encoder.finishDefault();
    device.queue.submit([command]);
    if (!await device.queue.onSubmittedWorkDone()) { print("FAIL submit"); return; }
    checkFrames(renderer);
  }
  gpu.dispose();
  print("PASS");
}
