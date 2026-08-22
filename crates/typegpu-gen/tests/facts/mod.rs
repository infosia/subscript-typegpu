use subscript_compiler::SourceFile;

#[test]
fn nested_offsets_and_array_strides_have_unambiguous_names() {
    let source = r#"
import {
  ArrayRoot_OFFSET_items,
  ArrayRoot_STRIDE_items,
  Outer_OFFSET_inner_value,
} from "./facts.typegpu";

@CStruct
class Inner {
  value: u32;
}

@CStruct
class Outer {
  inner: Inner;
}

@CStruct
class ArrayRoot {
  items: FixedArray<Inner, 2>;
}
"#;
    let generated = subscript_typegpu_gen::generate(&[SourceFile::new("facts.ts", source)])
        .expect("generate nested facts");
    assert!(generated
        .support_module
        .contains("Outer_OFFSET_inner_value"));
    assert!(generated.support_module.contains("ArrayRoot_STRIDE_items"));
    assert!(!generated
        .support_module
        .contains("ArrayRoot_OFFSET_items_value"));
}

#[test]
fn an_imported_reference_class_is_not_a_schema() {
    let source = r#"
import { Host_SIZE } from "./not-schema.typegpu";

class Host {
  value: u32;
}
"#;
    let diagnostics = subscript_typegpu_gen::generate(&[SourceFile::new("not-schema.ts", source)])
        .expect_err("reference class must not generate schema facts");
    let message = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(message.contains("SC1"), "{message}");
    assert!(message.contains("not a schema"), "{message}");
    assert!(message.contains("(author)"), "{message}");
}

#[test]
fn imported_names_without_generated_facts_report_schema_rules() {
    let cases = [
        (
            r#"
import { Missing_SIZE } from "./missing.typegpu";
"#,
            "missing.ts",
            "SC1",
        ),
        (
            r#"
import { Bad_SIZE } from "./illegal.typegpu";

@CStruct
class Bad {
  value: string;
}
"#,
            "illegal.ts",
            "SC3",
        ),
    ];
    for (source, file, rule) in cases {
        let diagnostics = subscript_typegpu_gen::generate(&[SourceFile::new(file, source)])
            .expect_err("an imported name without a generated fact must fail");
        let messages = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            messages.contains(rule),
            "{file} expected {rule}:\n{messages}"
        );
        if rule == "SC1" {
            assert!(messages.contains("not a schema"), "{file}:\n{messages}");
        }
    }
}
