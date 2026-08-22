use std::collections::BTreeMap;
use std::error::Error;

use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::TypeInner;
use subscript_typegpu_gen::layout::{Scalar, TypeTree};

use crate::support;

fn capabilities(source: &str) -> Capabilities {
    if source.contains("enable f16;") {
        Capabilities::SHADER_FLOAT16
    } else {
        Capabilities::empty()
    }
}

fn parse(program: &str, source: &str) -> naga::Module {
    naga::front::wgsl::parse_str(source).unwrap_or_else(|error| {
        panic!(
            "{program}: WGSL parse failed:\n{}",
            error.emit_to_string(source)
        )
    })
}

fn cause_chain(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str("\ncaused by: ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

fn validate(program: &str, source: &str, module: &naga::Module) {
    Validator::new(ValidationFlags::all(), capabilities(source))
        .validate(module)
        .unwrap_or_else(|error| {
            panic!(
                "{program}: WGSL validation failed:\n{}",
                cause_chain(&error)
            )
        });
}

fn library_types(tree: &TypeTree, names: &mut std::collections::BTreeSet<&'static str>) {
    match tree {
        TypeTree::Vector(vector) => {
            names.insert(match (vector.scalar, vector.lanes) {
                (Scalar::F32, 2) => "Vec2f",
                (Scalar::F32, 3) => "Vec3f",
                (Scalar::F32, 4) => "Vec4f",
                (Scalar::I32, 2) => "Vec2i",
                (Scalar::I32, 3) => "Vec3i",
                (Scalar::I32, 4) => "Vec4i",
                (Scalar::U32, 2) => "Vec2u",
                (Scalar::U32, 3) => "Vec3u",
                (Scalar::U32, 4) => "Vec4u",
                (Scalar::F16, 2) => "Vec2h",
                (Scalar::F16, 3) => "Vec3h",
                (Scalar::F16, 4) => "Vec4h",
                shape => panic!("unexpected vector shape {shape:?}"),
            });
        }
        TypeTree::Matrix(matrix) => {
            names.insert(match (matrix.columns, matrix.rows) {
                (2, 2) => "Mat2x2f",
                (3, 3) => "Mat3x3f",
                (4, 4) => "Mat4x4f",
                shape => panic!("unexpected matrix shape {shape:?}"),
            });
        }
        TypeTree::Array(element, _) => library_types(element, names),
        TypeTree::Struct(structure) => {
            for member in &structure.members {
                library_types(&member.ty, names);
            }
        }
        TypeTree::Scalar(_) => {}
    }
}

#[test]
fn naga_offsets_and_spans_match_the_engine() {
    let programs = support::b_programs();
    assert!(!programs.is_empty(), "no b programs found");
    for path in programs {
        let program = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("program name");
        let generated = subscript_typegpu_gen::generate(&support::program_files(&path))
            .unwrap_or_else(|diagnostics| panic!("generate {program}: {diagnostics:?}"));
        let module = parse(program, &generated.wgsl_module);
        validate(program, &generated.wgsl_module, &module);

        let expected = generated
            .layouts
            .iter()
            .map(|layout| (layout.name.as_str(), layout))
            .collect::<BTreeMap<_, _>>();
        let mut matched = 0;
        for (_, ty) in module.types.iter() {
            let Some(name) = ty.name.as_deref() else {
                continue;
            };
            let Some(expected) = expected.get(name) else {
                continue;
            };
            matched += 1;
            let TypeInner::Struct { members, span } = &ty.inner else {
                panic!("{program}: {name} is not a struct");
            };
            assert_eq!(*span, expected.wgsl.size, "{program}: {name} span");
            assert_eq!(
                members.len(),
                expected.wgsl.members.len(),
                "{program}: {name} member count"
            );
            let TypeTree::Struct(structure) = &expected.tree else {
                panic!("{program}: {name} has no struct tree");
            };
            for ((member, expected_member), tree_member) in members
                .iter()
                .zip(&expected.wgsl.members)
                .zip(&structure.members)
            {
                assert_eq!(
                    member.offset, expected_member.offset,
                    "{program}: {name}.{:?}",
                    member.name
                );
                if matches!(tree_member.ty, TypeTree::Array(_, _)) {
                    let TypeInner::Array { stride, .. } = module.types[member.ty].inner else {
                        panic!("{program}: {name}.{} is not an array", tree_member.name);
                    };
                    assert_eq!(
                        stride,
                        expected_member.layout.stride.expect("array stride"),
                        "{program}: {name}.{} stride",
                        tree_member.name
                    );
                }
            }
        }
        assert_eq!(
            matched,
            expected.len(),
            "{program}: not every emitted struct matched an engine layout"
        );

        if program == "b01-layout.ts" {
            assert_eq!(generated.wgsl_module.matches("enable f16;").count(), 1);
            assert!(generated.wgsl_module.starts_with("enable f16;\n"));
            let mut covered_library_types = std::collections::BTreeSet::new();
            for layout in &generated.layouts {
                library_types(&layout.tree, &mut covered_library_types);
            }
            assert_eq!(
                covered_library_types,
                [
                    "Mat2x2f", "Mat3x3f", "Mat4x4f", "Vec2f", "Vec2h", "Vec2i", "Vec2u", "Vec3f",
                    "Vec3h", "Vec3i", "Vec3u", "Vec4f", "Vec4h", "Vec4i", "Vec4u",
                ]
                .into_iter()
                .collect(),
                "{program}: every library class must appear in a schema"
            );
            let uniform_source = format!(
                "{}\n@group(0) @binding(0) var<uniform> params: Params;\n",
                generated.wgsl_module
            );
            let uniform = parse(program, &uniform_source);
            validate(program, &uniform_source, &uniform);
        }
    }
}
