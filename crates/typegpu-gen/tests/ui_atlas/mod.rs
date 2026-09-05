use crate::support;

#[test]
fn atlas_regeneration_is_byte_identical() {
    let root = support::root();
    let generated = subscript_typegpu_gen::generate_ui_atlas(&root).expect("generate atlas");
    assert_eq!(
        generated,
        support::read(&root.join("lib/typegpu-ui-atlas.generated.ts"))
    );
    let hex = generated
        .split("UI_ATLAS_ALPHA_HEX: string = \"")
        .nth(1)
        .expect("alpha constant")
        .split('"')
        .next()
        .expect("alpha data");
    assert_eq!(hex.len(), 32768);
    assert!(hex
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    for name in ["X", "Y", "W", "H"] {
        let marker = format!("UI_ATLAS_RECT_{name}: i32[] = [");
        let values = generated
            .split(&marker)
            .nth(1)
            .expect("rect column")
            .split(']')
            .next()
            .expect("rect values");
        assert_eq!(values.split(',').count(), 100);
    }
}

#[test]
fn atlas_cli_writes_the_library_result() {
    let root = support::root();
    let temp = std::env::temp_dir().join(format!("typegpu-atlas-cli-{}", std::process::id()));
    std::fs::create_dir_all(temp.join("third_party/microui/demo")).expect("create atlas directory");
    std::fs::create_dir_all(temp.join("lib")).expect("create library directory");
    std::fs::copy(
        root.join("third_party/microui/demo/atlas.inl"),
        temp.join("third_party/microui/demo/atlas.inl"),
    )
    .expect("copy atlas data");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-gen"))
        .arg("ui-atlas")
        .arg(&temp)
        .output()
        .expect("run atlas CLI");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        support::read(&temp.join("lib/typegpu-ui-atlas.generated.ts")),
        subscript_typegpu_gen::generate_ui_atlas(&root).expect("generate atlas")
    );
    std::fs::write(temp.join("third_party/microui/demo/atlas.inl"), "")
        .expect("write invalid atlas");
    assert!(subscript_typegpu_gen::generate_ui_atlas(&temp).is_err());
    std::fs::remove_dir_all(temp).expect("remove atlas directory");
}
