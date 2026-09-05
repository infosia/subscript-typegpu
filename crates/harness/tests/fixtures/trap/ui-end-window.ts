// expected-rule: UIT2
import { UiContext } from "./typegpu-ui";
export function main(): void {
  const ui: UiContext = new UiContext();
  ui.begin();
  ui.endWindow();
}
