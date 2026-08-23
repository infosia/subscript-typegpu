use subscript_compiler::SourceFile;

use crate::support;

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
        23
    );
    for class in module
        .classes
        .iter()
        .filter(|class| class.pos.file == "typegpu-types.ts")
    {
        assert!(
            class.methods.iter().all(|method| !method.body.is_empty()),
            "{} has an empty method body",
            class.name
        );
    }
    for factory in [
        "v3fFrom2",
        "v4fFrom2",
        "v4fFrom3",
        "v2fSplat",
        "v3fSplat",
        "v4fSplat",
        "v3iFrom2",
        "v4iFrom2",
        "v4iFrom3",
        "v2iSplat",
        "v3iSplat",
        "v4iSplat",
        "v3uFrom2",
        "v4uFrom2",
        "v4uFrom3",
        "v2uSplat",
        "v3uSplat",
        "v4uSplat",
        "mat2x2fIdentity",
        "mat3x3fIdentity",
        "mat4x4fIdentity",
    ] {
        let function = module
            .functions
            .iter()
            .find(|function| function.name == factory)
            .unwrap_or_else(|| panic!("missing {factory}"));
        assert!(!function.body.is_empty(), "{factory} has an empty body");
    }
}
