// expected-rule: UIT4
import { UiContext, UiRect } from "./typegpu-ui";
export function main(): void {
  const ui: UiContext = new UiContext();
  ui.begin();
  for (let i: i32 = 0; i < 49; i += 1) {
    ui.beginWindow(`window${i}`, new UiRect(0, 0, 100, 100));
    ui.endWindow();
  }
  ui.end();
}
