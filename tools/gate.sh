#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

: "${CARGO_BUILD_JOBS:=4}"
export CARGO_BUILD_JOBS

cargo_program=${CARGO:-cargo}
cargo() {
    subcommand=$1
    shift
    if [ "${1:-}" != "--offline" ]; then
        echo "gate error: cargo $subcommand lacks --offline" >&2
        exit 1
    fi
    shift
    command "$cargo_program" --offline "$subcommand" "$@"
}

gate_tmp=$(mktemp -d "${TMPDIR:-/tmp}/subscript-typegpu-gate.XXXXXX")
codegen_backup=
program_backup=

restore_measure_files() {
    if [ -n "$codegen_backup" ] && [ -f "$codegen_backup" ]; then
        cp "$codegen_backup" crates/typegpu-gen/src/lib.rs
    fi
    if [ -n "$program_backup" ] && [ -f "$program_backup" ]; then
        cp "$program_backup" programs/a01-smoke.ts
    fi
}

cleanup() {
    restore_measure_files
    rm -rf "$gate_tmp"
}
trap cleanup EXIT HUP INT TERM

gate_run=0
last_test_log=
require_backend=0

run_gate() {
    gate_run=$((gate_run + 1))
    cargo fmt --offline --all -- --check
    cargo clippy --offline --workspace -- -D warnings

    last_test_log="$gate_tmp/test-$gate_run.log"
    if cargo test --offline --workspace -- --nocapture >"$last_test_log" 2>&1; then
        cat "$last_test_log"
    else
        cat "$last_test_log"
        return 1
    fi

    tools/hygiene.sh
    pending_count=$(grep -c '^pending:' "$last_test_log" || true)
    grep '^pending:' "$last_test_log" || true
    if [ "$pending_count" -gt 0 ]; then
        if [ "$require_backend" -eq 1 ]; then
            echo "gate: red, pending $pending_count"
            return 1
        fi
        echo "gate: green, pending $pending_count"
    else
        echo "gate: green"
    fi
}

elapsed() {
    started=$1
    finished=$(date +%s)
    echo $((finished - started))
}

measure() {
    echo "CAUTION: tools/gate.sh --measure runs cargo clean. It removes the workspace target directory."

    cargo clean --offline
    if [ -d target/debug ] || [ -d target/ship-build ]; then
        echo "measurement error: cargo clean left a build directory under target" >&2
        exit 1
    fi
    if [ -d target ] && [ -n "$(find target -mindepth 1 -print -quit)" ]; then
        echo "measurement error: cargo clean did not empty target" >&2
        exit 1
    fi

    started=$(date +%s)
    cargo build --offline --workspace --tests
    cargo rustc --offline --release -p subscript-typegpu-facade \
        --message-format=json --target-dir target/ship-build -- \
        --print native-static-libs
    cargo build --offline --release -p subscript-runtime \
        --message-format=json --target-dir target/ship-build
    cold=$(elapsed "$started")

    started=$(date +%s)
    cargo test --offline --workspace --no-run
    warm=$(elapsed "$started")

    run_gate

    codegen_backup="$gate_tmp/typegpu-gen-lib.rs"
    cp crates/typegpu-gen/src/lib.rs "$codegen_backup"
    printf '\n// T12 measurement probe.\n' >> crates/typegpu-gen/src/lib.rs
    started=$(date +%s)
    run_gate
    codegen_gate=$(elapsed "$started")
    cp "$codegen_backup" crates/typegpu-gen/src/lib.rs
    codegen_backup=

    run_gate

    program_backup="$gate_tmp/a01-smoke.ts"
    cp programs/a01-smoke.ts "$program_backup"
    printf '\n// T12 measurement probe.\n' >> programs/a01-smoke.ts
    started=$(date +%s)
    run_gate
    program_gate=$(elapsed "$started")
    cp "$program_backup" programs/a01-smoke.ts
    program_backup=

    if ! executable_count=$(grep -c '^     Running ' "$last_test_log"); then
        echo "measurement error: cargo test log has no Running line" >&2
        exit 1
    fi
    measured_date=$(date +%F)
    revision=$(git rev-parse --short HEAD)
    echo "| $measured_date | $revision | $cold | $warm | $codegen_gate | $program_gate | $executable_count |"
}

case "$#" in
    0)
        run_gate
        ;;
    1)
        if [ "$1" = "--measure" ]; then
            echo "CAUTION: tools/gate.sh --measure runs cargo clean. Re-run with --measure --yes." >&2
        elif [ "$1" = "--require-backend" ]; then
            require_backend=1
            run_gate
            exit 0
        else
            echo "usage: tools/gate.sh [--require-backend | --measure --yes]" >&2
        fi
        exit 2
        ;;
    2)
        if [ "$1" = "--measure" ] && [ "$2" = "--yes" ]; then
            require_backend=1
            measure
        else
            echo "usage: tools/gate.sh [--require-backend | --measure --yes]" >&2
            exit 2
        fi
        ;;
    *)
        echo "usage: tools/gate.sh [--require-backend | --measure --yes]" >&2
        exit 2
        ;;
esac
