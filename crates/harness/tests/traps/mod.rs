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

#[test]
fn buffer_bounds_traps_are_named_and_numbered() {
    let directory = repository_root().join("crates/harness/tests/fixtures/trap");
    for (name, expected) in [
        (
            "write-past-end.ts",
            "BF8 Buffer.write elementIndex=1 elementCount=2 count=2",
        ),
        (
            "write-non-multiple.ts",
            "BF8 Buffer.write byteLength=3 elementSize=4 remainder=3",
        ),
        (
            "read-past-end.ts",
            "BF8 Buffer.read elementIndex=1 elementCount=2 count=2",
        ),
    ] {
        assert_trap(&directory.join(name), expected);
    }
}
