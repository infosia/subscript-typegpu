use subscript_compiler::SourceFile;

#[test]
fn missing_vec3_alignment_reports_both_offsets() {
    let library = r#"
@CStruct
export class Vec3f {
  x: f32;
  y: f32;
  z: f32;

  constructor(x: f32, y: f32, z: f32) {
    this.x = x;
    this.y = y;
    this.z = z;
  }
}
"#;
    let program = r#"
import { Vec3f } from "./typegpu-types";
import { Mixed_OFFSET_p } from "./identity.typegpu";

@CStruct
class Mixed {
  a: f32;
  p: Vec3f;

  constructor(a: f32, p: Vec3f) {
    this.a = a;
    this.p = p;
  }
}
"#;
    let diagnostics = subscript_typegpu_gen::generate(&[
        SourceFile::new("typegpu-types.ts", library),
        SourceFile::new("identity.ts", program),
    ])
    .expect_err("missing Vec3f alignment must fail");
    let message = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(message.contains("SC9"), "{message}");
    assert!(message.contains("C offset 4"), "{message}");
    assert!(message.contains("WGSL offset 16"), "{message}");
}

#[test]
fn a_library_shaped_name_in_a_program_is_an_ordinary_schema() {
    let source = r#"
import { Mat3x3f_OFFSET_value } from "./ordinary-name.typegpu";

@CStruct
class Mat3x3f {
  value: f32;

  constructor(value: f32) {
    this.value = value;
  }
}
"#;
    let generated = subscript_typegpu_gen::generate(&[SourceFile::new("ordinary-name.ts", source)])
        .expect("program class must be a schema");
    assert_eq!(generated.layouts.len(), 1);
    assert_eq!(generated.layouts[0].name, "Mat3x3f");
    assert!(generated.wgsl_module.contains("struct Mat3x3f"));
}

#[test]
fn size_mismatch_names_sizes_without_a_pseudo_field() {
    let library = r#"
@CStruct({ align: 16 })
export class Vec4f {
  x: f32;
  y: f32;
  z: f32;
  w: f32;

  constructor(x: f32, y: f32, z: f32, w: f32) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
  }
}
"#;
    let program = r#"
import { Vec4f } from "./typegpu-types";
import { Root_SIZE } from "./size-mismatch.typegpu";

@CStruct({ align: 16 })
class Padded {
  value: f32;

  constructor(value: f32) {
    this.value = value;
  }
}

@CStruct
class Root {
  head: Vec4f;
  items: FixedArray<Padded, 2>;

  constructor(head: Vec4f, items: FixedArray<Padded, 2>) {
    this.head = head;
    this.items = items;
  }
}
"#;
    let diagnostics = subscript_typegpu_gen::generate(&[
        SourceFile::new("typegpu-types.ts", library),
        SourceFile::new("size-mismatch.ts", program),
    ])
    .expect_err("different total sizes must fail");
    let message = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        message.contains("schema `Root` has C size 48 and WGSL size 32"),
        "{message}"
    );
    assert!(
        message.contains("schema `Padded` has C alignment 16 and WGSL alignment 4"),
        "{message}"
    );
    assert!(!message.contains("<size>"), "{message}");
}
