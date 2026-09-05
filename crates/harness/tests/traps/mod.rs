use std::path::{Path, PathBuf};

use subscript_codegen::ReloadSession;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn assert_trap(fixture: &Path, expected: &str) {
    let files = subscript_typegpu_harness::program_files(fixture)
        .unwrap_or_else(|error| panic!("load {}: {error}", fixture.display()));
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

fn assert_accept_export(fixture: &Path, export: &str) {
    let files = subscript_typegpu_harness::program_files(fixture)
        .unwrap_or_else(|error| panic!("load {}: {error}", fixture.display()));
    let libraries = [subscript_typegpu_harness::facade_library()];
    let mut session = ReloadSession::new_with_native_libraries(&files, &libraries)
        .unwrap_or_else(|error| panic!("compile {} dev: {error}", fixture.display()));
    let mut trapped = session.call_export(export).is_err();
    while !trapped && session.async_pending() != 0 {
        trapped = session.async_step().is_err();
    }
    assert!(
        !trapped,
        "{} export `{export}` unexpectedly trapped with output:\n{}",
        fixture.display(),
        String::from_utf8_lossy(&session.take_output()),
    );
}

#[test]
fn guarded_dispatch_accepts_fresh_encoder_each_frame() {
    if !super::differential::backend_is_available() {
        return;
    }
    let fixture =
        repository_root().join("crates/harness/tests/fixtures/trap/guarded-second-dispatch.ts");
    assert_accept_export(&fixture, "accept");
}

#[test]
fn runtime_traps_are_named_and_numbered() {
    if !super::differential::backend_is_available() {
        return;
    }
    let directory = repository_root().join("crates/harness/tests/fixtures/trap");
    for (name, expected) in [
        (
            "patch-past-field.ts",
            "EG2 Buffer.patch fieldOffset=3 byteLength=2 elementSize=4",
        ),
        (
            "patch-unaligned.ts",
            "BF2 Buffer.patch byteOffset=2 byteLength=2",
        ),
        (
            "copy-unaligned.ts",
            "BF8 Buffer.copyTo byteOffset=0 byteLength=2",
        ),
        (
            "write-past-end.ts",
            "BF8 Buffer.write elementIndex=1 elementCount=2 count=2",
        ),
        (
            "write-non-multiple.ts",
            "BF8 Buffer.write byteLength=3 elementSize=4 remainder=3",
        ),
        (
            "read-unaligned.ts",
            "BF9 Buffer.read byteOffset=0 byteLength=2",
        ),
        (
            "read-past-end.ts",
            "BF8 readOne elementIndex=2 elementCount=1 count=2",
        ),
        (
            "owned-read-past-end.ts",
            "BF9 Buffer.read elementIndex=1 elementCount=1 count=1",
        ),
        (
            "owned-read-one-past-end.ts",
            "BF9 Buffer.readOne elementIndex=1 elementCount=1 count=1",
        ),
        (
            "map-failure.ts",
            "BF9 Buffer.read elementIndex=0 elementCount=1 count=1",
        ),
        (
            "write-unaligned.ts",
            "BF2 Buffer.write byteOffset=0 byteLength=6",
        ),
        ("read-without-copy-src.ts", "BF10 Buffer.read usage=8"),
        ("write-without-copy-dst.ts", "BF10 Buffer.write usage=4"),
        (
            "write-one-without-copy-dst.ts",
            "BF10 Buffer.writeOne usage=4",
        ),
        ("patch-without-copy-dst.ts", "BF10 Buffer.patch usage=4"),
        (
            "resource-kind-mismatch.ts",
            "TX4 createBindGroup binding=3 expected=texture actual=buffer",
        ),
        (
            "resource-count-mismatch.ts",
            "PI9 createBindGroup expected 1 resources but received 0",
        ),
        (
            "resource-two-fields.ts",
            "TX4 createBindGroup binding=1 resourceFields=2",
        ),
        (
            "sampled-texture-store.ts",
            "TX3 store is not legal on Texture2d",
        ),
        (
            "storage-texture-no-format.ts",
            "TX5 storageTexture binding=2 has no format",
        ),
        (
            "unknown-layout-kind.ts",
            "TX5 bind group layout binding=4 has unknown kind=mystery",
        ),
        (
            "simulate-storage-barrier.ts",
            "CL2 simulateCompute pipeline=blocked",
        ),
        (
            "guarded-indirect.ts",
            "PI16 ComputePipeline.dispatchIndirect guarded=true",
        ),
        (
            "guarded-second-dispatch.ts",
            "PI15 ComputePipeline.dispatch x=2 y=1 z=1",
        ),
        (
            "index-buffer-without-format.ts",
            "RN18 RenderPipeline.setIndexBuffer indexFormat=undefined",
        ),
        (
            "texture-row-alignment.ts",
            "TX9 writeTextureBytes bytesPerRow=8 height=2",
        ),
        (
            "unsupported-host-blend.ts",
            "RN21 hostBlend color srcFactor=zero dstFactor=one-minus-src-alpha operation=add",
        ),
    ] {
        assert_trap(&directory.join(name), expected);
    }
}

#[test]
fn ui_runtime_traps_are_named_and_numbered() {
    let directory = repository_root().join("crates/harness/tests/fixtures/trap");
    for (name, expected) in [
        ("ui-end-window.ts", "UIT2 endWindow depth=0 (author)"),
        (
            "ui-row-widths.ts",
            "UIT3 layoutRow widths=17 maximum=16 (author)",
        ),
    ] {
        assert_trap(&directory.join(name), expected);
        println!("{expected}");
    }
}
