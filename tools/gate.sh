#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

case "$#" in
    0)
        ;;
    1)
        if [ "$1" = "--measure" ]; then
            echo "gate: --measure not implemented in this slice" >&2
        else
            echo "gate: unknown argument: $1" >&2
        fi
        exit 2
        ;;
    *)
        echo "usage: tools/gate.sh [--measure]" >&2
        exit 2
        ;;
esac

: "${CARGO_BUILD_JOBS:=4}"
export CARGO_BUILD_JOBS

cargo --offline fmt --all -- --check
cargo clippy --workspace --offline -- -D warnings

test_log=$(mktemp "${TMPDIR:-/tmp}/subscript-typegpu-gate.XXXXXX")
trap 'rm -f "$test_log"' EXIT HUP INT TERM
if cargo test --workspace --offline -- --nocapture >"$test_log" 2>&1; then
    cat "$test_log"
else
    cat "$test_log"
    exit 1
fi

tools/hygiene.sh

grep '^pending:' "$test_log" || true
echo "gate: green"
