// expected-rule: UIT3
import { UiContext, UiRect } from "./typegpu-ui";
export function main(): void {
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.pushLayout(new UiRect(0, 0, 100, 100));
  ui.layoutRow([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], 0);
}
