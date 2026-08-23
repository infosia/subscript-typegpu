//! EG9 documentation quote gate.

use std::path::Path;

fn check_program_quotes(root: &Path, document_path: &Path) {
    let document = std::fs::read_to_string(document_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", document_path.display()));
    let mut program: Option<String> = None;
    let mut source_lines = Vec::new();
    let mut quote_lines = Vec::new();
    let mut quote_count = 0usize;
    for (line_index, line) in document.lines().enumerate() {
        if let Some(relative) = line.strip_prefix("```ts program=") {
            assert!(
                program.is_none(),
                "{}:{} has a nested program fence",
                document_path.display(),
                line_index + 1
            );
            assert!(
                relative.starts_with("programs/") && relative.ends_with(".ts"),
                "{}:{} names an invalid program",
                document_path.display(),
                line_index + 1,
            );
            let path = root.join(relative);
            source_lines = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| {
                    panic!(
                        "{}:{} cannot read quoted program {}: {error}",
                        document_path.display(),
                        line_index + 1,
                        path.display(),
                    )
                })
                .lines()
                .map(str::to_owned)
                .collect();
            quote_lines.clear();
            program = Some(relative.to_owned());
            continue;
        }
        if line == "```" && program.is_some() {
            let relative = program.as_deref().expect("open program fence");
            assert!(
                !quote_lines.is_empty(),
                "{}:{} has an empty program fence for {relative}",
                document_path.display(),
                line_index + 1,
            );
            assert!(
                source_lines
                    .windows(quote_lines.len())
                    .any(|window| window == quote_lines),
                "{}:{} quotes no contiguous block from {relative}:\n{}",
                document_path.display(),
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
    assert!(
        program.is_none(),
        "{} has an unclosed program fence",
        document_path.display(),
    );
    assert!(
        quote_count != 0,
        "{} has no checked program lines",
        document_path.display(),
    );
}

#[test]
fn readme_and_documentation_program_quotes_match_the_named_programs() {
    let root = crate::repository_root();
    let docs_dir = root.join("docs");
    let mut documents = vec![root.join("README.md")];
    documents.extend(
        std::fs::read_dir(&docs_dir)
            .unwrap_or_else(|error| panic!("read {}: {error}", docs_dir.display()))
            .map(|entry| entry.expect("documentation directory entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "md")),
    );
    documents.sort();
    for document in documents {
        check_program_quotes(&root, &document);
    }
}
