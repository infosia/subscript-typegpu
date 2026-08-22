//! End-to-end C layout checks for every schema program.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use subscript_codegen::{
    add_c11_optimized_flags, add_executable_output, emit_c_without_main, host_c_compiler,
    runtime_system_libraries, tool_output_report, value_class_layouts,
};
use subscript_compiler::{hir, SourceFile};
use subscript_typegpu_gen::{Generated, GeneratedLayout};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("harness crate is under the repository root")
        .to_path_buf()
}

fn is_b_program(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let bytes = name.as_bytes();
    bytes.len() >= 8
        && bytes[0] == b'b'
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
        && name.ends_with(".ts")
        && !name.ends_with(".typegpu.ts")
}

fn programs() -> Vec<PathBuf> {
    let directory = repository_root().join("programs");
    let mut programs = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("read program entry").path())
        .filter(|path| is_b_program(path))
        .collect::<Vec<_>>();
    programs.sort();
    let names = programs
        .iter()
        .map(|path| path.file_name().expect("program name").to_string_lossy())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "b01-layout.ts",
            "b02-vecadd.ts",
            "b03-saxpy-uniform.ts",
            "b04-particles.ts",
            "b05-buffer.ts",
            "b06-render.ts",
            "b07-draw-variants.ts",
            "b08-render-bindings.ts",
            "b09-kernel-depth.ts",
            "b10-workgroup.ts",
        ],
        "C layout program set changed"
    );
    programs
}

fn checked(program: &Path) -> (Generated, hir::Module) {
    let program_name = program
        .file_name()
        .and_then(|name| name.to_str())
        .expect("UTF-8 program name")
        .to_owned();
    let support_name = format!(
        "{}.typegpu.ts",
        program
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("UTF-8 program stem")
    );
    let mut files = subscript_typegpu_harness::program_files(program).expect("load program files");
    files.retain(|file| {
        matches!(
            file.name.as_str(),
            "subscript-typegpu.generated.d.ts"
                | "wire-enum-aliases.generated.d.ts"
                | "webgpu.ts"
                | "typegpu-types.ts"
                | "typegpu.ts"
        ) || file.name == program_name
            || file.name == support_name
    });
    let inputs = files
        .iter()
        .filter(|file| !file.name.ends_with(".typegpu.ts"))
        .cloned()
        .collect::<Vec<SourceFile>>();
    let generated = subscript_typegpu_gen::generate(&inputs).expect("generate schema support");
    let module = subscript_compiler::check_program(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "{} failed the complete check:\n{}",
            program.display(),
            subscript_compiler::render_diagnostics(&files, &diagnostics)
        )
    });
    (generated, module)
}

fn assert_language_layouts(program: &Path, generated: &Generated, module: &hir::Module) {
    let language = value_class_layouts(module)
        .expect("compute subscript C layouts")
        .into_iter()
        .map(|layout| (layout.name.clone(), layout))
        .collect::<BTreeMap<_, _>>();
    for expected in &generated.layouts {
        let actual = language.get(&expected.name).unwrap_or_else(|| {
            panic!(
                "{}: subscript layout lacks schema `{}`",
                program.display(),
                expected.name
            )
        });
        assert_eq!(
            actual.size,
            expected.c.size,
            "{}: {} size",
            program.display(),
            expected.name
        );
        assert_eq!(
            actual.align,
            expected.c.align,
            "{}: {} align",
            program.display(),
            expected.name
        );
        assert_eq!(
            actual.fields.len(),
            expected.c.members.len(),
            "{}: {} field count",
            program.display(),
            expected.name
        );
        for (actual, expected) in actual.fields.iter().zip(&expected.c.members) {
            assert_eq!(
                actual.name,
                expected.name,
                "{}: {} field name",
                program.display(),
                expected.name
            );
            assert_eq!(
                actual.offset,
                expected.offset,
                "{}: {}.{} offset",
                program.display(),
                expected.name,
                actual.name
            );
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProbeLayout {
    size: u32,
    align: u32,
    fields: BTreeMap<String, u32>,
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let number = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "subscript-typegpu-c-layout-{}-{number}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create C layout directory");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn c_type(module: &hir::Module, expected: &GeneratedLayout, source: &str) -> String {
    let index = module
        .classes
        .iter()
        .position(|class| class.name == expected.name && class.pos.file != "typegpu-types.ts")
        .unwrap_or_else(|| panic!("emitted C cannot name schema `{}`", expected.name));
    let name = format!("Sub_{index}_{}", expected.name);
    assert!(
        source.contains(&format!("typedef struct {name}")),
        "emitted C does not contain schema `{}`",
        expected.name
    );
    name
}

fn compile_probe(
    program: &Path,
    generated: &Generated,
    module: &hir::Module,
) -> BTreeMap<String, ProbeLayout> {
    let mut source = emit_c_without_main(module)
        .unwrap_or_else(|error| panic!("{}: emit C: {error}", program.display()))
        .source;
    source.push_str("\n#include <stddef.h>\n#include <stdio.h>\nint main(void) {\n");
    for layout in &generated.layouts {
        let c_type = c_type(module, layout, &source);
        let mut format = format!("{}|%zu|%zu", layout.name);
        let mut arguments = format!("sizeof({c_type}), _Alignof({c_type})");
        for member in &layout.c.members {
            format.push_str(&format!("|{}=%zu", member.name));
            arguments.push_str(&format!(", offsetof({c_type}, {})", member.name));
        }
        source.push_str(&format!("  printf(\"{format}\\n\", {arguments});\n"));
    }
    source.push_str("  return 0;\n}\n");

    let directory = TempDir::new();
    let source_path = directory.0.join("probe.c");
    let executable = directory
        .0
        .join(format!("probe{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(&source_path, source).expect("write C layout probe");
    let compiler = host_c_compiler().expect("resolve host C compiler");
    let mut command = compiler.command();
    add_c11_optimized_flags(&mut command, compiler.style());
    command
        .arg("-I")
        .arg(repository_root().join("crates/facade"))
        .arg(&source_path)
        .args(subscript_typegpu_harness::ship_link_inputs().expect("facade link inputs"))
        .arg(subscript_typegpu_harness::ensure_runtime_staticlib().expect("runtime archive"))
        .args(runtime_system_libraries(compiler.style()));
    add_executable_output(&mut command, &executable, compiler.style());
    let compiled = command.output().expect("run host C compiler");
    assert!(
        compiled.status.success(),
        "{}: C layout probe compile failed:\n{}",
        program.display(),
        tool_output_report(&compiled)
    );
    let output = Command::new(&executable)
        .output()
        .expect("run C layout probe");
    assert!(
        output.status.success(),
        "{}: C layout probe failed:\n{}",
        program.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("C layout output is UTF-8");
    stdout
        .lines()
        .map(|line| {
            let mut parts = line.split('|');
            let name = parts.next().expect("probe schema name").to_owned();
            let size = parts
                .next()
                .expect("probe size")
                .parse()
                .expect("numeric size");
            let align = parts
                .next()
                .expect("probe align")
                .parse()
                .expect("numeric align");
            let fields = parts
                .map(|field| {
                    let (name, offset) = field.split_once('=').expect("field offset pair");
                    (name.to_owned(), offset.parse().expect("numeric offset"))
                })
                .collect();
            (
                name,
                ProbeLayout {
                    size,
                    align,
                    fields,
                },
            )
        })
        .collect()
}

#[test]
fn engine_c_layouts_match_subscript_codegen_for_every_b_program() {
    for program in programs() {
        let (generated, module) = checked(&program);
        assert_language_layouts(&program, &generated, &module);
    }
}

#[test]
fn emitted_c_layouts_match_the_engine_for_every_b_program() {
    for program in programs() {
        let (generated, module) = checked(&program);
        let actual = compile_probe(&program, &generated, &module);
        for expected in &generated.layouts {
            let layout = actual.get(&expected.name).unwrap_or_else(|| {
                panic!(
                    "{}: C probe lacks schema `{}`",
                    program.display(),
                    expected.name
                )
            });
            assert_eq!(
                layout.size,
                expected.c.size,
                "{}: {} C size",
                program.display(),
                expected.name
            );
            assert_eq!(
                layout.align,
                expected.c.align,
                "{}: {} C align",
                program.display(),
                expected.name
            );
            let offsets = expected
                .c
                .members
                .iter()
                .map(|member| (member.name.clone(), member.offset))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(
                layout.fields,
                offsets,
                "{}: {} C field offsets",
                program.display(),
                expected.name
            );
        }
        assert_eq!(
            actual.len(),
            generated.layouts.len(),
            "{}: C probe schema count",
            program.display()
        );
    }
}
