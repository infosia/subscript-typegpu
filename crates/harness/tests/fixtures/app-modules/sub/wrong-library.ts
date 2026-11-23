// The registered module is at the root.
import { MutStorage } from "./typegpu";
export function width(): u32 { return new MutStorage<u32>([1])[0]; }
