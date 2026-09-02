// expected-rule: S018
// expected-owner: checker
// expected-message: `Vec3h` has no method `abs`
import { Vec3h } from "./typegpu-types";

function rejected(value: Vec3h): void {
  value.abs();
}
