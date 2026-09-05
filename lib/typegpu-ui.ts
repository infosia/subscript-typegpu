// Immediate-mode UI state, layout, and draw commands.
import { UI_ATLAS_FONT, UI_ATLAS_RECT_W, UI_TEXT_HEIGHT } from "./typegpu-ui-atlas.generated";

function uiTrap(rule: string, method: string, values: string): void {
  print(`${rule} ${method} ${values} (author)`);
  unreachable();
}

export class UiState<T> {
  value: T;
  constructor(value: T) { this.value = value; }
}

export class UiRect {
  x: i32;
  y: i32;
  w: i32;
  h: i32;
  constructor(x: i32, y: i32, w: i32, h: i32) {
    this.x = x;
    this.y = y;
    this.w = w;
    this.h = h;
  }
}

export class UiCommand {
  kind: i32;
  x: i32;
  y: i32;
  w: i32;
  h: i32;
  color: u32;
  id: i32;
  text: string;
  constructor(kind: i32, rect: UiRect, color: u32 = 0, id: i32 = 0, text: string = "") {
    this.kind = kind;
    this.x = rect.x;
    this.y = rect.y;
    this.w = rect.w;
    this.h = rect.h;
    this.color = color;
    this.id = id;
    this.text = text;
  }
}

export const UI_OPT_ALIGN_CENTER: u32 = 1;
export const UI_OPT_ALIGN_RIGHT: u32 = 2;
export const UI_OPT_NO_INTERACT: u32 = 4;
export const UI_OPT_NO_FRAME: u32 = 8;
export const UI_OPT_NO_RESIZE: u32 = 16;
export const UI_OPT_NO_SCROLL: u32 = 32;
export const UI_OPT_NO_CLOSE: u32 = 64;
export const UI_OPT_NO_TITLE: u32 = 128;
export const UI_OPT_HOLD_FOCUS: u32 = 256;
export const UI_OPT_AUTO_SIZE: u32 = 512;
export const UI_OPT_POPUP: u32 = 1024;
export const UI_OPT_CLOSED: u32 = 2048;
export const UI_OPT_EXPANDED: u32 = 4096;
export const UI_MOUSE_LEFT: u32 = 1;
export const UI_MOUSE_RIGHT: u32 = 2;
export const UI_MOUSE_MIDDLE: u32 = 4;
export const UI_KEY_SHIFT: u32 = 1;
export const UI_KEY_CTRL: u32 = 2;
export const UI_KEY_ALT: u32 = 4;
export const UI_KEY_BACKSPACE: u32 = 8;
export const UI_KEY_RETURN: u32 = 16;
export const UI_RES_ACTIVE: u32 = 1;
export const UI_RES_SUBMIT: u32 = 2;
export const UI_RES_CHANGE: u32 = 4;
export const UI_ICON_CLOSE: i32 = 0;
export const UI_ICON_CHECK: i32 = 1;
export const UI_ICON_COLLAPSED: i32 = 2;
export const UI_ICON_EXPANDED: i32 = 3;
export const UI_COLOR_TEXT: i32 = 0;
export const UI_COLOR_BORDER: i32 = 1;
export const UI_COLOR_WINDOW_BG: i32 = 2;
export const UI_COLOR_TITLE_BG: i32 = 3;
export const UI_COLOR_TITLE_TEXT: i32 = 4;
export const UI_COLOR_PANEL_BG: i32 = 5;
export const UI_COLOR_BUTTON: i32 = 6;
export const UI_COLOR_BUTTON_HOVER: i32 = 7;
export const UI_COLOR_BUTTON_FOCUS: i32 = 8;
export const UI_COLOR_BASE: i32 = 9;
export const UI_COLOR_BASE_HOVER: i32 = 10;
export const UI_COLOR_BASE_FOCUS: i32 = 11;
export const UI_COLOR_SCROLL_BASE: i32 = 12;
export const UI_COLOR_SCROLL_THUMB: i32 = 13;

export class UiStyle {
  width: i32 = 68;
  height: i32 = 10;
  padding: i32 = 5;
  spacing: i32 = 4;
  indent: i32 = 24;
  titleHeight: i32 = 24;
  scrollbarSize: i32 = 12;
  thumbSize: i32 = 8;
  colors: u32[] = [0xffe6e6e6, 0xff191919, 0xff323232, 0xff191919,
    0xfff0f0f0, 0, 0xff4b4b4b, 0xff5f5f5f, 0xff737373,
    0xff1e1e1e, 0xff232323, 0xff282828, 0xff2b2b2b, 0xff1e1e1e];
}

function uiMax(a: i32, b: i32): i32 { return a > b ? a : b; }
function uiMin(a: i32, b: i32): i32 { return a < b ? a : b; }
function uiCopy(r: UiRect): UiRect { return new UiRect(r.x, r.y, r.w, r.h); }
function uiIntersection(a: UiRect, b: UiRect): UiRect {
  const x: i32 = uiMax(a.x, b.x);
  const y: i32 = uiMax(a.y, b.y);
  return new UiRect(x, y, uiMax(0, uiMin(a.x + a.w, b.x + b.w) - x),
    uiMax(0, uiMin(a.y + a.h, b.y + b.h) - y));
}
function uiContains(r: UiRect, x: i32, y: i32): boolean {
  return x >= r.x && y >= r.y && x < r.x + r.w && y < r.y + r.h;
}
function uiColorText(color: u32): string {
  const digits: string = "0123456789abcdef";
  let result: string = "#";
  for (let i: i32 = 0; i < 4; i += 1) {
    const byte: u32 = color % 256;
    result += digits.charAt((byte / 16) as i32) + digits.charAt((byte % 16) as i32);
    color /= 256;
  }
  return result;
}

class UiLayout {
  body: UiRect;
  widths: i32[] = [0];
  height: i32 = 0;
  item: i32 = 0;
  x: i32 = 0;
  y: i32 = 0;
  indent: i32 = 0;
  nextRow: i32 = 0;
  maxX: i32 = -16777216;
  maxY: i32 = -16777216;
  next: UiRect = new UiRect(0, 0, 0, 0);
  nextType: i32 = 0;
  column: boolean = false;
  constructor(body: UiRect) { this.body = uiCopy(body); }
}

// Root ranges index the command array. The end index is exclusive.
export class UiRoot {
  id: u32;
  rect: UiRect;
  scrollX: i32 = 0;
  scrollY: i32 = 0;
  zindex: i32;
  start: i32 = 0;
  end: i32 = 0;
  constructor(id: u32, rect: UiRect, zindex: i32) {
    this.id = id;
    this.rect = uiCopy(rect);
    this.zindex = zindex;
  }
}

export class UiContext {
  style: UiStyle = new UiStyle();
  commands: UiCommand[] = [];
  roots: UiRoot[] = [];
  hover: u32 = 0;
  focus: u32 = 0;
  lastId: u32 = 0;
  lastRect: UiRect = new UiRect(0, 0, 0, 0);
  mouseX: i32 = 0;
  mouseY: i32 = 0;
  mouseDeltaX: i32 = 0;
  mouseDeltaY: i32 = 0;
  mouseDown: u32 = 0;
  mousePressed: u32 = 0;
  keyDown: u32 = 0;
  keyPressed: u32 = 0;
  scrollX: i32 = 0;
  scrollY: i32 = 0;
  textInput: string = "";
  frame: i32 = 0;
  hoverRoot: u32 = 0;
  currentRoot: u32 = 0;
  private active: boolean = false;
  private previousX: i32 = 0;
  private previousY: i32 = 0;
  private nextHoverRoot: u32 = 0;
  private nextHoverZ: i32 = -2147483647;
  private updatedFocus: boolean = false;
  private ids: u32[] = [];
  private clips: UiRect[] = [];
  private layouts: UiLayout[] = [];
  private order: i32[] = [];

  private requireFrame(method: string): void {
    if (!this.active) uiTrap("UIT2", method, `frame=${this.frame}`);
  }
  begin(): void {
    if (this.active) uiTrap("UIT2", "begin", `frame=${this.frame}`);
    this.active = true;
    this.frame += 1;
    this.commands = [];
    this.roots = [];
    this.order = [];
    this.mouseDeltaX = this.mouseX - this.previousX;
    this.mouseDeltaY = this.mouseY - this.previousY;
    this.hoverRoot = this.nextHoverRoot;
    this.nextHoverRoot = 0;
    this.nextHoverZ = -2147483647;
    this.clips = [new UiRect(0, 0, 16777216, 16777216)];
  }
  end(): void {
    this.requireFrame("end");
    if (this.ids.length !== 0 || this.layouts.length !== 0 || this.clips.length !== 1 || this.currentRoot !== 0) {
      uiTrap("UIT2", "end", `ids=${this.ids.length} layouts=${this.layouts.length} clips=${this.clips.length} root=${this.currentRoot}`);
    }
    if (!this.updatedFocus) this.focus = 0;
    this.updatedFocus = false;
    this.order = [];
    for (let i: i32 = 0; i < this.roots.length; i += 1) {
      this.order.push(i);
      let j: i32 = this.order.length - 1;
      while (j > 0 && this.roots[this.order[j - 1]].zindex > this.roots[this.order[j]].zindex) {
        const prior: i32 = this.order[j - 1];
        this.order[j - 1] = this.order[j];
        this.order[j] = prior;
        j -= 1;
      }
    }
    this.mousePressed = 0;
    this.keyPressed = 0;
    this.textInput = "";
    this.scrollX = 0;
    this.scrollY = 0;
    this.previousX = this.mouseX;
    this.previousY = this.mouseY;
    this.active = false;
  }
  inputMouseMove(x: i32, y: i32): void { this.mouseX = x; this.mouseY = y; }
  inputMouseDown(x: i32, y: i32, button: u32): void {
    this.inputMouseMove(x, y);
    this.mouseDown |= button;
    this.mousePressed |= button;
  }
  inputMouseUp(x: i32, y: i32, button: u32): void {
    this.inputMouseMove(x, y);
    this.mouseDown &= ~button;
  }
  inputScroll(dx: i32, dy: i32): void { this.scrollX += dx; this.scrollY += dy; }
  inputKeyDown(key: u32): void { this.keyDown |= key; this.keyPressed |= key; }
  inputKeyUp(key: u32): void { this.keyDown &= ~key; }
  inputText(codePoint: u32): void {
    const ascii: string = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    if (codePoint >= 32 && codePoint <= 126) this.textInput += ascii.charAt((codePoint - 32) as i32);
  }
  getId(label: string): u32 {
    let hash: u32 = this.ids.length === 0 ? 2166136261 : this.ids[this.ids.length - 1];
    for (let i: i32 = 0; i < label.length; i += 1) hash = (hash ^ (label.charCodeAt(i) as u32)) * 16777619;
    this.lastId = hash;
    return hash;
  }
  pushId(label: string): void { this.requireFrame("pushId"); this.ids.push(this.getId(label)); }
  popId(): void {
    this.requireFrame("popId");
    if (this.ids.length === 0) uiTrap("UIT2", "popId", "depth=0");
    this.ids.pop();
  }
  setFocus(id: u32): void { this.requireFrame("setFocus"); this.focus = id; this.updatedFocus = true; }
  mouseOver(rect: UiRect): boolean {
    this.requireFrame("mouseOver");
    return this.currentRoot !== 0 && this.currentRoot === this.hoverRoot
      && uiContains(rect, this.mouseX, this.mouseY) && uiContains(this.getClip(), this.mouseX, this.mouseY);
  }
  updateControl(id: u32, rect: UiRect, opt: u32 = 0): void {
    this.requireFrame("updateControl");
    if (this.focus === id) this.updatedFocus = true;
    if ((opt & UI_OPT_NO_INTERACT) !== 0) return;
    const over: boolean = this.mouseOver(rect);
    if (over && this.mouseDown === 0) this.hover = id;
    if (over && this.mousePressed !== 0) this.setFocus(id);
    if (this.focus === id) {
      if ((this.mouseDown === 0 && (opt & UI_OPT_HOLD_FOCUS) === 0) || (this.mousePressed !== 0 && !over)) this.setFocus(0);
    }
    if (!over && this.hover === id) this.hover = 0;
  }
  getClip(): UiRect { this.requireFrame("getClip"); return uiCopy(this.clips[this.clips.length - 1]); }
  pushClip(rect: UiRect): void { this.requireFrame("pushClip"); this.clips.push(uiIntersection(this.getClip(), rect)); }
  popClip(): void {
    this.requireFrame("popClip");
    if (this.clips.length <= 1) uiTrap("UIT2", "popClip", `depth=${this.clips.length}`);
    this.clips.pop();
  }
  private emit(command: UiCommand): void {
    this.requireFrame("draw");
    const clip: UiRect = this.getClip();
    const rect: UiRect = new UiRect(command.x, command.y, command.w, command.h);
    const visible: UiRect = uiIntersection(rect, clip);
    if (visible.w <= 0 || visible.h <= 0) return;
    const partial: boolean = visible.x !== rect.x || visible.y !== rect.y || visible.w !== rect.w || visible.h !== rect.h;
    if (partial) this.commands.push(new UiCommand(1, visible));
    this.commands.push(command);
    if (partial) this.commands.push(new UiCommand(1, clip));
  }
  drawRect(rect: UiRect, color: u32): void { this.emit(new UiCommand(2, rect, color)); }
  drawIcon(icon: i32, rect: UiRect, color: u32): void { this.emit(new UiCommand(4, rect, color, icon)); }
  textWidth(text: string): i32 {
    let width: i32 = 0;
    for (let i: i32 = 0; i < text.length; i += 1) {
      const byte: i32 = text.charCodeAt(i);
      if (byte >= 32 && byte <= 126) width += UI_ATLAS_RECT_W[UI_ATLAS_FONT + byte];
    }
    return width;
  }
  drawText(text: string, x: i32, y: i32, color: u32): void {
    let visible: string = "";
    for (let i: i32 = 0; i < text.length; i += 1) {
      const byte: i32 = text.charCodeAt(i);
      if (byte >= 32 && byte <= 126) visible += text.charAt(i);
    }
    this.emit(new UiCommand(3, new UiRect(x, y, this.textWidth(visible), UI_TEXT_HEIGHT), color, 0, visible));
  }

  // A layout body uses content coordinates after the scroll offset.
  pushLayout(body: UiRect, scrollX: i32 = 0, scrollY: i32 = 0): void {
    this.requireFrame("pushLayout");
    this.layouts.push(new UiLayout(new UiRect(body.x - scrollX, body.y - scrollY, body.w, body.h)));
  }
  popLayout(): UiRect {
    const layout: UiLayout = this.layout("popLayout");
    this.layouts.pop();
    return new UiRect(layout.body.x, layout.body.y, uiMax(0, layout.maxX - layout.body.x), uiMax(0, layout.maxY - layout.body.y));
  }
  private layout(method: string): UiLayout {
    this.requireFrame(method);
    if (this.layouts.length === 0) uiTrap("UIT2", method, "depth=0");
    return this.layouts[this.layouts.length - 1];
  }
  layoutRow(widths: i32[], height: i32): void {
    const layout: UiLayout = this.layout("layoutRow");
    if (widths.length > 16) uiTrap("UIT3", "layoutRow", `widths=${widths.length} maximum=16`);
    layout.widths = [];
    for (let i: i32 = 0; i < widths.length; i += 1) layout.widths.push(widths[i]);
    layout.height = height;
    layout.item = 0;
    layout.x = layout.indent;
    layout.y = layout.nextRow;
  }
  layoutSetNext(rect: UiRect, relative: boolean): void {
    const layout: UiLayout = this.layout("layoutSetNext");
    layout.next = uiCopy(rect);
    layout.nextType = relative ? 1 : 2;
  }
  layoutNext(): UiRect {
    const layout: UiLayout = this.layout("layoutNext");
    let rect: UiRect = uiCopy(layout.next);
    if (layout.nextType !== 0) {
      const absolute: boolean = layout.nextType === 2;
      layout.nextType = 0;
      if (absolute) { this.lastRect = uiCopy(rect); return rect; }
    } else {
      if (layout.item === layout.widths.length) {
        layout.item = 0;
        layout.x = layout.indent;
        layout.y = layout.nextRow;
      }
      let width: i32 = layout.widths.length === 0 ? 0 : layout.widths[layout.item];
      let height: i32 = layout.height;
      if (width === 0) width = this.style.width + this.style.padding * 2;
      if (height === 0) height = this.style.height + this.style.padding * 2;
      if (width < 0) width += layout.body.w - layout.x + 1;
      if (height < 0) height += layout.body.h - layout.y + 1;
      rect = new UiRect(layout.x, layout.y, width, height);
      layout.item += 1;
    }
    layout.x += rect.w + this.style.spacing;
    layout.nextRow = uiMax(layout.nextRow, rect.y + rect.h + this.style.spacing);
    rect.x += layout.body.x;
    rect.y += layout.body.y;
    layout.maxX = uiMax(layout.maxX, rect.x + rect.w);
    layout.maxY = uiMax(layout.maxY, rect.y + rect.h);
    this.lastRect = uiCopy(rect);
    return rect;
  }
  layoutBeginColumn(): void {
    this.pushLayout(this.layoutNext());
    this.layout("layoutBeginColumn").column = true;
  }
  layoutEndColumn(): void {
    this.requireFrame("layoutEndColumn");
    if (this.layouts.length < 2) uiTrap("UIT2", "layoutEndColumn", `depth=${this.layouts.length}`);
    const child: UiLayout = this.layout("layoutEndColumn");
    if (!child.column) uiTrap("UIT2", "layoutEndColumn", "column=false");
    this.layouts.pop();
    const parent: UiLayout = this.layout("layoutEndColumn");
    parent.x = uiMax(parent.x, child.x + child.body.x - parent.body.x);
    parent.nextRow = uiMax(parent.nextRow, child.nextRow + child.body.y - parent.body.y);
    parent.maxX = uiMax(parent.maxX, child.maxX);
    parent.maxY = uiMax(parent.maxY, child.maxY);
  }

  // Container code records each root through these range boundaries.
  beginRoot(root: UiRoot): void {
    this.requireFrame("beginRoot");
    if (this.currentRoot !== 0) uiTrap("UIT2", "beginRoot", `root=${this.currentRoot}`);
    this.currentRoot = root.id;
    root.start = this.commands.length;
    this.roots.push(root);
    if (uiContains(root.rect, this.mouseX, this.mouseY) && root.zindex > this.nextHoverZ) {
      this.nextHoverRoot = root.id;
      this.nextHoverZ = root.zindex;
    }
  }
  endRoot(): void {
    this.requireFrame("endRoot");
    if (this.currentRoot === 0) uiTrap("UIT2", "endRoot", "root=0");
    this.roots[this.roots.length - 1].end = this.commands.length;
    this.currentRoot = 0;
  }
  drawOrder(): i32[] {
    const result: i32[] = [];
    for (let i: i32 = 0; i < this.order.length; i += 1) result.push(this.order[i]);
    return result;
  }
  private commandLine(command: UiCommand): string {
    if (command.kind === 1) return `clip ${command.x} ${command.y} ${command.w} ${command.h}`;
    const color: string = uiColorText(command.color);
    if (command.kind === 2) return `rect ${command.x} ${command.y} ${command.w} ${command.h} ${color}`;
    if (command.kind === 3) return `text ${command.x} ${command.y} ${color} "${command.text}"`;
    return `icon ${command.id} ${command.x} ${command.y} ${command.w} ${command.h} ${color}`;
  }
  dumpCommands(): string[] {
    const lines: string[] = [];
    if (this.roots.length === 0) {
      for (let i: i32 = 0; i < this.commands.length; i += 1) lines.push(this.commandLine(this.commands[i]));
    }
    for (let i: i32 = 0; i < this.order.length; i += 1) {
      const root: UiRoot = this.roots[this.order[i]];
      for (let j: i32 = root.start; j < root.end; j += 1) lines.push(this.commandLine(this.commands[j]));
    }
    for (let i: i32 = 0; i < this.order.length; i += 1) {
      const root: UiRoot = this.roots[this.order[i]];
      lines.push(`container ${root.id} ${root.rect.x} ${root.rect.y} ${root.rect.w} ${root.rect.h} ${root.scrollX} ${root.scrollY} ${root.zindex}`);
    }
    return lines;
  }
}
