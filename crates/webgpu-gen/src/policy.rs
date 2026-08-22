//! Serde model of `policy.toml` (F18) and the named policy
//! failure classes of the two-way validation.

use std::fmt;

use serde::Deserialize;

/// The committed policy document.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Policy {
    /// Subset membership and the future-anchor object.
    pub slice: SliceSection,
    /// Freestanding webgpu.h functions in the subset.
    #[serde(default)]
    pub functions: Vec<FunctionRow>,
    /// Method rows: membership plus pattern (and reshape data), in
    /// emission order.
    #[serde(default)]
    pub map: Vec<MapRow>,
    /// Exclusions with reasons: the exclusion rows
    /// (facade-generator.md F18).
    #[serde(default)]
    pub exclude: Vec<ExcludeRow>,
    /// Generated facade exports which have no selected API-layer consumer.
    #[serde(default)]
    pub export_exclude: Vec<ExportExcludeRow>,
    /// Enum/bitflag constant sets emitted into the generated Rust.
    #[serde(default)]
    pub constants: Vec<ConstantRow>,
    /// F2 method or generated struct-pair count renames.
    #[serde(default)]
    pub renames: Vec<RenameRow>,
    /// F15 facade-safe representations of otherwise unrepresentable
    /// sentinel values.
    #[serde(default)]
    pub sentinels: Vec<SentinelRow>,
    /// F12 extension structs flattened into chain-free descriptors.
    #[serde(default)]
    pub chain_flattenings: Vec<ChainFlatteningRow>,
    /// S3 typed siblings derived from selected F20 byte-pair methods.
    #[serde(default)]
    pub typed_pairs: Vec<TypedPairRow>,
    /// API-layer subset and J9 trichotomy policy.
    #[serde(default)]
    pub api: Option<ApiSection>,
}

/// `[api]` section: the API subset plus its two-way J9 catalogue.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiSection {
    /// IDL interfaces emitted as wrapper classes, in output order.
    pub interfaces: Vec<String>,
    /// IDL dictionaries emitted as Q33 descriptors, in output order.
    pub dictionaries: Vec<String>,
    /// Pre-pass namespaces emitted as numeric singleton objects.
    pub namespaces: Vec<String>,
    /// Explicit IDL namespace to facade flag-alias joins when the names differ.
    #[serde(default)]
    pub namespace_mappings: Vec<ApiNamespaceMappingRow>,
    /// WebIDL string enums emitted as Q32 literal-union aliases.
    #[serde(default)]
    pub enums: Vec<String>,
    /// Reason for reshaping selected WebIDL namespaces to numeric singleton
    /// objects instead of JavaScript namespace objects.
    pub namespace_reason: String,
    /// Explicit clearances for selected interfaces whose inherited surface is
    /// outside this API slice.
    #[serde(default)]
    pub interface_parents: Vec<ApiInterfaceParentRow>,
    /// Facade-owned fill records projected from read-only IDL interfaces.
    #[serde(default)]
    pub result_records: Vec<ApiResultRecordRow>,
    /// IDL subclasses collapsed into a selected result-record base class.
    #[serde(default)]
    pub flattened_interfaces: Vec<ApiFlattenedInterfaceRow>,
    /// Explicit dictionary/typedef-to-public/boundary joins for WebIDL union
    /// dictionary branches and other named dictionary reshapes.
    #[serde(default)]
    pub dictionary_mappings: Vec<ApiDictionaryMappingRow>,
    /// Structurally identical IDL dictionaries represented by one public and
    /// boundary descriptor.
    #[serde(default)]
    pub dictionary_aliases: Vec<ApiDictionaryAliasRow>,
    /// IDL-inherited flat fields grouped in one nested facade aggregate.
    #[serde(default)]
    pub dictionary_nestings: Vec<ApiDictionaryNestingRow>,
    /// Public Q33 dictionaries whose facade fields are flattened into another
    /// boundary aggregate and therefore have no standalone mirror class.
    #[serde(default)]
    pub public_only_dictionaries: Vec<ApiPublicOnlyDictionaryRow>,
    /// Explicit names for default bridges when distinct fields share a name
    /// but not a type/default shape.
    #[serde(default)]
    pub default_helper_renames: Vec<ApiDefaultHelperRenameRow>,
    /// Explicit IDL string member to mirror integer constant joins.
    #[serde(default)]
    pub enum_mappings: Vec<ApiEnumMappingRow>,
    /// Boundary-only enum constants with no WebIDL string member.
    #[serde(default)]
    pub enum_exclusions: Vec<ApiEnumExclusionRow>,
    /// Interface represented by the module-level singleton.
    pub singleton_interface: String,
    /// Exported singleton identifier.
    pub singleton_name: String,
    /// Boundary handle stem used by the singleton interface.
    pub singleton_boundary: String,
    /// J9 reason for replacing the DOM entry point with the singleton.
    pub singleton_reason: String,
    /// J7 reason for adding `dispose()` to every wrapper.
    pub manual_dispose_reason: String,
    /// Members emitted by an unmodified join pattern.
    #[serde(default)]
    pub generate: Vec<ApiGenerateRow>,
    /// Members emitted with a policy-recorded shape deviation.
    #[serde(default)]
    pub deviations: Vec<ApiDeviationRow>,
    /// Reachable IDL members deliberately absent from this API slice.
    #[serde(default)]
    pub exclude: Vec<ApiExcludeRow>,
}

/// `[[api.namespace_mappings]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiNamespaceMappingRow {
    /// Exact selected WebIDL namespace.
    pub namespace: String,
    /// Exact generated subscript_typegpu mirror alias.
    pub boundary: String,
    /// Name-fidelity reason for the explicit join.
    pub reason: String,
}

/// `[[api.dictionary_mappings]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiDictionaryMappingRow {
    /// Exact selected WebIDL dictionary definition.
    pub dictionary: String,
    /// Public Q33 class name.
    pub api: String,
    /// WebIDL type name which selects this dictionary branch.
    pub idl_type: String,
    /// Exact generated subscript_typegpu mirror class.
    pub boundary: String,
    /// User-visible deviation reason.
    pub reason: String,
}

/// `[[api.dictionary_aliases]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiDictionaryAliasRow {
    /// Selected IDL dictionary represented by the canonical dictionary.
    pub dictionary: String,
    /// Selected canonical IDL dictionary which owns the emitted descriptor.
    pub canonical: String,
    /// User-visible reason the two IDL names collapse to one public type.
    pub reason: String,
}

/// `[[api.dictionary_nestings]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiDictionaryNestingRow {
    /// Selected IDL dictionary with flat inherited members.
    pub dictionary: String,
    /// Facade field holding the nested aggregate.
    pub boundary_field: String,
    /// Exact nested facade aggregate class.
    pub boundary: String,
    /// Exact IDL members grouped into the aggregate, in constructor order.
    pub members: Vec<String>,
    /// Why the facade groups the fields without changing the public shape.
    pub reason: String,
}

/// `[[api.public_only_dictionaries]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiPublicOnlyDictionaryRow {
    /// Exact selected WebIDL dictionary definition.
    pub dictionary: String,
    /// Exact `field=mirror-type` joins in inherited member order.
    pub field_types: Vec<String>,
    /// Why no standalone boundary aggregate exists.
    pub reason: String,
}

/// `[[api.default_helper_renames]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiDefaultHelperRenameRow {
    /// Exact selected `Dictionary.member`.
    pub member: String,
    /// Unique suffix used after `default`.
    pub helper: String,
    /// Why the shared field-name helper is ambiguous.
    pub reason: String,
}

/// `[[api.enum_mappings]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiEnumMappingRow {
    /// Selected WebIDL enum.
    pub enum_name: String,
    /// Exact string member in the WebIDL enum.
    pub member: String,
    /// Exact integer constant in the generated subscript_typegpu mirror enum.
    pub mirror: String,
    /// Reason for recording the spelling join explicitly.
    pub reason: String,
}

/// `[[api.enum_exclusions]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiEnumExclusionRow {
    /// Selected WebIDL enum.
    pub enum_name: String,
    /// Exact boundary-only mirror constant.
    pub mirror: String,
    /// Why the constant is not part of the public Q32 alias.
    pub reason: String,
}

/// `[[api.interface_parents]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiInterfaceParentRow {
    /// Selected child interface.
    pub interface: String,
    /// Exact parent named by the pinned IDL.
    pub parent: String,
    /// J9 reason inherited members are not traversed into this slice.
    pub reason: String,
}

/// `[[api.result_records]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiResultRecordRow {
    /// Exact read-only IDL interface represented by the result class.
    pub interface: String,
    /// Exact facade-owned fill aggregate.
    pub boundary: String,
    /// Whether conversion from the fill aggregate may produce null.
    pub nullable: bool,
    /// Facade-only fields deliberately omitted from the public IDL result.
    #[serde(default)]
    pub boundary_field_exclusions: Vec<String>,
    /// Optional facade-only field added to the public result class.
    #[serde(default)]
    pub synthetic_field: Option<String>,
    /// Public Q32 alias used by the synthetic field.
    #[serde(default)]
    pub synthetic_enum: Option<String>,
    /// Mirror enum read from the synthetic facade field.
    #[serde(default)]
    pub synthetic_boundary_enum: Option<String>,
    /// Exact `public-string=mirror-constant` reverse-lowering joins.
    #[serde(default)]
    pub synthetic_enum_mappings: Vec<String>,
    /// Boundary-only constants which do not become public result values.
    #[serde(default)]
    pub synthetic_enum_exclusions: Vec<String>,
    /// Why the IDL interface is materialized from an F11 fill record.
    pub reason: String,
}

/// `[[api.flattened_interfaces]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiFlattenedInterfaceRow {
    /// Exact selected IDL subclass.
    pub interface: String,
    /// Public result-record class receiving the collapsed subtype.
    pub target: String,
    /// User-visible loss caused by flattening the hierarchy.
    pub reason: String,
}

/// `[[api.generate]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiGenerateRow {
    /// `Interface.member`, `Dictionary.member`, or `Namespace.CONSTANT`.
    pub member: String,
    /// Named API emission pattern.
    pub pattern: String,
}

/// `[[api.deviations]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiDeviationRow {
    /// `Interface.member` in IDL spelling.
    pub member: String,
    /// Named API emission pattern.
    pub pattern: String,
    /// Boundary receiver stem when ordinary `GPUX` → `SubscriptTypegpuX` derivation
    /// does not apply.
    #[serde(default)]
    pub boundary_receiver: Option<String>,
    /// IDL arguments omitted by the facade-winning API shape.
    #[serde(default)]
    pub drop_arguments: Vec<String>,
    /// Makes an otherwise non-null IDL promise result nullable.
    #[serde(default)]
    pub nullable_return: bool,
    /// Replaces a `Promise<undefined>` result with a success boolean.
    #[serde(default)]
    pub boolean_result: bool,
    /// Exact facade future-begin function when the IDL async method name
    /// does not include the facade's `Begin` suffix.
    #[serde(default)]
    pub begin_function: Option<String>,
    /// Explicit IDL-name to mirror-name argument join in mirror call order.
    #[serde(default)]
    pub boundary_arguments: Vec<String>,
    /// Exact IDL argument-name sequence selecting one overloaded operation.
    #[serde(default)]
    pub overload_arguments: Vec<String>,
    /// IDL defaulted arguments deliberately kept required in the public method.
    #[serde(default)]
    pub required_arguments: Vec<String>,
    /// Additional zero-argument method name which sends `null` for an IDL
    /// optional empty descriptor while leaving the IDL-named method typed.
    #[serde(default)]
    pub default_variant: Option<String>,
    /// Makes an IDL optional dictionary field required because the language
    /// cannot express its absent value.
    #[serde(default)]
    pub required_field: bool,
    /// Supplies an explicit public Q33 default when WebIDL absence has a
    /// facade-level sentinel with the same meaning.
    #[serde(default)]
    pub field_default: Option<String>,
    /// Constructor arguments for a facade aggregate used when an optional
    /// nested descriptor is absent.
    #[serde(default)]
    pub absent_boundary_values: Vec<String>,
    /// `field=value` pairs for facade-only fields absent from WebIDL.
    #[serde(default)]
    pub boundary_defaults: Vec<String>,
    /// Public descriptor element used to reshape a WebIDL record/map.
    #[serde(default)]
    pub record_entry_api: Option<String>,
    /// Facade aggregate element used to reshape a WebIDL record/map.
    #[serde(default)]
    pub record_entry_boundary: Option<String>,
    /// Public fixed-key descriptor replacing `requiredLimits`' WebIDL record.
    #[serde(default)]
    pub required_limits_api: Option<String>,
    /// Selected result-record interface supplying the supported limit fields.
    #[serde(default)]
    pub required_limits_source: Option<String>,
    /// Facade u32 sentinel emitted when a public required-limit field is zero.
    #[serde(default)]
    pub required_limits_u32_unspecified: Option<u32>,
    /// J9 reason and exact deviating shape.
    pub reason: String,
}

/// `[[api.exclude]]` row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApiExcludeRow {
    /// Reachable IDL member omitted from the slice.
    pub member: String,
    /// J9 exclusion reason.
    pub reason: String,
}

/// `[slice]` section.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SliceSection {
    /// The object anchoring the future protocol (F6): its handle
    /// types `subscript_typegpu_future_completed` and the take functions, and its
    /// sync methods join the creation chunk.
    pub future_anchor: String,
    /// Subset objects in typedef order.
    pub objects: Vec<String>,
}

/// `[[functions]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FunctionRow {
    pub name: String,
    /// Only `create` in the generator.
    pub pattern: String,
    /// Optional subscript-typegpu.h comment (single line).
    #[serde(default)]
    pub doc: Option<String>,
}

/// `[[map]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MapRow {
    /// `object.method` in yml names.
    pub method: String,
    /// Named emission pattern.
    pub pattern: String,
    /// Required for reshape patterns.
    #[serde(default)]
    pub reason: Option<String>,
}

/// `[[exclude]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExcludeRow {
    /// `object.method`, or `addref` for the implicit AddRef family.
    pub construct: String,
    pub reason: String,
}

/// `[[export_exclude]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportExcludeRow {
    /// Exact generated `subscript_typegpu_*` export name.
    pub name: String,
    pub reason: String,
}

/// `[[constants]]` row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConstantRow {
    /// `bitflag.<name>` or `enum.<name>`.
    pub source: String,
    pub reason: String,
}

/// `[[renames]]` row (F2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RenameRow {
    /// `object.method`, `struct_name.generated_count_field`, or
    /// `object.method.generated_count_parameter` in yml names.
    pub construct: String,
    /// Exact public replacement name.
    pub to: String,
    pub reason: String,
}

/// `[[sentinels]]` row (F15/F18).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SentinelRow {
    /// `struct_name.member` in yml names.
    pub construct: String,
    /// The pinned yml constant substituted when the facade field is zero.
    pub zero_maps_to: String,
    pub reason: String,
}

/// `[[chain_flattenings]]` row (F12/F18).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChainFlatteningRow {
    /// `base_descriptor.extension_struct` in yml names.
    pub construct: String,
    /// Extension fields exposed directly on the public descriptor.
    pub fields: Vec<String>,
    pub reason: String,
}

/// `[[typed_pairs]]` row (S1-S3).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TypedPairRow {
    /// Existing `object.method` selected with the `byte-pair` pattern.
    pub source: String,
    /// Public C element type. S1 permits only `float`.
    pub element: String,
    /// Why this typed sibling exists without a webgpu.yml counterpart.
    pub reason: String,
}

/// The named failure classes of the two-way policy validation (F18),
/// plus `Invalid` for rows that name real constructs but carry data a
/// pattern cannot apply.
#[derive(Debug, PartialEq, Eq)]
pub enum PolicyError {
    /// Policy names a yml entry that does not exist.
    Unknown {
        /// The offending policy name.
        entry: String,
    },
    /// A policy entry no generation step consumed.
    Dead {
        /// The unconsumed policy name.
        entry: String,
    },
    /// The same construct is policed more than once.
    Duplicate {
        /// The repeated policy name.
        entry: String,
    },
    /// A yml construct reachable from the subset with neither rule
    /// pattern nor policy entry.
    Unpoliced {
        /// The unreached yml construct.
        construct: String,
    },
    /// A policy row a pattern cannot apply as written.
    Invalid {
        /// The offending policy name.
        entry: String,
        /// What the pattern rejected.
        message: String,
    },
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyError::Unknown { entry } => write!(
                f,
                "policy error (unknown): policy names `{entry}` but webgpu.yml has no such construct"
            ),
            PolicyError::Dead { entry } => write!(
                f,
                "policy error (dead): policy entry `{entry}` was consumed by no generation step"
            ),
            PolicyError::Duplicate { entry } => write!(
                f,
                "policy error (duplicate): policy lists `{entry}` more than once"
            ),
            PolicyError::Unpoliced { construct } => write!(
                f,
                "policy error (unpoliced): `{construct}` is reachable from the subset but has neither a rule pattern nor a policy entry"
            ),
            PolicyError::Invalid { entry, message } => {
                write!(f, "policy error (invalid): `{entry}`: {message}")
            }
        }
    }
}

impl std::error::Error for PolicyError {}
