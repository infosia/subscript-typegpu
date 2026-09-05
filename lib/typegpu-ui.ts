// Immediate-mode UI state, layout, and draw commands.
import {
  UI_ATLAS_FONT, UI_ATLAS_RECT_W, UI_ATLAS_RECT_X, UI_ATLAS_RECT_Y, UI_ATLAS_RECT_H,
  UI_ATLAS_WIDTH, UI_ATLAS_HEIGHT, UI_ATLAS_WHITE, UI_TEXT_HEIGHT, uiAtlasAlpha,
} from "./typegpu-ui-atlas.generated";
import { Vec2f, Vec4f } from "./typegpu-types";
import {
  Buffer, Uniform, Texture2d, Sampler, VertexInvocation, FragmentInvocation,
  RenderPipeline, RenderPipelineSpec, BindGroupLayoutSpec, VertexBufferLayoutSpec, createRenderPipeline, createRenderPipelineHost,
  createBindGroup, createBindGroupHost, bufferResource, textureResource, samplerResource, writeTextureBytes,
} from "./typegpu";
import {
  GPUBlendState, GPUDevice, GPUHostOwnedDevice, GPUQueue, GPUTexture, GPUTextureView, GPUSampler, GPUBindGroup,
  GPURenderPassEncoder, GPUBufferUsage, GPUTextureUsage,
} from "./webgpu";

function uiTrap(rule: string, method: string, values: string): void {
  print(`${rule} ${method} ${values} (author)`);
  unreachable();
}

export class UiState<T> {
  value: T;
  constructor(value: T) { this.value = value; }
}

@CStruct
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

const UI_UNCLIPPED: UiRect = new UiRect(0, 0, 16777216, 16777216);

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

export function uiNumberText(value: f32): string {
  const negative: boolean = value < 0;
  const magnitude: f32 = negative ? -value : value;
  const cents: i64 = (magnitude * 100 + 0.5) as i64;
  const whole: i64 = cents / 100;
  const fraction: i64 = cents % 100;
  return `${negative ? "-" : ""}${whole}.${fraction < 10 ? "0" : ""}${fraction}`;
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
  widths: i32[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  widthCount: i32 = 1;
  height: i32 = 0;
  item: i32 = 0;
  x: i32 = 0;
  y: i32 = 0;
  indent: i32 = 0;
  nextRow: i32 = 0;
  maxX: i32 = -UI_UNCLIPPED.w;
  maxY: i32 = -UI_UNCLIPPED.w;
  next: UiRect = new UiRect(0, 0, 0, 0);
  nextType: i32 = 0;
  column: boolean = false;
  constructor(body: UiRect) { this.body = uiCopy(body); }
}

// Root ranges index the command array. The end index is exclusive.
export class UiRoot {
  id: u32;
  rect: UiRect;
  body: UiRect = new UiRect(0, 0, 0, 0);
  contentWidth: i32 = 0;
  contentHeight: i32 = 0;
  open: boolean = true;
  lastUpdate: i32 = 0;
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
  commandCount: i32 = 0;
  rootCount: i32 = 0;
  private idCount: i32 = 0;
  private clipCount: i32 = 1;
  private layoutCount: i32 = 0;
  private orderCount: i32 = 0;
  private containers: UiRoot[] = [];
  private containerStack: i32[] = [];
  private containerKinds: i32[] = [];
  private containerDepth: i32 = 0;
  private treeIds: u32[] = [];
  private treeFrames: i32[] = [];
  private treeDepths: i32[] = [];
  private treeCount: i32 = 0;
  private lastZindex: i32 = 0;
  private scrollTarget: i32 = -1;
  private fullWidth: i32[] = [-1];
  private active: boolean = false;
  private previousX: i32 = 0;
  private previousY: i32 = 0;
  private nextHoverRoot: u32 = 0;
  private nextHoverZ: i32 = -2147483647;
  private updatedFocus: boolean = false;
  private ids: u32[] = [];
  private clips: UiRect[] = [uiCopy(UI_UNCLIPPED)];
  private layouts: UiLayout[] = [];
  private order: i32[] = [];
  private commandRoots: i32[] = [];
  private groupCommands: boolean = false;
  private rootStack: i32[] = [];
  private rootDepth: i32 = 0;

  constructor() {
    for (let i: i32 = 0; i < 48; i += 1) {
      const record: UiRoot = new UiRoot(0, new UiRect(0, 0, 0, 0), 0);
      record.open = false;
      this.containers.push(record); this.treeIds.push(0); this.treeFrames.push(0);
    }
  }
  private requireFrame(method: string): void {
    if (!this.active) uiTrap("UIT2", method, `frame=${this.frame}`);
  }
  begin(): void {
    if (this.active) uiTrap("UIT2", "begin", `frame=${this.frame}`);
    this.active = true;
    this.frame += 1;
    this.commandCount = 0;
    this.groupCommands = false;
    this.rootCount = 0;
    this.orderCount = 0;
    this.scrollTarget = -1;
    this.mouseDeltaX = this.mouseX - this.previousX;
    this.mouseDeltaY = this.mouseY - this.previousY;
    this.hoverRoot = this.nextHoverRoot;
    this.nextHoverRoot = 0;
    this.nextHoverZ = -2147483647;
    this.clipCount = 1;
  }
  end(): void {
    this.requireFrame("end");
    if (this.idCount !== 0 || this.layoutCount !== 0 || this.clipCount !== 1 || this.currentRoot !== 0 || this.containerDepth !== 0 || this.treeCount !== 0) {
      uiTrap("UIT2", "end", `ids=${this.idCount} layouts=${this.layoutCount} clips=${this.clipCount} root=${this.currentRoot} containers=${this.containerDepth} trees=${this.treeCount}`);
    }
    if (!this.updatedFocus) this.focus = 0;
    this.updatedFocus = false;
    if (this.scrollTarget >= 0) {
      this.containers[this.scrollTarget].scrollX += this.scrollX;
      this.containers[this.scrollTarget].scrollY += this.scrollY;
    }
    for (let i: i32 = 0; i < this.rootCount; i += 1) {
      const root: UiRoot = this.roots[i];
      if (this.mousePressed !== 0 && root.id === this.nextHoverRoot && root.zindex >= 0 && root.zindex < this.lastZindex) {
        this.lastZindex += 1;
        root.zindex = this.lastZindex;
      }
    }
    // Group nested root commands into contiguous ranges without new records.
    if (this.groupCommands) {
      for (let i: i32 = 1; i < this.commandCount; i += 1) {
        let j: i32 = i;
        while (j > 0 && this.commandRoots[j - 1] > this.commandRoots[j]) {
          const command: UiCommand = this.commands[j - 1];
          this.commands[j - 1] = this.commands[j]; this.commands[j] = command;
          const owner: i32 = this.commandRoots[j - 1];
          this.commandRoots[j - 1] = this.commandRoots[j]; this.commandRoots[j] = owner;
          j -= 1;
        }
      }
    }
    let cursor: i32 = 0;
    while (cursor < this.commandCount && this.commandRoots[cursor] < 0) cursor += 1;
    for (let i: i32 = 0; i < this.rootCount; i += 1) {
      this.roots[i].start = cursor;
      while (cursor < this.commandCount && this.commandRoots[cursor] === i) cursor += 1;
      this.roots[i].end = cursor;
    }
    this.orderCount = 0;
    for (let i: i32 = 0; i < this.rootCount; i += 1) {
      if (this.orderCount === this.order.length) this.order.push(i);
      else this.order[this.orderCount] = i;
      this.orderCount += 1;
      let j: i32 = this.orderCount - 1;
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
    let hash: u32 = this.idCount === 0 ? 2166136261 : this.ids[this.idCount - 1];
    for (let i: i32 = 0; i < label.length; i += 1) hash = (hash ^ (label.charCodeAt(i) as u32)) * 16777619;
    this.lastId = hash;
    return hash;
  }
  pushId(label: string): void { this.requireFrame("pushId"); this.pushIdValue(this.getId(label)); }
  private pushIdValue(id: u32): void {
    if (this.idCount === this.ids.length) this.ids.push(id);
    else this.ids[this.idCount] = id;
    this.idCount += 1;
  }
  popId(): void {
    this.requireFrame("popId");
    if (this.idCount === 0) uiTrap("UIT2", "popId", "depth=0");
    this.idCount -= 1;
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
    if (this.focus === id) {
      if ((this.mouseDown === 0 && (opt & UI_OPT_HOLD_FOCUS) === 0) || (this.mousePressed !== 0 && !over)) this.setFocus(0);
    }
    if (this.hover === id && this.mousePressed !== 0) this.setFocus(id);
    if (this.hover === id && this.mousePressed === 0 && !over) this.hover = 0;
  }
  getClip(): UiRect { this.requireFrame("getClip"); return uiCopy(this.clips[this.clipCount - 1]); }
  pushClip(rect: UiRect): void { this.requireFrame("pushClip"); this.pushClipValue(uiIntersection(this.getClip(), rect)); }
  private pushClipValue(rect: UiRect): void {
    if (this.clipCount === this.clips.length) this.clips.push(rect);
    else this.clips[this.clipCount] = rect;
    this.clipCount += 1;
  }
  popClip(): void {
    this.requireFrame("popClip");
    if (this.clipCount <= 1) uiTrap("UIT2", "popClip", `depth=${this.clipCount}`);
    this.clipCount -= 1;
  }
  private append(kind: i32, rect: UiRect, color: u32 = 0, id: i32 = 0, text: string = ""): void {
    if (this.commandCount === this.commands.length) this.commands.push(new UiCommand(0, rect));
    const command: UiCommand = this.commands[this.commandCount];
    command.kind = kind;
    command.x = rect.x; command.y = rect.y; command.w = rect.w; command.h = rect.h;
    command.color = color; command.id = id; command.text = text;
    const owner: i32 = this.rootDepth === 0 ? -1 : this.rootStack[this.rootDepth - 1];
    if (this.commandCount > 0 && owner < this.commandRoots[this.commandCount - 1]) this.groupCommands = true;
    if (this.commandCount === this.commandRoots.length) this.commandRoots.push(owner);
    else this.commandRoots[this.commandCount] = owner;
    this.commandCount += 1;
  }
  private emit(kind: i32, rect: UiRect, color: u32, id: i32 = 0, text: string = ""): void {
    this.requireFrame("draw");
    const clip: UiRect = this.getClip();
    const visible: UiRect = uiIntersection(rect, clip);
    if (visible.w <= 0 || visible.h <= 0) return;
    const partial: boolean = visible.x !== rect.x || visible.y !== rect.y || visible.w !== rect.w || visible.h !== rect.h;
    if (partial) this.append(1, visible);
    this.append(kind, rect, color, id, text);
    if (partial) this.append(1, uiCopy(UI_UNCLIPPED));
  }
  drawRect(rect: UiRect, color: u32): void { this.emit(2, rect, color); }
  drawIcon(icon: i32, rect: UiRect, color: u32): void { this.emit(4, rect, color, icon); }
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
    this.emit(3, new UiRect(x, y, this.textWidth(visible), UI_TEXT_HEIGHT), color, 0, visible);
  }

  // A layout body uses content coordinates after the scroll offset.
  pushLayout(body: UiRect, scrollX: i32 = 0, scrollY: i32 = 0): void {
    this.requireFrame("pushLayout");
    if (this.layoutCount === this.layouts.length) this.layouts.push(new UiLayout(body));
    const layout: UiLayout = this.layouts[this.layoutCount];
    layout.body = new UiRect(body.x - scrollX, body.y - scrollY, body.w, body.h);
    layout.widths[0] = 0; layout.widthCount = 1; layout.height = 0;
    layout.item = 0; layout.x = 0; layout.y = 0; layout.indent = 0; layout.nextRow = 0;
    layout.maxX = -UI_UNCLIPPED.w; layout.maxY = -UI_UNCLIPPED.w; layout.nextType = 0; layout.column = false;
    this.layoutCount += 1;
  }
  popLayout(): UiRect {
    const layout: UiLayout = this.layout("popLayout");
    this.layoutCount -= 1;
    return new UiRect(layout.body.x, layout.body.y, uiMax(0, layout.maxX - layout.body.x), uiMax(0, layout.maxY - layout.body.y));
  }
  private layout(method: string): UiLayout {
    this.requireFrame(method);
    if (this.layoutCount === 0) uiTrap("UIT2", method, "depth=0");
    return this.layouts[this.layoutCount - 1];
  }
  layoutRow(widths: i32[], height: i32): void {
    const layout: UiLayout = this.layout("layoutRow");
    if (widths.length > 16) uiTrap("UIT3", "layoutRow", `widths=${widths.length} maximum=16`);
    layout.widthCount = widths.length;
    for (let i: i32 = 0; i < widths.length; i += 1) layout.widths[i] = widths[i];
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
      if (layout.item === layout.widthCount) {
        layout.item = 0;
        layout.x = layout.indent;
        layout.y = layout.nextRow;
      }
      let width: i32 = layout.widthCount === 0 ? 0 : layout.widths[layout.item];
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
    if (this.layoutCount < 2) uiTrap("UIT2", "layoutEndColumn", `depth=${this.layoutCount}`);
    const child: UiLayout = this.layout("layoutEndColumn");
    if (!child.column) uiTrap("UIT2", "layoutEndColumn", "column=false");
    this.layoutCount -= 1;
    const parent: UiLayout = this.layout("layoutEndColumn");
    parent.x = uiMax(parent.x, child.x + child.body.x - parent.body.x);
    parent.nextRow = uiMax(parent.nextRow, child.nextRow + child.body.y - parent.body.y);
    parent.maxX = uiMax(parent.maxX, child.maxX);
    parent.maxY = uiMax(parent.maxY, child.maxY);
  }

  // Each root retains its command owner index.
  beginRoot(root: UiRoot): void {
    this.requireFrame("beginRoot");
    if (this.rootDepth === this.rootStack.length) this.rootStack.push(this.rootCount);
    else this.rootStack[this.rootDepth] = this.rootCount;
    this.rootDepth += 1;
    this.currentRoot = root.id;
    if (this.rootCount === this.roots.length) this.roots.push(root);
    else this.roots[this.rootCount] = root;
    this.rootCount += 1;
    if (uiContains(root.rect, this.mouseX, this.mouseY) && root.zindex > this.nextHoverZ) {
      this.nextHoverRoot = root.id;
      this.nextHoverZ = root.zindex;
    }
  }
  endRoot(): void {
    this.requireFrame("endRoot");
    if (this.currentRoot === 0) uiTrap("UIT2", "endRoot", "root=0");
    this.rootDepth -= 1;
    this.currentRoot = this.rootDepth === 0 ? 0 : this.roots[this.rootStack[this.rootDepth - 1]].id;
  }
  drawOrder(): i32[] {
    const result: i32[] = [];
    for (let i: i32 = 0; i < this.orderCount; i += 1) result.push(this.order[i]);
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
    if (this.rootCount === 0) {
      for (let i: i32 = 0; i < this.commandCount; i += 1) lines.push(this.commandLine(this.commands[i]));
    }
    for (let i: i32 = 0; i < this.orderCount; i += 1) {
      const root: UiRoot = this.roots[this.order[i]];
      for (let j: i32 = root.start; j < root.end; j += 1) lines.push(this.commandLine(this.commands[j]));
    }
    for (let i: i32 = 0; i < this.orderCount; i += 1) {
      const root: UiRoot = this.roots[this.order[i]];
      lines.push(`container ${root.id} ${root.rect.x} ${root.rect.y} ${root.rect.w} ${root.rect.h} ${root.scrollX} ${root.scrollY} ${root.zindex}`);
    }
    return lines;
  }
  drawFrame(rect: UiRect, colorId: i32): void {
    this.drawRect(rect, this.style.colors[colorId]);
    if (colorId === UI_COLOR_TITLE_BG || colorId === UI_COLOR_SCROLL_BASE || colorId === UI_COLOR_SCROLL_THUMB) return;
    const border: u32 = this.style.colors[UI_COLOR_BORDER];
    if (border / 16777216 === 0) return;
    this.drawRect(new UiRect(rect.x, rect.y - 1, rect.w, 1), border);
    this.drawRect(new UiRect(rect.x, rect.y + rect.h, rect.w, 1), border);
    this.drawRect(new UiRect(rect.x - 1, rect.y - 1, 1, rect.h + 2), border);
    this.drawRect(new UiRect(rect.x + rect.w, rect.y - 1, 1, rect.h + 2), border);
  }
  drawControlFrame(id: u32, rect: UiRect, colorId: i32, opt: u32 = 0): void {
    this.requireFrame("drawControlFrame");
    if ((opt & UI_OPT_NO_FRAME) === 0) this.drawFrame(rect, colorId + (this.focus === id ? 2 : this.hover === id ? 1 : 0));
  }
  drawControlText(text: string, rect: UiRect, colorId: i32, opt: u32 = 0): void {
    let x: i32 = rect.x + this.style.padding;
    const width: i32 = this.textWidth(text);
    if ((opt & UI_OPT_ALIGN_CENTER) !== 0) x = rect.x + (rect.w - width) / 2;
    else if ((opt & UI_OPT_ALIGN_RIGHT) !== 0) x = rect.x + rect.w - width - this.style.padding;
    this.pushClip(rect);
    this.drawText(text, x, rect.y + (rect.h - UI_TEXT_HEIGHT) / 2, this.style.colors[colorId]);
    this.popClip();
  }
  button(label: string, opt: u32 = 0): u32 {
    const id: u32 = this.getId(label);
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect, opt);
    this.drawControlFrame(id, rect, UI_COLOR_BUTTON, opt);
    this.drawControlText(label, rect, UI_COLOR_TEXT, opt);
    return this.focus === id && this.mousePressed === UI_MOUSE_LEFT ? UI_RES_SUBMIT : 0;
  }
  buttonIcon(icon: i32, opt: u32 = 0): u32 {
    let id: u32 = this.idCount === 0 ? 2166136261 : this.ids[this.idCount - 1];
    let bytes: u32 = icon as u32;
    for (let i: i32 = 0; i < 4; i += 1) { id = (id ^ (bytes % 256)) * 16777619; bytes /= 256; }
    this.lastId = id;
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect, opt);
    this.drawControlFrame(id, rect, UI_COLOR_BUTTON, opt);
    this.drawIcon(icon, rect, this.style.colors[UI_COLOR_TEXT]);
    return this.focus === id && this.mousePressed === UI_MOUSE_LEFT ? UI_RES_SUBMIT : 0;
  }
  checkbox(label: string, state: UiState<boolean>): u32 {
    const id: u32 = this.getId(label);
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect);
    let response: u32 = 0;
    if (this.focus === id && this.mousePressed === UI_MOUSE_LEFT) {
      state.value = !state.value; response = UI_RES_CHANGE;
    }
    const box: UiRect = new UiRect(rect.x, rect.y, rect.h, rect.h);
    this.drawControlFrame(id, box, UI_COLOR_BASE);
    if (state.value) this.drawIcon(UI_ICON_CHECK, box, this.style.colors[UI_COLOR_TEXT]);
    this.drawControlText(label, new UiRect(rect.x + rect.h, rect.y, rect.w - rect.h, rect.h), UI_COLOR_TEXT);
    return response;
  }
  slider(label: string, state: UiState<f32>, low: f32, high: f32, step: f32 = 0, opt: u32 = 0): u32 {
    const id: u32 = this.getId(label);
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect, opt);
    const old: f32 = state.value;
    let value: f32 = old;
    if (this.focus === id && (this.mouseDown | this.mousePressed) === UI_MOUSE_LEFT) {
      value = low + ((this.mouseX - rect.x) as f32) * (high - low) / (rect.w as f32);
      if (step !== 0) value = (((value + step / 2) / step) as i32 as f32) * step;
    }
    state.value = value < low ? low : value > high ? high : value;
    this.drawControlFrame(id, rect, UI_COLOR_BASE, opt);
    const x: i32 = high === low ? 0 : ((state.value - low) * ((rect.w - this.style.thumbSize) as f32) / (high - low)) as i32;
    this.drawControlFrame(id, new UiRect(rect.x + x, rect.y, this.style.thumbSize, rect.h), UI_COLOR_BUTTON, opt);
    this.drawControlText(uiNumberText(state.value), rect, UI_COLOR_TEXT,
      (opt & (UI_OPT_ALIGN_CENTER | UI_OPT_ALIGN_RIGHT)) === 0 ? opt | UI_OPT_ALIGN_CENTER : opt);
    return old !== state.value ? UI_RES_CHANGE : 0;
  }
  number(label: string, state: UiState<f32>, step: f32, opt: u32 = 0): u32 {
    const id: u32 = this.getId(label);
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect, opt);
    const old: f32 = state.value;
    if (this.focus === id && this.mouseDown === UI_MOUSE_LEFT) state.value += (this.mouseDeltaX as f32) * step;
    this.drawControlFrame(id, rect, UI_COLOR_BASE, opt);
    this.drawControlText(uiNumberText(state.value), rect, UI_COLOR_TEXT,
      (opt & (UI_OPT_ALIGN_CENTER | UI_OPT_ALIGN_RIGHT)) === 0 ? opt | UI_OPT_ALIGN_CENTER : opt);
    return old !== state.value ? UI_RES_CHANGE : 0;
  }
  textbox(label: string, state: UiState<string>, opt: u32 = 0): u32 {
    const id: u32 = this.getId(label);
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect, opt | UI_OPT_HOLD_FOCUS);
    let response: u32 = 0;
    if (this.focus === id) {
      if (this.textInput.length > 0) { state.value += this.textInput; response |= UI_RES_CHANGE; }
      if ((this.keyPressed & UI_KEY_BACKSPACE) !== 0 && state.value.length > 0) {
        state.value = state.value.slice(0, state.value.length - 1); response |= UI_RES_CHANGE;
      }
      if ((this.keyPressed & UI_KEY_RETURN) !== 0) { this.setFocus(0); response |= UI_RES_SUBMIT; }
    }
    this.drawControlFrame(id, rect, UI_COLOR_BASE, opt);
    if (this.focus === id) {
      const width: i32 = this.textWidth(state.value);
      const x: i32 = rect.x + uiMin(this.style.padding, rect.w - this.style.padding - width - 1);
      const y: i32 = rect.y + (rect.h - UI_TEXT_HEIGHT) / 2;
      this.pushClip(rect);
      this.drawText(state.value, x, y, this.style.colors[UI_COLOR_TEXT]);
      this.drawRect(new UiRect(x + width, y, 1, UI_TEXT_HEIGHT), this.style.colors[UI_COLOR_TEXT]);
      this.popClip();
    } else this.drawControlText(state.value, rect, UI_COLOR_TEXT, opt);
    return response;
  }
  label(text: string): void { this.drawControlText(text, this.layoutNext(), UI_COLOR_TEXT); }
  text(text: string): void {
    this.layoutBeginColumn();
    this.layoutRow(this.fullWidth, UI_TEXT_HEIGHT);
    let cursor: i32 = 0;
    let more: boolean = true;
    while (more) {
      const rect: UiRect = this.layoutNext();
      const start: i32 = cursor;
      let end: i32 = cursor;
      let width: i32 = 0;
      while (true) {
        const word: i32 = cursor;
        while (cursor < text.length && text.charCodeAt(cursor) !== 32 && text.charCodeAt(cursor) !== 10) cursor += 1;
        width += this.textWidth(text.slice(word, cursor));
        if (width > rect.w && end !== start) break;
        end = cursor;
        if (cursor === text.length || text.charCodeAt(cursor) === 10) break;
        width += this.textWidth(" "); cursor += 1;
      }
      this.drawText(text.slice(start, end), rect.x, rect.y, this.style.colors[UI_COLOR_TEXT]);
      more = end < text.length;
      cursor = end + 1;
    }
    this.layoutEndColumn();
  }
  private treeHeader(label: string, opt: u32, tree: boolean): u32 {
    const id: u32 = this.getId(label);
    let slot: i32 = -1;
    let oldest: i32 = -1;
    let age: i32 = this.frame;
    for (let i: i32 = 0; i < 48; i += 1) {
      if (this.treeIds[i] === id) slot = i;
      if (this.treeFrames[i] < age) { age = this.treeFrames[i]; oldest = i; }
    }
    let active: boolean = slot >= 0;
    const expanded: boolean = (opt & UI_OPT_EXPANDED) !== 0 ? !active : active;
    this.layoutRow(this.fullWidth, 0);
    const rect: UiRect = this.layoutNext();
    this.updateControl(id, rect);
    if (this.focus === id && this.mousePressed === UI_MOUSE_LEFT) active = !active;
    if (active) {
      if (slot < 0) {
        if (oldest < 0) uiTrap("UIT4", "treeHeader", "slots=48 available=0");
        slot = oldest;
      }
      this.treeIds[slot] = id; this.treeFrames[slot] = this.frame;
    } else if (slot >= 0) { this.treeIds[slot] = 0; this.treeFrames[slot] = 0; }
    if (!tree) this.drawControlFrame(id, rect, UI_COLOR_BUTTON);
    else if (this.hover === id) this.drawFrame(rect, UI_COLOR_BUTTON_HOVER);
    this.drawIcon(expanded ? UI_ICON_EXPANDED : UI_ICON_COLLAPSED,
      new UiRect(rect.x, rect.y, rect.h, rect.h), this.style.colors[UI_COLOR_TEXT]);
    this.drawControlText(label, new UiRect(rect.x + rect.h - this.style.padding, rect.y,
      rect.w - rect.h + this.style.padding, rect.h), UI_COLOR_TEXT);
    return expanded ? UI_RES_ACTIVE : 0;
  }
  header(label: string, opt: u32 = 0): u32 { return this.treeHeader(label, opt, false); }
  beginTreenode(label: string, opt: u32 = 0): u32 {
    const response: u32 = this.treeHeader(label, opt, true);
    if (response !== 0) {
      this.layout("beginTreenode").indent += this.style.indent;
      this.pushIdValue(this.lastId);
      if (this.treeCount === this.treeDepths.length) this.treeDepths.push(this.idCount);
      else this.treeDepths[this.treeCount] = this.idCount;
      this.treeCount += 1;
    }
    return response;
  }
  endTreenode(): void {
    this.requireFrame("endTreenode");
    if (this.treeCount === 0 || this.treeDepths[this.treeCount - 1] !== this.idCount) uiTrap("UIT2", "endTreenode", `depth=${this.treeCount}`);
    this.treeCount -= 1;
    this.layout("endTreenode").indent -= this.style.indent;
    this.popId();
  }
  private container(id: u32, opt: u32): i32 {
    let oldest: i32 = -1;
    let age: i32 = this.frame;
    for (let i: i32 = 0; i < 48; i += 1) {
      const record: UiRoot = this.containers[i];
      if (record.id === id) {
        if (record.open || (opt & UI_OPT_CLOSED) === 0) record.lastUpdate = this.frame;
        return i;
      }
      if (record.lastUpdate < age) { age = record.lastUpdate; oldest = i; }
    }
    if ((opt & UI_OPT_CLOSED) !== 0) return -1;
    // The pool retains every container that the current frame uses.
    if (oldest < 0) uiTrap("UIT4", "container", "slots=48 available=0");
    const record: UiRoot = this.containers[oldest];
    record.id = id; record.rect = new UiRect(0, 0, 0, 0); record.body = record.rect;
    record.scrollX = 0; record.scrollY = 0; record.contentWidth = 0; record.contentHeight = 0;
    record.open = true; record.lastUpdate = this.frame;
    this.lastZindex += 1; record.zindex = this.lastZindex;
    return oldest;
  }
  private pushContainer(slot: i32, kind: i32): void {
    if (this.containerDepth === this.containerStack.length) {
      this.containerStack.push(slot); this.containerKinds.push(kind);
    } else {
      this.containerStack[this.containerDepth] = slot; this.containerKinds[this.containerDepth] = kind;
    }
    this.containerDepth += 1;
  }
  private scrollbar(slot: i32, body: UiRect, vertical: boolean, content: i32): void {
    const record: UiRoot = this.containers[slot];
    const length: i32 = vertical ? body.h : body.w;
    const maximum: i32 = content - length;
    if (maximum <= 0 || length <= 0) {
      if (vertical) record.scrollY = 0; else record.scrollX = 0;
      return;
    }
    const id: u32 = this.getId(vertical ? "!scrollbary" : "!scrollbarx");
    const base: UiRect = vertical ? new UiRect(body.x + body.w, body.y, this.style.scrollbarSize, body.h)
      : new UiRect(body.x, body.y + body.h, body.w, this.style.scrollbarSize);
    this.updateControl(id, base);
    let scroll: i32 = vertical ? record.scrollY : record.scrollX;
    if (this.focus === id && this.mouseDown === UI_MOUSE_LEFT) scroll += (vertical ? this.mouseDeltaY : this.mouseDeltaX) * content / length;
    scroll = uiMax(0, uiMin(maximum, scroll));
    if (vertical) record.scrollY = scroll; else record.scrollX = scroll;
    this.drawFrame(base, UI_COLOR_SCROLL_BASE);
    const size: i32 = uiMax(this.style.thumbSize, length * length / content);
    const offset: i32 = scroll * (length - size) / maximum;
    this.drawFrame(vertical ? new UiRect(base.x, base.y + offset, base.w, size)
      : new UiRect(base.x + offset, base.y, size, base.h), UI_COLOR_SCROLL_THUMB);
    if (this.mouseOver(body)) this.scrollTarget = slot;
  }
  private containerBody(slot: i32, body: UiRect, opt: u32): void {
    const record: UiRoot = this.containers[slot];
    if ((opt & UI_OPT_NO_SCROLL) === 0) {
      const width: i32 = record.contentWidth + this.style.padding * 2;
      const height: i32 = record.contentHeight + this.style.padding * 2;
      this.pushClip(body);
      if (height > record.body.h) body.w -= this.style.scrollbarSize;
      if (width > record.body.w) body.h -= this.style.scrollbarSize;
      this.scrollbar(slot, body, true, height);
      this.scrollbar(slot, body, false, width);
      this.popClip();
    }
    record.body = body;
    const padding: i32 = this.style.padding;
    this.pushLayout(new UiRect(body.x + padding, body.y + padding, body.w - padding * 2, body.h - padding * 2), record.scrollX, record.scrollY);
  }
  beginWindow(title: string, rect: UiRect, opt: u32 = 0): u32 {
    this.requireFrame("beginWindow");
    const id: u32 = this.getId(title);
    const slot: i32 = this.container(id, opt);
    if (slot < 0) return 0;
    const record: UiRoot = this.containers[slot];
    if (!record.open) return 0;
    this.pushIdValue(id);
    if (record.rect.w === 0) record.rect = rect;
    this.pushContainer(slot, 1);
    this.beginRoot(record);
    this.pushClipValue(uiCopy(UI_UNCLIPPED));
    rect = record.rect;
    let body: UiRect = rect;
    if ((opt & UI_OPT_NO_FRAME) === 0) this.drawFrame(rect, UI_COLOR_WINDOW_BG);
    if ((opt & UI_OPT_NO_TITLE) === 0) {
      const titleRect: UiRect = new UiRect(rect.x, rect.y, rect.w, this.style.titleHeight);
      this.drawFrame(titleRect, UI_COLOR_TITLE_BG);
      const titleId: u32 = this.getId("!title");
      this.updateControl(titleId, titleRect, opt);
      this.drawControlText(title, titleRect, UI_COLOR_TITLE_TEXT, opt);
      if (this.focus === titleId && this.mouseDown === UI_MOUSE_LEFT) {
        record.rect = new UiRect(record.rect.x + this.mouseDeltaX, record.rect.y + this.mouseDeltaY, record.rect.w, record.rect.h);
      }
      body.y += titleRect.h; body.h -= titleRect.h;
      if ((opt & UI_OPT_NO_CLOSE) === 0) {
        const closeId: u32 = this.getId("!close");
        const closeRect: UiRect = new UiRect(titleRect.x + titleRect.w - titleRect.h, titleRect.y, titleRect.h, titleRect.h);
        this.drawIcon(UI_ICON_CLOSE, closeRect, this.style.colors[UI_COLOR_TITLE_TEXT]);
        this.updateControl(closeId, closeRect, opt);
        if (this.focus === closeId && this.mousePressed === UI_MOUSE_LEFT) record.open = false;
      }
    }
    this.containerBody(slot, body, opt);
    if ((opt & UI_OPT_NO_RESIZE) === 0) {
      const size: i32 = this.style.titleHeight;
      const resizeId: u32 = this.getId("!resize");
      this.updateControl(resizeId, new UiRect(rect.x + rect.w - size, rect.y + rect.h - size, size, size), opt);
      if (this.focus === resizeId && this.mouseDown === UI_MOUSE_LEFT) {
        record.rect = new UiRect(record.rect.x, record.rect.y, uiMax(96, record.rect.w + this.mouseDeltaX), uiMax(64, record.rect.h + this.mouseDeltaY));
      }
    }
    if ((opt & UI_OPT_AUTO_SIZE) !== 0) {
      const layout: UiLayout = this.layout("beginWindow");
      record.rect = new UiRect(record.rect.x, record.rect.y, record.contentWidth + record.rect.w - layout.body.w,
        record.contentHeight + record.rect.h - layout.body.h);
    }
    if ((opt & UI_OPT_POPUP) !== 0 && this.mousePressed !== 0 && this.hoverRoot !== id) record.open = false;
    this.pushClip(record.body);
    return UI_RES_ACTIVE;
  }
  private endContainer(method: string, kind: i32): void {
    this.requireFrame(method);
    if (this.containerDepth === 0) uiTrap("UIT2", method, "depth=0");
    if (this.containerKinds[this.containerDepth - 1] !== kind) uiTrap("UIT2", method, `kind=${this.containerKinds[this.containerDepth - 1]}`);
    const record: UiRoot = this.containers[this.containerStack[this.containerDepth - 1]];
    this.popClip();
    if (kind !== 2) { this.popClip(); this.endRoot(); }
    const extent: UiRect = this.popLayout();
    record.contentWidth = extent.w; record.contentHeight = extent.h;
    this.popId();
    this.containerDepth -= 1;
  }
  endWindow(): void { this.endContainer("endWindow", 1); }
  beginPanel(label: string, opt: u32 = 0): void {
    this.pushId(label);
    const slot: i32 = this.container(this.lastId, opt);
    if (slot < 0) uiTrap("UIT2", "beginPanel", "closed=true");
    const record: UiRoot = this.containers[slot];
    record.rect = this.layoutNext();
    if ((opt & UI_OPT_NO_FRAME) === 0) this.drawFrame(record.rect, UI_COLOR_PANEL_BG);
    this.pushContainer(slot, 2);
    this.containerBody(slot, record.rect, opt);
    this.pushClip(record.body);
  }
  endPanel(): void { this.endContainer("endPanel", 2); }
  openPopup(label: string): void {
    this.requireFrame("openPopup");
    const slot: i32 = this.container(this.getId(label), 0);
    const record: UiRoot = this.containers[slot];
    record.rect = new UiRect(this.mouseX, this.mouseY, 1, 1); record.open = true;
    this.lastZindex += 1; record.zindex = this.lastZindex;
    this.hoverRoot = record.id; this.nextHoverRoot = record.id; this.nextHoverZ = record.zindex;
  }
  beginPopup(label: string): u32 {
    const response: u32 = this.beginWindow(label, new UiRect(0, 0, 0, 0), UI_OPT_POPUP | UI_OPT_AUTO_SIZE | UI_OPT_NO_RESIZE | UI_OPT_NO_SCROLL | UI_OPT_NO_TITLE | UI_OPT_CLOSED);
    if (response !== 0) this.containerKinds[this.containerDepth - 1] = 3;
    return response;
  }
  endPopup(): void { this.endContainer("endPopup", 3); }
}

@CStruct
export class UiVertex {
  position: Vec2f;
  uv: Vec2f;
  color: u32;
  constructor(position: Vec2f, uv: Vec2f, color: u32) {
    this.position = position; this.uv = uv; this.color = color;
  }
}

@CStruct
export class UiViewport {
  width: f32;
  height: f32;
  constructor(width: f32, height: f32) { this.width = width; this.height = height; }
}

export class UiRenderLayout {
  viewport!: Uniform<UiViewport>;
  atlas!: Texture2d<f32>;
  nearest!: Sampler;
}

@CStruct
export class UiVarying {
  position: Vec4f;
  uv: Vec2f;
  color: Vec4f;
  constructor(position: Vec4f, uv: Vec2f, color: Vec4f) {
    this.position = position; this.uv = uv; this.color = color;
  }
}

export function uiVertex(res: UiRenderLayout, vertex: UiVertex, ctx: VertexInvocation): UiVarying {
  return new UiVarying(
    new Vec4f(vertex.position.x * 2.0 / res.viewport.$.width - 1.0,
      1.0 - vertex.position.y * 2.0 / res.viewport.$.height, 0.0, 1.0),
    vertex.uv,
    new Vec4f((vertex.color % 256) as f32 / 255.0,
      ((vertex.color / 256) % 256) as f32 / 255.0,
      ((vertex.color / 65536) % 256) as f32 / 255.0,
      (vertex.color / 16777216) as f32 / 255.0),
  );
}

export function uiFragment(res: UiRenderLayout, input: UiVarying, ctx: FragmentInvocation): Vec4f {
  const alpha: f32 = res.atlas.sample(res.nearest, input.uv).x;
  return new Vec4f(input.color.x, input.color.y, input.color.z, input.color.w * alpha);
}

export const UI_BLEND: GPUBlendState = {
  color: { operation: "add", srcFactor: "src-alpha", dstFactor: "one-minus-src-alpha" },
  alpha: { operation: "add", srcFactor: "src-alpha", dstFactor: "one-minus-src-alpha" },
};

export class UiPipelineFacts {
  wgsl: string;
  vertexEntry: string;
  fragmentEntry: string;
  layout: BindGroupLayoutSpec;
  vertexLayout: VertexBufferLayoutSpec;
  spec: RenderPipelineSpec;
  constructor(wgsl: string, vertexEntry: string, fragmentEntry: string,
    layout: BindGroupLayoutSpec, vertexLayout: VertexBufferLayoutSpec,
    spec: RenderPipelineSpec) {
    this.wgsl = wgsl; this.vertexEntry = vertexEntry; this.fragmentEntry = fragmentEntry;
    this.layout = layout; this.vertexLayout = vertexLayout;
    this.spec = spec;
  }
}

export class UiDrawRange {
  first: u32;
  count: u32 = 0;
  clip: UiRect;
  constructor(first: u32, clip: UiRect) { this.first = first; this.clip = uiCopy(clip); }
}

function uiValidateRenderer(facts: UiPipelineFacts, capacity: u32): void {
  // A uint16 index addresses at most 65,536 vertices.
  if (capacity === 0 || capacity > 16384) {
    uiTrap("UIT1", "UiRenderer", `capacity=${capacity} maximum=16384`);
  }
  if (facts.spec.topology !== "triangle-list") {
    uiTrap("UIT1", "UiRenderer", `topology=${facts.spec.topology}`);
  }
  if (facts.spec.indexFormat !== "uint16") {
    uiTrap("UIT1", "UiRenderer", `indexFormat=${facts.spec.indexFormat}`);
  }
}

function uiUploadRenderer(queue: GPUQueue, indices: Buffer<u16>, atlas: GPUTexture, capacity: u32): void {
  const indexBytes: u8[] = [];
  for (let q: u32 = 0; q < capacity; q += 1) {
    const pattern: FixedArray<u16, 6> = [
      (q * 4) as u16, (q * 4 + 1) as u16, (q * 4 + 2) as u16,
      (q * 4) as u16, (q * 4 + 2) as u16, (q * 4 + 3) as u16,
    ];
    const bytes: u8[] = Context.bytesOf<FixedArray<u16, 6>>(pattern);
    for (let i: i32 = 0; i < bytes.length; i += 1) indexBytes.push(bytes[i]);
  }
  indices.write(queue, 0, indexBytes);
  const alpha: u8[] = uiAtlasAlpha();
  const atlasBytes: u8[] = [];
  for (let i: i32 = 0; i < alpha.length; i += 1) {
    atlasBytes.push(alpha[i]); atlasBytes.push(alpha[i]); atlasBytes.push(alpha[i]); atlasBytes.push(alpha[i]);
  }
  writeTextureBytes(queue, atlas, atlasBytes, (UI_ATLAS_WIDTH * 4) as u32,
    UI_ATLAS_WIDTH as u32, UI_ATLAS_HEIGHT as u32);
}

export class UiRenderer {
  readonly capacity: u32;
  quadCount: u32 = 0;
  indexCount: u32 = 0;
  rangeCount: i32 = 0;
  vertexBytes: u8[] = [];
  ranges: UiDrawRange[] = [];
  private queue: GPUQueue;
  private ownsQueue: boolean;
  private vertices: Buffer<UiVertex>;
  private indices: Buffer<u16>;
  private atlas: GPUTexture;
  private atlasView: GPUTextureView;
  private nearest: GPUSampler;
  private viewport: Buffer<UiViewport>;
  private group: GPUBindGroup;
  private pipeline: RenderPipeline;
  private clip: UiRect = uiCopy(UI_UNCLIPPED);

  constructor(queue: GPUQueue, ownsQueue: boolean, capacity: u32,
    vertices: Buffer<UiVertex>, indices: Buffer<u16>, atlas: GPUTexture,
    atlasView: GPUTextureView, nearest: GPUSampler, viewport: Buffer<UiViewport>,
    group: GPUBindGroup, pipeline: RenderPipeline) {
    this.queue = queue; this.ownsQueue = ownsQueue; this.capacity = capacity;
    this.vertices = vertices; this.indices = indices; this.atlas = atlas;
    this.atlasView = atlasView; this.nearest = nearest; this.viewport = viewport;
    this.group = group; this.pipeline = pipeline;
  }

  static create(device: GPUDevice, facts: UiPipelineFacts, capacity: u32 = 16384): UiRenderer {
    uiValidateRenderer(facts, capacity);
    const vertexStride: u32 = Context.bytesOf<UiVertex>(new UiVertex(new Vec2f(0, 0), new Vec2f(0, 0), 0)).length as u32;
    const viewportStride: u32 = Context.bytesOf<UiViewport>(new UiViewport(0, 0)).length as u32;
    const queue: GPUQueue = device.queue;
    const vertexUsage: u64 = GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST;
    const indexUsage: u64 = GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST;
    const viewportUsage: u64 = GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST;
    const vertices: Buffer<UiVertex> = new Buffer<UiVertex>(device.createBuffer({
      label: "ui-vertices", size: (vertexStride as u64) * (capacity as u64) * 4, usage: vertexUsage,
    }), vertexStride, capacity * 4, vertexUsage);
    const indices: Buffer<u16> = new Buffer<u16>(device.createBuffer({
      label: "ui-indices", size: (capacity as u64) * 12, usage: indexUsage,
    }), 2, capacity * 6, indexUsage);
    const atlas: GPUTexture = device.createTexture({
      label: "ui-atlas", size: { width: UI_ATLAS_WIDTH as u32, height: UI_ATLAS_HEIGHT as u32 },
      format: "rgba8unorm", usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    uiUploadRenderer(queue, indices, atlas, capacity);
    const atlasView: GPUTextureView = atlas.createView();
    const nearest: GPUSampler = device.createSampler({ minFilter: "nearest", magFilter: "nearest" });
    const viewport: Buffer<UiViewport> = new Buffer<UiViewport>(device.createBuffer({
      label: "ui-viewport", size: viewportStride as u64, usage: viewportUsage,
    }), viewportStride, 1, viewportUsage);
    const pipeline: RenderPipeline = createRenderPipeline(device, facts.wgsl, facts.vertexEntry,
      facts.fragmentEntry, [facts.layout], [facts.vertexLayout], facts.spec);
    using layout = pipeline.bindGroupLayout(0);
    const group: GPUBindGroup = createBindGroup(device, layout, facts.layout, [
      bufferResource(viewport.handle()), textureResource(atlasView), samplerResource(nearest),
    ]);
    return new UiRenderer(queue, false, capacity, vertices, indices, atlas, atlasView,
      nearest, viewport, group, pipeline);
  }

  static createHost(device: GPUHostOwnedDevice, facts: UiPipelineFacts, capacity: u32 = 16384): UiRenderer {
    uiValidateRenderer(facts, capacity);
    const vertexStride: u32 = Context.bytesOf<UiVertex>(new UiVertex(new Vec2f(0, 0), new Vec2f(0, 0), 0)).length as u32;
    const viewportStride: u32 = Context.bytesOf<UiViewport>(new UiViewport(0, 0)).length as u32;
    const queue: GPUQueue = device.queue();
    const vertexUsage: u64 = GPUBufferUsage.VERTEX + GPUBufferUsage.COPY_DST;
    const indexUsage: u64 = GPUBufferUsage.INDEX + GPUBufferUsage.COPY_DST;
    const viewportUsage: u64 = GPUBufferUsage.UNIFORM + GPUBufferUsage.COPY_DST;
    const vertices: Buffer<UiVertex> = new Buffer<UiVertex>(device.createBuffer({
      label: "ui-vertices", size: (vertexStride as u64) * (capacity as u64) * 4, usage: vertexUsage,
    }), vertexStride, capacity * 4, vertexUsage);
    const indices: Buffer<u16> = new Buffer<u16>(device.createBuffer({
      label: "ui-indices", size: (capacity as u64) * 12, usage: indexUsage,
    }), 2, capacity * 6, indexUsage);
    const atlas: GPUTexture = device.createTexture({
      label: "ui-atlas", size: { width: UI_ATLAS_WIDTH as u32, height: UI_ATLAS_HEIGHT as u32 },
      format: "rgba8unorm", usage: GPUTextureUsage.TEXTURE_BINDING + GPUTextureUsage.COPY_DST,
    });
    uiUploadRenderer(queue, indices, atlas, capacity);
    const atlasView: GPUTextureView = atlas.createView();
    const nearest: GPUSampler = device.createSampler({ minFilter: "nearest", magFilter: "nearest" });
    const viewport: Buffer<UiViewport> = new Buffer<UiViewport>(device.createBuffer({
      label: "ui-viewport", size: viewportStride as u64, usage: viewportUsage,
    }), viewportStride, 1, viewportUsage);
    const pipeline: RenderPipeline = createRenderPipelineHost(device, facts.wgsl, facts.vertexEntry,
      facts.fragmentEntry, [facts.layout], [facts.vertexLayout], facts.spec);
    using layout = pipeline.bindGroupLayout(0);
    const group: GPUBindGroup = createBindGroupHost(device, layout, facts.layout, [
      bufferResource(viewport.handle()), textureResource(atlasView), samplerResource(nearest),
    ]);
    return new UiRenderer(queue, true, capacity, vertices, indices, atlas, atlasView,
      nearest, viewport, group, pipeline);
  }

  private quad(x: i32, y: i32, w: i32, h: i32, atlas: i32, color: u32): void {
    const count: u32 = this.quadCount + 1;
    if (count > this.capacity) uiTrap("UIT1", "UiRenderer.build", `capacity=${this.capacity} count=${count}`);
    const u: f32 = (UI_ATLAS_RECT_X[atlas] as f32) / (UI_ATLAS_WIDTH as f32);
    const v: f32 = (UI_ATLAS_RECT_Y[atlas] as f32) / (UI_ATLAS_HEIGHT as f32);
    const right: f32 = ((UI_ATLAS_RECT_X[atlas] + UI_ATLAS_RECT_W[atlas]) as f32) / (UI_ATLAS_WIDTH as f32);
    const bottom: f32 = ((UI_ATLAS_RECT_Y[atlas] + UI_ATLAS_RECT_H[atlas]) as f32) / (UI_ATLAS_HEIGHT as f32);
    const values: FixedArray<UiVertex, 4> = [
      new UiVertex(new Vec2f(x as f32, y as f32), new Vec2f(u, v), color),
      new UiVertex(new Vec2f((x + w) as f32, y as f32), new Vec2f(right, v), color),
      new UiVertex(new Vec2f((x + w) as f32, (y + h) as f32), new Vec2f(right, bottom), color),
      new UiVertex(new Vec2f(x as f32, (y + h) as f32), new Vec2f(u, bottom), color),
    ];
    const bytes: u8[] = Context.bytesOf<FixedArray<UiVertex, 4>>(values);
    const offset: i32 = (this.quadCount as i32) * bytes.length;
    for (let i: i32 = 0; i < bytes.length; i += 1) {
      if (offset + i === this.vertexBytes.length) this.vertexBytes.push(bytes[i]);
      else this.vertexBytes[offset + i] = bytes[i];
    }
    if (this.rangeCount === 0) this.startRange(this.clip);
    this.ranges[this.rangeCount - 1].count += 6;
    this.quadCount = count;
    this.indexCount += 6;
  }

  private startRange(clip: UiRect): void {
    this.clip = uiCopy(clip);
    if (this.rangeCount === 0 || this.ranges[this.rangeCount - 1].count !== 0) {
      if (this.rangeCount === this.ranges.length) this.ranges.push(new UiDrawRange(this.indexCount, clip));
      this.rangeCount += 1;
    }
    const range: UiDrawRange = this.ranges[this.rangeCount - 1];
    range.first = this.indexCount; range.count = 0; range.clip = uiCopy(clip);
  }

  private commands(context: UiContext, start: i32, end: i32): void {
    for (let i: i32 = start; i < end; i += 1) {
      const command: UiCommand = context.commands[i];
      if (command.kind === 1) {
        this.startRange(new UiRect(command.x, command.y, command.w, command.h));
        continue;
      }
      if (command.kind === 2) {
        this.quad(command.x, command.y, command.w, command.h, UI_ATLAS_WHITE, command.color);
      } else if (command.kind === 4) {
        const w: i32 = UI_ATLAS_RECT_W[command.id];
        const h: i32 = UI_ATLAS_RECT_H[command.id];
        this.quad(command.x + (command.w - w) / 2, command.y + (command.h - h) / 2,
          w, h, command.id, command.color);
      } else if (command.kind === 3) {
        let x: i32 = command.x;
        for (let j: i32 = 0; j < command.text.length; j += 1) {
          const byte: i32 = command.text.charCodeAt(j);
          if (byte >= 32 && byte <= 126) {
            const glyph: i32 = UI_ATLAS_FONT + byte;
            const w: i32 = UI_ATLAS_RECT_W[glyph];
            this.quad(x, command.y, w, UI_ATLAS_RECT_H[glyph], glyph, command.color);
            x += w;
          }
        }
      }
    }
  }

  build(context: UiContext): void {
    this.quadCount = 0; this.indexCount = 0;
    this.rangeCount = 0;
    this.clip = uiCopy(UI_UNCLIPPED);
    const order: i32[] = context.drawOrder();
    if (context.rootCount === 0) this.commands(context, 0, context.commandCount);
    for (let i: i32 = 0; i < order.length; i += 1) {
      const root: UiRoot = context.roots[order[i]];
      this.commands(context, root.start, root.end);
    }
    if (this.rangeCount > 0 && this.ranges[this.rangeCount - 1].count === 0) this.rangeCount -= 1;
  }

  render(context: UiContext, pass: GPURenderPassEncoder, width: u32, height: u32): void {
    this.build(context);
    this.viewport.write(this.queue, 0, Context.bytesOf<UiViewport>(new UiViewport(width as f32, height as f32)));
    if (this.quadCount === 0) { pass.setScissorRect(0, 0, width, height); return; }
    this.vertices.write(this.queue, 0, this.vertexBytes);
    this.pipeline.bind(pass, [this.group], [this.vertices.handle()]);
    this.pipeline.setIndexBuffer(pass, this.indices.handle());
    for (let i: i32 = 0; i < this.rangeCount; i += 1) {
      const range: UiDrawRange = this.ranges[i];
      const clip: UiRect = uiIntersection(range.clip, new UiRect(0, 0, width as i32, height as i32));
      if (clip.w === 0 || clip.h === 0) continue;
      pass.setScissorRect(clip.x as u32, clip.y as u32, clip.w as u32, clip.h as u32);
      pass.drawIndexed(range.count, 1, range.first, 0, 0);
    }
    pass.setScissorRect(0, 0, width, height);
  }

  dispose(): void {
    this.group.dispose(); this.pipeline.dispose(); this.viewport.dispose(); this.nearest.dispose();
    this.atlasView.dispose(); this.atlas.dispose(); this.indices.dispose(); this.vertices.dispose();
    if (this.ownsQueue) this.queue.dispose();
  }
  [Symbol.dispose](): void { this.dispose(); }
}
