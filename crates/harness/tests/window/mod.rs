use std::path::PathBuf;

use subscript_compiler::{hir, CheckOptions, SourceFile, Type};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn checked_example() -> hir::Module {
    let program = root().join("examples/window-triangle/main.ts");
    let mut files = subscript_typegpu_harness::program_files(&program)
        .unwrap_or_else(|error| panic!("load {}: {error}", program.display()));
    let generated = subscript_typegpu_gen::generate(&files)
        .unwrap_or_else(|diagnostics| panic!("generate {}: {diagnostics:?}", program.display()));
    files.push(SourceFile::new("main.typegpu.ts", generated.support_module));
    subscript_compiler::check_program_with(&files, &CheckOptions::default())
        .unwrap_or_else(|diagnostics| panic!("check {}: {diagnostics:?}", program.display()))
}

fn type_name(module: &hir::Module, ty: &Type) -> String {
    match ty {
        Type::Void => "void".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::I32 => "i32".to_owned(),
        Type::Class(id) => module.classes[id.0].name.clone(),
        Type::StringAlias(id) => module.string_aliases[id.0].name.clone(),
        other => format!("{other:?}"),
    }
}

#[test]
fn window_example_has_the_three_host_entry_signatures() {
    let module = checked_example();
    let expected = [
        (
            "init",
            vec![
                ("instance", "SubscriptTypegpuInstance"),
                ("device", "SubscriptTypegpuDevice"),
                ("format", "GPUTextureFormatWire"),
            ],
        ),
        (
            "frame",
            vec![
                ("view", "SubscriptTypegpuTextureView"),
                ("width", "u32"),
                ("height", "u32"),
                ("key", "u32"),
            ],
        ),
        ("shutdown", Vec::new()),
    ];
    let entries = module
        .functions
        .iter()
        .filter(|function| function.exported && function.pos.file == "main.ts")
        .collect::<Vec<_>>();
    assert_eq!(
        entries.len(),
        expected.len(),
        "unexpected exported entry count"
    );
    for (name, params) in expected {
        let function = entries
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("missing exported {name}"));
        assert!(!function.is_async, "{name} must be synchronous");
        assert_eq!(type_name(&module, &function.ret), "void", "{name} return");
        let actual = function
            .params
            .iter()
            .map(|parameter| (parameter.name.as_str(), type_name(&module, &parameter.ty)))
            .collect::<Vec<_>>();
        let expected = params
            .iter()
            .map(|(parameter, ty)| (*parameter, (*ty).to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{name} signature");
    }
}
