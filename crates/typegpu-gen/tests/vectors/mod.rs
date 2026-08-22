use std::collections::BTreeMap;

use serde_json::Value;
use subscript_typegpu_gen::layout::{
    wgsl_layout, Matrix, Member, Scalar, Struct, TypeTree, Vector,
};

use crate::support;

fn vector(name: &str) -> TypeTree {
    let (scalar, lanes, align) = match name {
        "vec2f" => (Scalar::F32, 2, 8),
        "vec3f" => (Scalar::F32, 3, 16),
        "vec4f" => (Scalar::F32, 4, 16),
        "vec2i" => (Scalar::I32, 2, 8),
        "vec3i" => (Scalar::I32, 3, 16),
        "vec4i" => (Scalar::I32, 4, 16),
        "vec2u" => (Scalar::U32, 2, 8),
        "vec3u" => (Scalar::U32, 3, 16),
        "vec4u" => (Scalar::U32, 4, 16),
        "vec2h" => (Scalar::F16, 2, 4),
        "vec3h" => (Scalar::F16, 3, 8),
        "vec4h" => (Scalar::F16, 4, 8),
        _ => panic!("unknown vector {name}"),
    };
    TypeTree::Vector(Vector {
        scalar,
        lanes,
        c_alignment: Some(align),
    })
}

fn scalar(name: &str) -> TypeTree {
    TypeTree::Scalar(match name {
        "f32" => Scalar::F32,
        "i32" => Scalar::I32,
        "u32" => Scalar::U32,
        "f16" => Scalar::F16,
        _ => panic!("unknown scalar {name}"),
    })
}

fn matrix(name: &str) -> TypeTree {
    let (columns, rows, align) = match name {
        "mat2x2f" => (2, 2, 8),
        "mat3x3f" => (3, 3, 16),
        "mat4x4f" => (4, 4, 16),
        _ => panic!("unknown matrix {name}"),
    };
    TypeTree::Matrix(Matrix {
        columns,
        rows,
        c_alignment: Some(align),
        column_alignment: Some(align),
    })
}

fn shape(name: &str) -> TypeTree {
    let one = TypeTree::Struct(Struct {
        name: "One".into(),
        members: vec![
            Member {
                name: "a".into(),
                ty: scalar("u32"),
            },
            Member {
                name: "b".into(),
                ty: vector("vec3f"),
            },
        ],
        c_alignment: None,
    });
    let array = TypeTree::Array(Box::new(one.clone()), 3);
    match name {
        "One" => one,
        "OneArray3" => array,
        "Two" => TypeTree::Struct(Struct {
            name: "Two".into(),
            members: vec![
                Member {
                    name: "c".into(),
                    ty: array,
                },
                Member {
                    name: "d".into(),
                    ty: vector("vec4u"),
                },
            ],
            c_alignment: None,
        }),
        _ => panic!("unknown shape {name}"),
    }
}

fn number(value: &Value, key: &str) -> u32 {
    value[key]
        .as_u64()
        .unwrap_or_else(|| panic!("{key} is not a number in {value}")) as u32
}

fn compare(entries: &[Value], tree: impl Fn(&str) -> TypeTree) {
    for entry in entries {
        let name = entry["name"].as_str().expect("vector name");
        let layout = wgsl_layout(&tree(name));
        assert_eq!(layout.align, number(entry, "align"), "{name} align");
        assert_eq!(layout.size, number(entry, "size"), "{name} size");
        if let Some(stride) = entry.get("stride") {
            assert_eq!(layout.stride, Some(stride.as_u64().expect("stride") as u32));
        }
        if let Some(offsets) = entry.get("offsets").and_then(Value::as_object) {
            let actual = layout
                .members
                .iter()
                .map(|member| (member.name.as_str(), member.offset))
                .collect::<BTreeMap<_, _>>();
            for (member, expected) in offsets {
                if member.starts_with("element") {
                    let index = member
                        .trim_start_matches("element")
                        .parse::<u32>()
                        .expect("array element index");
                    assert_eq!(
                        layout.stride.map(|stride| stride * index),
                        Some(expected.as_u64().expect("offset") as u32),
                        "{name}.{member}"
                    );
                    continue;
                }
                assert_eq!(
                    actual.get(member.as_str()),
                    Some(&(expected.as_u64().expect("offset") as u32)),
                    "{name}.{member}"
                );
            }
        }
    }
}

#[test]
fn committed_typegpu_vectors_match_the_layout_engine() {
    let path = support::root().join("specs/layout-vectors.json");
    if !path.is_file() {
        println!(
            "pending: TypeGPU layout vectors — run tools/gen-layout-vectors.mjs with SUBSCRIPT_TYPEGPU_UPSTREAM_DIR"
        );
        return;
    }
    let document: Value = serde_json::from_str(&support::read(&path)).expect("parse vectors");
    assert!(
        document["typegpuVersion"]
            .as_str()
            .is_some_and(|version| !version.is_empty()),
        "missing TypeGPU version"
    );
    let scalars = document["scalars"].as_array().expect("scalars");
    let vectors = document["vectors"].as_array().expect("vectors");
    let matrices = document["matrices"].as_array().expect("matrices");
    let shapes = document["shapes"].as_array().expect("shapes");
    assert_eq!(scalars.len(), 4, "scalar vector count");
    assert_eq!(vectors.len(), 12, "vector vector count");
    assert_eq!(matrices.len(), 3, "matrix vector count");
    assert_eq!(shapes.len(), 3, "shape vector count");
    compare(scalars, scalar);
    compare(vectors, vector);
    compare(matrices, matrix);
    compare(shapes, shape);
    let array = wgsl_layout(&shape("OneArray3"));
    assert_eq!(array.stride, Some(32));
}
