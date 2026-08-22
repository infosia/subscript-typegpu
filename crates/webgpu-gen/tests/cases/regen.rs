//! T6 regeneration runs the shipped generator binary against a scratch root.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

const INPUTS: [&str; 4] = [
    "third_party/webgpu-headers/webgpu.yml",
    "third_party/gpuweb/spec/index.bs",
    "third_party/gpuweb/spec/sections/copies.bs",
    "crates/webgpu-gen/policy.toml",
];

const OUTPUTS: [&str; 6] = [
    "crates/facade/subscript-typegpu.h",
    "crates/facade/src/generated.rs",
    "lib/subscript-typegpu.generated.d.ts",
    "lib/wire-enum-aliases.generated.d.ts",
    "lib/webgpu.ts",
    "crates/harness/src/native_symbols.generated.rs",
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

struct ScratchRoot(PathBuf);

impl ScratchRoot {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "subscript-typegpu-regen-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)
            .unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
        ScratchRoot(path)
    }

    fn copy_from_repository(&self, relative: &str) {
        let source = repository_root().join(relative);
        let destination = self.0.join(relative);
        std::fs::create_dir_all(destination.parent().expect("copied file has a parent"))
            .unwrap_or_else(|error| panic!("create parent for {}: {error}", destination.display()));
        std::fs::copy(&source, &destination).unwrap_or_else(|error| {
            panic!(
                "copy {} to {}: {error}",
                source.display(),
                destination.display()
            )
        });
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Regeneration {
    scratch: ScratchRoot,
    libclang_pending: bool,
}

fn regeneration() -> &'static Regeneration {
    static REGENERATION: OnceLock<Regeneration> = OnceLock::new();
    REGENERATION.get_or_init(|| {
        let scratch = ScratchRoot::new();
        for relative in INPUTS.into_iter().chain(OUTPUTS) {
            scratch.copy_from_repository(relative);
        }
        let output = Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-webgpu-gen"))
            .arg(&scratch.0)
            .output()
            .expect("run subscript-typegpu-webgpu-gen");
        let stderr = String::from_utf8_lossy(&output.stderr);
        let libclang_pending = !output.status.success()
            && stderr.to_ascii_lowercase().contains("libclang")
            && stderr.contains("libclang output pass");
        if !output.status.success() && !libclang_pending {
            panic!(
                "generator failed for scratch root\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                stderr
            );
        }
        Regeneration {
            scratch,
            libclang_pending,
        }
    })
}

fn compare(relative: &str) {
    let regeneration = regeneration();
    assert_eq!(
        read(&repository_root().join(relative)),
        read(&regeneration.scratch.0.join(relative)),
        "{relative} differs; run tools/regen.sh"
    );
}

fn compare_libclang_output(relative: &str) {
    if regeneration().libclang_pending {
        crate::support::pend_libclang();
        return;
    }
    compare(relative);
}

#[test]
fn facade_header_is_current() {
    compare("crates/facade/subscript-typegpu.h");
}

#[test]
fn facade_rust_is_current() {
    compare("crates/facade/src/generated.rs");
}

#[test]
fn ambient_mirror_is_current() {
    compare_libclang_output("lib/subscript-typegpu.generated.d.ts");
}

#[test]
fn wire_enum_aliases_are_current() {
    compare_libclang_output("lib/wire-enum-aliases.generated.d.ts");
}

#[test]
fn webgpu_api_is_current() {
    compare_libclang_output("lib/webgpu.ts");
}

#[test]
fn ambient_mirror_is_raw_bindgen_output() {
    let header = read(&repository_root().join("crates/facade/subscript-typegpu.h"));
    let Some(mirror) = crate::support::bind_header(&header) else {
        return;
    };
    assert_eq!(
        read(&repository_root().join("lib/subscript-typegpu.generated.d.ts")),
        mirror,
        "ambient mirror differs from bindgen output; run tools/regen.sh"
    );
}

#[test]
fn native_symbols_are_current() {
    compare("crates/harness/src/native_symbols.generated.rs");
}

#[test]
fn hard_coded_calls_are_in_the_function_table() {
    let generated = read(&repository_root().join("crates/facade/src/generated.rs"));
    for name in [
        "wgpuBufferRelease",
        "wgpuDeviceRelease",
        "wgpuAdapterInfoFreeMembers",
    ] {
        assert!(
            generated.contains(&format!("{name}: unsafe extern \"C\" fn")),
            "function table misses {name}"
        );
    }
}

#[test]
fn every_symbol_table_name_is_emitted() {
    let generated = read(&repository_root().join("crates/facade/src/generated.rs"));
    let symbols = read(&repository_root().join("crates/harness/src/native_symbols.generated.rs"));
    for line in symbols.lines() {
        let Some(name) = line.trim().strip_prefix("(\"") else {
            continue;
        };
        let Some((name, _)) = name.split_once('"') else {
            continue;
        };
        assert!(
            generated.contains(&format!("pub extern \"C\" fn {name}")),
            "facade source misses planned export {name}"
        );
    }
}

#[test]
fn shims_name_the_symbol_and_backend_variable_before_abort() {
    let generated = read(&repository_root().join("crates/facade/src/generated.rs"));
    assert!(generated.contains(
        "eprintln!(\"subscript-typegpu: cannot call wgpuCreateInstance: set SUBSCRIPT_TYPEGPU_BACKEND_LIB\")"
    ));
}
