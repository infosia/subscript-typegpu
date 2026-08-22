use std::collections::BTreeSet;

use subscript_compiler::SourceFile;

use crate::support;

fn names(values: &[subscript_compiler::hir::Function]) -> BTreeSet<&str> {
    values
        .iter()
        .map(|function| function.name.as_str())
        .collect()
}

#[test]
fn every_library_method_has_the_sc6_body() {
    let root = support::root();
    let files = [SourceFile::new(
        "typegpu-types.ts",
        support::read(&root.join("lib/typegpu-types.ts")),
    )];
    let module = subscript_compiler::check_program(&files).expect("check type library");
    assert_eq!(
        module
            .classes
            .iter()
            .filter(|class| class.pos.file == "typegpu-types.ts")
            .count(),
        17
    );
    for class in module
        .classes
        .iter()
        .filter(|class| class.pos.file == "typegpu-types.ts")
    {
        let expected = match class.name.as_str() {
            "Vec2f" | "Vec4f" => {
                ["add", "dot", "length", "mul", "normalize", "scale", "sub"].as_slice()
            }
            "Vec3f" => [
                "add",
                "cross",
                "dot",
                "length",
                "mul",
                "normalize",
                "scale",
                "sub",
            ]
            .as_slice(),
            "Vec2i" | "Vec3i" | "Vec4i" | "Vec2u" | "Vec3u" | "Vec4u" => {
                ["add", "dot", "mul", "scale", "sub"].as_slice()
            }
            "Vec2h" | "Vec3h" | "Vec4h" => [].as_slice(),
            "AtomicU32" | "AtomicI32" => {
                ["add", "exchange", "load", "max", "min", "store", "sub"].as_slice()
            }
            "Mat2x2f" | "Mat3x3f" | "Mat4x4f" => ["mul", "mulVec", "transpose"].as_slice(),
            name => panic!("unexpected library class {name}"),
        };
        assert_eq!(
            names(&class.methods),
            expected.iter().copied().collect(),
            "{} methods",
            class.name
        );
        assert!(
            class.methods.iter().all(|method| !method.body.is_empty()),
            "{} has an empty method body",
            class.name
        );
    }
    for factory in ["mat2x2fIdentity", "mat3x3fIdentity", "mat4x4fIdentity"] {
        let function = module
            .functions
            .iter()
            .find(|function| function.name == factory)
            .unwrap_or_else(|| panic!("missing {factory}"));
        assert!(!function.body.is_empty(), "{factory} has an empty body");
    }
}
