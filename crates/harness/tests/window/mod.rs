use std::path::PathBuf;

use subscript_compiler::{hir, CheckOptions, Type};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn example_programs() -> Vec<PathBuf> {
    let directory = root().join("examples");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("main.ts"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    programs.sort();
    assert!(!programs.is_empty(), "example program list is empty");
    programs
}

fn checked_example(program: &PathBuf) -> hir::Module {
    let files = subscript_typegpu_harness::program_files(&program)
        .unwrap_or_else(|error| panic!("load {}: {error}", program.display()));
    subscript_compiler::check_program_with(&files, &CheckOptions::default())
        .unwrap_or_else(|diagnostics| panic!("check {}: {diagnostics:?}", program.display()))
}

#[test]
fn window_example_compiles_through_the_host_loader_without_a_device() {
    for program in example_programs() {
        let session = subscript_typegpu_harness::load_program(&program)
            .unwrap_or_else(|error| panic!("compile {}: {error}", program.display()));
        drop(session);
    }
}

fn type_name(module: &hir::Module, ty: &Type) -> String {
    match ty {
        Type::Void => "void".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::I32 => "i32".to_owned(),
        Type::F32 => "f32".to_owned(),
        Type::Class(id) => module.classes[id.0].name.clone(),
        Type::StringAlias(id) => module.string_aliases[id.0].name.clone(),
        other => format!("{other:?}"),
    }
}

#[test]
fn window_example_has_the_three_host_entry_signatures() {
    let expected = [
        (
            "init",
            vec![
                ("instance", "SubscriptTypegpuInstance"),
                ("device", "SubscriptTypegpuDevice"),
                ("format", "GPUTextureFormat"),
            ],
        ),
        (
            "frame",
            vec![
                ("view", "SubscriptTypegpuTextureView"),
                ("width", "u32"),
                ("height", "u32"),
                ("key", "u32"),
                ("pointerX", "f32"),
                ("pointerY", "f32"),
                ("buttons", "u32"),
            ],
        ),
        ("shutdown", Vec::new()),
    ];
    for program in example_programs() {
        let module = checked_example(&program);
        let entries = module
            .functions
            .iter()
            .filter(|function| function.exported && function.pos.file == "main.ts")
            .collect::<Vec<_>>();
        if entries.iter().any(|function| function.name == "main") {
            continue;
        }
        assert_eq!(
            entries.len(),
            expected.len(),
            "{} exported entry count",
            program.display(),
        );
        for (name, params) in &expected {
            let function = entries
                .iter()
                .find(|function| function.name == *name)
                .unwrap_or_else(|| panic!("{} missing exported {name}", program.display()));
            if *name != "init" {
                assert!(!function.is_async, "{name} must be synchronous");
            }
            assert_eq!(type_name(&module, &function.ret), "void", "{name} return");
            let actual = function
                .params
                .iter()
                .map(|parameter| (parameter.name.as_str(), type_name(&module, &parameter.ty)))
                .collect::<Vec<_>>();
            let wanted = params
                .iter()
                .map(|(parameter, ty)| (*parameter, (*ty).to_owned()))
                .collect::<Vec<_>>();
            assert_eq!(actual, wanted, "{} {name} signature", program.display());
        }
    }
}

#[test]
fn only_the_harness_manifest_depends_on_subscript_codegen() {
    let crates = root().join("crates");
    let mut direct = std::fs::read_dir(&crates)
        .unwrap_or_else(|error| panic!("read {}: {error}", crates.display()))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let manifest = entry.path().join("Cargo.toml");
            let source = std::fs::read_to_string(&manifest).ok()?;
            source
                .lines()
                .any(|line| line.trim_start().starts_with("subscript-codegen"))
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    direct.sort();
    assert_eq!(direct, ["harness"]);
}
