import { Uniform } from "../sub/typegpu";
import { Width_SIZE } from "../sub/main.typegpu";

@ValueType class Width { value: u32 = 0; }
export function width(): u32 { return new Uniform<u32>(3).$ + Width_SIZE; }
