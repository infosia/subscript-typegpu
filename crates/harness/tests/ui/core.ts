import { UiContext, UiRect, UiRoot, UiCommand, UiState, UiStyle,
  UI_OPT_HOLD_FOCUS, UI_OPT_NO_INTERACT, UI_MOUSE_LEFT, UI_MOUSE_RIGHT,
  UI_KEY_SHIFT, UI_KEY_RETURN, UI_ICON_CHECK, UI_COLOR_TEXT } from "./typegpu-ui";
import { uiAtlasAlpha, UI_ATLAS_ALPHA_HEX, UI_ATLAS_WIDTH, UI_ATLAS_HEIGHT,
  UI_ATLAS_RECT_X, UI_ATLAS_RECT_Y, UI_ATLAS_RECT_W, UI_ATLAS_RECT_H,
  UI_ATLAS_FONT, UI_ATLAS_WHITE, UI_TEXT_HEIGHT } from "./typegpu-ui-atlas.generated";

function check(value: boolean, name: string): void {
  if (!value) { print(`FAIL ${name}`); unreachable(); }
}
function rect(r: UiRect, x: i32, y: i32, w: i32, h: i32): boolean {
  return r.x === x && r.y === y && r.w === w && r.h === h;
}
export function main(): void {
  const alpha: u8[] = uiAtlasAlpha();
  check(alpha.length === UI_ATLAS_WIDTH * UI_ATLAS_HEIGHT && UI_ATLAS_ALPHA_HEX.length === 32768, "atlas size");
  check(UI_ATLAS_RECT_X.length === 100 && UI_ATLAS_RECT_Y.length === 100
    && UI_ATLAS_RECT_W.length === 100 && UI_ATLAS_RECT_H.length === 100, "atlas rects");
  check(UI_ATLAS_RECT_X[UI_ATLAS_WHITE] === 125 && UI_ATLAS_RECT_Y[UI_ATLAS_WHITE] === 68
    && alpha[68 * 128 + 125] === 255 && alpha[16383] === 0, "atlas pixels");
  check(UI_ATLAS_RECT_W[UI_ATLAS_FONT + 65] === 7 && UI_TEXT_HEIGHT === 18, "glyph A");
  let checksum: u32 = 2166136261;
  for (let i: i32 = 0; i < alpha.length; i += 1) checksum = (checksum ^ (alpha[i] as u32)) * 16777619;
  check(checksum === 4088512439, "atlas checksum");
  print("atlas PASS");
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
  print("ids PASS");
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
  print("input PASS");
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
  print("layout PASS");
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
    && lines[2] === "clip 10 10 10 10" && lines[3] === "icon 1 10 10 5 5 #ffffffff"
    && lines[4] === "clip 17 10 3 10" && lines[5] === "text 17 10 #ff0000ff \"A\""
    && lines[6] === "clip 10 10 10 10", "commands");
  const command: UiCommand = context.commands[1];
  check(command.kind === 2 && command.id === 0 && command.text === "", "command fields");
  print("commands PASS");
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
  check(context.focus === 300, "direct press");
  context.endRoot(); context.end();
  context.inputMouseUp(20, 20, UI_MOUSE_LEFT);
  context.inputMouseDown(60, 60, UI_MOUSE_LEFT);
  context.begin(); context.beginRoot(front);
  context.updateControl(300, new UiRect(10, 10, 30, 30), UI_OPT_HOLD_FOCUS);
  check(context.focus === 0, "outside press");
  context.pushClip(new UiRect(0, 0, 10, 10));
  check(!context.mouseOver(front.rect), "clip hover"); context.popClip();
  context.endRoot(); context.beginRoot(back);
  check(!context.mouseOver(back.rect), "occluded root"); context.endRoot(); context.end();
  print("focus PASS");
}
