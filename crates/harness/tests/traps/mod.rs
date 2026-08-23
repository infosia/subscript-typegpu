use std::path::{Path, PathBuf};

use subscript_codegen::ReloadSession;
use subscript_compiler::SourceFile;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn assert_trap(fixture: &Path, expected: &str, generate_support: bool) {
    let mut files = subscript_typegpu_harness::program_files(fixture)
        .unwrap_or_else(|error| panic!("load {}: {error}", fixture.display()));
    if generate_support {
        let generated = subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
            panic!(
                "generate {}:\n{}",
                fixture.display(),
                subscript_compiler::render_diagnostics(&files, &diagnostics),
            )
        });
        let stem = fixture
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("trap fixture has a UTF-8 stem");
        files.push(SourceFile::new(
            format!("{stem}.typegpu.ts"),
            generated.support_module,
        ));
    }
    let libraries = [subscript_typegpu_harness::facade_library()];
    let mut session = ReloadSession::new_with_native_libraries(&files, &libraries)
        .unwrap_or_else(|error| panic!("compile {} dev: {error}", fixture.display()));
    let mut trapped = session.call_main().is_err();
    while !trapped && session.async_pending() != 0 {
        trapped = session.async_step().is_err();
    }
    assert!(
        trapped,
        "{} unexpectedly passed with output:\n{}",
        fixture.display(),
        String::from_utf8_lossy(&session.take_output()),
    );
    let text = String::from_utf8(session.take_output())
        .unwrap_or_else(|error| panic!("{} output is not UTF-8: {error}", fixture.display()));
    assert!(
        text.contains(expected),
        "{} trap lacks `{expected}`:\n{text}",
        fixture.display(),
    );
}

#[test]
fn runtime_traps_are_named_and_numbered() {
    if !super::differential::backend_is_available() {
        return;
    }
    let directory = repository_root().join("crates/harness/tests/fixtures/trap");
    for (name, expected, generate_support) in [
        (
            "patch-past-field.ts",
            "EG2 Buffer.patch fieldOffset=3 byteLength=2 elementSize=4",
            false,
        ),
        (
            "patch-unaligned.ts",
            "BF2 Buffer.patch byteOffset=2 byteLength=2",
            false,
        ),
        (
            "copy-unaligned.ts",
            "BF8 Buffer.copyTo byteOffset=0 byteLength=2",
            false,
        ),
        (
            "write-past-end.ts",
            "BF8 Buffer.write elementIndex=1 elementCount=2 count=2",
            false,
        ),
        (
            "write-non-multiple.ts",
            "BF8 Buffer.write byteLength=3 elementSize=4 remainder=3",
            false,
        ),
        (
            "read-unaligned.ts",
            "BF9 Buffer.read byteOffset=0 byteLength=2",
            false,
        ),
        (
            "read-past-end.ts",
            "BF8 readOne elementIndex=2 elementCount=1 count=2",
            false,
        ),
        (
            "owned-read-past-end.ts",
            "BF9 Buffer.read elementIndex=1 elementCount=1 count=1",
            false,
        ),
        (
            "owned-read-one-past-end.ts",
            "BF9 Buffer.readOne elementIndex=1 elementCount=1 count=1",
            false,
        ),
        (
            "map-failure.ts",
            "BF9 Buffer.read elementIndex=0 elementCount=1 count=1",
            false,
        ),
        (
            "write-unaligned.ts",
            "BF2 Buffer.write byteOffset=0 byteLength=6",
            false,
        ),
        (
            "read-without-copy-src.ts",
            "BF10 Buffer.read usage=8",
            false,
        ),
        (
            "write-without-copy-dst.ts",
            "BF10 Buffer.write usage=4",
            false,
        ),
        (
            "write-one-without-copy-dst.ts",
            "BF10 Buffer.writeOne usage=4",
            false,
        ),
        (
            "patch-without-copy-dst.ts",
            "BF10 Buffer.patch usage=4",
            false,
        ),
        (
            "resource-kind-mismatch.ts",
            "TX4 createBindGroup binding=3 expected=texture actual=buffer",
            false,
        ),
        (
            "resource-count-mismatch.ts",
            "PI9 createBindGroup expected 1 resources but received 0",
            false,
        ),
        (
            "resource-two-fields.ts",
            "TX4 createBindGroup binding=1 resourceFields=2",
            false,
        ),
        (
            "sampled-texture-store.ts",
            "TX3 store is not legal on Texture2d",
            false,
        ),
        (
            "storage-texture-no-format.ts",
            "TX5 storageTexture binding=2 has no format",
            false,
        ),
        (
            "unknown-layout-kind.ts",
            "TX5 bind group layout binding=4 has unknown kind=mystery",
            false,
        ),
        (
            "simulate-storage-barrier.ts",
            "CL2 simulateCompute pipeline=blocked",
            true,
        ),
        (
            "guarded-indirect.ts",
            "PI16 ComputePipeline.dispatchIndirect guarded=true",
            false,
        ),
        (
            "guarded-second-dispatch.ts",
            "PI15 ComputePipeline.dispatch x=2 y=1 z=1",
            false,
        ),
        (
            "index-buffer-without-format.ts",
            "RN18 RenderPipeline.setIndexBuffer indexFormat=undefined",
            false,
        ),
    ] {
        assert_trap(&directory.join(name), expected, generate_support);
    }
}
