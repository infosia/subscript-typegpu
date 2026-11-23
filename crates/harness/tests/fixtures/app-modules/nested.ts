import { tileSize } from "./sub/tile";
import { Edge_SIZE } from "./nested.typegpu";

@ValueType class Edge { width: u32 = 0; height: u32 = 0; }
export function main(): void { print(`${tileSize()},${Edge_SIZE}`); }
