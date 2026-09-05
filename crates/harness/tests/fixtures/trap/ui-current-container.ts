// expected-rule: UIT2
import { UiContext, UiRect } from "./typegpu-ui";

export function main(): void {
  const ui = new UiContext();
  ui.begin();
  ui.currentContainer();
}

export function accept(): void {
  const ui = new UiContext();
  ui.begin();
  if (ui.beginWindow("Window", new UiRect(0, 0, 300, 300)) === 0) unreachable();
  const window = ui.currentContainer();
  window.rect = new UiRect(0, 0, 320, 340);
  ui.layoutRow([-1], 100);
  ui.beginPanel("Panel");
  const panel = ui.currentContainer();
  if (panel.id === window.id) unreachable();
  ui.layoutRow([-1], 200);
  ui.label("Content");
  ui.endPanel();
  panel.scrollY = panel.contentHeight;
  if (panel.scrollY <= 0) unreachable();
  if (ui.currentContainer().id !== window.id) unreachable();
  if (ui.currentContainer().rect.w !== 320) unreachable();
  ui.endWindow();
  ui.end();
}

export function outsideFrame(): void {
  const ui = new UiContext();
  ui.currentContainer();
}
