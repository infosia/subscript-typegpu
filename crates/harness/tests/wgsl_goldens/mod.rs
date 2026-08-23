//! Byte-exact WGSL goldens and naga validation for every generated pipeline.

use std::path::{Path, PathBuf};

use naga::valid::{Capabilities, ValidationFlags, Validator};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is below the repository root")
        .to_path_buf()
}

fn pipeline_programs() -> Vec<PathBuf> {
    let mut programs = std::fs::read_dir(root().join("programs"))
        .expect("read programs")
        .map(|entry| entry.expect("read program entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    (name.starts_with('b') || name.starts_with('x')) && name.ends_with(".ts")
                })
        })
        .collect::<Vec<_>>();
    programs.sort();
    programs
}

fn first_differing_line(actual: &str, expected: &str) -> usize {
    actual
        .lines()
        .zip(expected.lines())
        .position(|(actual, expected)| actual != expected)
        .map_or_else(
            || actual.lines().count().min(expected.lines().count()) + 1,
            |index| index + 1,
        )
}

fn generated(program: &Path) -> subscript_typegpu_gen::Generated {
    let mut files = subscript_typegpu_harness::program_files(program).expect("load program files");
    files.retain(|file| !file.name.ends_with(".typegpu.ts"));
    subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "{} generation failed:\n{}",
            program.display(),
            subscript_compiler::render_diagnostics(&files, &diagnostics)
        )
    })
}

fn attribution<'a>(
    generated: &'a subscript_typegpu_gen::Generated,
    pipeline: &str,
    line: u32,
) -> Option<&'a str> {
    generated
        .wgsl_spans
        .iter()
        .find(|span| span.pipeline == pipeline && line >= span.start_line && line <= span.end_line)
        .map(|span| span.label.as_str())
}

fn naga_message(
    program: &Path,
    pipeline: &str,
    generated: &subscript_typegpu_gen::Generated,
    line: Option<u32>,
    stage: &str,
    error: &dyn std::fmt::Display,
) -> String {
    let label = line.and_then(|line| attribution(generated, pipeline, line));
    match label {
        Some(label) => format!(
            "{label}: {} pipeline {pipeline}: naga {stage}: {error}",
            program.display()
        ),
        None => format!(
            "{} pipeline {pipeline}: K15 (generator): naga {stage}: {error}",
            program.display()
        ),
    }
}

#[test]
fn every_pipeline_matches_its_golden_and_validates() {
    let mut count = 0;
    let mut named_goldens = std::collections::BTreeSet::new();
    for program in pipeline_programs() {
        let stem = program
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("program stem");
        let generated = generated(&program);
        for (pipeline, actual) in &generated.pipelines {
            count += 1;
            let golden = root()
                .join("programs")
                .join(format!("{stem}.{pipeline}.wgsl"));
            named_goldens.insert(golden.clone());
            let expected = std::fs::read_to_string(&golden)
                .unwrap_or_else(|error| panic!("read {}: {error}", golden.display()));
            assert_eq!(
                actual,
                &expected,
                "{} pipeline {pipeline} first differs at line {}. Run tools/regen.sh",
                program.display(),
                first_differing_line(actual, &expected),
            );
            let module = naga::front::wgsl::parse_str(actual).unwrap_or_else(|error| {
                let line = error.location(actual).map(|location| location.line_number);
                panic!(
                    "{}",
                    naga_message(&program, pipeline, &generated, line, "parse", &error)
                )
            });
            let capabilities = if actual.starts_with("enable f16;") {
                Capabilities::SHADER_FLOAT16
            } else {
                Capabilities::empty()
            };
            Validator::new(ValidationFlags::all(), capabilities)
                .validate(&module)
                .unwrap_or_else(|error| {
                    let line = error.location(actual).map(|location| location.line_number);
                    panic!(
                        "{}",
                        naga_message(&program, pipeline, &generated, line, "validate", &error)
                    )
                });
        }
    }
    assert!(count > 0, "pipeline golden count");
    for entry in std::fs::read_dir(root().join("programs")).expect("read programs") {
        let path = entry.expect("read program entry").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "wgsl")
        {
            assert!(
                named_goldens.contains(&path),
                "orphan WGSL golden: {}",
                path.display()
            );
        }
    }
}

#[test]
fn naga_errors_inside_shells_start_with_the_shell_name() {
    let program = root().join("crates/harness/tests/fixtures/wgsl/shell-naga-error.ts");
    let generated = generated(&program);
    let (pipeline, source) = generated.pipelines.first().expect("fixture pipeline");
    let message = match naga::front::wgsl::parse_str(source) {
        Err(error) => {
            let line = error.location(source).map(|location| location.line_number);
            naga_message(&program, pipeline, &generated, line, "parse", &error)
        }
        Ok(module) => {
            let error = Validator::new(ValidationFlags::all(), Capabilities::empty())
                .validate(&module)
                .expect_err("shell fixture must fail naga validation");
            let line = error.location(source).map(|location| location.line_number);
            naga_message(&program, pipeline, &generated, line, "validate", &error)
        }
    };
    assert!(
        message.starts_with("shell badShell:"),
        "unexpected shell attribution: {message}"
    );
}
