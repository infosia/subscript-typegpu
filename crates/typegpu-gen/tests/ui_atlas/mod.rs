use crate::support;

fn source_commit(module: &str) -> &str {
    module
        .lines()
        .find_map(|line| line.strip_prefix("// Source commit: "))
        .expect("source commit header")
        .trim_end_matches('.')
}

#[test]
fn atlas_regeneration_is_byte_identical() {
    let root = support::root();
    let generated = subscript_typegpu_gen::generate_ui_atlas(&root).expect("generate atlas");
    let committed = support::read(&root.join("lib/typegpu-ui-atlas.generated.ts"));
    assert!(
        generated == committed,
        "atlas differs: committed header {}, submodule {}",
        source_commit(&committed),
        source_commit(&generated),
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
    let committed = support::read(&root.join("lib/typegpu-ui-atlas.generated.ts"));
    let source = support::read(&root.join("third_party/microui/demo/atlas.inl"));
    std::fs::write(temp.join("third_party/microui/demo/atlas.inl"), &source)
        .expect("write atlas fixture");
    let gitdir = temp.join("third_party/microui/git-data");
    std::fs::create_dir_all(&gitdir).expect("create git directory");
    std::fs::write(temp.join("third_party/microui/.git"), "gitdir: git-data\n")
        .expect("write gitfile");
    let commit = support::read(&root.join("lib/typegpu-ui-atlas.generated.ts"))
        .lines()
        .nth(1)
        .expect("source header")
        .strip_prefix("// Source commit: ")
        .expect("commit prefix")
        .trim_end_matches('.')
        .to_owned();
    std::fs::write(gitdir.join("HEAD"), &commit).expect("write detached HEAD");
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
        committed
    );
    std::fs::create_dir_all(gitdir.join("refs/heads")).expect("create refs");
    std::fs::write(gitdir.join("refs/heads/main"), &commit).expect("write ref");
    std::fs::write(gitdir.join("HEAD"), "ref: refs/heads/main\n").expect("write symbolic HEAD");
    assert_eq!(
        subscript_typegpu_gen::generate_ui_atlas(&temp).expect("resolve ref"),
        committed
    );
    std::fs::remove_file(gitdir.join("refs/heads/main")).expect("remove ref");
    assert!(subscript_typegpu_gen::generate_ui_atlas(&temp).is_err());
    std::fs::remove_file(gitdir.join("HEAD")).expect("remove HEAD");
    assert!(subscript_typegpu_gen::generate_ui_atlas(&temp).is_err());
    std::fs::write(gitdir.join("HEAD"), &commit).expect("restore HEAD");
    std::fs::remove_file(temp.join("third_party/microui/.git")).expect("remove gitfile");
    let directory = temp.join("third_party/microui/.git");
    std::fs::rename(&gitdir, &directory).expect("move git directory");
    assert_eq!(
        subscript_typegpu_gen::generate_ui_atlas(&temp).expect("resolve directory HEAD"),
        committed
    );
    let texture_start = source.find("atlas_texture[").expect("texture initializer");
    let body_start = texture_start + source[texture_start..].find('{').expect("texture body") + 1;
    let body_end = body_start + source[body_start..].find("};").expect("texture end");
    for count in [0, 16383, 16384, 16385] {
        let fixture = format!(
            "{}{}{}",
            &source[..body_start],
            "0xff,".repeat(count),
            &source[body_end..]
        );
        std::fs::write(temp.join("third_party/microui/demo/atlas.inl"), fixture)
            .expect("write atlas size fixture");
        let result = subscript_typegpu_gen::generate_ui_atlas(&temp);
        if count > 16384 {
            assert_eq!(
                result.unwrap_err(),
                format!("atlas has {count} bytes, expected 16384")
            );
        } else {
            let generated = result.expect("generate zero-filled atlas");
            let hex = generated
                .split_once("UI_ATLAS_ALPHA_HEX: string = \"")
                .expect("alpha constant")
                .1
                .split_once('"')
                .expect("alpha data")
                .0;
            let expected = format!("{}{}", "ff".repeat(count), "00".repeat(16384 - count));
            assert_eq!(hex, expected);
        }
    }
    std::fs::write(temp.join("third_party/microui/demo/atlas.inl"), "")
        .expect("write invalid atlas");
    assert!(subscript_typegpu_gen::generate_ui_atlas(&temp).is_err());
    std::fs::remove_dir_all(temp).expect("remove atlas directory");
}
