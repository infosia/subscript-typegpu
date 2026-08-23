#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

if [ "$#" -ne 1 ]; then
  echo "usage: tools/example.sh <program.ts>" >&2
  exit 2
fi
program=$1

backend=${SUBSCRIPT_TYPEGPU_BACKEND:-}
case "$backend" in
  default|metal|vulkan|gles|d3d11|d3d12) ;;
  *)
    echo "example: set SUBSCRIPT_TYPEGPU_BACKEND to default, metal, vulkan, gles, d3d11, or d3d12" >&2
    exit 1
    ;;
esac
if [ "$backend" = "default" ]; then
  unset SUBSCRIPT_TYPEGPU_BACKEND
fi

backend_lib=${SUBSCRIPT_TYPEGPU_BACKEND_LIB:-}
if [ -z "$backend_lib" ] || [ ! -f "$backend_lib" ]; then
  echo "example: SUBSCRIPT_TYPEGPU_BACKEND_LIB must name a backend library file" >&2
  exit 1
fi

if [ "$(uname -s)" = "Darwin" ] && [ "$backend" = "metal" ]; then
  if ! otool -L "$backend_lib" | grep -F 'Metal.framework' >/dev/null; then
    echo "example: the Metal backend library does not link Metal.framework" >&2
    exit 1
  fi
fi

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
exec cargo run --offline -p subscript-typegpu-harness -- dev "$program"
