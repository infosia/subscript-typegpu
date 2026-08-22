//! EG9 documentation quote gate.

#[test]
fn tutorial_program_quotes_match_the_named_programs() {
    let root = crate::repository_root();
    let tutorial_path = root.join("docs/tutorial.md");
    let tutorial = std::fs::read_to_string(&tutorial_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", tutorial_path.display()));
    let mut program: Option<String> = None;
    let mut source_lines = Vec::new();
    let mut quote_lines = Vec::new();
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
            source_lines = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
                .lines()
                .map(str::to_owned)
                .collect();
            quote_lines.clear();
            program = Some(relative.to_owned());
            continue;
        }
        if line == "```" && program.is_some() {
            let relative = program.as_deref().expect("open tutorial fence");
            assert!(
                !quote_lines.is_empty(),
                "empty tutorial fence for {relative}"
            );
            assert!(
                source_lines
                    .windows(quote_lines.len())
                    .any(|window| window == quote_lines),
                "{}:{} quotes no contiguous block from {relative}:\n{}",
                tutorial_path.display(),
                line_index + 1,
                quote_lines.join("\n"),
            );
            quote_count += quote_lines.len();
            program = None;
            source_lines.clear();
            quote_lines.clear();
            continue;
        }
        if program.is_some() {
            quote_lines.push(line.to_owned());
        }
    }
    assert!(program.is_none(), "tutorial has an unclosed program fence");
    assert!(quote_count != 0, "tutorial has no checked program lines");
}
