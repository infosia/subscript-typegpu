#[test]
fn ui_core_matches_on_both_tiers() {
    let _guard = crate::differential::suite_lock();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui/core.ts");
    let mut outputs = Vec::new();
    for tier in ["dev", "ship"] {
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_subscript-typegpu-harness"))
            .arg(tier)
            .arg(&path)
            .output()
            .expect("run UI core");
        assert!(
            result.status.success(),
            "{tier}: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            result.stdout,
            b"atlas PASS\nids PASS\ninput PASS\nlayout PASS\ncommands PASS\nfocus PASS\n"
        );
        outputs.push(result.stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
}
