#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"

cargo run --offline -p subscript-typegpu-webgpu-gen -- "$(pwd)"

scratch_dir=$(mktemp -d "${TMPDIR:-/tmp}/subscript-typegpu-regen.XXXXXX")
trap 'rm -rf -- "$scratch_dir"' EXIT HUP INT TERM

for program in programs/[bx]*.ts; do
  case "$program" in
    *.typegpu.ts) continue ;;
  esac
  cargo run --offline -p subscript-typegpu-gen -- gen "$program" --lib lib -o "$scratch_dir"
done

for wgsl in "$scratch_dir"/*.wgsl; do
  if [ ! -f "$wgsl" ]; then
    continue
  fi
  cp "$wgsl" programs/
done
