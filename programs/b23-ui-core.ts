// program: b23-ui-core
// purpose: Exercise host UI commands and input across consecutive frames.
// exercises: UI2, UI3, UI4, UI5, UI6, UI7, UI8, UI9, UI10, UI11, UI12, UI13, UI19
// questions: none

import { uiNumberText, UiContext, UiRect, UiRoot, UiCommand, UiState, UiStyle,
  UI_OPT_ALIGN_CENTER, UI_OPT_ALIGN_RIGHT, UI_OPT_HOLD_FOCUS, UI_OPT_NO_INTERACT, UI_MOUSE_LEFT, UI_MOUSE_RIGHT,
  UI_KEY_SHIFT, UI_KEY_RETURN, UI_ICON_CHECK, UI_COLOR_TEXT } from "./typegpu-ui";
import { uiAtlasAlpha, UI_ATLAS_ALPHA_CHUNK, UI_ATLAS_ALPHA_HEX, UI_ATLAS_WIDTH, UI_ATLAS_HEIGHT,
  UI_ATLAS_RECT_X, UI_ATLAS_RECT_Y, UI_ATLAS_RECT_W, UI_ATLAS_RECT_H,
  UI_ATLAS_FONT, UI_ATLAS_WHITE, UI_TEXT_HEIGHT } from "./typegpu-ui-atlas.generated";


export function main(): void {
  verifyCore();
  verifyWidgets();
  verifyContainers();
  const ui: UiContext = new UiContext();
  const checked: UiState<boolean> = new UiState<boolean>(true);
  const slider: UiState<f32> = new UiState<f32>(10);
  const text: UiState<string> = new UiState<string>("");
  const widths: i32[] = [-1];
  const window: UiRect = new UiRect(10, 10, 300, 400);
  for (let frame: i32 = 1; frame <= 16; frame += 1) {
    if (frame === 1) ui.inputMouseMove(0, 0);
    if (frame === 2) ui.inputMouseMove(30, 49);
    if (frame === 3) ui.inputMouseDown(30, 49, UI_MOUSE_LEFT);
    if (frame === 4) ui.inputMouseUp(30, 49, UI_MOUSE_LEFT);
    if (frame === 5) ui.inputMouseMove(44, 97);
    if (frame === 6) ui.inputMouseDown(44, 97, UI_MOUSE_LEFT);
    if (frame === 7) ui.inputMouseMove(54, 97);
    if (frame === 8) ui.inputMouseMove(64, 97);
    if (frame === 9) ui.inputMouseUp(64, 97, UI_MOUSE_LEFT);
    if (frame === 10) ui.inputMouseMove(35, 174);
    if (frame === 11) ui.inputMouseDown(35, 174, UI_MOUSE_LEFT);
    if (frame === 12) ui.inputMouseUp(35, 174, UI_MOUSE_LEFT);
    if (frame === 13) ui.inputText(65);
    if (frame === 14) ui.inputMouseMove(35, 145);
    if (frame === 15) ui.inputMouseDown(35, 145, UI_MOUSE_LEFT);
    if (frame === 16) ui.inputMouseUp(35, 145, UI_MOUSE_LEFT);
    ui.begin();
    let button: u32 = 0;
    let checkbox: u32 = 0;
    let slide: u32 = 0;
    let textbox: u32 = 0;
    let tree: u32 = 0;
    if (ui.beginWindow("Core", window) !== 0) {
      ui.layoutRow(widths, 0);
      button = ui.button("Button");
      checkbox = ui.checkbox("Checkbox", checked);
      slide = ui.slider("Slider", slider, 0, 100, 1);
      ui.label("Label");
      tree = ui.beginTreenode("Tree");
      if (tree !== 0) { ui.label("Inside"); ui.endTreenode(); }
      ui.layoutRow(widths, 100);
      ui.beginPanel("Panel");
      ui.layoutRow(widths, 0);
      textbox = ui.textbox("Text", text);
      ui.endPanel();
      ui.endWindow();
    }
    ui.end();
    print(`frame ${frame}`);
    const lines: string[] = ui.dumpCommands();
    for (let i: i32 = 0; i < lines.length; i += 1) print(lines[i]);
    print(`state button=${button} checkbox=${checked.value} slider=${uiNumberText(slider.value)} text="${text.value}" tree=${tree} checkboxResponse=${checkbox} sliderResponse=${slide} textboxResponse=${textbox}`);
  }
  verifyScrollbars();
  print("PASS");
}


function check(value: boolean, name: string): void {
  if (!value) { print(`FAIL ${name}`); unreachable(); }
}
function rect(r: UiRect, x: i32, y: i32, w: i32, h: i32): boolean {
  return r.x === x && r.y === y && r.w === w && r.h === h;
}
function verifyCore(): void {
  const alpha: u8[] = uiAtlasAlpha();
  let hexLength: i32 = 0;
  for (let i: i32 = 0; i < UI_ATLAS_ALPHA_HEX.length; i += 1) {
    check(UI_ATLAS_ALPHA_HEX[i].length === UI_ATLAS_ALPHA_CHUNK, "atlas chunk size");
    hexLength += UI_ATLAS_ALPHA_HEX[i].length;
  }
  check(alpha.length === UI_ATLAS_WIDTH * UI_ATLAS_HEIGHT && hexLength === 32768
    && UI_ATLAS_ALPHA_CHUNK === 4096 && UI_ATLAS_ALPHA_HEX.length === 8, "atlas size");
  check(UI_ATLAS_RECT_X.length === 100 && UI_ATLAS_RECT_Y.length === 100
    && UI_ATLAS_RECT_W.length === 100 && UI_ATLAS_RECT_H.length === 100, "atlas rects");
  check(UI_ATLAS_RECT_X[UI_ATLAS_WHITE] === 125 && UI_ATLAS_RECT_Y[UI_ATLAS_WHITE] === 68
    && alpha[68 * 128 + 125] === 255 && alpha[16383] === 0, "atlas pixels");
  check(UI_ATLAS_RECT_W[UI_ATLAS_FONT + 65] === 7 && UI_TEXT_HEIGHT === 18, "glyph A");
  let checksum: u32 = 2166136261;
  for (let i: i32 = 0; i < alpha.length; i += 1) checksum = (checksum ^ (alpha[i] as u32)) * 16777619;
  check(checksum === 4088512439, "atlas checksum");
  const context: UiContext = new UiContext();
  const holder: UiState<i32> = new UiState<i32>(7);
  holder.value += 1;
  check(holder.value === 8, "holder");
  const style: UiStyle = context.style;
  check(style.width === 68 && style.height === 10 && style.padding === 5 && style.spacing === 4
    && style.indent === 24 && style.titleHeight === 24 && style.scrollbarSize === 12 && style.thumbSize === 8
    && style.colors[UI_COLOR_TEXT] === 0xffe6e6e6 && style.colors.length === 14, "style");
  check(context.getId("") === 2166136261 && context.getId("hello") === 1335831723, "fnv");
  context.begin();
  context.pushId("a");
  const scoped: u32 = context.getId("b");
  context.popId();
  check(scoped === context.getId("ab") && scoped !== context.getId("b"), "id scope");
  context.end();
  context.inputMouseMove(10, 20);
  context.inputMouseDown(12, 22, UI_MOUSE_LEFT);
  context.inputMouseDown(12, 22, UI_MOUSE_RIGHT);
  context.inputMouseUp(13, 23, UI_MOUSE_RIGHT);
  context.inputKeyDown(UI_KEY_SHIFT);
  context.inputKeyDown(UI_KEY_RETURN);
  context.inputKeyUp(UI_KEY_RETURN);
  context.inputText(31); context.inputText(32); context.inputText(65); context.inputText(126); context.inputText(127);
  context.inputScroll(2, 3); context.inputScroll(-1, 4);
  context.begin();
  check(context.mouseDeltaX === 13 && context.mouseDeltaY === 23 && context.mouseDown === 1
    && context.mousePressed === 3 && context.keyDown === 1 && context.keyPressed === 17
    && context.textInput === " A~" && context.scrollX === 1 && context.scrollY === 7, "input edges");
  context.end();
  check(context.mousePressed === 0 && context.keyPressed === 0 && context.textInput === ""
    && context.scrollX === 0 && context.scrollY === 0 && context.mouseDown === 1 && context.keyDown === 1, "edge reset");
  context.inputMouseUp(13, 23, UI_MOUSE_LEFT); context.inputKeyUp(UI_KEY_SHIFT);
  context.begin();
  check(context.mouseDeltaX === 0 && context.mouseDeltaY === 0 && context.mouseDown === 0 && context.keyDown === 0, "level reset");
  context.end();
  context.begin();
  context.pushLayout(new UiRect(10, 20, 200, 100));
  context.layoutRow([0, -1], 0);
  check(rect(context.layoutNext(), 10, 20, 78, 20), "default item");
  check(rect(context.layoutNext(), 92, 20, 118, 20), "fill item");
  check(rect(context.layoutNext(), 10, 44, 78, 20), "next row");
  context.layoutSetNext(new UiRect(1, 2, 3, 4), false);
  check(rect(context.layoutNext(), 1, 2, 3, 4), "absolute item");
  context.layoutSetNext(new UiRect(3, 4, 5, 6), true);
  check(rect(context.layoutNext(), 13, 24, 5, 6), "relative item");
  const extent: UiRect = context.popLayout();
  check(rect(extent, 10, 20, 200, 44), "extent");
  context.pushLayout(new UiRect(0, 0, 200, 100));
  context.layoutRow([50, 50], 10);
  context.layoutBeginColumn();
  context.layoutRow([20], 30);
  check(rect(context.layoutNext(), 0, 0, 20, 30), "column first");
  check(rect(context.layoutNext(), 0, 34, 20, 30), "column second");
  context.layoutEndColumn();
  check(rect(context.layoutNext(), 54, 0, 50, 10), "column parent");
  check(rect(context.layoutNext(), 0, 68, 50, 10), "column next row");
  context.popLayout();
  context.pushLayout(new UiRect(10, 20, 100, 100), 2, 3);
  context.layoutRow([], -1);
  check(rect(context.layoutNext(), 8, 17, 78, 100), "scroll layout");
  context.popLayout(); context.end();
  context.begin();
  context.pushClip(new UiRect(10, 10, 20, 20));
  context.pushClip(new UiRect(0, 0, 20, 20));
  check(rect(context.getClip(), 10, 10, 10, 10), "clip intersection");
  context.drawRect(new UiRect(0, 0, 5, 5), 0xffffffff);
  context.drawRect(new UiRect(5, 5, 10, 10), 0x44332211);
  context.drawIcon(UI_ICON_CHECK, new UiRect(10, 10, 5, 5), 0xffffffff);
  context.drawText("A\n", 17, 10, 0xff0000ff);
  check(context.textWidth("A\nB") === 14, "text width");
  context.popClip(); context.popClip(); context.end();
  const lines: string[] = context.dumpCommands();
  check(lines.length === 7 && lines[0] === "clip 10 10 5 5" && lines[1] === "rect 5 5 10 10 #11223344"
    && lines[2] === "clip 0 0 16777216 16777216" && lines[3] === "icon 1 10 10 5 5 #ffffffff"
    && lines[4] === "clip 17 10 3 10" && lines[5] === "text 17 10 #ff0000ff \"A\""
    && lines[6] === "clip 0 0 16777216 16777216", "commands");
  const command: UiCommand = context.commands[1];
  check(command.kind === 2 && command.id === 0 && command.text === "", "command fields");
  const back: UiRoot = new UiRoot(11, new UiRect(0, 0, 100, 100), 1);
  const front: UiRoot = new UiRoot(22, new UiRect(0, 0, 100, 100), 2);
  context.inputMouseMove(20, 20);
  context.begin();
  context.beginRoot(front); context.drawRect(front.rect, 0xff000002); context.endRoot();
  context.beginRoot(back); context.drawRect(back.rect, 0xff000001); context.endRoot(); context.end();
  check(context.drawOrder()[0] === 1 && context.drawOrder()[1] === 0
    && context.dumpCommands()[0] === "rect 0 0 100 100 #010000ff"
    && context.dumpCommands()[2] === "container 11 0 0 100 100 0 0 1", "root order");
  context.begin(); context.beginRoot(front);
  check(context.mouseOver(front.rect), "hover root");
  context.updateControl(100, front.rect);
  check(context.hover === 100 && context.focus === 0, "hover");
  context.endRoot(); context.end();
  context.inputMouseDown(20, 20, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front); context.updateControl(100, front.rect, UI_OPT_HOLD_FOCUS);
  check(context.focus === 100, "press focus"); context.endRoot(); context.end();
  context.inputMouseUp(20, 20, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front); context.updateControl(100, front.rect, UI_OPT_HOLD_FOCUS);
  check(context.focus === 100, "hold focus");
  context.updateControl(100, front.rect);
  check(context.focus === 0, "release focus");
  context.setFocus(200); context.updateControl(100, front.rect, UI_OPT_NO_INTERACT);
  check(context.focus === 200, "no interact"); context.endRoot(); context.end();
  context.begin(); context.end(); check(context.focus === 0, "untouched focus");
  context.inputMouseMove(100, 100);
  context.begin(); context.beginRoot(front); context.updateControl(100, front.rect);
  check(!context.mouseOver(front.rect) && context.hover === 0, "exclusive edge");
  context.endRoot(); context.end();
  context.inputMouseMove(20, 20);
  context.begin(); context.beginRoot(front); context.endRoot(); context.end();
  context.inputMouseDown(20, 20, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front);
  context.updateControl(300, new UiRect(10, 10, 30, 30), UI_OPT_HOLD_FOCUS);
  check(context.focus === 0, "press without hover");
  context.endRoot(); context.end();
  context.inputMouseUp(20, 20, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front);
  context.updateControl(300, new UiRect(10, 10, 30, 30), UI_OPT_HOLD_FOCUS);
  context.endRoot(); context.end();
  context.inputMouseDown(20, 20, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front);
  context.updateControl(300, new UiRect(10, 10, 30, 30), UI_OPT_HOLD_FOCUS);
  check(context.focus === 300, "press after hover");
  context.endRoot(); context.end();
  context.inputMouseUp(20, 20, UI_MOUSE_LEFT);
  context.inputMouseMove(60, 60);
  context.begin(); context.beginRoot(front);
  context.updateControl(300, new UiRect(10, 10, 30, 30), UI_OPT_HOLD_FOCUS);
  check(context.focus === 300 && context.hover === 0, "hold focus after pointer leaves");
  context.endRoot(); context.end();
  context.inputMouseDown(60, 60, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front);
  context.updateControl(300, new UiRect(10, 10, 30, 30), UI_OPT_HOLD_FOCUS);
  check(context.focus === 0, "outside press");
  context.pushClip(new UiRect(0, 0, 10, 10));
  check(!context.mouseOver(front.rect), "clip hover"); context.popClip();
  context.endRoot(); context.beginRoot(back);
  check(!context.mouseOver(back.rect), "occluded root"); context.endRoot(); context.end();
}

function verifyWidgets(): void {
  check(uiNumberText(1.125) === "1.13" && uiNumberText(-1.125) === "-1.13"
    && uiNumberText(-0.004) === "-0.00", "number format");
  const ui: UiContext = new UiContext();
  const value: UiState<f32> = new UiState<f32>(1);
  const checked: UiState<boolean> = new UiState<boolean>(false);
  const input: UiState<string> = new UiState<string>("AB");
  const root: UiRoot = new UiRoot(1, new UiRect(0, 0, 200, 200), 1);
  ui.inputMouseMove(10, 10);
  ui.begin(); ui.beginRoot(root); ui.endRoot(); ui.end();
  for (let frame: i32 = 0; frame < 6; frame += 1) {
    if (frame % 2 === 0) ui.inputMouseUp(10, 10, UI_MOUSE_LEFT);
    else ui.inputMouseDown(10, 10, UI_MOUSE_LEFT);
    ui.begin(); ui.beginRoot(root); ui.pushLayout(root.rect);
    ui.layoutSetNext(new UiRect(0, 0, 100, 20), false);
    if (frame < 2) {
      const response: u32 = ui.checkbox("check", checked);
      check(response === ((frame === 1 ? 4 : 0) as u32) && checked.value === (frame === 1), "checkbox press");
    } else if (frame < 4) {
      check(ui.buttonIcon(UI_ICON_CHECK) === ((frame === 3 ? 2 : 0) as u32), "icon press");
    } else {
      check(ui.number("number", value, 0.5) === 0 && value.value === 1, "number initial delta");
    }
    ui.popLayout(); ui.endRoot(); ui.end();
  }
  ui.inputMouseMove(20, 10);
  ui.begin(); ui.beginRoot(root); ui.pushLayout(root.rect);
  ui.layoutSetNext(new UiRect(0, 0, 100, 20), false);
  check(ui.number("number", value, 0.5) === 4 && value.value === 6, "number drag");
  ui.popLayout(); ui.endRoot(); ui.end();
  ui.inputMouseUp(20, 10, UI_MOUSE_LEFT);
  ui.begin(); ui.beginRoot(root); ui.pushLayout(root.rect);
  ui.layoutSetNext(new UiRect(0, 0, 100, 20), false);
  const id: u32 = ui.getId("textbox");
  ui.setFocus(id);
  ui.textbox("textbox", input);
  ui.popLayout(); ui.endRoot(); ui.end();
  ui.inputKeyDown(8); ui.inputKeyDown(UI_KEY_RETURN); ui.inputText(67);
  ui.begin(); ui.beginRoot(root); ui.pushLayout(root.rect);
  check(ui.textbox("textbox", input) === 6 && input.value === "AB" && ui.focus === 0, "textbox keys");
  ui.popLayout(); ui.endRoot(); ui.end();
  ui.inputKeyUp(8); ui.inputKeyUp(UI_KEY_RETURN);
  ui.begin(); ui.beginRoot(root); ui.pushLayout(root.rect);
  check(ui.header("expanded", 4096) === 1, "expanded header");
  ui.layoutRow([50], 0); ui.text("one two three\nfour");
  ui.drawControlText("A", new UiRect(0, 0, 100, 20), UI_COLOR_TEXT, 1);
  ui.drawControlText("A", new UiRect(0, 0, 100, 20), UI_COLOR_TEXT, 2);
  ui.popLayout(); ui.endRoot(); ui.end();
  const lines: string[] = ui.dumpCommands();
  let center: boolean = false;
  let right: boolean = false;
  let wrapped: i32 = 0;
  for (let i: i32 = 0; i < lines.length; i += 1) {
    if (lines[i] === "text 46 1 #e6e6e6ff \"A\"") center = true;
    if (lines[i] === "text 88 1 #e6e6e6ff \"A\"") right = true;
    if (lines[i].includes("\"one two\"") || lines[i].includes("\"three\"") || lines[i].includes("\"four\"")) wrapped += 1;
  }
  check(center && right && wrapped === 3, "aligned and wrapped text");
  ui.begin(); ui.beginRoot(root); ui.pushLayout(root.rect);
  for (let alignment: i32 = 0; alignment < 3; alignment += 1) {
    const opt: u32 = alignment === 0 ? 0 : alignment === 1 ? UI_OPT_ALIGN_CENTER : UI_OPT_ALIGN_RIGHT;
    ui.layoutSetNext(new UiRect(0, 0, 100, 20), false);
    ui.number("aligned number", value, 0.5, opt);
    const numberText: UiCommand = ui.commands[ui.commandCount - 1];
    check(numberText.kind === 3 && numberText.x === (alignment === 2 ? 74 : 39), "number alignment");
    ui.layoutSetNext(new UiRect(0, 0, 100, 20), false);
    ui.slider("aligned slider", value, 0, 100, 0, opt);
    const sliderText: UiCommand = ui.commands[ui.commandCount - 1];
    check(sliderText.kind === 3 && sliderText.x === (alignment === 2 ? 74 : 39), "slider alignment");
  }
  ui.popLayout(); ui.endRoot(); ui.end();
  const commands: UiCommand[] = ui.commands;
  const roots: UiRoot[] = ui.roots;
  const first: UiCommand = commands[0];
  ui.begin(); ui.drawRect(new UiRect(1, 1, 2, 2), 0xffffffff); ui.end();
  check(commands[0] === first && first.x === 1 && roots.length > 0 && ui.commands[0] === first
    && ui.commandCount === 1 && ui.rootCount === 0 && ui.drawOrder().length === 0
    && ui.dumpCommands().length === 1, "reused records and counts");
  ui.begin();
  ui.style.colors[1] = 0;
  ui.drawFrame(new UiRect(0, 0, 10, 10), 6);
  ui.drawControlFrame(10, new UiRect(0, 0, 10, 10), 6, 8);
  ui.end();
  check(ui.commandCount === 1, "frame options");
}

function verifyContainers(): void {
  const ui: UiContext = new UiContext();
  const bounds: UiRect = new UiRect(10, 10, 200, 150);
  ui.inputMouseMove(30, 20);
  ui.begin();
  check(ui.beginWindow("Window", bounds) === 1, "window active");
  ui.label("content"); ui.endWindow(); ui.end();
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  ui.inputMouseDown(30, 20, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  ui.inputMouseMove(40, 30);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  check(ui.roots[0].rect.x === 20 && ui.roots[0].rect.y === 20, "title drag");
  ui.inputMouseUp(40, 30, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  ui.inputMouseMove(205, 155);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  ui.inputMouseDown(205, 155, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  const width: i32 = ui.roots[0].rect.w;
  const height: i32 = ui.roots[0].rect.h;
  ui.inputMouseMove(215, 165);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  check(ui.roots[0].rect.w === width + 10 && ui.roots[0].rect.h === height + 10, "resize drag");
  ui.inputMouseUp(215, 165, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  const closeX: i32 = ui.roots[0].rect.x + ui.roots[0].rect.w - 12;
  ui.inputMouseMove(closeX, 30);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  ui.inputMouseDown(closeX, 30, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Window", bounds); ui.label("content"); ui.endWindow(); ui.end();
  ui.inputMouseUp(closeX, 30, UI_MOUSE_LEFT);
  ui.begin(); check(ui.beginWindow("Window", bounds) === 0, "close window"); ui.end();
  ui.begin();
  ui.beginWindow("Parent", bounds);
  ui.label("before");
  ui.openPopup("Popup");
  check(ui.beginPopup("Popup") === 1, "popup open");
  ui.label("popup"); ui.endPopup();
  ui.label("after"); ui.endWindow(); ui.end();
  check(ui.rootCount === 2 && ui.roots[0].end === ui.roots[1].start
    && ui.roots[1].end === ui.commandCount, "nested root ranges");
  ui.inputMouseMove(0, 0);
  ui.begin(); ui.beginWindow("Parent", bounds);
  if (ui.beginPopup("Popup") !== 0) { ui.label("popup"); ui.endPopup(); }
  ui.endWindow(); ui.end();
  ui.inputMouseDown(0, 0, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Parent", bounds);
  if (ui.beginPopup("Popup") !== 0) ui.endPopup();
  ui.endWindow(); ui.end();
  ui.inputMouseUp(0, 0, UI_MOUSE_LEFT);
  ui.begin(); ui.beginWindow("Parent", bounds);
  check(ui.beginPopup("Popup") === 0, "popup outside press");
  ui.endWindow(); ui.end();
  ui.begin();
  check(ui.beginWindow("Popup option", bounds, 1024) === 1, "window popup option");
  ui.endWindow(); ui.end();
  const scroll: UiContext = new UiContext();
  scroll.inputMouseMove(30, 50);
  for (let frame: i32 = 0; frame < 4; frame += 1) {
    if (frame === 2) scroll.inputScroll(0, 20);
    scroll.begin(); scroll.beginWindow("Scroll", bounds);
    scroll.layoutRow([250], 300); scroll.label("large");
    scroll.endWindow(); scroll.end();
    if (frame >= 2) check(scroll.roots[0].scrollY === 20, "wheel scroll");
  }
  const pool: UiContext = new UiContext();
  for (let frame: i32 = 0; frame < 50; frame += 1) {
    pool.begin(); pool.beginWindow(`pool${frame}`, bounds); pool.label("pool"); pool.endWindow(); pool.end();
  }
  pool.begin();
  check(pool.beginWindow("pool0", bounds, 2048) === 0, "container eviction");
  pool.beginWindow("pool49", bounds);
  check(pool.header("tree", 4096) === 1, "tree default");
  pool.endWindow(); pool.end();
}

function verifyScrollbars(): void {
  const ui: UiContext = new UiContext();
  const bounds: UiRect = new UiRect(10, 10, 200, 150);
  ui.inputMouseMove(30, 50);
  for (let frame: i32 = 0; frame < 7; frame += 1) {
    if (frame === 2) ui.inputScroll(10, 20);
    if (frame === 3) ui.inputMouseMove(204, 50);
    if (frame === 4) ui.inputMouseDown(204, 50, UI_MOUSE_LEFT);
    if (frame === 5) ui.inputMouseMove(204, 60);
    if (frame === 6) ui.inputMouseUp(204, 60, UI_MOUSE_LEFT);
    ui.begin(); ui.beginWindow("Scrollbars", bounds);
    ui.layoutRow([250], 300); ui.label("large");
    ui.endWindow(); ui.end();
    if (frame >= 1) {
      print(`scrollbar frame ${frame}`);
      const lines: string[] = ui.dumpCommands();
      for (let i: i32 = 0; i < lines.length; i += 1) print(lines[i]);
    }
    if (frame >= 2) check(ui.roots[0].scrollX === 10, "horizontal wheel");
    if (frame >= 2 && frame <= 4) check(ui.roots[0].scrollY === 20, "wheel and thumb press");
    if (frame >= 5) check(ui.roots[0].scrollY === 47, "vertical thumb drag");
  }
}
