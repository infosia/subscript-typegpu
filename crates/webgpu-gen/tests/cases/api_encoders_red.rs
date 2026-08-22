//! Encoder API-generator honesty guards: every join axis fails by construct name.

fn document(idl: &str) -> String {
    format!("<script type=idl>\n{idl}\n</script>\n")
}

fn policy(interfaces: &str, dictionaries: &str, enums: &str, extra: &str) -> String {
    format!(
        r#"
[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = [{interfaces}]
dictionaries = [{dictionaries}]
namespaces = []
enums = [{enums}]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUThing"
singleton_name = "fixture"
singleton_boundary = "Thing"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

{extra}
"#
    )
}

fn mirror(extra: &str) -> String {
    format!(
        r#"
interface SubscriptTypegpuInstance {{
  readonly brand: never;
}}
interface SubscriptTypegpuThing {{
  readonly brand: never;
}}
declare function subscript_typegpu_create_instance(): SubscriptTypegpuInstance;
declare function subscript_typegpu_instance_process_events(instance: SubscriptTypegpuInstance): void;
declare function subscript_typegpu_future_status(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): i32;
declare function subscript_typegpu_future_drop(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): void;
declare function subscript_typegpu_thing_release(value: SubscriptTypegpuThing): void;
{extra}
"#
    )
}

fn red(idl: &str, mirror_source: &str, policy_source: &str, message: &str) {
    let error = subscript_typegpu_webgpu_gen::generate_api(idl, mirror_source, policy_source)
        .expect_err("fixture unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(error, message);
}

#[test]
fn overloaded_operations_require_an_explicit_selector() {
    red(
        &document(
            r#"
interface GPUThing {
  undefined act(unsigned long value);
  undefined act(unsigned long value, unsigned long other);
};
"#,
        ),
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u32): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            "[[api.generate]]\nmember = \"GPUThing.act\"\npattern = \"operation\"",
        ),
        "api policy error (invalid): `GPUThing.act`: IDL operation has 2 overloads; select one with overload_arguments in a deviation row",
    );
}

#[test]
fn overload_selectors_must_name_exact_argument_order() {
    red(
        &document(
            r#"
interface GPUThing {
  undefined act(unsigned long value);
  undefined act(unsigned long value, unsigned long other);
};
"#,
        ),
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u32): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
overload_arguments = ["other", "value"]
reason = "fixture overload"
"#,
        ),
        "api policy error (invalid): `GPUThing.act`: overload_arguments [\"other\", \"value\"] selects 0 overloads, expected exactly one",
    );
}

#[test]
fn required_argument_claims_must_name_defaulted_optionals() {
    red(
        &document(
            r#"
interface GPUThing {
  undefined act(optional unsigned long value = 0);
};
"#,
        ),
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u32): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
required_arguments = ["missing"]
reason = "fixture required argument"
"#,
        ),
        "api policy error (unknown): policy names `GPUThing.act.argument.missing` but the selected IDL/mirror join has no such construct",
    );
}

#[test]
fn scalar_arguments_are_joined_by_exact_width() {
    red(
        &document(
            r#"
interface GPUThing {
  undefined act(unsigned long value);
};
"#,
        ),
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u64): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            "[[api.generate]]\nmember = \"GPUThing.act\"\npattern = \"operation\"",
        ),
        "api policy error (invalid): `GPUThing.act.argument.value`: IDL scalar `u32` does not match mirror parameter `u64`",
    );
}

#[test]
fn deviation_rows_do_not_disable_unreshaped_scalar_width_joins() {
    red(
        &document(
            r#"
interface GPUThing {
  undefined act(unsigned long value, unsigned long tag);
};
"#,
        ),
        &mirror(
            "declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u64, renamedTag: u32): void;",
        ),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
boundary_arguments = ["tag=renamedTag"]
reason = "fixture renames only tag"
"#,
        ),
        "api policy error (invalid): `GPUThing.act.argument.value`: IDL scalar `u32` does not match mirror parameter `u64`",
    );
}

#[test]
fn deviation_rows_do_not_disable_unreshaped_scalar_sequence_joins() {
    red(
        &document(
            r#"
typedef unsigned long GPUOffset;
interface GPUThing {
  undefined act(sequence<GPUOffset> offsets, unsigned long tag);
};
"#,
        ),
        &mirror(
            "declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, offsets: u64[], renamedTag: u32): void;",
        ),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
boundary_arguments = ["tag=renamedTag"]
reason = "fixture renames only tag"
"#,
        ),
        "api policy error (invalid): `GPUThing.act.argument.offsets`: IDL scalar sequence `u32[]` does not match mirror parameter `u64[]`",
    );
}

#[test]
fn sparse_boundary_joins_fail_on_an_unmapped_rename() {
    red(
        &document(
            "interface GPUThing { undefined act(unsigned long first, unsigned long second); };",
        ),
        &mirror(
            "declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, renamedFirst: u32, renamedSecond: u32): void;",
        ),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
boundary_arguments = ["first=renamedFirst"]
reason = "fixture incomplete sparse join"
"#,
        ),
        "api policy error (invalid): `GPUThing.act.arguments`: mirror parameter `renamedSecond` has no explicit or same-named IDL argument join",
    );
}

#[test]
fn sparse_boundary_joins_reject_duplicate_mirror_parameters() {
    red(
        &document(
            "interface GPUThing { undefined act(unsigned long first, unsigned long second); };",
        ),
        &mirror(
            "declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, target: u32, second: u32): void;",
        ),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
boundary_arguments = ["first=target", "second=target"]
reason = "fixture duplicate mirror join"
"#,
        ),
        "api policy error (duplicate): api policy lists `mirror.GPUThing.act.parameter.target` more than once",
    );
}

#[test]
fn sparse_boundary_joins_reject_unknown_mirror_parameters() {
    red(
        &document("interface GPUThing { undefined act(unsigned long value); };"),
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u32): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
boundary_arguments = ["value=missing"]
reason = "fixture unknown mirror join"
"#,
        ),
        "api policy error (unknown): policy names `mirror.GPUThing.act.parameter.missing` but the selected IDL/mirror join has no such construct",
    );
}

#[test]
fn dictionary_aliases_require_exact_field_shapes() {
    red(
        &document(
            r#"
interface GPUThing {};
dictionary GPUCanonical { required unsigned long first; };
dictionary GPUAlias { required unsigned long second; };
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuCanonical {
  first: u32;
  constructor(first: u32);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUCanonical\", \"GPUAlias\"",
            "",
            r#"
[[api.dictionary_aliases]]
dictionary = "GPUAlias"
canonical = "GPUCanonical"
reason = "fixture alias"

[[api.generate]]
member = "GPUCanonical.first"
pattern = "dictionary-field"

[[api.generate]]
member = "GPUAlias.second"
pattern = "dictionary-field"
"#,
        ),
        "api policy error (invalid): `GPUAlias.dictionary_alias`: alias field `second` does not exactly match canonical `GPUCanonical.first`",
    );
}

#[test]
fn dictionary_nestings_require_exact_nested_member_order() {
    red(
        &document(
            r#"
interface GPUThing {};
dictionary GPUThingDescriptor {
  required unsigned long offset;
  required unsigned long bytes;
};
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuLayout {
  offset: u32;
  constructor(offset: u32);
}
declare class SubscriptTypegpuThingDescriptor {
  layout: SubscriptTypegpuLayout;
  constructor(layout: SubscriptTypegpuLayout);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.dictionary_nestings]]
dictionary = "GPUThingDescriptor"
boundary_field = "layout"
boundary = "SubscriptTypegpuLayout"
members = ["offset", "bytes"]
reason = "fixture nesting"

[[api.generate]]
member = "GPUThingDescriptor.offset"
pattern = "dictionary-field"

[[api.generate]]
member = "GPUThingDescriptor.bytes"
pattern = "dictionary-field"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.layout`: nested mirror fields are [\"offset\"], expected policy members [\"offset\", \"bytes\"]",
    );
}

#[test]
fn required_handle_claims_reject_modelled_idl_fields() {
    red(
        &document(
            r#"
interface GPUThing {};
interface GPUTextureView {};
dictionary GPUThingDescriptor { required GPUTextureView view; };
"#,
        ),
        &mirror(
            r#"
interface SubscriptTypegpuTextureView {
  readonly brand: never;
}
declare function subscript_typegpu_texture_view_release(value: SubscriptTypegpuTextureView): void;
declare class SubscriptTypegpuThingDescriptor {
  view: SubscriptTypegpuTextureView | null;
  constructor(view: SubscriptTypegpuTextureView | null);
}
"#,
        ),
        &policy(
            "\"GPUThing\", \"GPUTextureView\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.deviations]]
member = "GPUThingDescriptor.view"
pattern = "dictionary-required-handle"
reason = "fixture required handle"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.view`: dictionary-required-handle requires a required non-modelled IDL type (union)",
    );
}

#[test]
fn optional_handle_claims_require_nullable_mirror_handles() {
    red(
        &document(
            r#"
interface GPUThing {};
interface GPUQuerySet {};
dictionary GPUThingDescriptor { GPUQuerySet occlusionQuerySet; };
"#,
        ),
        &mirror(
            r#"
interface SubscriptTypegpuQuerySet {
  readonly brand: never;
}
declare function subscript_typegpu_query_set_release(value: SubscriptTypegpuQuerySet): void;
declare class SubscriptTypegpuThingDescriptor {
  occlusionQuerySet: SubscriptTypegpuQuerySet;
  constructor(occlusionQuerySet: SubscriptTypegpuQuerySet);
}
"#,
        ),
        &policy(
            "\"GPUThing\", \"GPUQuerySet\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.deviations]]
member = "GPUThingDescriptor.occlusionQuerySet"
pattern = "dictionary-optional-handle"
field_default = "null"
reason = "fixture optional handle"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.occlusionQuerySet`: dictionary-optional-handle requires an optional IDL field joined to a nullable mirror handle, found `SubscriptTypegpuQuerySet`",
    );
}

#[test]
fn enum_array_claims_require_nullable_idl_elements() {
    red(
        &document(
            r#"
interface GPUThing {};
enum GPUFormat { "a" };
dictionary GPUThingDescriptor { required sequence<GPUFormat> formats; };
"#,
        ),
        &mirror(
            r#"
declare enum SubscriptTypegpuFormat {
  SUBSCRIPT_TYPEGPU_FORMAT_A = 1,
}
declare class SubscriptTypegpuThingDescriptor {
  formats: SubscriptTypegpuFormat[];
  constructor(formats: SubscriptTypegpuFormat[]);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUThingDescriptor\"",
            "\"GPUFormat\"",
            r#"
[[api.deviations]]
member = "GPUThingDescriptor.formats"
pattern = "dictionary-enum-array"
reason = "fixture enum array"

[[api.generate]]
member = "GPUFormat.a"
pattern = "enum-value"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.formats`: dictionary-enum-array requires nullable IDL enum elements",
    );
}

#[test]
fn union_descriptor_claims_require_nonnullable_mirror_aggregates() {
    red(
        &document(
            r#"
interface GPUThing {};
dictionary GPUColorDict { required double r; };
typedef (GPUColorDict or sequence<double>) GPUColor;
dictionary GPUThingDescriptor { GPUColor clearValue = {}; };
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuColor {
  r: f64;
  constructor(r: f64);
}
declare class SubscriptTypegpuThingDescriptor {
  clearValue: SubscriptTypegpuColor | null;
  constructor(clearValue: SubscriptTypegpuColor | null);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUColorDict\", \"GPUThingDescriptor\"",
            "",
            r#"
[[api.dictionary_mappings]]
dictionary = "GPUColorDict"
api = "GPUColor"
idl_type = "GPUColor"
boundary = "SubscriptTypegpuColor"
reason = "fixture dictionary branch"

[[api.generate]]
member = "GPUColorDict.r"
pattern = "dictionary-field"

[[api.deviations]]
member = "GPUThingDescriptor.clearValue"
pattern = "dictionary-union-descriptor"
field_default = "{ r: 0 }"
reason = "fixture union descriptor"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.clearValue`: dictionary-union-descriptor requires a non-null mirror aggregate, found `SubscriptTypegpuColor | null`",
    );
}

#[test]
fn parameter_handle_arrays_join_exact_mirror_element_types() {
    red(
        &document(
            r#"
interface GPUThing { undefined submit(sequence<GPUItem> items); };
interface GPUItem {};
"#,
        ),
        &mirror(
            r#"
interface SubscriptTypegpuItem {
  readonly brand: never;
}
interface SubscriptTypegpuWrong {
  readonly brand: never;
}
declare function subscript_typegpu_item_release(value: SubscriptTypegpuItem): void;
declare function subscript_typegpu_thing_submit(thing: SubscriptTypegpuThing, items: SubscriptTypegpuWrong[]): void;
"#,
        ),
        &policy(
            "\"GPUThing\", \"GPUItem\"",
            "",
            "",
            "[[api.generate]]\nmember = \"GPUThing.submit\"\npattern = \"operation\"",
        ),
        "api policy error (invalid): `GPUThing.submit.argument.items`: IDL handle sequence `GPUItem` joins `SubscriptTypegpuItem[]`, but the mirror parameter declares `SubscriptTypegpuWrong[]`",
    );
}

#[test]
fn nullable_handle_parameters_join_nullability_exactly() {
    red(
        &document(
            r#"
interface GPUThing { undefined setItem(GPUItem? item); };
interface GPUItem {};
"#,
        ),
        &mirror(
            r#"
interface SubscriptTypegpuItem {
  readonly brand: never;
}
declare function subscript_typegpu_item_release(value: SubscriptTypegpuItem): void;
declare function subscript_typegpu_thing_set_item(thing: SubscriptTypegpuThing, item: SubscriptTypegpuItem): void;
"#,
        ),
        &policy(
            "\"GPUThing\", \"GPUItem\"",
            "",
            "",
            "[[api.generate]]\nmember = \"GPUThing.setItem\"\npattern = \"operation\"",
        ),
        "api policy error (invalid): `GPUThing.setItem.argument.item`: IDL handle nullability `true` does not match mirror parameter `SubscriptTypegpuItem`",
    );
}

#[test]
fn overload_selectors_cannot_attach_to_non_overloaded_operations() {
    red(
        &document("interface GPUThing { undefined act(unsigned long value); };") ,
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u32): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
overload_arguments = ["value"]
reason = "fixture stray overload selector"
"#,
        ),
        "api policy error (invalid): `GPUThing.act`: overload_arguments names an IDL operation without overloads",
    );
}

#[test]
fn required_argument_claims_reject_required_idl_arguments() {
    red(
        &document("interface GPUThing { undefined act(unsigned long value); };") ,
        &mirror("declare function subscript_typegpu_thing_act(thing: SubscriptTypegpuThing, value: u32): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            r#"
[[api.deviations]]
member = "GPUThing.act"
pattern = "operation"
required_arguments = ["value"]
reason = "fixture invalid required argument"
"#,
        ),
        "api policy error (invalid): `GPUThing.act.argument.value`: required_arguments must name an optional IDL argument with a default",
    );
}

#[test]
fn dictionary_aliases_require_matching_policy_classification() {
    red(
        &document(
            r#"
interface GPUThing {};
dictionary GPUCanonical { unsigned long value = 1; };
dictionary GPUAlias { unsigned long value = 1; };
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuCanonical {
  value: u32;
  constructor(value: u32);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUCanonical\", \"GPUAlias\"",
            "",
            r#"
[[api.dictionary_aliases]]
dictionary = "GPUAlias"
canonical = "GPUCanonical"
reason = "fixture alias"

[[api.generate]]
member = "GPUCanonical.value"
pattern = "dictionary-field"

[[api.deviations]]
member = "GPUAlias.value"
pattern = "dictionary-default"
field_default = "1"
reason = "fixture mismatched alias policy"
"#,
        ),
        "api policy error (invalid): `GPUAlias.value`: dictionary alias policy does not match canonical `GPUCanonical.value`",
    );
}

#[test]
fn dictionary_nestings_join_the_named_boundary_field_type() {
    red(
        &document(
            r#"
interface GPUThing {};
dictionary GPUThingDescriptor { required unsigned long offset; };
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuExpectedLayout {
  offset: u32;
  constructor(offset: u32);
}
declare class SubscriptTypegpuThingDescriptor {
  layout: SubscriptTypegpuActualLayout;
  constructor(layout: SubscriptTypegpuActualLayout);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.dictionary_nestings]]
dictionary = "GPUThingDescriptor"
boundary_field = "layout"
boundary = "SubscriptTypegpuExpectedLayout"
members = ["offset"]
reason = "fixture nesting"

[[api.generate]]
member = "GPUThingDescriptor.offset"
pattern = "dictionary-field"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.layout`: dictionary nesting names `SubscriptTypegpuExpectedLayout`, but mirror field declares `SubscriptTypegpuActualLayout`",
    );
}

#[test]
fn required_handle_claims_require_a_mirror_handle() {
    red(
        &document(
            r#"
interface GPUThing {};
interface GPUTexture {};
interface GPUTextureView {};
dictionary GPUThingDescriptor { required (GPUTexture or GPUTextureView) view; };
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuThingDescriptor {
  view: u32;
  constructor(view: u32);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.deviations]]
member = "GPUThingDescriptor.view"
pattern = "dictionary-required-handle"
reason = "fixture required handle"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.view`: dictionary-required-handle requires a mirror handle, found `u32`",
    );
}

#[test]
fn optional_handle_claims_reject_nullable_idl_types() {
    red(
        &document(
            r#"
interface GPUThing {};
interface GPUQuerySet {};
dictionary GPUThingDescriptor { GPUQuerySet? querySet; };
"#,
        ),
        &mirror(
            r#"
interface SubscriptTypegpuQuerySet {
  readonly brand: never;
}
declare function subscript_typegpu_query_set_release(value: SubscriptTypegpuQuerySet): void;
declare class SubscriptTypegpuThingDescriptor {
  querySet: SubscriptTypegpuQuerySet | null;
  constructor(querySet: SubscriptTypegpuQuerySet | null);
}
"#,
        ),
        &policy(
            "\"GPUThing\", \"GPUQuerySet\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.deviations]]
member = "GPUThingDescriptor.querySet"
pattern = "dictionary-optional-handle"
field_default = "null"
reason = "fixture optional handle"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.querySet`: dictionary-optional-handle requires a non-null named IDL handle type",
    );
}

#[test]
fn union_descriptor_claims_require_a_selected_dictionary_branch() {
    red(
        &document(
            r#"
interface GPUThing {};
dictionary GPUThingDescriptor { required GPUColor clearValue; };
"#,
        ),
        &mirror(
            r#"
declare class SubscriptTypegpuColor {
  r: f64;
  constructor(r: f64);
}
declare class SubscriptTypegpuThingDescriptor {
  clearValue: SubscriptTypegpuColor;
  constructor(clearValue: SubscriptTypegpuColor);
}
"#,
        ),
        &policy(
            "\"GPUThing\"",
            "\"GPUThingDescriptor\"",
            "",
            r#"
[[api.deviations]]
member = "GPUThingDescriptor.clearValue"
pattern = "dictionary-union-descriptor"
field_default = "{ r: 0 }"
reason = "fixture missing branch"
"#,
        ),
        "api policy error (invalid): `GPUThingDescriptor.clearValue`: mirror aggregate `SubscriptTypegpuColor` has no selected public dictionary branch",
    );
}

#[test]
fn parameter_handle_arrays_reject_nullable_elements() {
    red(
        &document(
            r#"
interface GPUThing { undefined submit(sequence<GPUItem?> items); };
interface GPUItem {};
"#,
        ),
        &mirror(
            r#"
interface SubscriptTypegpuItem {
  readonly brand: never;
}
declare function subscript_typegpu_item_release(value: SubscriptTypegpuItem): void;
declare function subscript_typegpu_thing_submit(thing: SubscriptTypegpuThing, items: SubscriptTypegpuItem[]): void;
"#,
        ),
        &policy(
            "\"GPUThing\", \"GPUItem\"",
            "",
            "",
            "[[api.generate]]\nmember = \"GPUThing.submit\"\npattern = \"operation\"",
        ),
        "api policy error (invalid): `GPUThing.submit.argument.items`: parameter-position handle arrays cannot contain nullable elements",
    );
}

#[test]
fn typedef_scalar_arrays_are_joined_by_exact_width() {
    red(
        &document(
            r#"
typedef unsigned long GPUOffset;
interface GPUThing { undefined setOffsets(sequence<GPUOffset> offsets); };
"#,
        ),
        &mirror("declare function subscript_typegpu_thing_set_offsets(thing: SubscriptTypegpuThing, offsets: u64[]): void;"),
        &policy(
            "\"GPUThing\"",
            "",
            "",
            "[[api.generate]]\nmember = \"GPUThing.setOffsets\"\npattern = \"operation\"",
        ),
        "api policy error (invalid): `GPUThing.setOffsets.argument.offsets`: IDL scalar sequence `u32[]` does not match mirror parameter `u64[]`",
    );
}
