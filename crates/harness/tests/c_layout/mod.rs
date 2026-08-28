//! End-to-end C layout checks for every schema program.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use subscript_codegen::{
    add_c11_optimized_flags, add_executable_output, add_object_directory, host_c_compiler,
    include_directory_arg, tool_output_report, value_class_layouts,
};
use subscript_compiler::{hir, SourceFile, Type};
use subscript_typegpu_gen::Generated;

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
    assert!(!programs.is_empty(), "C layout program set is empty");
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

/// Projects managed mirror fields to their C aggregate forms.
///
/// The projection is the mirror's own contract, not a measurement: a
/// `T[]` field stands for the adjacent pair `size_t {name}Count` then
/// `const T* {name}` (subscript `compiler.md` §30.2), a string field
/// stands for `SubscriptTypegpuStringView`, and a function field
/// stands for one pointer. The probe reads the header's member offsets
/// under those names, so a header that breaks the pair form fails here.
/// The pointer widths assume a 64-bit host.
fn facade_mirror_layouts(module: &mut hir::Module) -> BTreeMap<String, ProbeLayout> {
    let mut boundary_fields = BTreeMap::new();
    for class in module.classes.iter().filter(|class| class.is_boundary) {
        boundary_fields.insert(
            class.name.clone(),
            class
                .fields
                .iter()
                .map(|field| (field.name.clone(), matches!(field.ty, Type::Array(_))))
                .collect::<Vec<_>>(),
        );
    }
    assert!(
        !boundary_fields.is_empty(),
        "facade mirror class set is empty"
    );

    for class in module.classes.iter_mut().filter(|class| class.is_boundary) {
        for field in &mut class.fields {
            field.ty = match &field.ty {
                Type::Str | Type::Array(_) => Type::FixedArray(Box::new(Type::U64), 2),
                Type::Func(_) => Type::U64,
                other => other.clone(),
            };
        }
    }

    let layouts = value_class_layouts(module)
        .expect("compute facade mirror C layouts")
        .into_iter()
        .map(|layout| (layout.name.clone(), layout))
        .collect::<BTreeMap<_, _>>();
    boundary_fields
        .into_iter()
        .map(|(name, fields)| {
            let layout = layouts
                .get(&name)
                .unwrap_or_else(|| panic!("subscript layout lacks facade mirror `{name}`"));
            assert_eq!(
                layout.fields.len(),
                fields.len(),
                "{name}: facade mirror field count"
            );
            let fields = layout
                .fields
                .iter()
                .zip(fields)
                .flat_map(|(layout, (name, is_array))| {
                    assert_eq!(layout.name, name, "facade mirror field name");
                    let mut offsets = vec![(name.clone(), layout.offset)];
                    if is_array {
                        offsets[0].1 = layout
                            .offset
                            .checked_add(8)
                            .expect("array pointer offset is within u32");
                        offsets.push((format!("{name}Count"), layout.offset));
                    }
                    offsets
                })
                .collect();
            (
                name,
                ProbeLayout {
                    size: layout.size,
                    align: layout.align,
                    fields,
                },
            )
        })
        .collect()
}

fn compile_probe(
    program: &Path,
    expected: &BTreeMap<String, ProbeLayout>,
) -> BTreeMap<String, ProbeLayout> {
    let mut source = String::from(
        "#include \"subscript-typegpu.h\"\n#include <stddef.h>\n#include <stdio.h>\nint main(void) {\n",
    );
    for (name, layout) in expected {
        let mut format = format!("{name}|%zu|%zu");
        let mut arguments = format!("sizeof({name}), _Alignof({name})");
        for member in layout.fields.keys() {
            format.push_str(&format!("|{member}=%zu"));
            arguments.push_str(&format!(", offsetof({name}, {member})"));
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
    add_object_directory(&mut command, &directory.0, compiler.style());
    command
        .arg(include_directory_arg(
            compiler.style(),
            &repository_root().join("crates/facade"),
        ))
        .arg(&source_path);
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

/// Proves both layout agreements from one check of each program: the
/// generator's schema layouts against the engine, and the facade
/// header's mirror structs against the engine. The first comparison
/// runs before the second, because the second rewrites the module's
/// mirror field types.
#[test]
fn schema_and_header_layouts_match_the_engine_for_every_b_program() {
    subscript_typegpu_harness::run_program_pool(programs(), |program| {
        let (generated, mut module) = checked(program);
        assert_language_layouts(program, &generated, &module);
        let expected = facade_mirror_layouts(&mut module);
        let actual = compile_probe(program, &expected);
        for (name, expected) in &expected {
            let layout = actual.get(name).unwrap_or_else(|| {
                panic!("{}: C probe lacks schema `{}`", program.display(), name)
            });
            assert_eq!(
                layout.size,
                expected.size,
                "{}: {} C size",
                program.display(),
                name
            );
            assert_eq!(
                layout.align,
                expected.align,
                "{}: {} C align",
                program.display(),
                name
            );
            assert_eq!(
                layout.fields,
                expected.fields,
                "{}: {} C field offsets",
                program.display(),
                name
            );
        }
    });
}
