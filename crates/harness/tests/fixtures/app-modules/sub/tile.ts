import { Tile_SIZE } from "../nested.typegpu";

@ValueType class Tile { edge: u32 = 0; }
export function tileSize(): u32 { return Tile_SIZE; }
