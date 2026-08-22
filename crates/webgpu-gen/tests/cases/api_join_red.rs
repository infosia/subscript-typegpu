//! Latent API-join ambiguity must fail with the construct named.

fn document(idl: &str) -> String {
    format!("<script type=idl>\n{idl}\n</script>\n")
}

fn policy(interface: &str, boundary: &str, rows: &str) -> String {
    format!(
        r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["{interface}"]
dictionaries = []
namespaces = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "{interface}"
singleton_name = "fixture"
singleton_boundary = "{boundary}"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

{rows}
"#
    )
}

fn mirror(boundary: &str, extra: &str) -> String {
    let mut boundary_snake = String::new();
    for (index, character) in boundary.chars().enumerate() {
        if index != 0 && character.is_ascii_uppercase() {
            boundary_snake.push('_');
        }
        boundary_snake.push(character.to_ascii_lowercase());
    }
    format!(
        r#"
interface SubscriptTypegpu{boundary} {{
  readonly brand: never;
}}
declare function subscript_typegpu_create_instance(): SubscriptTypegpuInstance;
declare function subscript_typegpu_instance_process_events(instance: SubscriptTypegpuInstance): void;
declare function subscript_typegpu_future_status(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): i32;
declare function subscript_typegpu_future_drop(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): void;
declare function subscript_typegpu_{boundary_snake}_release(value: SubscriptTypegpu{boundary}): void;
{extra}
"#
    )
}

#[test]
fn selected_parented_interface_requires_an_explicit_parent_record() {
    let idl = document(
        r#"
interface GPUError {
  readonly attribute DOMString message;
};
interface GPUValidationError : GPUError {
};
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &idl,
        &mirror("ValidationError", ""),
        &policy("GPUValidationError", "ValidationError", ""),
    )
    .expect_err(
        "parented IDL interface unexpectedly generated without an explicit parent policy record",
    )
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUValidationError.parent`: IDL interface inherits `GPUError`; add an [[api.interface_parents]] record with that parent and a reason"
    );
}

#[test]
fn transposed_same_type_arguments_fail_the_ordered_join() {
    let idl = document(
        r#"
interface GPUThing {
  undefined pair(GPUBuffer first, GPUBuffer second);
};
"#,
    );
    let mirror = format!(
        "{}\ninterface SubscriptTypegpuBuffer {{\n  readonly brand: never;\n}}\n\
         declare function subscript_typegpu_thing_pair(thing: SubscriptTypegpuThing, second: SubscriptTypegpuBuffer, first: SubscriptTypegpuBuffer): void;\n",
        mirror("Thing", "")
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &idl,
        &mirror,
        &policy(
            "GPUThing",
            "Thing",
            "[[api.generate]]\nmember = \"GPUThing.pair\"\npattern = \"operation\"",
        ),
    )
    .expect_err("transposed same-type mirror arguments unexpectedly generated")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThing.pair.arguments`: kept IDL argument names [\"first\", \"second\"] do not match mirror parameter names [\"second\", \"first\"] in order"
    );
}

#[test]
fn named_idl_argument_type_must_match_the_derived_mirror_type() {
    let idl = document(
        r#"
interface GPUThing {
  undefined accept(GPUBuffer buffer);
};
"#,
    );
    let mirror = format!(
        "{}\ninterface SubscriptTypegpuTexture {{\n  readonly brand: never;\n}}\n\
         declare function subscript_typegpu_thing_accept(thing: SubscriptTypegpuThing, buffer: SubscriptTypegpuTexture): void;\n",
        mirror("Thing", "")
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &idl,
        &mirror,
        &policy(
            "GPUThing",
            "Thing",
            "[[api.generate]]\nmember = \"GPUThing.accept\"\npattern = \"operation\"",
        ),
    )
    .expect_err("mismatched named IDL and mirror argument types unexpectedly joined")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThing.accept.argument.buffer`: IDL named type `GPUBuffer` joins `SubscriptTypegpuBuffer`, but the mirror declares `SubscriptTypegpuTexture`"
    );
}

#[test]
fn q32_enum_join_names_a_missing_mirror_constant() {
    let idl = document(
        r#"
interface GPUThing {
};
enum GPUBufferMapState {
  "unmapped",
  "pending",
  "mapped",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuBufferMapState {
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED = 1,
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_MAPPED = 3,
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = []
namespaces = []
enums = ["GPUBufferMapState"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUBufferMapState.unmapped"
pattern = "enum-value"

[[api.generate]]
member = "GPUBufferMapState.pending"
pattern = "enum-value"

[[api.generate]]
member = "GPUBufferMapState.mapped"
pattern = "enum-value"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("missing mirror enum constant unexpectedly joined")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUBufferMapState.pending`: mirror enum `SubscriptTypegpuBufferMapState` has no constant `SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_PENDING`"
    );
}

#[test]
fn joined_enums_pass_directly_to_cenum_facade_calls() {
    let idl = document(
        r#"
interface GPUThing {
  undefined setState(GPUBufferMapState state);
};
enum GPUBufferMapState {
  "unmapped",
  "pending",
  "mapped",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuBufferMapState {
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED = 1,
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_PENDING = 2,
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_MAPPED = 3,
}
declare function subscript_typegpu_thing_set_state(thing: SubscriptTypegpuThing, state: SubscriptTypegpuBufferMapState): void;
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = []
namespaces = []
enums = ["GPUBufferMapState"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThing.setState"
pattern = "operation"

[[api.generate]]
member = "GPUBufferMapState.unmapped"
pattern = "enum-value"

[[api.generate]]
member = "GPUBufferMapState.pending"
pattern = "enum-value"

[[api.generate]]
member = "GPUBufferMapState.mapped"
pattern = "enum-value"
"#;
    let generated = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect("Q32 input fixture joins");
    assert!(generated
        .source
        .contains("subscript_typegpu_thing_set_state(this.thing, state);"));
    assert!(!generated
        .source
        .contains("toSubscriptTypegpuBufferMapState"));
    assert!(generated
        .wire_enum_aliases
        .contains("type GPUBufferMapState = CEnum<{\n  \"unmapped\": 1,"));
}

#[test]
fn migrated_enum_arrays_cannot_enter_c_to_script_readback_positions() {
    let idl = document(
        r#"
interface GPUThing {
};
enum GPUBufferMapState {
  "unmapped",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuBufferMapState {
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED = 1,
}
declare class SubscriptTypegpuReadback {
  states: SubscriptTypegpuBufferMapState[];
  constructor(states: SubscriptTypegpuBufferMapState[]);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = []
namespaces = []
enums = ["GPUBufferMapState"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUBufferMapState.unmapped"
pattern = "enum-value"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("migrated enum readback array unexpectedly generated")
        .to_string();
    assert_eq!(
        error,
        "api policy error (invalid): `mirror.SubscriptTypegpuReadback.states`: migrated enum appears in a C-to-script array-read position"
    );
}

#[test]
fn absence_capable_q32_dictionary_fields_use_presence_tests_and_sentinels() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  GPUCompareFunction compare;
};
enum GPUCompareFunction {
  "less-equal",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuCompareFunction {
  SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_UNDEFINED = 0,
  SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_LESS_EQUAL = 1,
}
declare class SubscriptTypegpuThingDescriptor {
  compare: SubscriptTypegpuCompareFunction;
  constructor(compare: SubscriptTypegpuCompareFunction);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = ["GPUCompareFunction"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.compare"
pattern = "dictionary-field"

[[api.generate]]
member = "GPUCompareFunction.less-equal"
pattern = "enum-value"

[[api.enum_exclusions]]
enum_name = "GPUCompareFunction"
mirror = "SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_UNDEFINED"
reason = "fixture boundary-only sentinel"
"#;
    let generated = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect("absence-capable Q32 dictionary fixture joins");
    assert_eq!(
        generated.absence_enum_members,
        ["GPUThingDescriptor.compare"]
    );
    assert!(generated.source.contains("compare?: GPUCompareFunction;"));
    assert!(generated.source.contains(
        "if (value.compare !== undefined) {\n    return value.compare;\n  }\n  return \"undefined\";"
    ));
    assert!(generated.wire_enum_aliases.contains("  \"undefined\": 0,"));
}

#[test]
fn absence_capable_q32_dictionary_fields_require_an_undefined_sentinel() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  GPUCompareFunction compare;
};
enum GPUCompareFunction {
  "less-equal",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuCompareFunction {
  SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_LESS_EQUAL = 1,
}
declare class SubscriptTypegpuThingDescriptor {
  compare: SubscriptTypegpuCompareFunction;
  constructor(compare: SubscriptTypegpuCompareFunction);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = ["GPUCompareFunction"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.compare"
pattern = "dictionary-field"

[[api.generate]]
member = "GPUCompareFunction.less-equal"
pattern = "enum-value"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("absence-capable Q32 dictionary field generated without a sentinel")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.compare`: absence-capable IDL enum `GPUCompareFunction` requires mirror constant `SUBSCRIPT_TYPEGPU_COMPARE_FUNCTION_UNDEFINED`"
    );
}

#[test]
fn dictionary_mirror_enums_cannot_leak_without_idl_lowering() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  required GPUTextureFormat format;
};
enum GPUTextureFormat {
  "rgba8unorm",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuTextureFormat {
  SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_UNORM = 1,
}
declare class SubscriptTypegpuThingDescriptor {
  format: SubscriptTypegpuTextureFormat;
  constructor(format: SubscriptTypegpuTextureFormat);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.format"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("unlowered mirror enum unexpectedly leaked into a public dictionary")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.format`: pattern `dictionary-field` has no public dictionary conversion plan for mirror type `SubscriptTypegpuTextureFormat`; direct emission is restricted to non-null scalar, boolean, and string fields"
    );
}

#[test]
fn dictionary_mirror_enum_arrays_cannot_leak_without_idl_lowering() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  required sequence<GPUTextureFormat> formats;
};
enum GPUTextureFormat {
  "rgba8unorm",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuTextureFormat {
  SUBSCRIPT_TYPEGPU_TEXTURE_FORMAT_RGBA8_UNORM = 1,
}
declare class SubscriptTypegpuThingDescriptor {
  formats: SubscriptTypegpuTextureFormat[];
  constructor(formats: SubscriptTypegpuTextureFormat[]);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.formats"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("unlowered mirror enum array unexpectedly leaked into a public dictionary")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.formats`: pattern `dictionary-field` has no public dictionary conversion plan for mirror type `SubscriptTypegpuTextureFormat[]`; direct emission is restricted to non-null scalar, boolean, and string fields"
    );
}

#[test]
fn dictionary_mirror_classes_cannot_leak_without_idl_lowering() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  required sequence<GPUNestedDescriptor> nested;
};
dictionary GPUNestedDescriptor {
  required unsigned long count;
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuNestedDescriptor {
  count: u32;
  constructor(count: u32);
}
declare class SubscriptTypegpuThingDescriptor {
  nested: SubscriptTypegpuNestedDescriptor[];
  constructor(nested: SubscriptTypegpuNestedDescriptor[]);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.nested"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("unlowered mirror class unexpectedly leaked into a public dictionary")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.nested`: pattern `dictionary-field` has no public dictionary conversion plan for mirror type `SubscriptTypegpuNestedDescriptor[]`; direct emission is restricted to non-null scalar, boolean, and string fields"
    );
}

#[test]
fn selected_dictionary_arrays_lower_element_wise() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUEntry {
  required unsigned long binding;
};
dictionary GPUThingDescriptor {
  required sequence<GPUEntry> entries;
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuEntry {
  binding: u32;
  constructor(binding: u32);
}
declare class SubscriptTypegpuThingDescriptor {
  entries: SubscriptTypegpuEntry[];
  constructor(entries: SubscriptTypegpuEntry[]);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUEntry", "GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUEntry.binding"
pattern = "dictionary-field"

[[api.generate]]
member = "GPUThingDescriptor.entries"
pattern = "dictionary-field"
"#;
    let generated = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect("selected dictionary arrays lower element-wise");
    assert!(generated.source.contains(
        "function toSubscriptTypegpuEntryArray(values: GPUEntry[]): SubscriptTypegpuEntry[]"
    ));
    assert!(generated
        .source
        .contains("toSubscriptTypegpuEntryArray(value.entries)"));
}

#[test]
fn selected_handle_arrays_lower_wrappers_element_wise() {
    let idl = document(
        r#"
interface GPUThing {
};
interface GPUItem {
};
dictionary GPUThingDescriptor {
  required sequence<GPUItem?> items;
};
"#,
    );
    let mirror = format!(
        "{}\ninterface SubscriptTypegpuItem {{\n  readonly brand: never;\n}}\n\
         declare function subscript_typegpu_item_release(value: SubscriptTypegpuItem): void;\n\
         declare class SubscriptTypegpuThingDescriptor {{\n  items: SubscriptTypegpuItem[];\n  constructor(items: SubscriptTypegpuItem[]);\n}}\n",
        mirror("Thing", "")
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing", "GPUItem"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.deviations]]
member = "GPUThingDescriptor.items"
pattern = "dictionary-handle-array"
reason = "fixture drops nullable sequence holes"
"#;
    let generated = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect("selected handle arrays lower wrappers element-wise");
    assert!(generated.source.contains(
        "function toSubscriptTypegpuItemArray(values: GPUItem[]): SubscriptTypegpuItem[]"
    ));
    assert!(generated
        .source
        .contains("lowered.push(values[index].item);"));
}

#[test]
fn q32_enum_join_rejects_an_unaccounted_mirror_constant() {
    let idl = document(
        r#"
interface GPUThing {
};
enum GPUBufferMapState {
  "unmapped",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuBufferMapState {
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNDEFINED = 0,
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED = 1,
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = []
namespaces = []
enums = ["GPUBufferMapState"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUBufferMapState.unmapped"
pattern = "enum-value"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("unaccounted mirror enum sentinel unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `mirror.SubscriptTypegpuBufferMapState.SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNDEFINED`: mirror enum constant has no member in selected IDL enum `GPUBufferMapState`"
    );
}

#[test]
fn dead_enum_exclusion_rows_fail_by_name() {
    let idl = document(
        r#"
interface GPUThing {
};
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = []
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.enum_exclusions]]
enum_name = "GPUBufferMapState"
mirror = "SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNDEFINED"
reason = "fixture boundary-only sentinel"
"#;
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuBufferMapState {
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNDEFINED = 0,
}
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("dead enum exclusion unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (dead): api policy entry `GPUBufferMapState.SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNDEFINED` was consumed by no generation step"
    );
}

#[test]
fn enum_exclusions_cannot_name_an_already_mapped_constant() {
    let idl = document(
        r#"
interface GPUThing {
};
enum GPUBufferMapState {
  "unmapped",
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare enum SubscriptTypegpuBufferMapState {
  SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED = 1,
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = []
namespaces = []
enums = ["GPUBufferMapState"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.enum_exclusions]]
enum_name = "GPUBufferMapState"
mirror = "SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED"
reason = "fixture invalid overlap"

[[api.generate]]
member = "GPUBufferMapState.unmapped"
pattern = "enum-value"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("already-mapped mirror enum constant unexpectedly consumed as an exclusion")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUBufferMapState.SUBSCRIPT_TYPEGPU_BUFFER_MAP_STATE_UNMAPPED`: enum exclusion names mirror constant already mapped from IDL member `GPUBufferMapState.unmapped`"
    );
}

#[test]
fn mirror_alias_cycles_fail_by_name() {
    let idl = document(
        r#"
interface GPUThing {
  unsigned long value();
};
"#,
    );
    let mirror = mirror(
        "Thing",
        "type AliasA = AliasB;\ntype AliasB = AliasA;\ndeclare function subscript_typegpu_thing_value(thing: SubscriptTypegpuThing): AliasA;",
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &idl,
        &mirror,
        &policy(
            "GPUThing",
            "Thing",
            "[[api.generate]]\nmember = \"GPUThing.value\"\npattern = \"operation\"",
        ),
    )
    .expect_err("mirror alias cycle unexpectedly recursed through generation")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `mirror.alias.AliasA`: type-alias cycle detected: AliasA -> AliasB -> AliasA"
    );
}

fn duplicate_default_policy() -> String {
    r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPU"]
dictionaries = ["GPUFirstDescriptor", "GPUSecondDescriptor"]
namespaces = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPU"
singleton_name = "fixture"
singleton_boundary = "Instance"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUFirstDescriptor.label"
pattern = "dictionary-field"

[[api.generate]]
member = "GPUSecondDescriptor.label"
pattern = "dictionary-field"
"#
    .to_owned()
}

fn duplicate_default_document() -> String {
    document(
        r#"
interface GPU {
};
dictionary GPUObjectDescriptorBase {
  DOMString label = "";
};
dictionary GPUFirstDescriptor : GPUObjectDescriptorBase {
};
dictionary GPUSecondDescriptor : GPUObjectDescriptorBase {
};
"#,
    )
}

fn duplicate_default_mirror(second_type: &str) -> String {
    mirror(
        "Instance",
        &format!(
            r#"
declare class SubscriptTypegpuFirstDescriptor {{
  label: string;
  constructor(label: string);
}}
declare class SubscriptTypegpuSecondDescriptor {{
  label: {second_type};
  constructor(label: {second_type});
}}
"#
        ),
    )
}

#[test]
fn identical_default_helpers_are_emitted_once() {
    let generated = subscript_typegpu_webgpu_gen::generate_api(
        &duplicate_default_document(),
        &duplicate_default_mirror("string"),
        &duplicate_default_policy(),
    )
    .expect("identical default helper shapes generate");
    assert_eq!(generated.source.matches("function defaultLabel").count(), 1);
}

#[test]
fn conflicting_default_helper_shapes_fail_by_name() {
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &duplicate_default_document(),
        &duplicate_default_mirror("boolean"),
        &duplicate_default_policy(),
    )
    .expect_err("conflicting default helper shapes unexpectedly generated")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `defaultLabel`: default-helper collision: (\"boolean\", \"\\\"\\\"\") does not match (\"string\", \"\\\"\\\"\")"
    );
}

#[test]
fn handle_arrays_require_their_claiming_pattern() {
    let idl = document(
        r#"
interface GPUThing {
};
interface GPUItem {
};
dictionary GPUThingDescriptor {
  required sequence<GPUItem> items;
};
"#,
    );
    let mirror = format!(
        "{}\ninterface SubscriptTypegpuItem {{\n  readonly brand: never;\n}}\n\
         declare function subscript_typegpu_item_release(value: SubscriptTypegpuItem): void;\n\
         declare class SubscriptTypegpuThingDescriptor {{\n  items: SubscriptTypegpuItem[];\n  constructor(items: SubscriptTypegpuItem[]);\n}}\n",
        mirror("Thing", "")
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing", "GPUItem"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.items"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("handle array unexpectedly generated without its claiming pattern")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.items`: handle-element dictionary arrays require an explicit dictionary-handle-array policy row"
    );
}

#[test]
fn dead_default_helper_rename_rows_fail_by_name() {
    let policy = format!(
        "{}\n[[api.default_helper_renames]]\nmember = \"GPUFirstDescriptor.missing\"\nhelper = \"Missing\"\nreason = \"fixture dead helper rename\"\n",
        duplicate_default_policy()
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &duplicate_default_document(),
        &duplicate_default_mirror("string"),
        &policy,
    )
    .expect_err("dead default-helper rename unexpectedly generated")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (dead): api policy entry `GPUFirstDescriptor.missing.default_helper` was consumed by no generation step"
    );
}

#[test]
fn boundary_defaults_must_name_a_mirror_field() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  required unsigned long value;
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuThingDescriptor {
  value: u32;
  constructor(value: u32);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.deviations]]
member = "GPUThingDescriptor.value"
pattern = "dictionary-boundary-default"
boundary_defaults = ["missing=0"]
reason = "fixture unknown boundary field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("unknown boundary-default field unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (unknown): policy names `mirror.SubscriptTypegpuThingDescriptor.missing` but the selected IDL/mirror join has no such construct"
    );
}

#[test]
fn binding_resource_pattern_rejects_the_wrong_member() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  required GPUWhatever value;
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuThingDescriptor {
  value: u32;
  constructor(value: u32);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.deviations]]
member = "GPUThingDescriptor.value"
pattern = "binding-resource"
reason = "fixture wrong binding-resource owner"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("binding-resource unexpectedly applied to the wrong member")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.value`: binding-resource requires GPUBindGroupEntry.resource"
    );
}

#[test]
fn nullable_mirror_handles_require_a_claimed_conversion_plan() {
    let idl = document(
        r#"
interface GPUThing {
};
interface GPUPipelineLayout {
};
dictionary GPUThingDescriptor {
  required GPUPipelineLayout? layout;
};
"#,
    );
    let mirror = format!(
        "{}\ninterface SubscriptTypegpuPipelineLayout {{\n  readonly brand: never;\n}}\n\
         declare function subscript_typegpu_pipeline_layout_release(value: SubscriptTypegpuPipelineLayout): void;\n\
         declare class SubscriptTypegpuThingDescriptor {{\n  layout: SubscriptTypegpuPipelineLayout | null;\n  constructor(layout: SubscriptTypegpuPipelineLayout | null);\n}}\n",
        mirror("Thing", "")
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing", "GPUPipelineLayout"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.layout"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("nullable mirror handle unexpectedly generated as an unconditional read")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.layout`: nullable mirror handle `SubscriptTypegpuPipelineLayout | null` requires an explicit nullable-handle conversion plan; unconditional handle reads are forbidden"
    );
}

#[test]
fn unknown_mirror_aliases_cannot_cross_dictionary_fallthrough() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  required GPUOpaque value;
};
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuThingDescriptor {
  value: MysteryAlias;
  constructor(value: MysteryAlias);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.value"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("unknown mirror alias unexpectedly crossed the public dictionary")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.value`: pattern `dictionary-field` has no public dictionary conversion plan for mirror type `MysteryAlias`; direct emission is restricted to non-null scalar, boolean, and string fields"
    );
}

#[test]
fn nullable_pipeline_layout_union_requires_and_honors_an_explicit_claim() {
    let idl = document(
        r#"
interface GPUThing {
};
interface GPUPipelineLayout {
};
enum GPUAutoLayoutMode {
  "auto",
};
dictionary GPUThingDescriptor {
  required (GPUPipelineLayout or GPUAutoLayoutMode) layout;
};
"#,
    );
    let mirror = format!(
        "{}\ninterface SubscriptTypegpuPipelineLayout {{\n  readonly brand: never;\n}}\n\
         declare function subscript_typegpu_pipeline_layout_release(value: SubscriptTypegpuPipelineLayout): void;\n\
         declare class SubscriptTypegpuThingDescriptor {{\n  layout: SubscriptTypegpuPipelineLayout | null;\n  constructor(layout: SubscriptTypegpuPipelineLayout | null);\n}}\n",
        mirror("Thing", "")
    );
    let unclaimed_policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing", "GPUPipelineLayout"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.layout"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, unclaimed_policy)
        .expect_err("nullable pipeline-layout union unexpectedly crossed without a claim")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.layout`: pattern `dictionary-field` has no public dictionary conversion plan for mirror type `SubscriptTypegpuPipelineLayout | null`; direct emission is restricted to non-null scalar, boolean, and string fields"
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing", "GPUPipelineLayout"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.deviations]]
member = "GPUThingDescriptor.layout"
pattern = "dictionary-nullable-handle"
field_default = "null"
reason = "fixture auto-layout reshape"
"#;
    let generated = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect("claimed nullable pipeline-layout union joins");
    assert!(generated
        .source
        .contains("layout?: GPUPipelineLayout | null = null;"));
    assert!(generated.source.contains(
        "function toNullableSubscriptTypegpuPipelineLayout(value: GPUPipelineLayout | null): SubscriptTypegpuPipelineLayout | null"
    ));
}

#[test]
fn webidl_records_require_an_explicit_entry_array_reshape() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  record<USVString, GPUPipelineConstantValue> constants = {};
};
typedef double GPUPipelineConstantValue;
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuConstantEntry {
  key: string;
  value: f64;
  constructor(key: string, value: f64);
}
declare class SubscriptTypegpuThingDescriptor {
  constants: SubscriptTypegpuConstantEntry[];
  constructor(constants: SubscriptTypegpuConstantEntry[]);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.generate]]
member = "GPUThingDescriptor.constants"
pattern = "dictionary-field"
"#;
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect_err("WebIDL record unexpectedly crossed the dictionary allowlist")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.constants`: pattern `dictionary-field` has no public dictionary conversion plan for mirror type `SubscriptTypegpuConstantEntry[]`; direct emission is restricted to non-null scalar, boolean, and string fields"
    );
}

#[test]
fn claimed_webidl_records_emit_public_entries_and_element_wise_lowering() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  record<USVString, GPUPipelineConstantValue> constants = {};
};
typedef double GPUPipelineConstantValue;
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuConstantEntry {
  key: string;
  value: f64;
  constructor(key: string, value: f64);
}
declare class SubscriptTypegpuThingDescriptor {
  constants: SubscriptTypegpuConstantEntry[];
  constructor(constants: SubscriptTypegpuConstantEntry[]);
}
"#,
    );
    let policy = r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUThing"]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.deviations]]
member = "GPUThingDescriptor.constants"
pattern = "dictionary-record-entries"
field_default = "[]"
record_entry_api = "GPUPipelineConstantEntry"
record_entry_boundary = "SubscriptTypegpuConstantEntry"
reason = "fixture record reshape"
"#;
    let generated = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, policy)
        .expect("claimed WebIDL record joins");
    assert!(generated
        .source
        .contains("export class GPUPipelineConstantEntry {\n  key!: string;\n  value!: f64;"));
    assert!(generated.source.contains(
        "function toSubscriptTypegpuConstantEntryArray(values: GPUPipelineConstantEntry[]): SubscriptTypegpuConstantEntry[]"
    ));
}

fn dictionary_fixture_policy(interfaces: &str, row: &str) -> String {
    format!(
        r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = [{interfaces}]
dictionaries = ["GPUThingDescriptor"]
namespaces = []
enums = []
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

{row}
"#
    )
}

fn nullable_layout_fixture_mirror(nullable: bool) -> String {
    let layout_type = if nullable {
        "SubscriptTypegpuPipelineLayout | null"
    } else {
        "SubscriptTypegpuPipelineLayout"
    };
    mirror(
        "Thing",
        &format!(
            r#"
interface SubscriptTypegpuPipelineLayout {{
  readonly brand: never;
}}
declare function subscript_typegpu_pipeline_layout_release(value: SubscriptTypegpuPipelineLayout): void;
declare class SubscriptTypegpuThingDescriptor {{
  layout: {layout_type};
  constructor(layout: {layout_type});
}}
"#
        ),
    )
}

fn record_entry_policy() -> String {
    dictionary_fixture_policy(
        "\"GPUThing\"",
        r#"
[[api.deviations]]
member = "GPUThingDescriptor.constants"
pattern = "dictionary-record-entries"
field_default = "[]"
record_entry_api = "GPUPipelineConstantEntry"
record_entry_boundary = "SubscriptTypegpuConstantEntry"
reason = "fixture record reshape"
"#,
    )
}

#[test]
fn nullable_handle_claim_rejects_modelled_idl_types() {
    let idl = document(
        r#"
interface GPUThing {
};
interface GPUPipelineLayout {
};
dictionary GPUThingDescriptor {
  required GPUPipelineLayout? layout;
};
"#,
    );
    let policy = dictionary_fixture_policy(
        "\"GPUThing\", \"GPUPipelineLayout\"",
        r#"
[[api.deviations]]
member = "GPUThingDescriptor.layout"
pattern = "dictionary-nullable-handle"
field_default = "null"
reason = "fixture nullable handle"
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &idl,
        &nullable_layout_fixture_mirror(true),
        &policy,
    )
    .expect_err("modelled IDL type unexpectedly accepted as a union")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.layout`: dictionary-nullable-handle requires a non-modelled IDL type (union)"
    );
}

#[test]
fn nullable_handle_claim_rejects_nonnullable_mirror_handles() {
    let idl = document(
        r#"
interface GPUThing {
};
interface GPUPipelineLayout {
};
enum GPUAutoLayoutMode {
  "auto",
};
dictionary GPUThingDescriptor {
  required (GPUPipelineLayout or GPUAutoLayoutMode) layout;
};
"#,
    );
    let policy = dictionary_fixture_policy(
        "\"GPUThing\", \"GPUPipelineLayout\"",
        r#"
[[api.deviations]]
member = "GPUThingDescriptor.layout"
pattern = "dictionary-nullable-handle"
field_default = "null"
reason = "fixture nullable handle"
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(
        &idl,
        &nullable_layout_fixture_mirror(false),
        &policy,
    )
    .expect_err("non-nullable mirror handle unexpectedly accepted")
    .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.layout`: dictionary-nullable-handle requires a nullable mirror handle, found `SubscriptTypegpuPipelineLayout`"
    );
}

#[test]
fn record_entry_claim_rejects_wrong_idl_record_shape() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  record<DOMString, GPUPipelineConstantValue> constants = {};
};
typedef double GPUPipelineConstantValue;
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuConstantEntry {
  key: string;
  value: f64;
  constructor(key: string, value: f64);
}
declare class SubscriptTypegpuThingDescriptor {
  constants: SubscriptTypegpuConstantEntry[];
  constructor(constants: SubscriptTypegpuConstantEntry[]);
}
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, &record_entry_policy())
        .expect_err("wrong IDL record shape unexpectedly accepted")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.constants`: dictionary-record-entries requires record<USVString, GPUPipelineConstantValue>, found record<DOMString, GPUPipelineConstantValue>"
    );
}

#[test]
fn record_entry_claim_rejects_mirror_array_type_mismatch() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  record<USVString, GPUPipelineConstantValue> constants = {};
};
typedef double GPUPipelineConstantValue;
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuConstantEntry {
  key: string;
  value: f64;
  constructor(key: string, value: f64);
}
declare class SubscriptTypegpuThingDescriptor {
  constants: SubscriptTypegpuWrongEntry[];
  constructor(constants: SubscriptTypegpuWrongEntry[]);
}
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, &record_entry_policy())
        .expect_err("mismatched mirror record-entry array unexpectedly accepted")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `GPUThingDescriptor.constants`: record entry array joins `SubscriptTypegpuConstantEntry[]`, but the mirror field declares `SubscriptTypegpuWrongEntry[]`"
    );
}

#[test]
fn record_entry_claim_rejects_malformed_entry_aggregate() {
    let idl = document(
        r#"
interface GPUThing {
};
dictionary GPUThingDescriptor {
  record<USVString, GPUPipelineConstantValue> constants = {};
};
typedef double GPUPipelineConstantValue;
"#,
    );
    let mirror = mirror(
        "Thing",
        r#"
declare class SubscriptTypegpuConstantEntry {
  key: string;
  value: u32;
  constructor(key: string, value: u32);
}
declare class SubscriptTypegpuThingDescriptor {
  constants: SubscriptTypegpuConstantEntry[];
  constructor(constants: SubscriptTypegpuConstantEntry[]);
}
"#,
    );
    let error = subscript_typegpu_webgpu_gen::generate_api(&idl, &mirror, &record_entry_policy())
        .expect_err("malformed mirror record-entry aggregate unexpectedly accepted")
        .to_string();
    eprintln!("{error}");
    assert_eq!(
        error,
        "api policy error (invalid): `mirror.SubscriptTypegpuConstantEntry`: record entry aggregate must declare exactly key: string and value: f64"
    );
}
