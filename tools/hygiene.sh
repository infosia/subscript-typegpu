#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

failed=0
file_list=$(mktemp "${TMPDIR:-/tmp}/subscript-typegpu-hygiene.XXXXXX")
trap 'rm -f "$file_list"' EXIT HUP INT TERM
git ls-files --cached --others --exclude-standard >"$file_list"
while IFS= read -r file; do
    case "$file" in
        tools/hygiene.sh|HANDOFF*.md|third_party/*|target/*)
            continue
            ;;
    esac

    mode=$(git ls-files -s -- "$file" | sed -n '1s/ .*//p')
    if [ "$mode" = "160000" ]; then
        continue
    fi
    if [ ! -f "$file" ]; then
        continue
    fi

    if grep -nE '/(Users|home|Documents|workspace|Projects|repos)/|~/|\.\./\.\.|\.\./(subscript|subscript-gpu|yawgpu|gpuweb|webgpu-native-cts|webgpu-headers|TypeGPU)|(^|[^[:alnum:]])[A-Za-z]:[\\/]|(^|[^[:alnum:]_])(sgpu|SGPU|stgpu|STGPU)([^[:alnum:]_]|$)|(^|[^[:alnum:]_])tgpu([^[:alnum:]_]|$)' "$file"; then
        echo "hygiene: forbidden text in $file" >&2
        failed=1
    fi
done <"$file_list"

if find crates -name build.rs -type f -print | grep .; then
    echo "hygiene: build.rs exists" >&2
    failed=1
fi

if { grep -nH '^\[features\]' Cargo.toml || find crates -name Cargo.toml -type f -exec grep -nH '^\[features\]' {} +; }; then
    echo "hygiene: a features table exists" >&2
    failed=1
fi

for tests_dir in crates/*/tests; do
    if [ ! -d "$tests_dir" ]; then
        continue
    fi
    count=$(find "$tests_dir" -maxdepth 1 -name '*.rs' -type f | wc -l | tr -d ' ')
    if [ "$count" -gt 1 ]; then
        echo "hygiene: more than one Rust test file in $tests_dir" >&2
        failed=1
    fi
done

exit "$failed"
