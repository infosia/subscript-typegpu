// expected-rule: UIT4
import { UiContext, UiRect, UI_OPT_EXPANDED, UI_MOUSE_LEFT } from "./typegpu-ui";
export function main(): void {
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.pushLayout(new UiRect(0, 0, 100, 2000));
  for (let i: i32 = 0; i < 49; i += 1) {
    const label: string = `tree${i}`;
    // A toggle reserves a pool slot for the default-expanded node.
    ui.hover = ui.getId(label);
    ui.inputMouseDown(0, 0, UI_MOUSE_LEFT);
    if (ui.beginTreenode(label, UI_OPT_EXPANDED) !== 0) ui.endTreenode();
  }
  ui.popLayout();
  ui.end();
}
