// example: ui-demo
// Two interactive windows show widgets, a text log, and background color controls.
// The style editor is omitted.
// Ported from microui's demo (https://github.com/rxi/microui/blob/0850aba860959c3e75fb3e97120ca92957f9d057/demo/main.c).

import {
  UiContext, UiRect, UiState, UiRenderer, UiPipelineFacts,
  UiRenderLayout, UiVertex, UiVarying, uiVertex, uiFragment,
  UI_BLEND, UI_OPT_EXPANDED, UI_RES_SUBMIT, UI_COLOR_TEXT, UI_OPT_ALIGN_CENTER,
} from "./typegpu-ui";
import { RenderPipelineSpec, renderPipelineL } from "./typegpu";
import { GPUHostOwnedDevice, GPUTextureView, hostOwnedGPUDevice } from "./webgpu";
import {
  uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY, uiPipeline_FRAGMENT_ENTRY,
  uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, uiPipeline_TARGET_FORMAT,
} from "./main.typegpu";

export const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "bgra8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

const ui: UiContext = new UiContext();
const red: UiState<f32> = new UiState<f32>(90);
const green: UiState<f32> = new UiState<f32>(95);
const blue: UiState<f32> = new UiState<f32>(100);
const checks: UiState<boolean>[] = [
  new UiState<boolean>(true), new UiState<boolean>(false), new UiState<boolean>(true),
];
const input: UiState<string> = new UiState<string>("");
let logText: string = "";
let logUpdated: boolean = false;
let wheelX: f32 = 0;
let wheelY: f32 = 0;
let previousButtons: u32 = 0;
let ownedDevice: GPUHostOwnedDevice | null = null;
let renderer: UiRenderer | null = null;

function appendLog(message: string): void {
  if (logText.length !== 0) logText += "\n";
  logText += message;
  while (logText.length > 8000) {
    let end: i32 = 0;
    while (end < logText.length && logText.charCodeAt(end) !== 10) end += 1;
    logText = logText.slice(end < logText.length ? end + 1 : end);
  }
  logUpdated = true;
}

function logButton(number: i32): void {
  if ((ui.button(`Button ${number}`) & UI_RES_SUBMIT) !== 0) {
    appendLog(`Pressed button ${number}`);
  }
}

function colorHex(value: f32): string {
  const digits: string = "0123456789ABCDEF";
  const byte: i32 = value as i32;
  return digits.slice(byte / 16, byte / 16 + 1) + digits.slice(byte % 16, byte % 16 + 1);
}

function demoWindow(): void {
  if (ui.beginWindow("Demo Window", new UiRect(40, 40, 300, 450)) === 0) return;
  const window = ui.currentContainer();
  window.rect = new UiRect(window.rect.x, window.rect.y,
    window.rect.w < 240 ? 240 : window.rect.w, window.rect.h < 300 ? 300 : window.rect.h);
  if (ui.header("Window Info") !== 0) {
    ui.layoutRow([54, -1], 0);
    ui.label("Position:");
    ui.label(`${window.rect.x}, ${window.rect.y}`);
    ui.label("Size:");
    ui.label(`${window.rect.w}, ${window.rect.h}`);
  }
  if (ui.header("Test Buttons", UI_OPT_EXPANDED) !== 0) {
    ui.layoutRow([86, -110, -1], 0);
    ui.label("Test buttons 1:");
    logButton(1);
    logButton(2);
    ui.label("Test buttons 2:");
    logButton(3);
    if ((ui.button("Popup") & UI_RES_SUBMIT) !== 0) ui.openPopup("Test Popup");
    if (ui.beginPopup("Test Popup") !== 0) {
      ui.button("Hello");
      ui.button("World");
      ui.endPopup();
    }
  }
  if (ui.header("Tree and Text", UI_OPT_EXPANDED) !== 0) {
    ui.layoutRow([140, -1], 0);
    ui.layoutBeginColumn();
    if (ui.beginTreenode("Test 1") !== 0) {
      if (ui.beginTreenode("Test 1a") !== 0) {
        ui.label("Hello");
        ui.label("world");
        ui.endTreenode();
      }
      if (ui.beginTreenode("Test 1b") !== 0) {
        logButton(1);
        logButton(2);
        ui.endTreenode();
      }
      ui.endTreenode();
    }
    if (ui.beginTreenode("Test 2") !== 0) {
      ui.layoutRow([54, 54], 0);
      for (let number: i32 = 3; number <= 6; number += 1) logButton(number);
      ui.endTreenode();
    }
    if (ui.beginTreenode("Test 3") !== 0) {
      for (let index: i32 = 0; index < 3; index += 1) {
        ui.checkbox(`Checkbox ${index + 1}`, checks[index]);
      }
      ui.endTreenode();
    }
    ui.layoutEndColumn();
    ui.layoutBeginColumn();
    ui.layoutRow([-1], 0);
    ui.text("Open a tree to explore its controls. Buttons add messages to the log. Drag a slider to change the background.");
    ui.layoutEndColumn();
  }
  if (ui.header("Background Color", UI_OPT_EXPANDED) !== 0) {
    ui.layoutRow([-78, -1], 74);
    ui.layoutBeginColumn();
    ui.layoutRow([46, -1], 0);
    ui.label("Red:");
    ui.slider("red", red, 0, 255);
    ui.label("Green:");
    ui.slider("green", green, 0, 255);
    ui.label("Blue:");
    ui.slider("blue", blue, 0, 255);
    ui.layoutEndColumn();
    const preview: UiRect = ui.layoutNext();
    ui.drawRect(preview, (red.value as u32) | ((green.value as u32) << 8)
      | ((blue.value as u32) << 16) | 0xff000000);
    ui.drawControlText(`#${colorHex(red.value)}${colorHex(green.value)}${colorHex(blue.value)}`,
      preview, UI_COLOR_TEXT, UI_OPT_ALIGN_CENTER);
  }
  ui.endWindow();
}

function logWindow(): void {
  if (ui.beginWindow("Log Window", new UiRect(350, 40, 300, 200)) === 0) return;
  ui.layoutRow([-1], -25);
  ui.beginPanel("Log Output");
  const panel = ui.currentContainer();
  ui.layoutRow([-1], -1);
  ui.text(logText);
  ui.endPanel();
  if (logUpdated) {
    panel.scrollY = panel.contentHeight;
    logUpdated = false;
  }
  ui.layoutRow([-70, -1], 0);
  let submitted: boolean = (ui.textbox("message", input) & UI_RES_SUBMIT) !== 0;
  if (submitted) ui.setFocus(ui.lastId);
  if ((ui.button("Submit") & UI_RES_SUBMIT) !== 0) submitted = true;
  if (submitted) {
    appendLog(input.value);
    input.value = "";
  }
  ui.endWindow();
}

export function init(
  instance: SubscriptTypegpuInstance,
  device: SubscriptTypegpuDevice,
  format: GPUTextureFormat,
): void {
  if (format !== uiPipeline_TARGET_FORMAT) {
    print(`FAIL format expected=${uiPipeline_TARGET_FORMAT} actual=${format}`);
    return;
  }
  const deviceWrapper: GPUHostOwnedDevice = hostOwnedGPUDevice(instance, device);
  const facts: UiPipelineFacts = new UiPipelineFacts(
    uiPipeline_WGSL, uiPipeline_VERTEX_ENTRY, uiPipeline_FRAGMENT_ENTRY,
    uiPipeline_LAYOUT0, uiPipeline_VERTEX_LAYOUT0, uiPipeline,
  );
  ownedDevice = deviceWrapper;
  renderer = UiRenderer.createHost(deviceWrapper, facts);
}

export function wheel(deltaX: f32, deltaY: f32): void {
  // Host wheel deltas describe motion. The UI scroll offset moves in the opposite direction.
  wheelX -= deltaX;
  wheelY -= deltaY;
  const x: i32 = wheelX as i32;
  const y: i32 = wheelY as i32;
  ui.inputScroll(x, y);
  wheelX -= x as f32;
  wheelY -= y as f32;
}

export function keyDown(key: u32): void { ui.inputKeyDown(key); }
export function keyUp(key: u32): void { ui.inputKeyUp(key); }
export function textInput(codePoint: u32): void { ui.inputText(codePoint); }

export function frame(
  view: SubscriptTypegpuTextureView,
  width: u32,
  height: u32,
  key: u32,
  pointerX: f32,
  pointerY: f32,
  buttons: u32,
): void {
  const activeDevice: GPUHostOwnedDevice | null = ownedDevice;
  const activeRenderer: UiRenderer | null = renderer;
  if (activeDevice === null) return;
  if (activeRenderer === null) return;
  const x: i32 = pointerX as i32;
  const y: i32 = pointerY as i32;
  ui.inputMouseMove(x, y);
  for (let bit: u32 = 1; bit <= 4; bit *= 2) {
    if ((buttons & bit) !== 0 && (previousButtons & bit) === 0) ui.inputMouseDown(x, y, bit);
    if ((buttons & bit) === 0 && (previousButtons & bit) !== 0) ui.inputMouseUp(x, y, bit);
  }
  previousButtons = buttons;
  ui.begin();
  demoWindow();
  logWindow();
  ui.end();
  using encoder = activeDevice.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({ colorAttachments: [{
    view: new GPUTextureView(view),
    clearValue: { r: (red.value as f64) / 255.0, g: (green.value as f64) / 255.0, b: (blue.value as f64) / 255.0, a: 1.0 },
    loadOp: "clear", storeOp: "store",
  }] });
  activeRenderer.render(ui, pass, width, height);
  pass.end();
  using command = encoder.finishDefault();
  using queue = activeDevice.queue();
  queue.submit([command]);
}

export function shutdown(): void {
  if (renderer !== null) {
    renderer.dispose();
    renderer = null;
  }
  ownedDevice = null;
}
