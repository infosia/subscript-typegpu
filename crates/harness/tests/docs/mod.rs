//! EG9 documentation quote gate.

#[test]
fn tutorial_program_quotes_match_the_named_programs() {
    let root = crate::repository_root();
    let tutorial_path = root.join("docs/tutorial.md");
    let tutorial = std::fs::read_to_string(&tutorial_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", tutorial_path.display()));
    let mut program = None;
    let mut source = String::new();
    let mut quote_count = 0usize;
    for (line_index, line) in tutorial.lines().enumerate() {
        if let Some(relative) = line.strip_prefix("```ts program=") {
            assert!(
                program.is_none(),
                "nested tutorial fence at line {}",
                line_index + 1
            );
            assert!(
                relative.starts_with("programs/") && relative.ends_with(".ts"),
                "tutorial fence names an invalid program at line {}",
                line_index + 1,
            );
            let path = root.join(relative);
            source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            program = Some(relative);
            continue;
        }
        if line == "```" && program.is_some() {
            program = None;
            source.clear();
            continue;
        }
        let Some(relative) = program else { continue };
        if line.is_empty() {
            continue;
        }
        quote_count += 1;
        assert!(
            source.lines().any(|candidate| candidate == line),
            "{}:{} quotes a line absent from {relative}: `{line}`",
            tutorial_path.display(),
            line_index + 1,
        );
    }
    assert!(program.is_none(), "tutorial has an unclosed program fence");
    assert!(quote_count != 0, "tutorial has no checked program lines");
}
