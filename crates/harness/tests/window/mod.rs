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

fn check_host_signatures(module: &hir::Module) {
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
        ("wheel", vec![("deltaX", "f32"), ("deltaY", "f32")]),
        ("keyDown", vec![("key", "u32")]),
        ("keyUp", vec![("key", "u32")]),
        ("textInput", vec![("codePoint", "u32")]),
    ];
    let entries = module
        .functions
        .iter()
        .filter(|function| function.exported && function.pos.file == "main.ts")
        .collect::<Vec<_>>();
    if entries.len() == 1 && entries[0].name == "main" {
        return;
    }
    for function in &entries {
        assert!(
            expected.iter().any(|(name, _)| *name == function.name),
            "unexpected export {}",
            function.name
        );
    }
    for (index, (name, params)) in expected.iter().enumerate() {
        let function = entries.iter().find(|function| function.name == *name);
        if index >= 3 && function.is_none() {
            continue;
        }
        let function = function.unwrap_or_else(|| panic!("missing exported {name}"));
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
        assert_eq!(actual, wanted, "{name} signature");
    }
}

#[test]
fn window_example_has_exact_host_entry_signatures() {
    for program in example_programs() {
        let module = checked_example(&program);
        check_host_signatures(&module);
        if program == root().join("examples/ui-demo/main.ts") {
            let (_session, exports) =
                subscript_typegpu_harness::load_program_with_exports(&program)
                    .unwrap_or_else(|error| panic!("compile {}: {error}", program.display()));
            let expected = module
                .functions
                .iter()
                .filter(|function| function.exported && function.pos.file == "main.ts")
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            assert_eq!(exports, expected, "{} checked exports", program.display());
        }
    }
}

#[test]
fn host_signature_rejections_name_the_invalid_entry() {
    let program = root().join("examples/window-triangle/main.ts");
    let original = checked_example(&program);
    let module = &original;
    let template = module
        .functions
        .iter()
        .find(|function| function.exported && function.name == "shutdown")
        .unwrap()
        .clone();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let results = [
        ("shutdown", 0, false),
        ("frame", 0, false),
        ("unexpectedInput", 0, false),
        ("wheel", 0, false),
        ("keyDown", 0, false),
        ("keyUp", 0, false),
        ("textInput", 0, false),
        ("wheel", 2, true),
    ]
    .into_iter()
    .map(|(name, params, is_async)| {
        let mut module = original.clone();
        let mut invalid = template.clone();
        invalid.name = name.to_owned();
        invalid.is_async = is_async;
        if params != 0 {
            let frame = module
                .functions
                .iter()
                .find(|function| function.name == "frame")
                .unwrap();
            invalid.params = frame.params[4..6].to_vec();
            invalid.params[0].name = "deltaX".to_owned();
            invalid.params[1].name = "deltaY".to_owned();
        }
        if name == "shutdown" || name == "frame" {
            module.functions.retain(|function| function.name != name);
        }
        if name != "shutdown" {
            module.functions.push(invalid);
        }
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            check_host_signatures(&module)
        }));
        (name, failure)
    })
    .collect::<Vec<_>>();
    std::panic::set_hook(hook);
    for (name, failure) in results {
        let failure = failure.expect_err("invalid host export must fail");
        let message = failure
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| failure.downcast_ref::<&str>().copied())
            .unwrap_or("");
        assert!(
            message.contains(name),
            "diagnostic must name {name}: {message}"
        );
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
