// example: ui-demo
// Two interactive windows show widgets, a text log, and background color controls.
// This port omits the style editor.
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

// The program declares the pipeline that draws the UI. The generator emits its
// WGSL and layout facts beside this file, and the renderer receives them.
export const uiPipeline: RenderPipelineSpec = renderPipelineL<UiRenderLayout, UiVertex, UiVarying>(
  uiVertex, uiFragment, { format: "bgra8unorm", indexFormat: "uint16", blend: UI_BLEND },
);

// One context for the program's life. It holds the frame's command list, the container
// records, and the widget state that survives between frames.
const ui: UiContext = new UiContext();
// The background color in 0-to-255 channels, the range the sliders below drive.
const red: UiState<f32> = new UiState<f32>(90);
const green: UiState<f32> = new UiState<f32>(95);
const blue: UiState<f32> = new UiState<f32>(100);
// Widget state crosses in holders. microui passes a pointer to the value.
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

// The log keeps the newest 8000 characters. It drops whole lines from the front, so a
// partial line never reaches the panel.
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

// A widget both draws and reports in one call, the immediate-mode contract. `UI_RES_SUBMIT`
// marks the frame on which a left press completes.
function logButton(number: i32): void {
  if ((ui.button(`Button ${number}`) & UI_RES_SUBMIT) !== 0) {
    appendLog(`Pressed button ${number}`);
  }
}

// Two hexadecimal digits per channel. The library carries no format helper, so the demo
// indexes a digit table.
function colorHex(value: f32): string {
  const digits: string = "0123456789ABCDEF";
  const byte: i32 = value as i32;
  return digits.slice(byte / 16, byte / 16 + 1) + digits.slice(byte % 16, byte % 16 + 1);
}

// The first window. `beginWindow` returns zero when the window is closed, and the rect
// argument applies on the first frame only.
function demoWindow(): void {
  if (ui.beginWindow("Demo Window", new UiRect(40, 40, 300, 450)) === 0) return;
  // The container record is live state the program reads and writes. This clamp restores the
  // minimum size on every frame, because a drag can shrink the window below it.
  const window = ui.currentContainer();
  window.rect = new UiRect(window.rect.x, window.rect.y,
    window.rect.w < 240 ? 240 : window.rect.w, window.rect.h < 300 ? 300 : window.rect.h);
  if (ui.header("Window Info") !== 0) {
    // A row of item widths in pixels. A negative width reaches to that many pixels from the
    // right edge, and a height of 0 takes the style height.
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
    // `layoutNext` takes the next item's rect without a widget, so the two draw calls fill
    // that cell with the live color and its hexadecimal text.
    const preview: UiRect = ui.layoutNext();
    ui.drawRect(preview, (red.value as u32) | ((green.value as u32) << 8)
      | ((blue.value as u32) << 16) | 0xff000000);
    ui.drawControlText(`#${colorHex(red.value)}${colorHex(green.value)}${colorHex(blue.value)}`,
      preview, UI_COLOR_TEXT, UI_OPT_ALIGN_CENTER);
  }
  ui.endWindow();
}

// The second window. It shows the log in a scrolled panel and appends a line from the
// textbox or the button.
function logWindow(): void {
  if (ui.beginWindow("Log Window", new UiRect(350, 40, 300, 200)) === 0) return;
  ui.layoutRow([-1], -25);
  ui.beginPanel("Log Output");
  const panel = ui.currentContainer();
  ui.layoutRow([-1], -1);
  ui.text(logText);
  ui.endPanel();
  // The scroll runs after `endPanel`, because the content extent of the panel is known only
  // when the panel closes.
  if (logUpdated) {
    panel.scrollY = panel.contentHeight;
    logUpdated = false;
  }
  // The textbox and the button share one submit path. A return key inside the textbox and a
  // button press both append the text and clear the field.
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
  // The pipeline declares its format literally. A surface with another format
  // ends the example before any draw.
  if (format !== uiPipeline_TARGET_FORMAT) {
    print(`FAIL format expected=${uiPipeline_TARGET_FORMAT} actual=${format}`);
    return;
  }
  // The host owns the device and the instance. The wrapper adds the API-layer surface and
  // disposes neither.
  const deviceWrapper: GPUHostOwnedDevice = hostOwnedGPUDevice(instance, device);
  // The facts carry everything the generator produced for the pipeline. The renderer builds
  // the pipeline, the glyph atlas, and its buffers from them, and it owns each one.
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
  // The UI takes whole pixels, so the fraction stays in the accumulator for the next event.
  const x: i32 = wheelX as i32;
  const y: i32 = wheelY as i32;
  ui.inputScroll(x, y);
  wheelX -= x as f32;
  wheelY -= y as f32;
}

// The host queues key and text events between frames and delivers them in event order. A
// script that exports none of these entries receives none of those events.
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
  // A failed `init` leaves the module state empty. The layers carry no exceptions, so a null
  // check ends the frame.
  const activeDevice: GPUHostOwnedDevice | null = ownedDevice;
  const activeRenderer: UiRenderer | null = renderer;
  if (activeDevice === null) return;
  if (activeRenderer === null) return;
  const x: i32 = pointerX as i32;
  const y: i32 = pointerY as i32;
  // The host reports the pointer in surface pixels, the space the UI layout uses.
  ui.inputMouseMove(x, y);
  // The host reports buttons as level state. microui's SDL loop receives press
  // and release events, so this loop derives the edges.
  for (let bit: u32 = 1; bit <= 4; bit *= 2) {
    if ((buttons & bit) !== 0 && (previousButtons & bit) === 0) ui.inputMouseDown(x, y, bit);
    if ((buttons & bit) === 0 && (previousButtons & bit) !== 0) ui.inputMouseUp(x, y, bit);
  }
  previousButtons = buttons;
  // One frame of UI. `begin` clears the command list, the two window functions fill it, and
  // `end` orders the windows by z-index and resolves the hover target.
  ui.begin();
  demoWindow();
  logWindow();
  ui.end();
  // The clear color comes from the three sliders, so the demo paints its own state behind
  // the windows.
  using encoder = activeDevice.createCommandEncoderDefault();
  using pass = encoder.beginRenderPass({ colorAttachments: [{
    view: new GPUTextureView(view),
    clearValue: { r: (red.value as f64) / 255.0, g: (green.value as f64) / 255.0, b: (blue.value as f64) / 255.0, a: 1.0 },
    loadOp: "clear", storeOp: "store",
  }] });
  // The renderer writes its vertex and index buffers, then records one scissored indexed draw
  // per clip rect. microui's demo walks the same command list in its own renderer.
  activeRenderer.render(ui, pass, width, height);
  pass.end();
  // The GPU runs nothing until the queue receives the command buffer. The host presents the
  // surface after this call returns.
  using command = encoder.finishDefault();
  using queue = activeDevice.queue();
  queue.submit([command]);
}

// The host calls this one time before it releases the device. `dispose` releases the
// pipeline, the atlas, and the buffers, and the host owns the device the wrapper held.
export function shutdown(): void {
  if (renderer !== null) {
    renderer.dispose();
    renderer = null;
  }
  ownedDevice = null;
}
