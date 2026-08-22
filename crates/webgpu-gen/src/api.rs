//! API-layer join, J9 validation, and byte-stable source emission.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use weedle::Parse;

use crate::api_model::{
    IdlArgument, IdlMember, IdlMemberKind, IdlModel, IdlType, MirrorField, MirrorModel, MirrorParam,
};
use crate::naming;
use crate::policy::{ApiDeviationRow, ApiSection, Policy};

const GPU_DEVICE_CONSTRUCTOR: &str = "GPUDevice.@constructor";
const HOST_OWNED_WRAPPER_PATTERN: &str = "host-owned-wrapper";
const GPU_QUEUE_WRITE_BUFFER_F32: &str = "GPUQueue.writeBufferF32";
const GPU_BUFFER_READ_MAPPED_RANGE_F32: &str = "GPUBuffer.readMappedRangeF32";
const TYPED_WRITE_F32_PATTERN: &str = "typed-write-f32";
const TYPED_READ_F32_PATTERN: &str = "typed-read-f32";

/// One IDL-joined enum mapping consumed by the facade header generator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CEnumAlias {
    /// Generated subscript_typegpu enum typedef.
    pub boundary_name: String,
    /// GPUWeb IDL enum alias.
    pub public_name: String,
}

/// Generated API source plus the pattern/override measurement.
#[derive(Debug)]
pub struct GeneratedApi {
    /// Exact `api/webgpu.ts` bytes.
    pub source: String,
    /// Exact ambient aliases for enums that cross host C boundaries.
    pub wire_enum_aliases: String,
    /// IDL-joined enum typedefs that migrate through `@subscript-cenum`.
    pub cenum_aliases: Vec<CEnumAlias>,
    /// Emitted members produced without a per-member shape override.
    pub pattern_members: usize,
    /// Emitted members whose shape is controlled by a deviation row.
    pub override_members: usize,
    /// Selected IDL members explicitly excluded by J9 policy.
    pub excluded_members: usize,
    /// Wrapper classes emitted around the member surface, accounted
    /// separately from IDL members.
    pub wrapper_constructs: usize,
    /// Facade-owned result classes emitted from F11 fill records.
    pub result_constructs: usize,
    /// Named policy notes for namespace-to-singleton reshapes.
    pub namespace_reshape_notes: Vec<String>,
    /// Selected optional-without-default Q32 dictionary members whose
    /// absence lowers to a boundary-only Undefined sentinel.
    pub absence_enum_members: Vec<String>,
}

/// Named J9 two-way policy failures for the API section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiPolicyError {
    /// Policy names no selected IDL or joined mirror construct.
    Unknown {
        /// Offending policy key.
        entry: String,
    },
    /// A valid policy row was consumed by no generation step.
    Dead {
        /// Unconsumed policy key.
        entry: String,
    },
    /// A policy classification is repeated.
    Duplicate {
        /// Repeated policy key.
        entry: String,
    },
    /// A selected IDL member has no J9 classification.
    Unpoliced {
        /// Reachable IDL member.
        construct: String,
    },
    /// A real construct does not satisfy the selected emission pattern.
    Invalid {
        /// Policy key or join construct.
        entry: String,
        /// Shape mismatch.
        message: String,
    },
}

impl fmt::Display for ApiPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiPolicyError::Unknown { entry } => write!(
                f,
                "api policy error (unknown): policy names `{entry}` but the selected IDL/mirror join has no such construct"
            ),
            ApiPolicyError::Dead { entry } => write!(
                f,
                "api policy error (dead): api policy entry `{entry}` was consumed by no generation step"
            ),
            ApiPolicyError::Duplicate { entry } => write!(
                f,
                "api policy error (duplicate): api policy lists `{entry}` more than once"
            ),
            ApiPolicyError::Unpoliced { construct } => write!(
                f,
                "api policy error (unpoliced): `{construct}` is reachable from the selected IDL subset but is neither generated, deviation-rowed, nor excluded"
            ),
            ApiPolicyError::Invalid { entry, message } => {
                write!(f, "api policy error (invalid): `{entry}`: {message}")
            }
        }
    }
}

impl std::error::Error for ApiPolicyError {}

/// API extraction, mirror ingestion, TOML, or J9 failure.
#[derive(Debug)]
pub enum ApiError {
    /// GPUWeb `<script type=idl>` extraction or pre-pass failure.
    IdlExtraction(String),
    /// `weedle2` rejected the post-pre-pass source or left a remainder.
    IdlParse(String),
    /// Owned IDL projection failed.
    IdlModel(String),
    /// Generated mirror projection failed.
    Mirror(String),
    /// `policy.toml` failed to deserialize.
    Toml(toml::de::Error),
    /// J9 or join validation failed.
    Policy(ApiPolicyError),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::IdlExtraction(error) => write!(f, "gpuweb IDL extraction: {error}"),
            ApiError::IdlParse(error) => write!(f, "gpuweb IDL parse: {error}"),
            ApiError::IdlModel(error) => write!(f, "gpuweb IDL model: {error}"),
            ApiError::Mirror(error) => write!(f, "subscript_typegpu mirror model: {error}"),
            ApiError::Toml(error) => write!(f, "policy.toml: {error}"),
            ApiError::Policy(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<ApiPolicyError> for ApiError {
    fn from(error: ApiPolicyError) -> Self {
        ApiError::Policy(error)
    }
}

#[derive(Clone, Debug)]
enum Classification {
    Generate(String),
    Deviation(Box<ApiDeviationRow>),
    Exclude,
}

#[derive(Clone, Debug)]
struct ClassifiedMember {
    classification: Classification,
}

#[derive(Clone, Debug)]
struct DescriptorFieldPlan {
    name: String,
    ty: String,
    required: bool,
    default: Option<String>,
    conversion: DescriptorFieldConversion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RequiredLimitFieldPlan {
    name: String,
    ty: String,
}

#[derive(Clone, Debug)]
enum DescriptorFieldConversion {
    Direct,
    Enum,
    OptionalEnum {
        public_name: String,
        undefined_key: String,
    },
    RequiredOptionalBool,
    EnumArray,
    Descriptor(String),
    DescriptorArray(String),
    RecordEntries {
        api_name: String,
        boundary_name: String,
    },
    RequiredLimits {
        api_name: String,
        boundary_name: String,
        fields: Vec<RequiredLimitFieldPlan>,
        u32_unspecified: u32,
    },
    OptionalDescriptor {
        boundary_name: String,
        absent_values: Vec<String>,
    },
    NullableDescriptor {
        boundary_name: String,
    },
    Handle(String),
    HandleArray {
        boundary_name: String,
        api_name: String,
        raw_field: String,
    },
    NullableHandle {
        boundary_name: String,
        api_name: String,
        raw_field: String,
    },
}

#[derive(Clone, Debug)]
struct DescriptorPlan {
    idl_name: String,
    idl_type: String,
    idl_aliases: Vec<String>,
    name: String,
    boundary_name: String,
    public_only: bool,
    fields: Vec<DescriptorFieldPlan>,
    boundary_fields: Vec<MirrorField>,
    boundary_defaults: BTreeMap<String, String>,
    nested_boundaries: Vec<NestedBoundaryPlan>,
}

#[derive(Clone, Debug)]
struct NestedBoundaryPlan {
    field_name: String,
    boundary_name: String,
    members: Vec<String>,
}

#[derive(Clone, Debug)]
struct NamespacePlan {
    name: String,
    value_type: String,
    constants: Vec<(String, u64)>,
}

#[derive(Clone, Debug)]
struct EnumMemberPlan {
    idl_name: String,
    mirror_name: String,
    wire_value: i64,
}

#[derive(Clone, Debug)]
struct EnumPlan {
    name: String,
    mirror_name: String,
    members: Vec<EnumMemberPlan>,
    exclusions: Vec<EnumMemberPlan>,
}

#[derive(Clone, Debug)]
struct SyntheticEnumPlan {
    name: String,
    mirror_name: String,
    members: Vec<EnumMemberPlan>,
    exclusions: Vec<String>,
}

#[derive(Clone, Debug)]
enum ResultRecordFieldConversion {
    Direct,
    Enum,
    SyntheticEnum,
}

#[derive(Clone, Debug)]
struct ResultRecordFieldPlan {
    name: String,
    ty: String,
    conversion: ResultRecordFieldConversion,
}

#[derive(Clone, Debug)]
struct ResultRecordPlan {
    name: String,
    boundary_name: String,
    fields: Vec<ResultRecordFieldPlan>,
    synthetic_enum: Option<SyntheticEnumPlan>,
    nullable: bool,
    seed_values: Vec<String>,
}

#[derive(Clone, Debug)]
enum RecordFillSuccess {
    Boolean,
    StatusOne,
}

#[derive(Clone, Debug)]
struct DefaultHelperPlan {
    name: String,
    ty: String,
    default: String,
}

#[derive(Clone, Debug)]
enum MethodPlan {
    Async {
        name: String,
        begin: String,
        params: Vec<MethodParamPlan>,
        begin_args: Vec<String>,
        take: Option<String>,
        result_class: Option<String>,
        nullable: bool,
        boolean_result: bool,
    },
    Attribute {
        name: String,
        getter: String,
        return_type: String,
        result_class: Option<String>,
        enum_conversion: Option<String>,
    },
    Operation {
        name: String,
        function: String,
        params: Vec<MethodParamPlan>,
        return_type: String,
        result_class: Option<String>,
        call_args: Vec<String>,
        default_variant: Option<DefaultVariantPlan>,
    },
    MappedRange {
        read: String,
        write: String,
    },
    TypedWriteF32 {
        function: String,
    },
    TypedReadF32 {
        function: String,
    },
    ErrorScopePop {
        name: String,
        begin: String,
        take: String,
        record: String,
        result_class: String,
        nullable: bool,
        conversion: String,
        seed_values: Vec<String>,
    },
    RecordDrain {
        name: String,
        function: String,
        record: String,
        result_class: String,
        conversion: String,
        seed_values: Vec<String>,
        pump: bool,
    },
    RecordFill {
        name: String,
        function: String,
        record: String,
        result_class: String,
        conversion: String,
        seed_values: Vec<String>,
        success: RecordFillSuccess,
    },
}

fn build_mapped_range_method(
    mirror: &MirrorModel,
    member: &IdlMember,
    receiver: &str,
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Operation {
        return_type,
        arguments,
    } = &member.kind
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "mapped-range requires an IDL operation".to_owned(),
        });
    };
    if !matches!(
        return_type,
        IdlType::Named {
            name,
            nullable: false
        } if name == "ArrayBuffer"
    ) {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "mapped-range requires the IDL ArrayBuffer result".to_owned(),
        });
    }
    let argument_names = arguments
        .iter()
        .map(|argument| argument.name.as_str())
        .collect::<Vec<_>>();
    if argument_names != ["offset", "size"] {
        return Err(ApiPolicyError::Invalid {
            entry: format!("{}.arguments", member.key()),
            message: format!(
                "mapped-range expects IDL arguments [\"offset\", \"size\"], found {argument_names:?}"
            ),
        });
    }
    let read = format!(
        "subscript_typegpu_{}_read_mapped_range",
        naming::snake(receiver)
    );
    let write = format!(
        "subscript_typegpu_{}_write_mapped_range",
        naming::snake(receiver)
    );
    let read_fn = mirror
        .functions
        .get(&read)
        .ok_or_else(|| unknown(&format!("mirror.{read}")))?;
    validate_parameter_types(
        read_fn,
        &[
            format!("SubscriptTypegpu{receiver}"),
            "u64".to_owned(),
            "u8[]".to_owned(),
        ],
    )?;
    if read_fn.return_type != "i32" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{read}"),
            message: "mapped-range read must return i32 status".to_owned(),
        });
    }
    let write_fn = mirror
        .functions
        .get(&write)
        .ok_or_else(|| unknown(&format!("mirror.{write}")))?;
    validate_parameter_types(
        write_fn,
        &[
            format!("SubscriptTypegpu{receiver}"),
            "u64".to_owned(),
            "u8[]".to_owned(),
        ],
    )?;
    if write_fn.return_type != "i32" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{write}"),
            message: "mapped-range write must return i32 status".to_owned(),
        });
    }
    Ok(MethodPlan::MappedRange { read, write })
}

impl MethodPlan {
    fn result_class(&self) -> Option<&str> {
        match self {
            MethodPlan::Async { result_class, .. }
            | MethodPlan::Attribute { result_class, .. }
            | MethodPlan::Operation { result_class, .. } => result_class.as_deref(),
            MethodPlan::RecordDrain { result_class, .. }
            | MethodPlan::RecordFill { result_class, .. } => Some(result_class),
            MethodPlan::MappedRange { .. }
            | MethodPlan::TypedWriteF32 { .. }
            | MethodPlan::TypedReadF32 { .. }
            | MethodPlan::ErrorScopePop { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
struct MethodParamPlan {
    name: String,
    api_type: String,
    expression: String,
    default: Option<String>,
    helper: Option<MethodParamHelper>,
}

#[derive(Clone, Debug)]
enum MethodParamHelper {
    HandleArray {
        boundary_name: String,
        api_name: String,
        raw_field: String,
    },
    NullableHandle {
        boundary_name: String,
        api_name: String,
        raw_field: String,
    },
    NullableDescriptor {
        boundary_name: String,
        api_name: String,
    },
}

#[derive(Clone, Debug)]
struct DefaultVariantPlan {
    name: String,
    descriptor_expression: String,
}

#[derive(Clone, Debug)]
struct InterfacePlan {
    name: String,
    boundary: String,
    raw_field: String,
    methods: Vec<MethodPlan>,
    needs_instance: bool,
    host_owned: bool,
    idempotent_dispose: bool,
}

#[derive(Clone, Debug)]
struct ApiPlan {
    policy: ApiSection,
    descriptors: Vec<DescriptorPlan>,
    namespaces: Vec<NamespacePlan>,
    enums: Vec<EnumPlan>,
    result_records: Vec<ResultRecordPlan>,
    interfaces: Vec<InterfacePlan>,
    pattern_members: usize,
    override_members: usize,
    wrapper_constructs: usize,
    result_constructs: usize,
}

#[derive(Clone, Debug)]
struct LedgerRow {
    key: String,
    consumed: bool,
}

/// Generates the API layer from GPUWeb IDL, the committed facade mirror,
/// and the shared policy document.
pub fn generate_api(
    gpuweb_document: &str,
    mirror_source: &str,
    policy_text: &str,
) -> Result<GeneratedApi, ApiError> {
    let extracted =
        crate::idl::extract_gpuweb_idl(gpuweb_document).map_err(ApiError::IdlExtraction)?;
    let (remaining, definitions) = weedle::Definitions::parse(&extracted.weedle_source)
        .map_err(|error| ApiError::IdlParse(format!("{error:?}")))?;
    if !remaining.trim().is_empty() {
        return Err(ApiError::IdlParse(format!(
            "unconsumed remainder `{remaining}`"
        )));
    }
    let idl = IdlModel::from_definitions(&definitions, &extracted.namespace_constants)
        .map_err(ApiError::IdlModel)?;
    let mirror = MirrorModel::parse(mirror_source).map_err(ApiError::Mirror)?;
    let policy: Policy = toml::from_str(policy_text).map_err(ApiError::Toml)?;
    let api_policy = policy.api.ok_or_else(|| ApiPolicyError::Unknown {
        entry: "api".to_owned(),
    })?;
    let plan = build_plan(&idl, &mirror, api_policy)?;
    let source = render(&plan)?;
    let wire_enum_aliases = render_wire_enum_aliases(&plan)?;
    let cenum_aliases = plan
        .enums
        .iter()
        .map(|enum_plan| CEnumAlias {
            boundary_name: enum_plan.mirror_name.clone(),
            public_name: enum_plan.name.clone(),
        })
        .collect();
    Ok(GeneratedApi {
        source,
        wire_enum_aliases,
        cenum_aliases,
        pattern_members: plan.pattern_members,
        override_members: plan.override_members,
        excluded_members: plan.policy.exclude.len(),
        wrapper_constructs: plan.wrapper_constructs,
        result_constructs: plan.result_constructs,
        namespace_reshape_notes: plan
            .policy
            .namespaces
            .iter()
            .map(|name| format!("{name}: {}", plan.policy.namespace_reason))
            .collect(),
        absence_enum_members: plan
            .descriptors
            .iter()
            .flat_map(|descriptor| {
                descriptor
                    .fields
                    .iter()
                    .filter(|field| {
                        matches!(
                            &field.conversion,
                            DescriptorFieldConversion::OptionalEnum { .. }
                        )
                    })
                    .map(|field| format!("{}.{}", descriptor.idl_name, field.name))
            })
            .collect(),
    })
}

fn build_plan(
    idl: &IdlModel,
    mirror: &MirrorModel,
    policy: ApiSection,
) -> Result<ApiPlan, ApiPolicyError> {
    check_policy_duplicates(&policy)?;
    check_reasons(&policy)?;
    validate_interface_parents(idl, &policy)?;

    let mut reachable = BTreeMap::new();
    let mut ordered_interface_members = BTreeMap::new();
    let mut ordered_result_record_members = BTreeMap::new();
    let mut ordered_flattened_members = BTreeMap::new();
    let mut ordered_dictionary_members = BTreeMap::new();
    let mut ordered_namespace_members = BTreeMap::new();
    let mut ordered_enum_members = BTreeMap::new();

    for interface in &policy.interfaces {
        let members = idl
            .interface_members(interface)
            .map_err(|_| unknown(interface))?;
        let members = select_operation_overloads(interface, members, &policy)?;
        for member in &members {
            reachable.insert(member.key(), member.clone());
        }
        ordered_interface_members.insert(interface.clone(), members);
    }
    for row in &policy.result_records {
        let members = idl
            .interface_members(&row.interface)
            .map_err(|_| unknown(&row.interface))?;
        for member in &members {
            reachable.insert(member.key(), member.clone());
        }
        ordered_result_record_members.insert(row.interface.clone(), members);
    }
    for row in &policy.flattened_interfaces {
        let members = idl
            .interface_members(&row.interface)
            .map_err(|_| unknown(&row.interface))?;
        for member in &members {
            reachable.insert(member.key(), member.clone());
        }
        ordered_flattened_members.insert(row.interface.clone(), members);
    }
    for dictionary in &policy.dictionaries {
        let members = idl
            .dictionary_members(dictionary)
            .map_err(|_| unknown(dictionary))?;
        for member in &members {
            reachable.insert(member.key(), member.clone());
        }
        ordered_dictionary_members.insert(dictionary.clone(), members);
    }
    for namespace in &policy.namespaces {
        let members = idl
            .namespace_members(namespace)
            .map_err(|_| unknown(namespace))?;
        for member in &members {
            reachable.insert(member.key(), member.clone());
        }
        ordered_namespace_members.insert(namespace.clone(), members);
    }
    for enum_name in &policy.enums {
        let members = idl
            .enum_members(enum_name)
            .map_err(|_| unknown(enum_name))?;
        for member in &members {
            reachable.insert(member.key(), member.clone());
        }
        ordered_enum_members.insert(enum_name.clone(), members);
    }
    validate_flattened_interfaces(&policy, &ordered_flattened_members)?;
    let host_owned_device = validate_host_owned_device_policy(&policy, &ordered_interface_members)?;

    if !policy
        .interfaces
        .iter()
        .any(|name| name == &policy.singleton_interface)
    {
        return Err(ApiPolicyError::Invalid {
            entry: policy.singleton_interface.clone(),
            message: "singleton_interface must be a selected API interface".to_owned(),
        });
    }

    let mut ledger = Vec::new();
    for row in &policy.generate {
        if !reachable.contains_key(&row.member) {
            return Err(unknown(&row.member));
        }
        ledger.push(LedgerRow {
            key: format!("generate:{}", row.member),
            consumed: false,
        });
    }
    for row in &policy.deviations {
        if !(reachable.contains_key(&row.member)
            || host_owned_device && row.member == GPU_DEVICE_CONSTRUCTOR
            || synthetic_typed_anchor(row).is_some())
        {
            return Err(unknown(&row.member));
        }
        ledger.push(LedgerRow {
            key: format!("deviation:{}", row.member),
            consumed: false,
        });
    }
    for row in &policy.exclude {
        if !reachable.contains_key(&row.member) {
            return Err(unknown(&row.member));
        }
        ledger.push(LedgerRow {
            key: format!("exclude:{}", row.member),
            consumed: false,
        });
    }
    if host_owned_device {
        consume(&mut ledger, &format!("deviation:{GPU_DEVICE_CONSTRUCTOR}"));
    }
    for row in policy
        .deviations
        .iter()
        .filter(|row| synthetic_typed_anchor(row).is_some())
    {
        validate_synthetic_typed_deviation(row)?;
        consume(&mut ledger, &format!("deviation:{}", row.member));
    }

    let mut classified = BTreeMap::new();
    for (key, member) in &reachable {
        if let Some(row) = policy.generate.iter().find(|row| row.member == *key) {
            validate_pattern(member, &row.pattern, false)?;
            consume(&mut ledger, &format!("generate:{key}"));
            classified.insert(
                key.clone(),
                ClassifiedMember {
                    classification: Classification::Generate(row.pattern.clone()),
                },
            );
        } else if let Some(row) = policy.deviations.iter().find(|row| row.member == *key) {
            validate_pattern(member, &row.pattern, true)?;
            validate_deviation(member, row)?;
            consume(&mut ledger, &format!("deviation:{key}"));
            classified.insert(
                key.clone(),
                ClassifiedMember {
                    classification: Classification::Deviation(Box::new(row.clone())),
                },
            );
        } else if policy.exclude.iter().any(|row| row.member == *key) {
            consume(&mut ledger, &format!("exclude:{key}"));
            classified.insert(
                key.clone(),
                ClassifiedMember {
                    classification: Classification::Exclude,
                },
            );
        } else {
            return Err(ApiPolicyError::Unpoliced {
                construct: key.clone(),
            });
        }
    }
    if let Some(dead) = ledger.iter().find(|row| !row.consumed) {
        let entry = dead
            .key
            .split_once(':')
            .map_or(dead.key.as_str(), |(_, entry)| entry);
        return Err(ApiPolicyError::Dead {
            entry: entry.to_owned(),
        });
    }

    validate_common_mirror(mirror, &policy)?;
    let enums = build_enums(mirror, &policy, &ordered_enum_members, &classified)?;
    let result_records = build_result_records(
        mirror,
        &policy,
        &ordered_result_record_members,
        &classified,
        &enums,
    )?;
    let descriptors = build_descriptors(
        mirror,
        &policy,
        &ordered_dictionary_members,
        &classified,
        &enums,
        &result_records,
    )?;
    validate_no_migrated_enum_array_reads(mirror, &enums, &descriptors)?;
    let namespaces = build_namespaces(mirror, &policy, &ordered_namespace_members, &classified)?;
    let mut interfaces = build_interfaces(
        mirror,
        &policy,
        &ordered_interface_members,
        &classified,
        &descriptors,
        &enums,
        &result_records,
    )?;
    compute_instance_needs(&mut interfaces, &policy.singleton_interface);

    let pattern_members = policy.generate.len();
    let override_members = policy.deviations.len();
    let wrapper_constructs = policy.interfaces.len();
    let result_constructs = result_records.len();
    Ok(ApiPlan {
        policy,
        descriptors,
        namespaces,
        enums,
        result_records,
        interfaces,
        pattern_members,
        override_members,
        wrapper_constructs,
        result_constructs,
    })
}

fn synthetic_typed_anchor(row: &ApiDeviationRow) -> Option<&'static str> {
    match (row.member.as_str(), row.pattern.as_str()) {
        (GPU_QUEUE_WRITE_BUFFER_F32, TYPED_WRITE_F32_PATTERN) => Some("GPUQueue.writeBuffer"),
        (GPU_BUFFER_READ_MAPPED_RANGE_F32, TYPED_READ_F32_PATTERN) => {
            Some("GPUBuffer.getMappedRange")
        }
        _ => None,
    }
}

pub(crate) fn typed_pair_api_member(source: &str) -> Option<String> {
    let (object, method) = source.split_once('.')?;
    Some(format!(
        "GPU{}.{}F32",
        naming::pascal(object),
        naming::camel(method)
    ))
}

pub(crate) fn has_synthetic_typed_anchor(policy: &Policy, member: &str) -> bool {
    policy.api.as_ref().is_some_and(|api| {
        api.deviations
            .iter()
            .any(|row| row.member == member && synthetic_typed_anchor(row).is_some())
    })
}

fn validate_synthetic_typed_deviation(row: &ApiDeviationRow) -> Result<(), ApiPolicyError> {
    let controls_are_empty = row.boundary_receiver.is_none()
        && row.drop_arguments.is_empty()
        && !row.nullable_return
        && !row.boolean_result
        && row.begin_function.is_none()
        && row.boundary_arguments.is_empty()
        && row.overload_arguments.is_empty()
        && row.required_arguments.is_empty()
        && row.default_variant.is_none()
        && !row.required_field
        && row.field_default.is_none()
        && row.absent_boundary_values.is_empty()
        && row.boundary_defaults.is_empty()
        && row.record_entry_api.is_none()
        && row.record_entry_boundary.is_none()
        && row.required_limits_api.is_none()
        && row.required_limits_source.is_none()
        && row.required_limits_u32_unspecified.is_none();
    if controls_are_empty {
        Ok(())
    } else {
        Err(ApiPolicyError::Invalid {
            entry: row.member.clone(),
            message: "typed f32 deviations accept only member, pattern, and reason".to_owned(),
        })
    }
}

fn validate_no_migrated_enum_array_reads(
    mirror: &MirrorModel,
    enums: &[EnumPlan],
    descriptors: &[DescriptorPlan],
) -> Result<(), ApiPolicyError> {
    let migrated = enums
        .iter()
        .map(|enum_plan| enum_plan.mirror_name.as_str())
        .collect::<BTreeSet<_>>();
    for function in mirror.functions.values() {
        if function
            .return_type
            .strip_suffix("[]")
            .is_some_and(|element| migrated.contains(element))
        {
            return Err(ApiPolicyError::Invalid {
                entry: format!("mirror.{}.return", function.name),
                message: "migrated enum appears in a C-to-script array-read position".to_owned(),
            });
        }
    }

    let input_array_fields = descriptors
        .iter()
        .flat_map(|descriptor| {
            descriptor
                .fields
                .iter()
                .filter(|field| matches!(field.conversion, DescriptorFieldConversion::EnumArray))
                .map(|field| (descriptor.boundary_name.as_str(), field.name.as_str()))
        })
        .collect::<BTreeSet<_>>();
    for (class, fields) in &mirror.classes {
        for field in fields {
            if field
                .ty
                .strip_suffix("[]")
                .is_some_and(|element| migrated.contains(element))
                && !input_array_fields.contains(&(class.as_str(), field.name.as_str()))
            {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{class}.{}", field.name),
                    message: "migrated enum appears in a C-to-script array-read position"
                        .to_owned(),
                });
            }
        }
    }
    Ok(())
}

fn select_operation_overloads(
    interface: &str,
    members: Vec<IdlMember>,
    policy: &ApiSection,
) -> Result<Vec<IdlMember>, ApiPolicyError> {
    let mut groups: BTreeMap<&str, Vec<&IdlMember>> = BTreeMap::new();
    for member in &members {
        groups.entry(&member.name).or_default().push(member);
    }
    let mut selected = BTreeMap::new();
    for (name, overloads) in groups.iter().filter(|(_, members)| members.len() > 1) {
        let key = format!("{interface}.{name}");
        let row = policy
            .deviations
            .iter()
            .find(|row| row.member == key)
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: key.clone(),
                message: format!(
                    "IDL operation has {} overloads; select one with overload_arguments in a deviation row",
                    overloads.len()
                ),
            })?;
        if row.overload_arguments.is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: key,
                message:
                    "overloaded IDL operation requires a non-empty overload_arguments selector"
                        .to_owned(),
            });
        }
        let matches = overloads
            .iter()
            .filter(|member| operation_argument_names(member) == row.overload_arguments)
            .copied()
            .collect::<Vec<_>>();
        let [chosen] = matches.as_slice() else {
            return Err(ApiPolicyError::Invalid {
                entry: row.member.clone(),
                message: format!(
                    "overload_arguments {:?} selects {} overloads, expected exactly one",
                    row.overload_arguments,
                    matches.len()
                ),
            });
        };
        selected.insert((*name).to_owned(), operation_argument_names(chosen));
    }
    for row in policy.deviations.iter().filter(|row| {
        row.member.starts_with(&format!("{interface}.")) && !row.overload_arguments.is_empty()
    }) {
        let name = row.member.split_once('.').map_or("", |(_, name)| name);
        if groups.get(name).map_or(0, Vec::len) < 2 {
            return Err(ApiPolicyError::Invalid {
                entry: row.member.clone(),
                message: "overload_arguments names an IDL operation without overloads".to_owned(),
            });
        }
    }
    let mut emitted = BTreeSet::new();
    Ok(members
        .into_iter()
        .filter(|member| {
            let Some(chosen) = selected.get(member.name.as_str()) else {
                return true;
            };
            operation_argument_names(member) == *chosen && emitted.insert(member.name.clone())
        })
        .collect())
}

fn operation_argument_names(member: &IdlMember) -> Vec<String> {
    match &member.kind {
        IdlMemberKind::Operation { arguments, .. } => arguments
            .iter()
            .map(|argument| argument.name.clone())
            .collect(),
        _ => Vec::new(),
    }
}

fn unknown(entry: &str) -> ApiPolicyError {
    ApiPolicyError::Unknown {
        entry: entry.to_owned(),
    }
}

fn validate_host_owned_device_policy(
    policy: &ApiSection,
    ordered_interfaces: &BTreeMap<String, Vec<IdlMember>>,
) -> Result<bool, ApiPolicyError> {
    let required = ordered_interfaces
        .get("GPUDevice")
        .is_some_and(|members| members.iter().any(|member| member.name == "queue"));
    let row = policy
        .deviations
        .iter()
        .find(|row| row.member == GPU_DEVICE_CONSTRUCTOR);
    if !required {
        if row.is_some() {
            return Err(unknown(GPU_DEVICE_CONSTRUCTOR));
        }
        return Ok(false);
    }
    let row = row.ok_or_else(|| ApiPolicyError::Unpoliced {
        construct: GPU_DEVICE_CONSTRUCTOR.to_owned(),
    })?;
    if row.pattern != HOST_OWNED_WRAPPER_PATTERN {
        return Err(ApiPolicyError::Invalid {
            entry: GPU_DEVICE_CONSTRUCTOR.to_owned(),
            message: format!(
                "device ownership handoff requires pattern `{HOST_OWNED_WRAPPER_PATTERN}`, found `{}`",
                row.pattern
            ),
        });
    }
    if row.boundary_receiver.is_some()
        || !row.drop_arguments.is_empty()
        || row.nullable_return
        || row.boolean_result
        || row.begin_function.is_some()
        || !row.boundary_arguments.is_empty()
        || !row.overload_arguments.is_empty()
        || !row.required_arguments.is_empty()
        || row.default_variant.is_some()
        || row.required_field
        || row.field_default.is_some()
        || !row.absent_boundary_values.is_empty()
        || !row.boundary_defaults.is_empty()
        || row.record_entry_api.is_some()
        || row.record_entry_boundary.is_some()
        || row.required_limits_api.is_some()
        || row.required_limits_source.is_some()
        || row.required_limits_u32_unspecified.is_some()
    {
        return Err(ApiPolicyError::Invalid {
            entry: GPU_DEVICE_CONSTRUCTOR.to_owned(),
            message: "host-owned-wrapper accepts only member, pattern, and reason".to_owned(),
        });
    }
    Ok(true)
}

fn validate_interface_parents(idl: &IdlModel, policy: &ApiSection) -> Result<(), ApiPolicyError> {
    for row in &policy.interface_parents {
        idl.interface_parent(&row.interface)
            .map_err(|_| unknown(&row.interface))?;
    }

    let mut consumed = BTreeSet::new();
    let selected = policy
        .interfaces
        .iter()
        .map(String::as_str)
        .chain(
            policy
                .result_records
                .iter()
                .map(|row| row.interface.as_str()),
        )
        .chain(
            policy
                .flattened_interfaces
                .iter()
                .map(|row| row.interface.as_str()),
        )
        .collect::<Vec<_>>();
    for interface in selected {
        let parent = idl
            .interface_parent(interface)
            .map_err(|_| unknown(interface))?;
        let Some(parent) = parent else {
            continue;
        };
        let row = policy
            .interface_parents
            .iter()
            .find(|row| row.interface == interface)
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: format!("{interface}.parent"),
                message: format!(
                    "IDL interface inherits `{parent}`; add an [[api.interface_parents]] record with that parent and a reason"
                ),
            })?;
        if row.parent != parent {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{interface}.parent"),
                message: format!(
                    "policy names parent `{}`, but the pinned IDL declares `{parent}`",
                    row.parent
                ),
            });
        }
        consumed.insert(interface);
    }

    if let Some(row) = policy
        .interface_parents
        .iter()
        .find(|row| !consumed.contains(row.interface.as_str()))
    {
        return Err(ApiPolicyError::Dead {
            entry: format!("{}.parent", row.interface),
        });
    }
    Ok(())
}

fn validate_parameter_types(
    function: &crate::api_model::MirrorFunction,
    expected: &[String],
) -> Result<(), ApiPolicyError> {
    let actual = function
        .params
        .iter()
        .map(|parameter| parameter.ty.clone())
        .collect::<Vec<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{}", function.name),
            message: format!("parameter types are {actual:?}, expected {expected:?}"),
        })
    }
}

fn consume(ledger: &mut [LedgerRow], key: &str) {
    if let Some(row) = ledger.iter_mut().find(|row| row.key == key) {
        row.consumed = true;
    }
}

fn check_policy_duplicates(policy: &ApiSection) -> Result<(), ApiPolicyError> {
    for names in [
        &policy.interfaces,
        &policy.dictionaries,
        &policy.namespaces,
        &policy.enums,
    ] {
        let mut seen = BTreeSet::new();
        for name in names {
            if !seen.insert(name) {
                return Err(ApiPolicyError::Duplicate {
                    entry: name.clone(),
                });
            }
        }
    }
    let mut selected_interfaces = BTreeSet::new();
    for interface in &policy.interfaces {
        selected_interfaces.insert(interface.as_str());
    }
    for row in &policy.result_records {
        if !selected_interfaces.insert(row.interface.as_str()) {
            return Err(ApiPolicyError::Duplicate {
                entry: row.interface.clone(),
            });
        }
    }
    for row in &policy.flattened_interfaces {
        if !selected_interfaces.insert(row.interface.as_str()) {
            return Err(ApiPolicyError::Duplicate {
                entry: row.interface.clone(),
            });
        }
    }
    let mut interface_parents = BTreeSet::new();
    for row in &policy.interface_parents {
        if !interface_parents.insert(&row.interface) {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.parent", row.interface),
            });
        }
    }
    let mut namespace_mappings = BTreeSet::new();
    let mut namespace_boundaries = BTreeSet::new();
    for row in &policy.namespace_mappings {
        if !namespace_mappings.insert(&row.namespace) || !namespace_boundaries.insert(&row.boundary)
        {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.namespace_mapping", row.namespace),
            });
        }
    }
    let mut enum_members = BTreeSet::new();
    let mut mirror_members = BTreeSet::new();
    for row in &policy.enum_mappings {
        let key = format!("{}.{}", row.enum_name, row.member);
        if !enum_members.insert(key.clone()) {
            return Err(ApiPolicyError::Duplicate { entry: key });
        }
        if !mirror_members.insert(&row.mirror) {
            return Err(ApiPolicyError::Duplicate {
                entry: row.mirror.clone(),
            });
        }
    }
    let mut dictionary_mappings = BTreeSet::new();
    let mut dictionary_apis = BTreeSet::new();
    let mut dictionary_types = BTreeSet::new();
    let mut dictionary_boundaries = BTreeSet::new();
    for row in &policy.dictionary_mappings {
        if !dictionary_mappings.insert(&row.dictionary)
            || !dictionary_apis.insert(&row.api)
            || !dictionary_types.insert(&row.idl_type)
            || !dictionary_boundaries.insert(&row.boundary)
        {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.dictionary_mapping", row.dictionary),
            });
        }
    }
    let mut dictionary_aliases = BTreeSet::new();
    for row in &policy.dictionary_aliases {
        if !dictionary_aliases.insert(&row.dictionary) || row.dictionary == row.canonical {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.dictionary_alias", row.dictionary),
            });
        }
    }
    let mut dictionary_nestings = BTreeSet::new();
    for row in &policy.dictionary_nestings {
        let key = format!("{}.{}", row.dictionary, row.boundary_field);
        if !dictionary_nestings.insert(key.clone()) {
            return Err(ApiPolicyError::Duplicate { entry: key });
        }
    }
    let mut public_only_dictionaries = BTreeSet::new();
    for row in &policy.public_only_dictionaries {
        if !public_only_dictionaries.insert(&row.dictionary) {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.public_only_dictionary", row.dictionary),
            });
        }
        if dictionary_mappings.contains(&row.dictionary) {
            return Err(ApiPolicyError::Invalid {
                entry: row.dictionary.clone(),
                message: "dictionary cannot have both boundary and public-only mappings".to_owned(),
            });
        }
    }
    let mut default_helper_members = BTreeSet::new();
    let mut default_helper_names = BTreeSet::new();
    for row in &policy.default_helper_renames {
        if !default_helper_members.insert(&row.member) || !default_helper_names.insert(&row.helper)
        {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.default_helper", row.member),
            });
        }
    }
    let mut enum_exclusions = BTreeSet::new();
    for row in &policy.enum_exclusions {
        let key = format!("{}.{}", row.enum_name, row.mirror);
        if !enum_exclusions.insert(key.clone()) {
            return Err(ApiPolicyError::Duplicate { entry: key });
        }
    }
    let mut generated = BTreeSet::new();
    for member in policy
        .generate
        .iter()
        .map(|row| &row.member)
        .chain(policy.deviations.iter().map(|row| &row.member))
    {
        if !generated.insert(member) {
            return Err(ApiPolicyError::Duplicate {
                entry: member.clone(),
            });
        }
    }
    let mut excluded = BTreeSet::new();
    for row in &policy.exclude {
        if !excluded.insert(&row.member) {
            return Err(ApiPolicyError::Duplicate {
                entry: row.member.clone(),
            });
        }
    }
    Ok(())
}

fn check_reasons(policy: &ApiSection) -> Result<(), ApiPolicyError> {
    let mut required = vec![
        ("api.singleton", policy.singleton_reason.as_str()),
        ("api.dispose", policy.manual_dispose_reason.as_str()),
    ];
    if !policy.namespaces.is_empty() {
        required.push(("api.namespace", policy.namespace_reason.as_str()));
    }
    for row in &policy.result_records {
        required.push((row.interface.as_str(), row.reason.as_str()));
    }
    for row in &policy.flattened_interfaces {
        required.push((row.interface.as_str(), row.reason.as_str()));
    }
    for (entry, reason) in required {
        if reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: entry.to_owned(),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.interface_parents {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.parent", row.interface),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.namespace_mappings {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.namespace_mapping", row.namespace),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.dictionary_mappings {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.dictionary_mapping", row.dictionary),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.dictionary_aliases {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.dictionary_alias", row.dictionary),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.dictionary_nestings {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.{}", row.dictionary, row.boundary_field),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.public_only_dictionaries {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.public_only_dictionary", row.dictionary),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.default_helper_renames {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.default_helper", row.member),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.enum_mappings {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.{}", row.enum_name, row.member),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.enum_exclusions {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.{}", row.enum_name, row.mirror),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.deviations {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: row.member.clone(),
                message: "empty reason".to_owned(),
            });
        }
    }
    for row in &policy.exclude {
        if row.reason.trim().is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: row.member.clone(),
                message: "empty reason".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_pattern(
    member: &IdlMember,
    pattern: &str,
    deviation: bool,
) -> Result<(), ApiPolicyError> {
    let valid = match (&member.kind, pattern) {
        (IdlMemberKind::Operation { .. }, "operation") => true,
        (IdlMemberKind::Operation { .. }, "async-request") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "attribute-method") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-field") if !deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-required") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-default") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-optional-descriptor") if deviation => {
            true
        }
        (IdlMemberKind::DictionaryField { .. }, "dictionary-nullable-handle") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-required-handle") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-optional-handle") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-nullable-descriptor") if deviation => {
            true
        }
        (IdlMemberKind::DictionaryField { .. }, "dictionary-record-entries") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-required-limits") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-descriptor-array") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-required-optional-bool")
            if deviation =>
        {
            true
        }
        (IdlMemberKind::DictionaryField { .. }, "dictionary-handle-array") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-enum-array") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-union-descriptor") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "dictionary-boundary-default") if deviation => true,
        (IdlMemberKind::DictionaryField { .. }, "binding-resource") if deviation => true,
        (IdlMemberKind::NamespaceConstant { .. }, "namespace-constant") if !deviation => true,
        (IdlMemberKind::EnumValue, "enum-value") if !deviation => true,
        (IdlMemberKind::Operation { .. }, "mapped-range") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "label-method") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "result-record-field") => true,
        (IdlMemberKind::Operation { .. }, "error-scope-pop") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "device-lost-poll") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "uncaptured-error-drain") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "feature-probe") if deviation => true,
        (IdlMemberKind::Attribute { .. }, "result-record-fill") if deviation => true,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!("pattern `{pattern}` does not match the IDL member kind"),
        })
    }
}

fn validate_deviation(member: &IdlMember, row: &ApiDeviationRow) -> Result<(), ApiPolicyError> {
    let arguments = match &member.kind {
        IdlMemberKind::Operation { arguments, .. } => arguments,
        IdlMemberKind::DictionaryField { .. } => {
            if !row.drop_arguments.is_empty()
                || !row.boundary_arguments.is_empty()
                || !row.overload_arguments.is_empty()
                || !row.required_arguments.is_empty()
                || row.default_variant.is_some()
                || row.nullable_return
                || row.boolean_result
                || row.begin_function.is_some()
            {
                return Err(ApiPolicyError::Invalid {
                    entry: row.member.clone(),
                    message: "dictionary deviations cannot reshape operation results or arguments"
                        .to_owned(),
                });
            }
            if row.pattern != "dictionary-record-entries"
                && (row.record_entry_api.is_some() || row.record_entry_boundary.is_some())
            {
                return Err(ApiPolicyError::Invalid {
                    entry: row.member.clone(),
                    message: "record entry controls require dictionary-record-entries".to_owned(),
                });
            }
            let has_required_limits_controls = row.required_limits_api.is_some()
                || row.required_limits_source.is_some()
                || row.required_limits_u32_unspecified.is_some();
            if row.pattern != "dictionary-required-limits" && has_required_limits_controls {
                return Err(ApiPolicyError::Invalid {
                    entry: row.member.clone(),
                    message: "required-limits controls require dictionary-required-limits"
                        .to_owned(),
                });
            }
            match row.pattern.as_str() {
                "dictionary-required" if row.required_field => {}
                "dictionary-default" if row.field_default.is_some() => {}
                "dictionary-optional-descriptor" if !row.absent_boundary_values.is_empty() => {}
                "dictionary-nullable-handle" if row.field_default.as_deref() == Some("null") => {}
                "dictionary-required-handle" => {}
                "dictionary-optional-handle" if row.field_default.as_deref() == Some("null") => {}
                "dictionary-nullable-descriptor"
                    if row.field_default.as_deref() == Some("null") => {}
                "dictionary-record-entries"
                    if row.field_default.as_deref() == Some("[]")
                        && row.record_entry_api.is_some()
                        && row.record_entry_boundary.is_some() => {}
                "dictionary-required-limits"
                    if row.required_limits_api.is_some()
                        && row.required_limits_source.is_some()
                        && row.required_limits_u32_unspecified.is_some() => {}
                "dictionary-descriptor-array" => {}
                "dictionary-required-optional-bool" if row.required_field => {}
                "dictionary-handle-array" => {}
                "dictionary-enum-array" => {}
                "dictionary-union-descriptor" if row.field_default.is_some() => {}
                "dictionary-boundary-default" if !row.boundary_defaults.is_empty() => {}
                "binding-resource" => {}
                _ => {
                    return Err(ApiPolicyError::Invalid {
                        entry: row.member.clone(),
                        message: format!(
                            "dictionary deviation fields do not match pattern `{}`",
                            row.pattern
                        ),
                    })
                }
            }
            return Ok(());
        }
        _ => {
            if !row.drop_arguments.is_empty()
                || !row.boundary_arguments.is_empty()
                || !row.overload_arguments.is_empty()
                || !row.required_arguments.is_empty()
                || row.default_variant.is_some()
            {
                return Err(ApiPolicyError::Invalid {
                    entry: row.member.clone(),
                    message: "only operations may reshape IDL arguments".to_owned(),
                });
            }
            if row.required_field
                || row.field_default.is_some()
                || !row.absent_boundary_values.is_empty()
                || !row.boundary_defaults.is_empty()
                || row.record_entry_api.is_some()
                || row.record_entry_boundary.is_some()
                || row.required_limits_api.is_some()
                || row.required_limits_source.is_some()
                || row.required_limits_u32_unspecified.is_some()
                || row.begin_function.is_some()
            {
                return Err(ApiPolicyError::Invalid {
                    entry: row.member.clone(),
                    message: "dictionary deviation controls apply only to dictionary fields"
                        .to_owned(),
                });
            }
            return Ok(());
        }
    };
    for dropped in &row.drop_arguments {
        if !arguments.iter().any(|argument| argument.name == *dropped) {
            return Err(unknown(&format!("{}.argument.{dropped}", row.member)));
        }
    }
    if !row.overload_arguments.is_empty()
        && operation_argument_names(member) != row.overload_arguments
    {
        return Err(ApiPolicyError::Invalid {
            entry: format!("{}.overload_arguments", row.member),
            message: "selected overload arguments moved after overload selection".to_owned(),
        });
    }
    for required in &row.required_arguments {
        let argument = arguments
            .iter()
            .find(|argument| argument.name == *required)
            .ok_or_else(|| unknown(&format!("{}.argument.{required}", row.member)))?;
        if !argument.optional || argument.default.is_none() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.argument.{required}", row.member),
                message: "required_arguments must name an optional IDL argument with a default"
                    .to_owned(),
            });
        }
    }
    if row.boolean_result && row.nullable_return {
        return Err(ApiPolicyError::Invalid {
            entry: row.member.clone(),
            message: "boolean_result and nullable_return are mutually exclusive".to_owned(),
        });
    }
    if row.begin_function.is_some() && row.pattern != "async-request" {
        return Err(ApiPolicyError::Invalid {
            entry: row.member.clone(),
            message: "begin_function applies only to async-request operations".to_owned(),
        });
    }
    if row.required_field {
        return Err(ApiPolicyError::Invalid {
            entry: row.member.clone(),
            message: "required_field applies only to dictionary fields".to_owned(),
        });
    }
    if row.field_default.is_some()
        || !row.absent_boundary_values.is_empty()
        || !row.boundary_defaults.is_empty()
        || row.record_entry_api.is_some()
        || row.record_entry_boundary.is_some()
        || row.required_limits_api.is_some()
        || row.required_limits_source.is_some()
        || row.required_limits_u32_unspecified.is_some()
    {
        return Err(ApiPolicyError::Invalid {
            entry: row.member.clone(),
            message: "dictionary deviation controls apply only to dictionary fields".to_owned(),
        });
    }
    Ok(())
}

fn validate_common_mirror(mirror: &MirrorModel, policy: &ApiSection) -> Result<(), ApiPolicyError> {
    for function in [
        "subscript_typegpu_create_instance",
        "subscript_typegpu_instance_process_events",
        "subscript_typegpu_future_status",
        "subscript_typegpu_future_drop",
    ] {
        if !mirror.functions.contains_key(function) {
            return Err(unknown(&format!("mirror.{function}")));
        }
    }
    let singleton_handle = format!("SubscriptTypegpu{}", policy.singleton_boundary);
    if !mirror.handles.contains(&singleton_handle) {
        return Err(unknown(&format!("mirror.{singleton_handle}")));
    }
    Ok(())
}

fn build_descriptors(
    mirror: &MirrorModel,
    policy: &ApiSection,
    ordered: &BTreeMap<String, Vec<IdlMember>>,
    classified: &BTreeMap<String, ClassifiedMember>,
    enums: &[EnumPlan],
    result_records: &[ResultRecordPlan],
) -> Result<Vec<DescriptorPlan>, ApiPolicyError> {
    for row in &policy.dictionary_mappings {
        if !policy.dictionaries.contains(&row.dictionary) {
            return Err(unknown(&format!("{}.dictionary_mapping", row.dictionary)));
        }
        if !mirror.classes.contains_key(&row.boundary) {
            return Err(unknown(&format!("mirror.{}", row.boundary)));
        }
    }
    for row in &policy.dictionary_aliases {
        if !policy.dictionaries.contains(&row.dictionary) {
            return Err(unknown(&format!("{}.dictionary_alias", row.dictionary)));
        }
        if !policy.dictionaries.contains(&row.canonical) {
            return Err(unknown(&format!("{}.dictionary_alias", row.canonical)));
        }
        if policy
            .dictionary_aliases
            .iter()
            .any(|candidate| candidate.dictionary == row.canonical)
        {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.dictionary_alias", row.dictionary),
                message: "dictionary aliases cannot target another alias".to_owned(),
            });
        }
    }
    for row in &policy.dictionary_nestings {
        if !policy.dictionaries.contains(&row.dictionary) {
            return Err(unknown(&format!(
                "{}.{}",
                row.dictionary, row.boundary_field
            )));
        }
    }
    for row in &policy.public_only_dictionaries {
        if !policy.dictionaries.contains(&row.dictionary) {
            return Err(unknown(&format!(
                "{}.public_only_dictionary",
                row.dictionary
            )));
        }
    }
    let identities = policy
        .dictionaries
        .iter()
        .filter(|idl_name| {
            !policy
                .dictionary_aliases
                .iter()
                .any(|row| row.dictionary == **idl_name)
        })
        .map(|idl_name| {
            let idl_aliases = policy
                .dictionary_aliases
                .iter()
                .filter(|row| row.canonical == *idl_name)
                .map(|row| row.dictionary.clone())
                .collect::<Vec<_>>();
            if policy
                .public_only_dictionaries
                .iter()
                .any(|row| row.dictionary == *idl_name)
            {
                DescriptorPlan {
                    idl_name: idl_name.clone(),
                    idl_type: idl_name.clone(),
                    idl_aliases,
                    name: idl_name.clone(),
                    boundary_name: String::new(),
                    public_only: true,
                    fields: Vec::new(),
                    boundary_fields: Vec::new(),
                    boundary_defaults: BTreeMap::new(),
                    nested_boundaries: Vec::new(),
                }
            } else if let Some(row) = policy
                .dictionary_mappings
                .iter()
                .find(|row| row.dictionary == *idl_name)
            {
                DescriptorPlan {
                    idl_name: idl_name.clone(),
                    idl_type: row.idl_type.clone(),
                    idl_aliases,
                    name: row.api.clone(),
                    boundary_name: row.boundary.clone(),
                    public_only: false,
                    fields: Vec::new(),
                    boundary_fields: Vec::new(),
                    boundary_defaults: BTreeMap::new(),
                    nested_boundaries: Vec::new(),
                }
            } else {
                DescriptorPlan {
                    idl_name: idl_name.clone(),
                    idl_type: idl_name.clone(),
                    idl_aliases,
                    name: idl_name.clone(),
                    boundary_name: format!(
                        "SubscriptTypegpu{}",
                        idl_name.trim_start_matches("GPU")
                    ),
                    public_only: false,
                    fields: Vec::new(),
                    boundary_fields: Vec::new(),
                    boundary_defaults: BTreeMap::new(),
                    nested_boundaries: Vec::new(),
                }
            }
        })
        .collect::<Vec<_>>();
    let mut plans = Vec::new();
    for identity in &identities {
        let name = &identity.idl_name;
        let boundary_name = &identity.boundary_name;
        let boundary_fields = if identity.public_only {
            let row = policy
                .public_only_dictionaries
                .iter()
                .find(|row| row.dictionary == *name)
                .ok_or_else(|| unknown(&format!("{name}.public_only_dictionary")))?;
            let mut seen = BTreeSet::new();
            row.field_types
                .iter()
                .map(|entry| {
                    let (field, ty) =
                        entry
                            .split_once('=')
                            .ok_or_else(|| ApiPolicyError::Invalid {
                                entry: format!("{name}.public_only_dictionary"),
                                message: format!("malformed field type join `{entry}`"),
                            })?;
                    if !seen.insert(field) {
                        return Err(ApiPolicyError::Duplicate {
                            entry: format!("{name}.{field}"),
                        });
                    }
                    Ok(MirrorField {
                        name: field.to_owned(),
                        ty: ty.to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            mirror
                .classes
                .get(boundary_name)
                .ok_or_else(|| unknown(&format!("mirror.{boundary_name}")))?
                .clone()
        };
        let nested_boundaries =
            nested_boundaries_for(mirror, policy, name, boundary_name, &boundary_fields)?;
        let members = ordered.get(name).ok_or_else(|| unknown(name))?;
        let mut fields = Vec::new();
        let mut boundary_defaults = BTreeMap::new();
        for member in members {
            let classified_member = classified
                .get(&member.key())
                .ok_or_else(|| unknown(&member.key()))?;
            if matches!(classified_member.classification, Classification::Exclude) {
                continue;
            }
            let IdlMemberKind::DictionaryField {
                ty,
                required,
                default,
            } = &member.kind
            else {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "selected dictionary contains a non-field member".to_owned(),
                });
            };
            if let Classification::Deviation(row) = &classified_member.classification {
                if row.pattern == "binding-resource" {
                    if name != "GPUBindGroupEntry"
                        || !matches!(ty, IdlType::Named { name, .. } if name == "GPUBindingResource")
                    {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: "binding-resource requires GPUBindGroupEntry.resource"
                                .to_owned(),
                        });
                    }
                    let resource_fields = [
                        (
                            "buffer",
                            "GPUBuffer | null",
                            "null",
                            "SubscriptTypegpuBuffer",
                            "buffer",
                        ),
                        ("offset", "u64", "0", "", ""),
                        ("size", "u64", "0", "", ""),
                        (
                            "sampler",
                            "GPUSampler | null",
                            "null",
                            "SubscriptTypegpuSampler",
                            "sampler",
                        ),
                        (
                            "textureView",
                            "GPUTextureView | null",
                            "null",
                            "SubscriptTypegpuTextureView",
                            "textureView",
                        ),
                    ];
                    for (field_name, api_type, default_value, handle, raw_field) in resource_fields
                    {
                        let boundary_field = boundary_fields
                            .iter()
                            .find(|field| field.name == field_name)
                            .ok_or_else(|| {
                                unknown(&format!("mirror.{boundary_name}.{field_name}"))
                            })?;
                        let conversion = if handle.is_empty() {
                            if boundary_field.ty != api_type {
                                return Err(ApiPolicyError::Invalid {
                                    entry: member.key(),
                                    message: format!(
                                        "flattened `{field_name}` declares `{}`, expected `{api_type}`",
                                        boundary_field.ty
                                    ),
                                });
                            }
                            DescriptorFieldConversion::Direct
                        } else {
                            if boundary_field.ty != format!("{handle} | null") {
                                return Err(ApiPolicyError::Invalid {
                                    entry: member.key(),
                                    message: format!(
                                        "flattened `{field_name}` declares `{}`, expected `{handle} | null`",
                                        boundary_field.ty
                                    ),
                                });
                            }
                            DescriptorFieldConversion::NullableHandle {
                                boundary_name: handle.to_owned(),
                                api_name: api_type.trim_end_matches(" | null").to_owned(),
                                raw_field: raw_field.to_owned(),
                            }
                        };
                        fields.push(DescriptorFieldPlan {
                            name: field_name.to_owned(),
                            ty: api_type.to_owned(),
                            required: false,
                            default: Some(default_value.to_owned()),
                            conversion,
                        });
                    }
                    continue;
                }
            }
            let mirror_field = if let Some(nested) = nested_boundaries
                .iter()
                .find(|nested| nested.members.contains(&member.name))
            {
                mirror
                    .classes
                    .get(&nested.boundary_name)
                    .and_then(|fields| fields.iter().find(|field| field.name == member.name))
                    .ok_or_else(|| {
                        unknown(&format!("mirror.{}.{}", nested.boundary_name, member.name))
                    })?
            } else {
                boundary_fields
                    .iter()
                    .find(|field| field.name == member.name)
                    .ok_or_else(|| unknown(&format!("mirror.{boundary_name}.{}", member.name)))?
            };
            let field_pattern = match &classified_member.classification {
                Classification::Generate(pattern) => pattern.as_str(),
                Classification::Deviation(row) => row.pattern.as_str(),
                Classification::Exclude => unreachable!("excluded fields returned above"),
            };
            let (api_type, conversion) = descriptor_field_shape(
                mirror,
                policy,
                member,
                ty,
                mirror_field,
                &identities,
                enums,
                field_pattern,
                match &classified_member.classification {
                    Classification::Deviation(row) => Some(row),
                    _ => None,
                },
                result_records,
            )?;
            let (required, field_default) = match &classified_member.classification {
                Classification::Deviation(row) if row.pattern == "dictionary-required" => {
                    (true, None)
                }
                Classification::Deviation(row)
                    if row.pattern == "dictionary-required-optional-bool" =>
                {
                    (true, None)
                }
                Classification::Deviation(row) if row.pattern == "dictionary-default" => {
                    (false, row.field_default.clone())
                }
                Classification::Deviation(row) if row.pattern == "dictionary-union-descriptor" => {
                    (false, row.field_default.clone())
                }
                Classification::Deviation(row)
                    if row.pattern == "dictionary-optional-descriptor" =>
                {
                    (false, Some("null".to_owned()))
                }
                Classification::Deviation(row)
                    if row.pattern == "dictionary-nullable-handle"
                        || row.pattern == "dictionary-optional-handle"
                        || row.pattern == "dictionary-nullable-descriptor"
                        || row.pattern == "dictionary-record-entries" =>
                {
                    (false, row.field_default.clone())
                }
                Classification::Deviation(row) if row.pattern == "dictionary-required-limits" => {
                    (*required, default.clone())
                }
                Classification::Deviation(row)
                    if row.pattern == "dictionary-handle-array"
                        || row.pattern == "dictionary-enum-array"
                        || row.pattern == "dictionary-boundary-default"
                        || row.pattern == "dictionary-descriptor-array" =>
                {
                    (*required, default.clone())
                }
                Classification::Deviation(row) if row.pattern == "dictionary-required-handle" => {
                    (true, None)
                }
                Classification::Deviation(row) => {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!("unsupported dictionary pattern `{}`", row.pattern),
                    })
                }
                _ => (*required, default.clone()),
            };
            if let Classification::Deviation(row) = &classified_member.classification {
                for entry in &row.boundary_defaults {
                    let (field, value) =
                        entry
                            .split_once('=')
                            .ok_or_else(|| ApiPolicyError::Invalid {
                                entry: member.key(),
                                message: format!("malformed boundary default `{entry}`"),
                            })?;
                    if boundary_defaults
                        .insert(field.to_owned(), value.to_owned())
                        .is_some()
                    {
                        return Err(ApiPolicyError::Duplicate {
                            entry: format!("mirror.{boundary_name}.{field}"),
                        });
                    }
                }
            }
            fields.push(DescriptorFieldPlan {
                name: member.name.clone(),
                ty: api_type,
                required,
                default: if required { None } else { field_default },
                conversion,
            });
        }
        for field in &boundary_fields {
            if !fields.iter().any(|candidate| candidate.name == field.name)
                && !boundary_defaults.contains_key(&field.name)
                && !nested_boundaries
                    .iter()
                    .any(|nested| nested.field_name == field.name)
            {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{boundary_name}.{}", field.name),
                    message: "facade descriptor field has no generated IDL dictionary member"
                        .to_owned(),
                });
            }
        }
        for field in boundary_defaults.keys() {
            if !boundary_fields
                .iter()
                .any(|candidate| &candidate.name == field)
            {
                return Err(unknown(&format!("mirror.{boundary_name}.{field}")));
            }
        }
        plans.push(DescriptorPlan {
            idl_name: name.clone(),
            idl_type: identity.idl_type.clone(),
            idl_aliases: identity.idl_aliases.clone(),
            name: identity.name.clone(),
            boundary_name: boundary_name.clone(),
            public_only: identity.public_only,
            fields,
            boundary_fields,
            boundary_defaults,
            nested_boundaries,
        });
    }
    validate_dictionary_aliases(policy, ordered, classified)?;
    Ok(plans)
}

fn nested_boundaries_for(
    mirror: &MirrorModel,
    policy: &ApiSection,
    dictionary: &str,
    boundary_name: &str,
    boundary_fields: &[MirrorField],
) -> Result<Vec<NestedBoundaryPlan>, ApiPolicyError> {
    let mut plans = Vec::new();
    let mut claimed_members = BTreeSet::new();
    let mut claimed_fields = BTreeSet::new();
    for row in policy
        .dictionary_nestings
        .iter()
        .filter(|row| row.dictionary == dictionary)
    {
        if !claimed_fields.insert(&row.boundary_field) {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{dictionary}.{}", row.boundary_field),
            });
        }
        if row.members.is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{dictionary}.{}", row.boundary_field),
                message: "dictionary nesting must claim at least one IDL member".to_owned(),
            });
        }
        for member in &row.members {
            if !claimed_members.insert(member) {
                return Err(ApiPolicyError::Duplicate {
                    entry: format!("{dictionary}.{member}"),
                });
            }
        }
        let boundary_field = boundary_fields
            .iter()
            .find(|field| field.name == row.boundary_field)
            .ok_or_else(|| unknown(&format!("mirror.{boundary_name}.{}", row.boundary_field)))?;
        if boundary_field.ty != row.boundary {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{dictionary}.{}", row.boundary_field),
                message: format!(
                    "dictionary nesting names `{}`, but mirror field declares `{}`",
                    row.boundary, boundary_field.ty
                ),
            });
        }
        let nested_fields = mirror
            .classes
            .get(&row.boundary)
            .ok_or_else(|| unknown(&format!("mirror.{}", row.boundary)))?;
        let actual = nested_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>();
        let expected = row.members.iter().map(String::as_str).collect::<Vec<_>>();
        if actual != expected {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{dictionary}.{}", row.boundary_field),
                message: format!(
                    "nested mirror fields are {actual:?}, expected policy members {expected:?}"
                ),
            });
        }
        plans.push(NestedBoundaryPlan {
            field_name: row.boundary_field.clone(),
            boundary_name: row.boundary.clone(),
            members: row.members.clone(),
        });
    }
    Ok(plans)
}

fn validate_dictionary_aliases(
    policy: &ApiSection,
    ordered: &BTreeMap<String, Vec<IdlMember>>,
    classified: &BTreeMap<String, ClassifiedMember>,
) -> Result<(), ApiPolicyError> {
    for row in &policy.dictionary_aliases {
        let canonical = ordered
            .get(&row.canonical)
            .ok_or_else(|| unknown(&row.canonical))?;
        let alias = ordered
            .get(&row.dictionary)
            .ok_or_else(|| unknown(&row.dictionary))?;
        if canonical.len() != alias.len() {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.dictionary_alias", row.dictionary),
                message: format!(
                    "alias has {} fields, but canonical `{}` has {}",
                    alias.len(),
                    row.canonical,
                    canonical.len()
                ),
            });
        }
        for (canonical_member, alias_member) in canonical.iter().zip(alias) {
            if canonical_member.name != alias_member.name
                || canonical_member.kind != alias_member.kind
            {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{}.dictionary_alias", row.dictionary),
                    message: format!(
                        "alias field `{}` does not exactly match canonical `{}.{}`",
                        alias_member.name, row.canonical, canonical_member.name
                    ),
                });
            }
            let canonical_class = classified
                .get(&canonical_member.key())
                .ok_or_else(|| unknown(&canonical_member.key()))?;
            let alias_class = classified
                .get(&alias_member.key())
                .ok_or_else(|| unknown(&alias_member.key()))?;
            if classification_shape(canonical_class) != classification_shape(alias_class) {
                return Err(ApiPolicyError::Invalid {
                    entry: alias_member.key(),
                    message: format!(
                        "dictionary alias policy does not match canonical `{}.{}`",
                        row.canonical, canonical_member.name
                    ),
                });
            }
        }
    }
    Ok(())
}

fn classification_shape(classified: &ClassifiedMember) -> String {
    match &classified.classification {
        Classification::Generate(pattern) => format!("generate:{pattern}"),
        Classification::Exclude => "exclude".to_owned(),
        Classification::Deviation(row) => format!(
            "deviation:{}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
            row.pattern,
            row.drop_arguments,
            row.nullable_return,
            row.boolean_result,
            row.begin_function,
            row.boundary_arguments,
            row.default_variant,
            row.required_field,
            row.field_default,
            row.absent_boundary_values,
            row.boundary_defaults,
            row.record_entry_api,
            row.record_entry_boundary,
            row.required_limits_api,
            row.required_limits_source,
            row.required_limits_u32_unspecified,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn descriptor_field_shape(
    mirror: &MirrorModel,
    policy: &ApiSection,
    member: &IdlMember,
    idl_type: &IdlType,
    mirror_field: &MirrorField,
    descriptors: &[DescriptorPlan],
    enums: &[EnumPlan],
    pattern: &str,
    deviation: Option<&ApiDeviationRow>,
    result_records: &[ResultRecordPlan],
) -> Result<(String, DescriptorFieldConversion), ApiPolicyError> {
    let mirror_nullable = mirror_field.ty.ends_with(" | null");
    let mirror_type = mirror_field.ty.trim_end_matches(" | null");
    if let Some(row) = deviation.filter(|row| row.pattern == "dictionary-required-limits") {
        if !matches!(
            idl_type,
            IdlType::Record { key, value }
                if key == "DOMString" && matches!(value.as_ref(), IdlType::Other)
        ) {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: "dictionary-required-limits requires record<DOMString, (GPUSize64 or undefined)>"
                    .to_owned(),
            });
        }
        if !mirror_nullable {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-required-limits requires a nullable mirror aggregate, found `{}`",
                    mirror_field.ty
                ),
            });
        }
        let source_name = row.required_limits_source.as_deref().unwrap_or_default();
        let source = result_records
            .iter()
            .find(|record| record.name == source_name)
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-required-limits source `{source_name}` is not a selected result record"
                ),
            })?;
        if source.boundary_name != mirror_type {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "required-limits source `{source_name}` joins `{}`, but the mirror field declares `{}`",
                    source.boundary_name, mirror_field.ty
                ),
            });
        }
        let fields = source
            .fields
            .iter()
            .map(|field| {
                if !matches!(field.conversion, ResultRecordFieldConversion::Direct)
                    || !matches!(field.ty.as_str(), "u32" | "u64")
                {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{source_name}.{}", field.name),
                        message: format!(
                            "required-limits fields must be direct u32 or u64 scalars, found `{}`",
                            field.ty
                        ),
                    });
                }
                Ok(RequiredLimitFieldPlan {
                    name: field.name.clone(),
                    ty: field.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if fields.is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: source_name.to_owned(),
                message: "required-limits source has no public fields".to_owned(),
            });
        }
        let u32_unspecified = row.required_limits_u32_unspecified.unwrap_or_default();
        if u32_unspecified != u32::MAX {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "required-limits u32 unspecified sentinel is `{u32_unspecified}`, expected `{}`",
                    u32::MAX
                ),
            });
        }
        let api_name = row.required_limits_api.as_deref().unwrap_or_default();
        if api_name.is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: "required-limits public descriptor name is empty".to_owned(),
            });
        }
        return Ok((
            api_name.to_owned(),
            DescriptorFieldConversion::RequiredLimits {
                api_name: api_name.to_owned(),
                boundary_name: source.boundary_name.clone(),
                fields,
                u32_unspecified,
            },
        ));
    }
    if deviation.is_some_and(|row| row.pattern == "dictionary-union-descriptor") {
        if mirror_nullable {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-union-descriptor requires a non-null mirror aggregate, found `{}`",
                    mirror_field.ty
                ),
            });
        }
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.boundary_name == mirror_type)
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "mirror aggregate `{mirror_type}` has no selected public dictionary branch"
                ),
            })?;
        if !matches!(idl_type, IdlType::Other)
            && !matches!(
                idl_type,
                IdlType::Named {
                    name,
                    nullable: false
                } if descriptor.idl_type == *name
            )
        {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-union-descriptor requires the mapped IDL union type `{}`, found `{idl_type:?}`",
                    descriptor.idl_type
                ),
            });
        }
        return Ok((
            descriptor.name.clone(),
            DescriptorFieldConversion::Descriptor(descriptor.boundary_name.clone()),
        ));
    }
    if deviation.is_some_and(|row| row.pattern == "dictionary-required-handle") {
        if !matches!(
            &member.kind,
            IdlMemberKind::DictionaryField {
                ty: IdlType::Other,
                required: true,
                ..
            }
        ) {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message:
                    "dictionary-required-handle requires a required non-modelled IDL type (union)"
                        .to_owned(),
            });
        }
        if !mirror.handles.contains(mirror_type) {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-required-handle requires a mirror handle, found `{}`",
                    mirror_field.ty
                ),
            });
        }
        let api = policy
            .interfaces
            .iter()
            .find(|interface| {
                format!("SubscriptTypegpu{}", interface.trim_start_matches("GPU")) == mirror_type
            })
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!("mirror handle `{mirror_type}` has no selected public wrapper"),
            })?;
        return Ok((
            api.clone(),
            DescriptorFieldConversion::Handle(raw_field_for_api(api, policy)),
        ));
    }
    if deviation.is_some_and(|row| row.pattern == "dictionary-optional-handle") {
        let IdlType::Named {
            name,
            nullable: false,
        } = idl_type
        else {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: "dictionary-optional-handle requires a non-null named IDL handle type"
                    .to_owned(),
            });
        };
        if !matches!(
            &member.kind,
            IdlMemberKind::DictionaryField {
                required: false,
                default: None,
                ..
            }
        ) || !mirror_nullable
            || !mirror.handles.contains(mirror_type)
        {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-optional-handle requires an optional IDL field joined to a nullable mirror handle, found `{}`",
                    mirror_field.ty
                ),
            });
        }
        let api = policy
            .interfaces
            .iter()
            .find(|interface| {
                format!("SubscriptTypegpu{}", interface.trim_start_matches("GPU")) == mirror_type
            })
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!("mirror handle `{mirror_type}` has no selected public wrapper"),
            })?;
        if api != name {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "IDL handle `{name}` joins public wrapper `{api}`, but the names differ"
                ),
            });
        }
        return Ok((
            format!("{api} | null"),
            DescriptorFieldConversion::NullableHandle {
                boundary_name: mirror_type.to_owned(),
                api_name: api.clone(),
                raw_field: raw_field_for_api(api, policy),
            },
        ));
    }
    if deviation.is_some_and(|row| row.pattern == "dictionary-nullable-handle") {
        if !matches!(idl_type, IdlType::Other) {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: "dictionary-nullable-handle requires a non-modelled IDL type (union)"
                    .to_owned(),
            });
        }
        if !mirror_nullable || !mirror.handles.contains(mirror_type) {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-nullable-handle requires a nullable mirror handle, found `{}`",
                    mirror_field.ty
                ),
            });
        }
        let api = policy
            .interfaces
            .iter()
            .find(|interface| {
                format!("SubscriptTypegpu{}", interface.trim_start_matches("GPU")) == mirror_type
            })
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!("mirror handle `{mirror_type}` has no selected public wrapper"),
            })?;
        return Ok((
            format!("{api} | null"),
            DescriptorFieldConversion::NullableHandle {
                boundary_name: mirror_type.to_owned(),
                api_name: api.clone(),
                raw_field: raw_field_for_api(api, policy),
            },
        ));
    }
    if deviation.is_some_and(|row| row.pattern == "dictionary-required-optional-bool") {
        if !matches!(idl_type, IdlType::Boolean)
            || mirror_field.ty != "SubscriptTypegpuOptionalBool"
        {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "dictionary-required-optional-bool requires an IDL boolean joined to SubscriptTypegpuOptionalBool, found `{}`",
                    mirror_field.ty
                ),
            });
        }
        let optional_bool = mirror
            .enums
            .get("SubscriptTypegpuOptionalBool")
            .ok_or_else(|| unknown("mirror.SubscriptTypegpuOptionalBool"))?;
        for constant in [
            "SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_FALSE",
            "SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_TRUE",
        ] {
            if !optional_bool.members.contains_key(constant) {
                return Err(unknown(&format!(
                    "mirror.SubscriptTypegpuOptionalBool.{constant}"
                )));
            }
        }
        return Ok((
            "boolean".to_owned(),
            DescriptorFieldConversion::RequiredOptionalBool,
        ));
    }
    match idl_type {
        IdlType::Named { name, .. } => {
            if let Some(enum_plan) = enums.iter().find(|plan| plan.name == *name) {
                if mirror_nullable {
                    return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                }
                if !matches!(pattern, "dictionary-field" | "dictionary-required") {
                    return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                }
                if mirror_type != enum_plan.mirror_name {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "IDL enum `{name}` joins `{}`, but the mirror field declares `{}`",
                            enum_plan.mirror_name, mirror_field.ty
                        ),
                    });
                }
                let absence_capable = matches!(
                    &member.kind,
                    IdlMemberKind::DictionaryField {
                        required: false,
                        default: None,
                        ..
                    }
                );
                if absence_capable {
                    if pattern != "dictionary-field" {
                        return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                    }
                    let undefined_member = facade_enum_member(&enum_plan.mirror_name, "undefined");
                    let mirror_enum = mirror
                        .enums
                        .get(&enum_plan.mirror_name)
                        .ok_or_else(|| unknown(&format!("mirror.{}", enum_plan.mirror_name)))?;
                    if !mirror_enum.members.contains_key(&undefined_member) {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "absence-capable IDL enum `{name}` requires mirror constant `{undefined_member}`"
                            ),
                        });
                    }
                    if !policy
                        .enum_exclusions
                        .iter()
                        .any(|row| row.enum_name == *name && row.mirror == undefined_member)
                    {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "absence-capable IDL enum `{name}` requires `{undefined_member}` to be a boundary-only enum exclusion"
                            ),
                        });
                    }
                    return Ok((
                        name.clone(),
                        DescriptorFieldConversion::OptionalEnum {
                            public_name: enum_plan.name.clone(),
                            undefined_key: enum_plan
                                .exclusions
                                .iter()
                                .find(|member| member.mirror_name == undefined_member)
                                .map(|member| member.idl_name.clone())
                                .ok_or_else(|| ApiPolicyError::Invalid {
                                    entry: member.key(),
                                    message: format!(
                                        "absence-capable IDL enum `{name}` has no generated alias member for `{undefined_member}`"
                                    ),
                                })?,
                        },
                    ));
                }
                return Ok((name.clone(), DescriptorFieldConversion::Enum));
            }
            if let Some(descriptor) = descriptors
                .iter()
                .find(|descriptor| descriptor_matches_idl_type(descriptor, name))
            {
                if descriptor.public_only {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "public-only dictionary `{name}` has no standalone boundary aggregate"
                        ),
                    });
                }
                if mirror_type != descriptor.boundary_name {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "IDL dictionary type `{name}` joins `{}`, but the mirror field declares `{}`",
                            descriptor.boundary_name, mirror_field.ty
                        ),
                    });
                }
                if deviation.is_some_and(|row| row.pattern == "dictionary-nullable-descriptor") {
                    if !mirror_nullable {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "dictionary-nullable-descriptor requires a nullable mirror class, found `{}`",
                                mirror_field.ty
                            ),
                        });
                    }
                    return Ok((
                        format!("{} | null", descriptor.name),
                        DescriptorFieldConversion::NullableDescriptor {
                            boundary_name: descriptor.boundary_name.clone(),
                        },
                    ));
                }
                if mirror_nullable {
                    return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                }
                if deviation.is_some_and(|row| row.pattern == "dictionary-optional-descriptor") {
                    let absent_values: Vec<String> = deviation
                        .map(|row| {
                            row.absent_boundary_values
                                .iter()
                                .map(|value| cenum_boundary_value(value, enums))
                                .collect()
                        })
                        .unwrap_or_default();
                    if absent_values.len()
                        != mirror
                            .classes
                            .get(&descriptor.boundary_name)
                            .map_or(0, Vec::len)
                    {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "absent boundary constructor has {} values, expected {}",
                                absent_values.len(),
                                mirror
                                    .classes
                                    .get(&descriptor.boundary_name)
                                    .map_or(0, Vec::len)
                            ),
                        });
                    }
                    return Ok((
                        format!("{} | null", descriptor.name),
                        DescriptorFieldConversion::OptionalDescriptor {
                            boundary_name: descriptor.boundary_name.clone(),
                            absent_values,
                        },
                    ));
                }
                if pattern != "dictionary-field" {
                    return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                }
                return Ok((
                    descriptor.name.clone(),
                    DescriptorFieldConversion::Descriptor(descriptor.boundary_name.clone()),
                ));
            }
            if mirror.handles.contains(mirror_type) {
                if mirror_nullable {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "nullable mirror handle `{}` requires an explicit nullable-handle conversion plan; unconditional handle reads are forbidden",
                            mirror_field.ty
                        ),
                    });
                }
                if pattern != "dictionary-field" {
                    return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                }
                let api = policy
                    .interfaces
                    .iter()
                    .find(|interface| {
                        format!("SubscriptTypegpu{}", interface.trim_start_matches("GPU"))
                            == mirror_type
                    })
                    .ok_or_else(|| ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "mirror handle `{mirror_type}` has no selected public wrapper"
                        ),
                    })?;
                if api != name {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "IDL handle `{name}` joins public wrapper `{api}`, but the names differ"
                        ),
                    });
                }
                return Ok((
                    api.clone(),
                    DescriptorFieldConversion::Handle(raw_field_for_api(api, policy)),
                ));
            }
        }
        IdlType::Sequence(inner) => {
            if let IdlType::Named { name, nullable } = inner.as_ref() {
                if let Some(enum_plan) = enums.iter().find(|plan| plan.name == *name) {
                    let nullable_array_claim =
                        deviation.is_some_and(|row| row.pattern == "dictionary-enum-array");
                    if nullable_array_claim && !*nullable {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: "dictionary-enum-array requires nullable IDL enum elements"
                                .to_owned(),
                        });
                    }
                    if (*nullable && !nullable_array_claim)
                        || (!*nullable && pattern != "dictionary-field")
                    {
                        return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                    }
                    let expected = format!("{}[]", enum_plan.mirror_name);
                    if mirror_type != expected {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "IDL enum sequence `{name}` joins `{expected}`, but the mirror field declares `{}`",
                                mirror_field.ty
                            ),
                        });
                    }
                    return Ok((format!("{name}[]"), DescriptorFieldConversion::EnumArray));
                }
                if let Some(descriptor) = descriptors
                    .iter()
                    .find(|descriptor| descriptor_matches_idl_type(descriptor, name))
                {
                    if *nullable
                        && deviation.is_none_or(|row| row.pattern != "dictionary-descriptor-array")
                    {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: "nullable dictionary-element arrays require an explicit dictionary-descriptor-array policy row"
                                .to_owned(),
                        });
                    }
                    if !*nullable && pattern != "dictionary-field" {
                        return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
                    }
                    if descriptor.public_only {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "public-only dictionary `{name}` has no standalone boundary aggregate"
                            ),
                        });
                    }
                    let expected = format!("{}[]", descriptor.boundary_name);
                    if mirror_type != expected {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "IDL dictionary sequence `{name}` joins `{expected}`, but the mirror field declares `{}`",
                                mirror_field.ty
                            ),
                        });
                    }
                    return Ok((
                        format!("{}[]", descriptor.name),
                        DescriptorFieldConversion::DescriptorArray(
                            descriptor.boundary_name.clone(),
                        ),
                    ));
                }
                let expected_handle = format!("SubscriptTypegpu{}", name.trim_start_matches("GPU"));
                if mirror.handles.contains(&expected_handle) {
                    let expected = format!("{expected_handle}[]");
                    if mirror_type != expected {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "IDL handle sequence `{name}` joins `{expected}`, but the mirror field declares `{}`",
                                mirror_field.ty
                            ),
                        });
                    }
                    if deviation.is_none_or(|row| row.pattern != "dictionary-handle-array") {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: "handle-element dictionary arrays require an explicit dictionary-handle-array policy row"
                                .to_owned(),
                        });
                    }
                    let api = policy
                        .interfaces
                        .iter()
                        .find(|interface| *interface == name)
                        .ok_or_else(|| ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "mirror handle `{expected_handle}` has no selected public wrapper"
                            ),
                        })?;
                    return Ok((
                        format!("{api}[]"),
                        DescriptorFieldConversion::HandleArray {
                            boundary_name: expected_handle,
                            api_name: api.clone(),
                            raw_field: raw_field_for_api(api, policy),
                        },
                    ));
                }
            }
        }
        IdlType::Record { key, value } => {
            let Some(row) = deviation.filter(|row| row.pattern == "dictionary-record-entries")
            else {
                return Err(unclaimed_dictionary_field(member, pattern, mirror_field));
            };
            let value_name = match value.as_ref() {
                IdlType::Named {
                    name,
                    nullable: false,
                } => name.as_str(),
                _ => "",
            };
            if key != "USVString" || value_name != "GPUPipelineConstantValue" {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: format!(
                        "dictionary-record-entries requires record<USVString, GPUPipelineConstantValue>, found record<{key}, {value_name}>"
                    ),
                });
            }
            let api_name = row.record_entry_api.as_deref().unwrap_or_default();
            let boundary_name = row.record_entry_boundary.as_deref().unwrap_or_default();
            if mirror_type != format!("{boundary_name}[]") || mirror_nullable {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: format!(
                        "record entry array joins `{boundary_name}[]`, but the mirror field declares `{}`",
                        mirror_field.ty
                    ),
                });
            }
            let fields = mirror
                .classes
                .get(boundary_name)
                .ok_or_else(|| unknown(&format!("mirror.{boundary_name}")))?;
            if fields
                != &[
                    MirrorField {
                        name: "key".to_owned(),
                        ty: "string".to_owned(),
                    },
                    MirrorField {
                        name: "value".to_owned(),
                        ty: "f64".to_owned(),
                    },
                ]
            {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{boundary_name}"),
                    message:
                        "record entry aggregate must declare exactly key: string and value: f64"
                            .to_owned(),
                });
            }
            return Ok((
                format!("{api_name}[]"),
                DescriptorFieldConversion::RecordEntries {
                    api_name: api_name.to_owned(),
                    boundary_name: boundary_name.to_owned(),
                },
            ));
        }
        _ => {}
    }
    let direct_pattern = matches!(
        pattern,
        "dictionary-field"
            | "dictionary-required"
            | "dictionary-default"
            | "dictionary-boundary-default"
    );
    let direct_type = api_type_from_mirror(mirror, mirror_type)?;
    let direct_scalar = matches!(
        direct_type.as_str(),
        "boolean"
            | "string"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "f32"
            | "f64"
    );
    if direct_pattern && direct_scalar && !mirror_nullable {
        return Ok((direct_type, DescriptorFieldConversion::Direct));
    }
    Err(unclaimed_dictionary_field(member, pattern, mirror_field))
}

fn cenum_boundary_value(value: &str, enums: &[EnumPlan]) -> String {
    let Some((boundary_name, member_name)) = value.split_once('.') else {
        return value.to_owned();
    };
    enums
        .iter()
        .find(|enum_plan| enum_plan.mirror_name == boundary_name)
        .and_then(|enum_plan| {
            enum_plan
                .members
                .iter()
                .chain(&enum_plan.exclusions)
                .find(|member| member.mirror_name == member_name)
        })
        .map_or_else(
            || value.to_owned(),
            |member| format!("{:?}", member.idl_name),
        )
}

fn descriptor_matches_idl_type(descriptor: &DescriptorPlan, name: &str) -> bool {
    descriptor.idl_type == name
        || descriptor.idl_name == name
        || descriptor.idl_aliases.iter().any(|alias| alias == name)
}

fn unclaimed_dictionary_field(
    member: &IdlMember,
    pattern: &str,
    mirror_field: &MirrorField,
) -> ApiPolicyError {
    ApiPolicyError::Invalid {
        entry: member.key(),
        message: format!(
            "pattern `{pattern}` has no public dictionary conversion plan for mirror type `{}`; direct emission is restricted to non-null scalar, boolean, and string fields",
            mirror_field.ty
        ),
    }
}

fn build_namespaces(
    mirror: &MirrorModel,
    policy: &ApiSection,
    ordered: &BTreeMap<String, Vec<IdlMember>>,
    classified: &BTreeMap<String, ClassifiedMember>,
) -> Result<Vec<NamespacePlan>, ApiPolicyError> {
    for row in &policy.namespace_mappings {
        if !policy.namespaces.contains(&row.namespace) {
            return Err(unknown(&format!("{}.namespace_mapping", row.namespace)));
        }
        if !mirror.aliases.contains_key(&row.boundary) {
            return Err(unknown(&format!("mirror.{}", row.boundary)));
        }
    }
    let mut plans = Vec::new();
    for name in &policy.namespaces {
        let alias = policy
            .namespace_mappings
            .iter()
            .find(|row| row.namespace == *name)
            .map_or_else(
                || format!("SubscriptTypegpu{}", name.trim_start_matches("GPU")),
                |row| row.boundary.clone(),
            );
        let value_type = mirror
            .aliases
            .get(&alias)
            .ok_or_else(|| unknown(&format!("mirror.{alias}")))?
            .clone();
        let mut constants = Vec::new();
        for member in ordered.get(name).ok_or_else(|| unknown(name))? {
            let classified_member = classified
                .get(&member.key())
                .ok_or_else(|| unknown(&member.key()))?;
            if matches!(classified_member.classification, Classification::Exclude) {
                continue;
            }
            let IdlMemberKind::NamespaceConstant { value } = member.kind else {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "selected namespace contains a non-constant member".to_owned(),
                });
            };
            constants.push((member.name.clone(), value));
        }
        plans.push(NamespacePlan {
            name: name.clone(),
            value_type: api_type_from_mirror(mirror, &value_type)?,
            constants,
        });
    }
    Ok(plans)
}

fn facade_enum_prefix(mirror_name: &str) -> String {
    let suffix = mirror_name
        .strip_prefix("SubscriptTypegpu")
        .unwrap_or(mirror_name);
    format!(
        "SUBSCRIPT_TYPEGPU_{}_",
        naming::snake(suffix).to_ascii_uppercase()
    )
}

fn facade_enum_member(mirror_name: &str, member: &str) -> String {
    format!(
        "{}{}",
        facade_enum_prefix(mirror_name),
        naming::snake(member).to_ascii_uppercase()
    )
}

fn build_enums(
    mirror: &MirrorModel,
    policy: &ApiSection,
    ordered: &BTreeMap<String, Vec<IdlMember>>,
    classified: &BTreeMap<String, ClassifiedMember>,
) -> Result<Vec<EnumPlan>, ApiPolicyError> {
    let mut plans = Vec::new();
    let mut consumed_mappings = BTreeSet::new();
    let mut consumed_exclusions = BTreeSet::new();
    for name in &policy.enums {
        let mirror_name = format!("SubscriptTypegpu{}", name.trim_start_matches("GPU"));
        let mirror_enum = mirror
            .enums
            .get(&mirror_name)
            .ok_or_else(|| unknown(&format!("mirror.{mirror_name}")))?;
        let mut members = Vec::new();
        let mut exclusions = Vec::new();
        for member in ordered.get(name).ok_or_else(|| unknown(name))? {
            let classification = classified
                .get(&member.key())
                .ok_or_else(|| unknown(&member.key()))?;
            if !matches!(
                classification.classification,
                Classification::Generate(ref pattern) if pattern == "enum-value"
            ) {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message:
                        "selected Q32 enums require every member to use the enum-value pattern"
                            .to_owned(),
                });
            }
            let mapping = policy
                .enum_mappings
                .iter()
                .find(|row| row.enum_name == *name && row.member == member.name);
            let derived = facade_enum_member(&mirror_name, &member.name);
            let mirror_member = if let Some(row) = mapping {
                row.mirror.clone()
            } else if mirror_enum.members.contains_key(&derived) {
                derived
            } else {
                let normalized_idl = normalize_enum_spelling(&member.name);
                let candidates = mirror_enum
                    .members
                    .keys()
                    .filter(|constant| {
                        constant
                            .strip_prefix(&facade_enum_prefix(&mirror_name))
                            .is_some_and(|suffix| normalize_enum_spelling(suffix) == normalized_idl)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                match candidates.as_slice() {
                    [only] => only.clone(),
                    _ => derived,
                }
            };
            if let Some(row) = mapping {
                consumed_mappings.insert(format!("{}.{}", row.enum_name, row.member));
            }
            let wire_value = *mirror_enum.members.get(&mirror_member).ok_or_else(|| {
                ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: format!(
                        "mirror enum `{mirror_name}` has no constant `{mirror_member}`"
                    ),
                }
            })?;
            members.push(EnumMemberPlan {
                idl_name: member.name.clone(),
                wire_value,
                mirror_name: mirror_member,
            });
        }
        for exclusion in policy
            .enum_exclusions
            .iter()
            .filter(|row| row.enum_name == *name)
        {
            let wire_value = *mirror_enum
                .members
                .get(&exclusion.mirror)
                .ok_or_else(|| unknown(&format!("mirror.{}.{}", mirror_name, exclusion.mirror)))?;
            if let Some(mapped) = members
                .iter()
                .find(|member| member.mirror_name == exclusion.mirror)
            {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{}.{}", name, exclusion.mirror),
                    message: format!(
                        "enum exclusion names mirror constant already mapped from IDL member `{}.{}`",
                        name, mapped.idl_name
                    ),
                });
            }
            let suffix = exclusion
                .mirror
                .strip_prefix(&facade_enum_prefix(&mirror_name))
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: format!("{}.{}", name, exclusion.mirror),
                    message: "enum exclusion does not use the selected mirror enum prefix"
                        .to_owned(),
                })?;
            let idl_name = cenum_boundary_member_name(suffix);
            if let Some(mapped) = members.iter().find(|member| member.idl_name == idl_name) {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{}.{}", name, exclusion.mirror),
                    message: format!(
                        "boundary-only alias member collides with IDL member `{}.{}`",
                        name, mapped.idl_name
                    ),
                });
            }
            exclusions.push(EnumMemberPlan {
                idl_name,
                wire_value,
                mirror_name: exclusion.mirror.clone(),
            });
            consumed_exclusions.insert(format!("{}.{}", name, exclusion.mirror));
        }
        if let Some(extra) = mirror_enum.members.keys().find(|constant| {
            !members
                .iter()
                .any(|member| member.mirror_name.as_str() == constant.as_str())
                && !policy
                    .enum_exclusions
                    .iter()
                    .any(|row| row.enum_name == *name && row.mirror.as_str() == constant.as_str())
        }) {
            return Err(ApiPolicyError::Invalid {
                entry: format!("mirror.{mirror_name}.{extra}"),
                message: format!(
                    "mirror enum constant has no member in selected IDL enum `{name}`"
                ),
            });
        }
        plans.push(EnumPlan {
            name: name.clone(),
            mirror_name,
            members,
            exclusions,
        });
    }
    if let Some(row) = policy
        .enum_mappings
        .iter()
        .find(|row| !consumed_mappings.contains(&format!("{}.{}", row.enum_name, row.member)))
    {
        return Err(unknown(&format!("{}.{}", row.enum_name, row.member)));
    }
    if let Some(row) = policy
        .enum_exclusions
        .iter()
        .find(|row| !consumed_exclusions.contains(&format!("{}.{}", row.enum_name, row.mirror)))
    {
        let mirror_name = format!(
            "SubscriptTypegpu{}",
            row.enum_name.trim_start_matches("GPU")
        );
        let mirror_enum = mirror
            .enums
            .get(&mirror_name)
            .ok_or_else(|| unknown(&format!("mirror.{mirror_name}")))?;
        if !mirror_enum.members.contains_key(&row.mirror) {
            return Err(unknown(&format!("mirror.{}.{}", mirror_name, row.mirror)));
        }
        return Err(ApiPolicyError::Dead {
            entry: format!("{}.{}", row.enum_name, row.mirror),
        });
    }
    Ok(plans)
}

fn normalize_enum_spelling(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn cenum_boundary_member_name(value: &str) -> String {
    if !value
        .chars()
        .any(|character| character.is_ascii_lowercase())
    {
        return value.to_ascii_lowercase().replace('_', "-");
    }
    let mut out = String::new();
    for (index, character) in value.chars().enumerate() {
        if character == '_' {
            if !out.ends_with('-') {
                out.push('-');
            }
        } else {
            if index > 0 && character.is_ascii_uppercase() && !out.ends_with('-') {
                out.push('-');
            }
            out.push(character.to_ascii_lowercase());
        }
    }
    out
}

fn validate_flattened_interfaces(
    policy: &ApiSection,
    flattened: &BTreeMap<String, Vec<IdlMember>>,
) -> Result<(), ApiPolicyError> {
    for row in &policy.flattened_interfaces {
        if !policy
            .result_records
            .iter()
            .any(|record| record.interface == row.target)
        {
            return Err(ApiPolicyError::Invalid {
                entry: row.interface.clone(),
                message: format!(
                    "flattened interface target `{}` is not a selected result record",
                    row.target
                ),
            });
        }
        let parent = policy
            .interface_parents
            .iter()
            .find(|parent| parent.interface == row.interface)
            .ok_or_else(|| unknown(&format!("{}.parent", row.interface)))?;
        if parent.parent != row.target {
            return Err(ApiPolicyError::Invalid {
                entry: row.interface.clone(),
                message: format!(
                    "flattened interface target `{}` does not match its IDL parent `{}`",
                    row.target, parent.parent
                ),
            });
        }
        let members = flattened
            .get(&row.interface)
            .ok_or_else(|| unknown(&row.interface))?;
        if members.len() != 1
            || members[0].name != "@constructor"
            || !matches!(members[0].kind, IdlMemberKind::Special)
        {
            return Err(ApiPolicyError::Invalid {
                entry: row.interface.clone(),
                message: "flattened error subclasses must contain exactly their IDL constructor"
                    .to_owned(),
            });
        }
        if !policy
            .exclude
            .iter()
            .any(|exclude| exclude.member == members[0].key())
        {
            return Err(ApiPolicyError::Invalid {
                entry: members[0].key(),
                message: "a flattened error-subclass constructor must be excluded".to_owned(),
            });
        }
    }
    Ok(())
}

fn build_result_records(
    mirror: &MirrorModel,
    policy: &ApiSection,
    ordered: &BTreeMap<String, Vec<IdlMember>>,
    classified: &BTreeMap<String, ClassifiedMember>,
    enums: &[EnumPlan],
) -> Result<Vec<ResultRecordPlan>, ApiPolicyError> {
    let mut records = Vec::new();
    for row in &policy.result_records {
        let boundary_fields = mirror
            .classes
            .get(&row.boundary)
            .ok_or_else(|| unknown(&format!("mirror.{}", row.boundary)))?;
        let members = ordered
            .get(&row.interface)
            .ok_or_else(|| unknown(&row.interface))?;
        let mut boundary_exclusions = BTreeSet::new();
        for exclusion in &row.boundary_field_exclusions {
            if !boundary_exclusions.insert(exclusion.as_str()) {
                return Err(ApiPolicyError::Duplicate {
                    entry: format!("{}.boundary_field_exclusions.{exclusion}", row.interface),
                });
            }
            if !boundary_fields.iter().any(|field| field.name == *exclusion) {
                return Err(unknown(&format!("mirror.{}.{}", row.boundary, exclusion)));
            }
            if let Some(member) = members.iter().find(|member| member.name == *exclusion) {
                let classification = classified
                    .get(&member.key())
                    .ok_or_else(|| unknown(&member.key()))?;
                if !matches!(classification.classification, Classification::Exclude) {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.boundary_field_exclusions.{exclusion}", row.interface),
                        message: "boundary-field exclusion names a selected IDL result attribute"
                            .to_owned(),
                    });
                }
            }
        }
        let synthetic = match (
            row.synthetic_field.as_deref(),
            row.synthetic_enum.as_deref(),
            row.synthetic_boundary_enum.as_deref(),
        ) {
            (None, None, None) => {
                if !row.synthetic_enum_mappings.is_empty()
                    || !row.synthetic_enum_exclusions.is_empty()
                {
                    return Err(ApiPolicyError::Invalid {
                        entry: row.interface.clone(),
                        message: "synthetic enum mappings require all synthetic field controls"
                            .to_owned(),
                    });
                }
                None
            }
            (Some(field), Some(name), Some(boundary_name)) => {
                let mirror_enum = mirror
                    .enums
                    .get(boundary_name)
                    .ok_or_else(|| unknown(&format!("mirror.{boundary_name}")))?;
                let mut public_members = BTreeSet::new();
                let mut mirror_members = BTreeSet::new();
                let mut members = Vec::new();
                for mapping in &row.synthetic_enum_mappings {
                    let (public, mirror_member) =
                        mapping
                            .split_once('=')
                            .ok_or_else(|| ApiPolicyError::Invalid {
                                entry: format!("{}.synthetic_enum_mappings", row.interface),
                                message: format!("malformed synthetic enum mapping `{mapping}`"),
                            })?;
                    if public.is_empty() || mirror_member.is_empty() {
                        return Err(ApiPolicyError::Invalid {
                            entry: format!("{}.synthetic_enum_mappings", row.interface),
                            message: format!("malformed synthetic enum mapping `{mapping}`"),
                        });
                    }
                    if !public_members.insert(public) || !mirror_members.insert(mirror_member) {
                        return Err(ApiPolicyError::Duplicate {
                            entry: format!("{}.synthetic_enum.{mapping}", row.interface),
                        });
                    }
                    if !mirror_enum.members.contains_key(mirror_member) {
                        return Err(unknown(&format!("mirror.{boundary_name}.{mirror_member}")));
                    }
                    members.push(EnumMemberPlan {
                        idl_name: public.to_owned(),
                        mirror_name: mirror_member.to_owned(),
                        wire_value: *mirror_enum.members.get(mirror_member).ok_or_else(|| {
                            unknown(&format!("mirror.{boundary_name}.{mirror_member}"))
                        })?,
                    });
                }
                let mut exclusions = BTreeSet::new();
                for exclusion in &row.synthetic_enum_exclusions {
                    if !exclusions.insert(exclusion.as_str()) {
                        return Err(ApiPolicyError::Duplicate {
                            entry: format!("{}.synthetic_enum.{exclusion}", row.interface),
                        });
                    }
                    if mirror_members.contains(exclusion.as_str()) {
                        return Err(ApiPolicyError::Invalid {
                            entry: format!("{}.synthetic_enum.{exclusion}", row.interface),
                            message: "synthetic enum exclusion names an already-mapped constant"
                                .to_owned(),
                        });
                    }
                    if !mirror_enum.members.contains_key(exclusion) {
                        return Err(unknown(&format!("mirror.{boundary_name}.{exclusion}")));
                    }
                }
                if let Some(unaccounted) = mirror_enum.members.keys().find(|constant| {
                    !mirror_members.contains(constant.as_str())
                        && !exclusions.contains(constant.as_str())
                }) {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("mirror.{boundary_name}.{unaccounted}"),
                        message: format!(
                            "mirror enum constant is unaccounted by synthetic result enum `{name}`"
                        ),
                    });
                }
                let boundary_field = boundary_fields
                    .iter()
                    .find(|candidate| candidate.name == field)
                    .ok_or_else(|| ApiPolicyError::Invalid {
                        entry: format!("{}.{}", row.interface, field),
                        message: format!(
                            "synthetic result field has no mirror field in `{}`",
                            row.boundary
                        ),
                    })?;
                if boundary_field.ty != boundary_name {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.{}", row.interface, field),
                        message: format!(
                            "synthetic result field mirror type is `{}`, expected `{boundary_name}`",
                            boundary_field.ty
                        ),
                    });
                }
                Some((
                    field.to_owned(),
                    SyntheticEnumPlan {
                        name: name.to_owned(),
                        mirror_name: boundary_name.to_owned(),
                        members,
                        exclusions: row.synthetic_enum_exclusions.clone(),
                    },
                ))
            }
            _ => {
                return Err(ApiPolicyError::Invalid {
                    entry: row.interface.clone(),
                    message:
                        "synthetic field, public enum, and boundary enum must be stated together"
                            .to_owned(),
                })
            }
        };
        if row.nullable == row.synthetic_enum_exclusions.is_empty() {
            return Err(ApiPolicyError::Invalid {
                entry: row.interface.clone(),
                message: "result-record nullable must exactly match its null-producing synthetic enum exclusions"
                    .to_owned(),
            });
        }

        let mut fields = Vec::new();
        let mut seed_values = Vec::new();
        for boundary_field in boundary_fields {
            if boundary_exclusions.contains(boundary_field.name.as_str()) {
                seed_values.push(result_record_boundary_seed(mirror, boundary_field)?);
                continue;
            }
            if let Some((synthetic_field, synthetic_enum)) = &synthetic {
                if boundary_field.name == *synthetic_field {
                    fields.push(ResultRecordFieldPlan {
                        name: synthetic_field.clone(),
                        ty: synthetic_enum.name.clone(),
                        conversion: ResultRecordFieldConversion::SyntheticEnum,
                    });
                    let seed = row
                        .synthetic_enum_exclusions
                        .first()
                        .map(String::as_str)
                        .or_else(|| {
                            synthetic_enum
                                .members
                                .first()
                                .map(|member| member.mirror_name.as_str())
                        })
                        .ok_or_else(|| ApiPolicyError::Invalid {
                            entry: row.interface.clone(),
                            message: "synthetic result enum has no seed constant".to_owned(),
                        })?;
                    seed_values.push(format!("{}.{seed}", synthetic_enum.mirror_name));
                    continue;
                }
            }
            let member = members
                .iter()
                .find(|member| member.name == boundary_field.name)
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: format!("mirror.{}.{}", row.boundary, boundary_field.name),
                    message: format!(
                        "result record field has no selected IDL attribute on `{}`",
                        row.interface
                    ),
                })?;
            let classification = classified
                .get(&member.key())
                .ok_or_else(|| unknown(&member.key()))?;
            let pattern = match &classification.classification {
                Classification::Generate(pattern) => Some(pattern.as_str()),
                Classification::Deviation(row) => Some(row.pattern.as_str()),
                Classification::Exclude => None,
            };
            if pattern != Some("result-record-field") {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "result-record attributes require the result-record-field pattern"
                        .to_owned(),
                });
            }
            let IdlMemberKind::Attribute { ty } = &member.kind else {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "result-record-field requires a read-only IDL attribute".to_owned(),
                });
            };
            let (public_type, conversion) = match ty {
                IdlType::String if boundary_field.ty == "string" => {
                    seed_values.push("\"\"".to_owned());
                    ("string".to_owned(), ResultRecordFieldConversion::Direct)
                }
                IdlType::Named {
                    name,
                    nullable: false,
                } => {
                    let enum_plan =
                        enums
                            .iter()
                            .find(|plan| plan.name == *name)
                            .ok_or_else(|| ApiPolicyError::Invalid {
                                entry: member.key(),
                                message: format!(
                                    "result-record named field `{name}` is not a selected Q32 enum"
                                ),
                            })?;
                    if boundary_field.ty != enum_plan.mirror_name {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "IDL enum `{name}` joins `{}`, but result record field is `{}`",
                                enum_plan.mirror_name, boundary_field.ty
                            ),
                        });
                    }
                    let seed =
                        enum_plan
                            .members
                            .first()
                            .ok_or_else(|| ApiPolicyError::Invalid {
                                entry: member.key(),
                                message: "result-record enum has no public members".to_owned(),
                            })?;
                    seed_values.push(format!("{:?}", seed.idl_name));
                    (name.clone(), ResultRecordFieldConversion::Enum)
                }
                IdlType::Scalar(scalar) => {
                    if boundary_field.ty != *scalar {
                        return Err(ApiPolicyError::Invalid {
                            entry: member.key(),
                            message: format!(
                                "IDL result scalar `{scalar}` does not match mirror field `{}`",
                                boundary_field.ty
                            ),
                        });
                    }
                    seed_values.push("0".to_owned());
                    (scalar.clone(), ResultRecordFieldConversion::Direct)
                }
                IdlType::Boolean if boundary_field.ty == "boolean" => {
                    seed_values.push("false".to_owned());
                    ("boolean".to_owned(), ResultRecordFieldConversion::Direct)
                }
                _ => {
                    return Err(ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: format!(
                            "unsupported result-record field join from IDL `{ty:?}` to mirror `{}`",
                            boundary_field.ty
                        ),
                    })
                }
            };
            fields.push(ResultRecordFieldPlan {
                name: member.name.clone(),
                ty: public_type,
                conversion,
            });
        }
        let selected_members = members
            .iter()
            .filter(|member| {
                classified.get(&member.key()).is_some_and(|classification| {
                    !matches!(classification.classification, Classification::Exclude)
                })
            })
            .count();
        if fields.len() != selected_members + usize::from(synthetic.is_some()) {
            return Err(ApiPolicyError::Invalid {
                entry: row.interface.clone(),
                message: "selected result-record attributes do not exactly cover mirror fields"
                    .to_owned(),
            });
        }
        records.push(ResultRecordPlan {
            name: row.interface.clone(),
            boundary_name: row.boundary.clone(),
            fields,
            synthetic_enum: synthetic.map(|(_, plan)| plan),
            nullable: row.nullable,
            seed_values,
        });
    }
    Ok(records)
}

fn result_record_boundary_seed(
    mirror: &MirrorModel,
    field: &crate::api_model::MirrorField,
) -> Result<String, ApiPolicyError> {
    match field.ty.as_str() {
        "string" => Ok("\"\"".to_owned()),
        "boolean" => Ok("false".to_owned()),
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "f32" | "f64" => {
            Ok("0".to_owned())
        }
        ty => {
            let enum_shape = mirror
                .enums
                .get(ty)
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: format!("mirror.seed.{}", field.name),
                    message: format!(
                        "excluded result-record boundary field has unsupported type `{ty}`"
                    ),
                })?;
            let (member, _) = enum_shape
                .members
                .iter()
                .min_by(|(left_name, left_value), (right_name, right_value)| {
                    left_value
                        .cmp(right_value)
                        .then_with(|| left_name.cmp(right_name))
                })
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: format!("mirror.seed.{}", field.name),
                    message: format!("excluded result-record boundary enum `{ty}` has no members"),
                })?;
            Ok(format!("{ty}.{member}"))
        }
    }
}

fn build_interfaces(
    mirror: &MirrorModel,
    policy: &ApiSection,
    ordered: &BTreeMap<String, Vec<IdlMember>>,
    classified: &BTreeMap<String, ClassifiedMember>,
    descriptors: &[DescriptorPlan],
    enums: &[EnumPlan],
    result_records: &[ResultRecordPlan],
) -> Result<Vec<InterfacePlan>, ApiPolicyError> {
    let mut plans = Vec::new();
    for interface in &policy.interfaces {
        let boundary = if interface == &policy.singleton_interface {
            policy.singleton_boundary.clone()
        } else {
            interface
                .strip_prefix("GPU")
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: interface.clone(),
                    message: "API wrapper names must have the GPU prefix".to_owned(),
                })?
                .to_owned()
        };
        let handle = format!("SubscriptTypegpu{boundary}");
        if !mirror.handles.contains(&handle) {
            return Err(unknown(&format!("mirror.{handle}")));
        }
        let release = format!("subscript_typegpu_{}_release", naming::snake(&boundary));
        if !mirror.functions.contains_key(&release) {
            return Err(unknown(&format!("mirror.{release}")));
        }
        let mut methods = Vec::new();
        for member in ordered.get(interface).ok_or_else(|| unknown(interface))? {
            let member_key = member.key();
            let classified_member = classified
                .get(&member_key)
                .ok_or_else(|| unknown(&member_key))?;
            match &classified_member.classification {
                Classification::Exclude => {}
                Classification::Generate(pattern) => methods.push(build_method(
                    mirror,
                    policy,
                    &boundary,
                    member,
                    pattern,
                    None,
                    descriptors,
                    enums,
                    result_records,
                )?),
                Classification::Deviation(row) => methods.push(build_method(
                    mirror,
                    policy,
                    &boundary,
                    member,
                    &row.pattern,
                    Some(row),
                    descriptors,
                    enums,
                    result_records,
                )?),
            }
            for row in policy
                .deviations
                .iter()
                .filter(|row| synthetic_typed_anchor(row) == Some(member_key.as_str()))
            {
                methods.push(build_synthetic_typed_method(mirror, row, &boundary)?);
            }
        }
        plans.push(InterfacePlan {
            name: interface.clone(),
            raw_field: lower_first(&boundary),
            boundary,
            methods,
            needs_instance: false,
            host_owned: interface == "GPUDevice"
                && policy.deviations.iter().any(|row| {
                    row.member == GPU_DEVICE_CONSTRUCTOR
                        && row.pattern == HOST_OWNED_WRAPPER_PATTERN
                }),
            idempotent_dispose: interface == "GPUQueue"
                && policy.deviations.iter().any(|row| {
                    row.member == GPU_DEVICE_CONSTRUCTOR
                        && row.pattern == HOST_OWNED_WRAPPER_PATTERN
                }),
        });
    }
    Ok(plans)
}

fn build_synthetic_typed_method(
    mirror: &MirrorModel,
    row: &ApiDeviationRow,
    boundary: &str,
) -> Result<MethodPlan, ApiPolicyError> {
    match (row.member.as_str(), row.pattern.as_str(), boundary) {
        (GPU_QUEUE_WRITE_BUFFER_F32, TYPED_WRITE_F32_PATTERN, "Queue") => {
            let function = "subscript_typegpu_queue_write_buffer_f32";
            let declaration = mirror
                .functions
                .get(function)
                .ok_or_else(|| unknown(&format!("mirror.{function}")))?;
            validate_parameter_types(
                declaration,
                &[
                    "SubscriptTypegpuQueue".to_owned(),
                    "SubscriptTypegpuBuffer".to_owned(),
                    "u64".to_owned(),
                    "f32[]".to_owned(),
                ],
            )?;
            if declaration.return_type != "void" {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{function}"),
                    message: "typed queue write must return void".to_owned(),
                });
            }
            Ok(MethodPlan::TypedWriteF32 {
                function: function.to_owned(),
            })
        }
        (GPU_BUFFER_READ_MAPPED_RANGE_F32, TYPED_READ_F32_PATTERN, "Buffer") => {
            let function = "subscript_typegpu_buffer_read_mapped_range_f32";
            let declaration = mirror
                .functions
                .get(function)
                .ok_or_else(|| unknown(&format!("mirror.{function}")))?;
            validate_parameter_types(
                declaration,
                &[
                    "SubscriptTypegpuBuffer".to_owned(),
                    "u64".to_owned(),
                    "f32[]".to_owned(),
                ],
            )?;
            if declaration.return_type != "i32" {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{function}"),
                    message: "typed mapped read must return i32 status".to_owned(),
                });
            }
            Ok(MethodPlan::TypedReadF32 {
                function: function.to_owned(),
            })
        }
        _ => Err(ApiPolicyError::Invalid {
            entry: row.member.clone(),
            message: format!("typed f32 deviation does not match receiver `GPU{boundary}`"),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_method(
    mirror: &MirrorModel,
    policy: &ApiSection,
    boundary: &str,
    member: &IdlMember,
    pattern: &str,
    deviation: Option<&ApiDeviationRow>,
    descriptors: &[DescriptorPlan],
    enums: &[EnumPlan],
    result_records: &[ResultRecordPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let receiver = deviation
        .and_then(|row| row.boundary_receiver.as_deref())
        .unwrap_or(boundary);
    if receiver != boundary {
        let handle = format!("SubscriptTypegpu{receiver}");
        if !mirror.handles.contains(&handle) {
            return Err(unknown(&format!("mirror.{handle}")));
        }
    }
    match pattern {
        "async-request" => build_async_method(
            mirror,
            policy,
            member,
            receiver,
            deviation,
            descriptors,
            enums,
        ),
        "attribute-method" => build_attribute_method(mirror, policy, member, receiver, enums),
        "operation" => build_operation_method(
            mirror,
            policy,
            member,
            receiver,
            deviation,
            descriptors,
            enums,
        ),
        "mapped-range" => build_mapped_range_method(mirror, member, receiver),
        "label-method" => build_label_method(mirror, member, receiver),
        "error-scope-pop" => build_error_scope_pop_method(mirror, member, receiver, result_records),
        "device-lost-poll" | "uncaptured-error-drain" => {
            build_record_drain_method(mirror, member, receiver, result_records, pattern)
        }
        "feature-probe" => build_feature_probe_method(mirror, member, receiver, enums),
        "result-record-fill" => {
            build_result_record_fill_method(mirror, member, receiver, result_records)
        }
        _ => Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!("unsupported interface pattern `{pattern}`"),
        }),
    }
}

fn build_feature_probe_method(
    mirror: &MirrorModel,
    member: &IdlMember,
    receiver: &str,
    enums: &[EnumPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Attribute {
        ty: IdlType::Named {
            name: result_name,
            nullable: false,
        },
    } = &member.kind
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "feature-probe requires a non-null named IDL attribute".to_owned(),
        });
    };
    let expected_result = format!("GPUSupported{}", naming::pascal(&member.name));
    if result_name != &expected_result {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!(
                "feature-probe attribute type is `{result_name}`, expected `{expected_result}`"
            ),
        });
    }
    let singular = member
        .name
        .strip_suffix('s')
        .ok_or_else(|| ApiPolicyError::Invalid {
            entry: member.key(),
            message: "feature-probe attribute name must be plural".to_owned(),
        })?;
    let enum_name = format!("GPU{}Name", naming::pascal(singular));
    let enum_plan = enums
        .iter()
        .find(|enum_plan| enum_plan.name == enum_name)
        .ok_or_else(|| ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!("feature-probe requires the selected `{enum_name}` Q32 enum"),
        })?;
    let method_name = format!("has{}", naming::pascal(singular));
    let function_name = format!(
        "subscript_typegpu_{}_{}",
        naming::snake(receiver),
        naming::snake(&method_name)
    );
    let function = mirror
        .functions
        .get(&function_name)
        .ok_or_else(|| unknown(&format!("mirror.{function_name}")))?;
    validate_parameter_types(
        function,
        &[
            format!("SubscriptTypegpu{receiver}"),
            enum_plan.mirror_name.clone(),
        ],
    )?;
    if function.return_type != "boolean" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{function_name}"),
            message: "feature probe must return boolean".to_owned(),
        });
    }
    Ok(MethodPlan::Operation {
        name: method_name,
        function: function_name,
        params: vec![MethodParamPlan {
            name: "name".to_owned(),
            api_type: enum_name,
            expression: "name".to_owned(),
            default: None,
            helper: None,
        }],
        return_type: "boolean".to_owned(),
        result_class: None,
        call_args: vec!["name".to_owned()],
        default_variant: None,
    })
}

fn build_result_record_fill_method(
    mirror: &MirrorModel,
    member: &IdlMember,
    receiver: &str,
    result_records: &[ResultRecordPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Attribute {
        ty: IdlType::Named {
            name: result_name,
            nullable: false,
        },
    } = &member.kind
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "result-record-fill requires a non-null named IDL attribute".to_owned(),
        });
    };
    let record = result_records
        .iter()
        .find(|record| record.name == *result_name)
        .ok_or_else(|| ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!("result-record-fill requires the `{result_name}` result-record plan"),
        })?;
    let function_name = format!(
        "subscript_typegpu_{}_get_{}",
        naming::snake(receiver),
        naming::snake(&member.name)
    );
    let function = mirror
        .functions
        .get(&function_name)
        .ok_or_else(|| unknown(&format!("mirror.{function_name}")))?;
    validate_parameter_types(
        function,
        &[
            format!("SubscriptTypegpu{receiver}"),
            format!("{} | null", record.boundary_name),
        ],
    )?;
    let success = match function.return_type.as_str() {
        "boolean" => RecordFillSuccess::Boolean,
        "i32" => RecordFillSuccess::StatusOne,
        other => {
            return Err(ApiPolicyError::Invalid {
                entry: format!("mirror.{function_name}"),
                message: format!(
                    "result-record fill returns `{other}`, expected boolean or i32 status"
                ),
            })
        }
    };
    Ok(MethodPlan::RecordFill {
        name: member.name.clone(),
        function: function_name,
        record: record.boundary_name.clone(),
        result_class: record.name.clone(),
        conversion: format!("from{}", record.boundary_name),
        seed_values: record.seed_values.clone(),
        success,
    })
}

fn build_label_method(
    mirror: &MirrorModel,
    member: &IdlMember,
    receiver: &str,
) -> Result<MethodPlan, ApiPolicyError> {
    if !matches!(
        &member.kind,
        IdlMemberKind::Attribute {
            ty: IdlType::String
        }
    ) || member.name != "label"
    {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "label-method requires the IDL DOMString label attribute".to_owned(),
        });
    }
    let function_name = format!("subscript_typegpu_{}_set_label", naming::snake(receiver));
    let function = mirror
        .functions
        .get(&function_name)
        .ok_or_else(|| unknown(&format!("mirror.{function_name}")))?;
    validate_parameter_types(
        function,
        &[format!("SubscriptTypegpu{receiver}"), "string".to_owned()],
    )?;
    if function.return_type != "void" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{function_name}"),
            message: "label setter must return void".to_owned(),
        });
    }
    Ok(MethodPlan::Operation {
        name: "label".to_owned(),
        function: function_name,
        params: vec![MethodParamPlan {
            name: "value".to_owned(),
            api_type: "string".to_owned(),
            expression: "value".to_owned(),
            default: None,
            helper: None,
        }],
        return_type: "void".to_owned(),
        result_class: None,
        call_args: vec!["value".to_owned()],
        default_variant: None,
    })
}

fn build_error_scope_pop_method(
    mirror: &MirrorModel,
    member: &IdlMember,
    receiver: &str,
    result_records: &[ResultRecordPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Operation {
        return_type,
        arguments,
    } = &member.kind
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "error-scope-pop requires an IDL operation".to_owned(),
        });
    };
    let IdlType::Promise(inner) = return_type else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "error-scope-pop requires a promise result".to_owned(),
        });
    };
    let IdlType::Named {
        name: result_name,
        nullable,
    } = inner.as_ref()
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "error-scope-pop requires a named nullable result".to_owned(),
        });
    };
    if receiver != "Device" || !arguments.is_empty() || !nullable {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "error-scope-pop requires a zero-argument GPUDevice operation with a nullable named result"
                .to_owned(),
        });
    }
    let record = result_records
        .iter()
        .find(|record| record.name == *result_name)
        .ok_or_else(|| ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!("error-scope-pop requires the `{result_name}` result-record plan"),
        })?;
    if record.nullable != *nullable {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!(
                "IDL result nullability does not match the `{result_name}` result-record plan"
            ),
        });
    }
    let begin = format!(
        "subscript_typegpu_{}_{}",
        naming::snake(receiver),
        naming::snake(&member.name)
    );
    let begin_fn = mirror
        .functions
        .get(&begin)
        .ok_or_else(|| unknown(&format!("mirror.{begin}")))?;
    validate_parameter_types(begin_fn, &["SubscriptTypegpuDevice".to_owned()])?;
    if begin_fn.return_type != "SubscriptTypegpuFutureId" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{begin}"),
            message: "error-scope begin must return SubscriptTypegpuFutureId".to_owned(),
        });
    }
    let take = format!("subscript_typegpu_{}_take", naming::snake(&member.name));
    let take_fn = mirror
        .functions
        .get(&take)
        .ok_or_else(|| unknown(&format!("mirror.{take}")))?;
    validate_parameter_types(
        take_fn,
        &[
            "SubscriptTypegpuInstance".to_owned(),
            "SubscriptTypegpuFutureId".to_owned(),
            format!("{} | null", record.boundary_name),
        ],
    )?;
    if take_fn.return_type != "boolean" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{take}"),
            message: "error-scope fill take must return boolean".to_owned(),
        });
    }
    Ok(MethodPlan::ErrorScopePop {
        name: member.name.clone(),
        begin,
        take,
        record: record.boundary_name.clone(),
        result_class: record.name.clone(),
        nullable: *nullable,
        conversion: format!("from{}", record.boundary_name),
        seed_values: record.seed_values.clone(),
    })
}

fn build_record_drain_method(
    mirror: &MirrorModel,
    member: &IdlMember,
    receiver: &str,
    result_records: &[ResultRecordPlan],
    pattern: &str,
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Attribute { ty } = &member.kind else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "fill-record drains require an IDL attribute".to_owned(),
        });
    };
    if receiver != "Device" {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "fill-record drains require a GPUDevice attribute".to_owned(),
        });
    }
    let (method_name, function_name, record, pump) = match pattern {
        "device-lost-poll" => {
            let IdlType::Promise(inner) = ty else {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "device-lost-poll requires a promise attribute".to_owned(),
                });
            };
            let IdlType::Named {
                name: result_name,
                nullable: false,
            } = inner.as_ref()
            else {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "device-lost-poll requires a non-null named promise result".to_owned(),
                });
            };
            let record = result_records
                .iter()
                .find(|record| record.name == *result_name)
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: format!(
                        "fill-record drain requires the `{result_name}` result-record plan"
                    ),
                })?;
            let boundary_suffix = result_name
                .strip_prefix(&format!("GPU{receiver}"))
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: format!(
                        "device-lost result `{result_name}` does not derive from receiver `GPU{receiver}`"
                    ),
                })?;
            (
                lower_first(result_name.trim_start_matches("GPU")),
                format!(
                    "subscript_typegpu_{}_{}",
                    naming::snake(receiver),
                    naming::snake(boundary_suffix)
                ),
                record,
                true,
            )
        }
        "uncaptured-error-drain" => {
            if !matches!(
                ty,
                IdlType::Named {
                    name,
                    nullable: false
                } if name == "EventHandler"
            ) {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "uncaptured-error-drain requires an EventHandler attribute".to_owned(),
                });
            }
            let event_name =
                member
                    .name
                    .strip_prefix("on")
                    .ok_or_else(|| ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: "event drains require an IDL on... attribute".to_owned(),
                    })?;
            let mut record_matches = Vec::new();
            for record in result_records {
                let suffix = record.name.trim_start_matches("GPU");
                let lower_suffix = suffix.to_ascii_lowercase();
                if let Some(stem) = event_name.strip_suffix(&lower_suffix) {
                    record_matches.push((record, stem.to_owned(), suffix.to_owned()));
                }
            }
            let [(record, event_stem, result_suffix)] = record_matches.as_slice() else {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: format!(
                        "event name `{event_name}` selects {} result-record plans, expected exactly one",
                        record_matches.len()
                    ),
                });
            };
            let method_name = format!("next{}{}", naming::pascal(event_stem), result_suffix);
            let function_name = format!(
                "subscript_typegpu_{}_{}",
                naming::snake(receiver),
                naming::snake(&method_name)
            );
            (method_name, function_name, *record, false)
        }
        _ => {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!("unsupported fill-record drain pattern `{pattern}`"),
            })
        }
    };
    let function = mirror
        .functions
        .get(&function_name)
        .ok_or_else(|| unknown(&format!("mirror.{function_name}")))?;
    validate_parameter_types(
        function,
        &[
            "SubscriptTypegpuDevice".to_owned(),
            format!("{} | null", record.boundary_name),
        ],
    )?;
    if function.return_type != "boolean" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{function_name}"),
            message: "fill-record drain must return boolean".to_owned(),
        });
    }
    Ok(MethodPlan::RecordDrain {
        name: method_name,
        function: function_name,
        record: record.boundary_name.clone(),
        result_class: record.name.clone(),
        conversion: format!("from{}", record.boundary_name),
        seed_values: record.seed_values.clone(),
        pump,
    })
}

fn build_async_method(
    mirror: &MirrorModel,
    policy: &ApiSection,
    member: &IdlMember,
    receiver: &str,
    deviation: Option<&ApiDeviationRow>,
    descriptors: &[DescriptorPlan],
    enums: &[EnumPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Operation {
        return_type,
        arguments,
    } = &member.kind
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "async-request requires an IDL operation".to_owned(),
        });
    };
    let dropped: BTreeSet<&str> = deviation
        .map(|row| row.drop_arguments.iter().map(String::as_str).collect())
        .unwrap_or_default();
    let kept = arguments
        .iter()
        .filter(|argument| !dropped.contains(argument.name.as_str()))
        .collect::<Vec<_>>();
    let begin = deviation
        .and_then(|row| row.begin_function.clone())
        .unwrap_or_else(|| {
            format!(
                "subscript_typegpu_{}_{}",
                naming::snake(receiver),
                naming::snake(&member.name)
            )
        });
    let begin_fn = mirror
        .functions
        .get(&begin)
        .ok_or_else(|| unknown(&format!("mirror.{begin}")))?;
    let receiver_types = if receiver == "Instance" {
        vec!["SubscriptTypegpuInstance".to_owned()]
    } else {
        vec![
            "SubscriptTypegpuInstance".to_owned(),
            format!("SubscriptTypegpu{receiver}"),
        ]
    };
    let receiver_only_types = [format!("SubscriptTypegpu{receiver}")];
    let (mirror_arguments, mut begin_args) = if begin_fn
        .params
        .iter()
        .map(|param| param.ty.as_str())
        .take(receiver_types.len())
        .eq(receiver_types.iter().map(String::as_str))
    {
        let args = if receiver == "Instance" {
            vec!["this.instance".to_owned()]
        } else {
            vec![
                "this.instance".to_owned(),
                format!("this.{}", lower_first(receiver)),
            ]
        };
        (&begin_fn.params[receiver_types.len()..], args)
    } else if receiver != "Instance"
        && begin_fn
            .params
            .iter()
            .map(|param| param.ty.as_str())
            .take(1)
            .eq(receiver_only_types.iter().map(String::as_str))
    {
        (
            &begin_fn.params[1..],
            vec![format!("this.{}", lower_first(receiver))],
        )
    } else {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{begin}"),
            message: format!(
                "async begin receiver prefix does not match `{receiver}` future protocol"
            ),
        });
    };
    if mirror_arguments.len() != kept.len() {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!(
                "async begin exposes {} arguments but policy keeps {} IDL arguments",
                mirror_arguments.len(),
                kept.len()
            ),
        });
    }
    validate_argument_names(member, &kept, mirror_arguments)?;
    let params = kept
        .iter()
        .zip(mirror_arguments)
        .map(|(argument, parameter)| {
            let base_type = parameter.ty.trim_end_matches(" | null");
            if let Some(plan) = descriptors
                .iter()
                .find(|descriptor| descriptor.boundary_name == base_type)
            {
                let IdlType::Named {
                    name,
                    nullable: false,
                } = &argument.ty
                else {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.argument.{}", member.key(), argument.name),
                        message: format!(
                            "async descriptor argument must be the non-null IDL type `{}`",
                            plan.idl_type
                        ),
                    });
                };
                if name != &plan.idl_type && name != &plan.idl_name {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.argument.{}", member.key(), argument.name),
                        message: format!(
                            "IDL descriptor `{name}` does not join mirror descriptor `{base_type}`"
                        ),
                    });
                }
                if argument.optional && !parameter.ty.ends_with(" | null") {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.argument.{}", member.key(), argument.name),
                        message: format!(
                            "optional async descriptor `{name}` requires nullable mirror parameter `{}`",
                            parameter.ty
                        ),
                    });
                }
                if argument.optional && argument.default.as_deref() != Some("{}") {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.argument.{}", member.key(), argument.name),
                        message:
                            "optional async descriptors require the pinned empty-dictionary default"
                                .to_owned(),
                    });
                }
                return Ok(MethodParamPlan {
                    name: argument.name.clone(),
                    api_type: plan.name.clone(),
                    expression: format!("to{}({})", plan.boundary_name, argument.name),
                    default: argument.default.clone(),
                    helper: None,
                });
            }
            method_param_from_mirror(
                mirror,
                policy,
                &member.key(),
                argument,
                parameter,
                enums,
                deviation.is_some(),
                deviation.is_some(),
                false,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    begin_args.extend(params.iter().map(|param| param.expression.clone()));
    if begin_fn.return_type != "SubscriptTypegpuFutureId" {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{begin}"),
            message: "async begin must return SubscriptTypegpuFutureId".to_owned(),
        });
    }
    let inner = match return_type {
        IdlType::Promise(inner) => inner.as_ref(),
        _ => {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: "async-request requires an IDL Promise result".to_owned(),
            })
        }
    };
    let nullable_override = deviation.is_some_and(|row| row.nullable_return);
    let boolean_result = deviation.is_some_and(|row| row.boolean_result);
    match inner {
        IdlType::Named { name, nullable } => {
            if boolean_result {
                return Err(ApiPolicyError::Invalid {
                    entry: member.key(),
                    message: "handle-returning promise cannot use boolean_result".to_owned(),
                });
            }
            let take = format!("subscript_typegpu_{}_take", naming::snake(&member.name));
            let take_fn = mirror
                .functions
                .get(&take)
                .ok_or_else(|| unknown(&format!("mirror.{take}")))?;
            validate_parameter_types(
                take_fn,
                &[
                    "SubscriptTypegpuInstance".to_owned(),
                    "SubscriptTypegpuFutureId".to_owned(),
                ],
            )?;
            let expected = format!("SubscriptTypegpu{}", name.trim_start_matches("GPU"));
            if take_fn.return_type != expected {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{take}"),
                    message: format!(
                        "take result is `{}`, expected `{expected}`",
                        take_fn.return_type
                    ),
                });
            }
            Ok(MethodPlan::Async {
                name: member.name.clone(),
                begin,
                params,
                begin_args,
                take: Some(take),
                result_class: Some(name.clone()),
                nullable: *nullable || nullable_override,
                boolean_result: false,
            })
        }
        IdlType::Undefined if boolean_result => Ok(MethodPlan::Async {
            name: member.name.clone(),
            begin,
            params,
            begin_args,
            take: None,
            result_class: None,
            nullable: false,
            boolean_result: true,
        }),
        _ => Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "unsupported async result in the selected API slice".to_owned(),
        }),
    }
}

fn build_attribute_method(
    mirror: &MirrorModel,
    policy: &ApiSection,
    member: &IdlMember,
    receiver: &str,
    enums: &[EnumPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Attribute { ty } = &member.kind else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "attribute-method requires an IDL attribute".to_owned(),
        });
    };
    let getter = format!(
        "subscript_typegpu_{}_get_{}",
        naming::snake(receiver),
        naming::snake(member.name.as_str())
    );
    let function = mirror
        .functions
        .get(&getter)
        .ok_or_else(|| unknown(&format!("mirror.{getter}")))?;
    validate_parameter_types(function, &[format!("SubscriptTypegpu{receiver}")])?;
    let enum_plan = match ty {
        IdlType::Named {
            name,
            nullable: false,
        } => enums.iter().find(|plan| plan.name == *name),
        _ => None,
    };
    let (return_type, result_class, enum_conversion) = if let Some(enum_plan) = enum_plan {
        if function.return_type != enum_plan.mirror_name {
            return Err(ApiPolicyError::Invalid {
                entry: member.key(),
                message: format!(
                    "IDL enum `{}` joins `{}`, but the mirror getter returns `{}`",
                    enum_plan.name, enum_plan.mirror_name, function.return_type
                ),
            });
        }
        (enum_plan.name.clone(), None, None)
    } else {
        let (return_type, result_class) =
            api_return_from_mirror(mirror, policy, &function.return_type)?;
        (return_type, result_class, None)
    };
    Ok(MethodPlan::Attribute {
        name: member.name.clone(),
        getter,
        return_type,
        result_class,
        enum_conversion,
    })
}

fn build_operation_method(
    mirror: &MirrorModel,
    policy: &ApiSection,
    member: &IdlMember,
    receiver: &str,
    deviation: Option<&ApiDeviationRow>,
    descriptors: &[DescriptorPlan],
    enums: &[EnumPlan],
) -> Result<MethodPlan, ApiPolicyError> {
    let IdlMemberKind::Operation {
        return_type,
        arguments,
    } = &member.kind
    else {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "operation pattern requires an IDL operation".to_owned(),
        });
    };
    if matches!(return_type, IdlType::Promise(_)) {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: "Promise operations require async-request".to_owned(),
        });
    }
    let function_name = format!(
        "subscript_typegpu_{}_{}",
        naming::snake(receiver),
        naming::snake(&member.name)
    );
    let function = mirror
        .functions
        .get(&function_name)
        .ok_or_else(|| unknown(&format!("mirror.{function_name}")))?;
    let receiver_type = format!("SubscriptTypegpu{receiver}");
    let first = function
        .params
        .first()
        .ok_or_else(|| ApiPolicyError::Invalid {
            entry: format!("mirror.{function_name}"),
            message: "method boundary has no receiver parameter".to_owned(),
        })?;
    if first.ty != receiver_type {
        return Err(ApiPolicyError::Invalid {
            entry: format!("mirror.{function_name}"),
            message: format!("receiver is `{}`, expected `{receiver_type}`", first.ty),
        });
    }
    let dropped = deviation.map_or(&[][..], |row| row.drop_arguments.as_slice());
    let kept = arguments
        .iter()
        .filter(|argument| !dropped.contains(&argument.name))
        .collect::<Vec<_>>();
    let mirror_params = &function.params[1..];
    if mirror_params.len() != kept.len() {
        return Err(ApiPolicyError::Invalid {
            entry: member.key(),
            message: format!(
                "mirror exposes {} non-receiver parameters but the policy keeps {} IDL arguments",
                mirror_params.len(),
                kept.len()
            ),
        });
    }
    let joined = join_operation_arguments(member, &kept, mirror_params, deviation)?;
    let mut plans_by_name = BTreeMap::new();
    let mut call_args = Vec::new();
    let mut default_descriptor_expression = None;
    for (argument, parameter) in joined {
        let base_type = parameter.ty.trim_end_matches(" | null");
        if let Some(plan) = descriptors
            .iter()
            .find(|descriptor| descriptor.boundary_name == base_type)
        {
            let default_variant = deviation.and_then(|row| row.default_variant.as_deref());
            if default_variant.is_some()
                && (!argument.optional || argument.default.as_deref() != Some("{}"))
            {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{}.argument.{}", member.key(), argument.name),
                    message: "default_variant requires an optional IDL descriptor defaulted to {}"
                        .to_owned(),
                });
            }
            let expression = format!("to{}({})", plan.boundary_name, argument.name);
            if default_variant.is_some() {
                default_descriptor_expression = Some(expression.clone());
            }
            if argument.optional && argument.default.as_deref() == Some("{}") {
                if !parameter.ty.ends_with(" | null") {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{}.argument.{}", member.key(), argument.name),
                        message: format!(
                            "optional operation descriptor `{}` requires nullable mirror parameter `{}`",
                            plan.idl_type, parameter.ty
                        ),
                    });
                }
                let expression = format!("to{}({})", plan.boundary_name, argument.name);
                let param = MethodParamPlan {
                    name: argument.name.clone(),
                    api_type: format!("{} | null", plan.name),
                    expression: expression.clone(),
                    default: Some("null".to_owned()),
                    helper: Some(MethodParamHelper::NullableDescriptor {
                        boundary_name: plan.boundary_name.clone(),
                        api_name: plan.name.clone(),
                    }),
                };
                call_args.push(expression);
                plans_by_name.insert(argument.name.clone(), param);
                continue;
            }
            let param = MethodParamPlan {
                name: argument.name.clone(),
                api_type: plan.name.clone(),
                expression: expression.clone(),
                default: None,
                helper: None,
            };
            call_args.push(expression);
            plans_by_name.insert(argument.name.clone(), param);
        } else {
            let (allow_named_deviation, allow_scalar_deviation) = deviation
                .map_or((false, false), |row| {
                    operation_argument_deviation(row, &argument.name)
                });
            let param = method_param_from_mirror(
                mirror,
                policy,
                &member.key(),
                argument,
                parameter,
                enums,
                allow_named_deviation,
                allow_scalar_deviation,
                deviation.is_some_and(|row| row.required_arguments.contains(&argument.name)),
            )?;
            call_args.push(param.expression.clone());
            plans_by_name.insert(argument.name.clone(), param);
        }
    }
    let params = kept
        .iter()
        .map(|argument| {
            plans_by_name
                .remove(&argument.name)
                .ok_or_else(|| unknown(&format!("{}.argument.{}", member.key(), argument.name)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (api_return, result_class) = api_return_from_mirror(mirror, policy, &function.return_type)?;
    let return_type = if function.return_type == "void" {
        "void".to_owned()
    } else {
        api_return
    };
    Ok(MethodPlan::Operation {
        name: member.name.clone(),
        function: function_name,
        params,
        return_type,
        result_class,
        call_args,
        default_variant: deviation
            .and_then(|row| row.default_variant.clone())
            .map(|name| {
                default_descriptor_expression
                    .clone()
                    .map(|descriptor_expression| DefaultVariantPlan {
                        name,
                        descriptor_expression,
                    })
                    .ok_or_else(|| ApiPolicyError::Invalid {
                        entry: member.key(),
                        message: "default_variant has no descriptor argument".to_owned(),
                    })
            })
            .transpose()?,
    })
}

fn operation_argument_deviation(row: &ApiDeviationRow, argument: &str) -> (bool, bool) {
    if row.drop_arguments.iter().any(|name| name == argument) {
        return (true, true);
    }
    let mapping = row.boundary_arguments.iter().find_map(|mapping| {
        let (idl_name, mirror_name) = mapping.split_once('=')?;
        (idl_name == argument).then_some((idl_name, mirror_name))
    });
    match mapping {
        Some(_) => (true, true),
        None => (false, false),
    }
}

fn join_operation_arguments<'a>(
    member: &IdlMember,
    arguments: &[&'a IdlArgument],
    parameters: &'a [MirrorParam],
    deviation: Option<&ApiDeviationRow>,
) -> Result<Vec<(&'a IdlArgument, &'a MirrorParam)>, ApiPolicyError> {
    let mappings = deviation.map_or(&[][..], |row| row.boundary_arguments.as_slice());
    if mappings.is_empty() {
        validate_argument_names(member, arguments, parameters)?;
        return Ok(arguments.iter().copied().zip(parameters.iter()).collect());
    }
    let mut mapped_arguments = BTreeMap::new();
    let mut mapped_parameters = BTreeSet::new();
    for mapping in mappings {
        let (idl_name, mirror_name) =
            mapping
                .split_once('=')
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: format!("{}.arguments", member.key()),
                    message: format!("malformed boundary argument mapping `{mapping}`"),
                })?;
        if mapped_arguments.contains_key(idl_name) {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.argument.{idl_name}", member.key()),
            });
        }
        if !mapped_parameters.insert(mirror_name) {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("mirror.{}.parameter.{mirror_name}", member.key()),
            });
        }
        let argument = arguments
            .iter()
            .find(|argument| argument.name == idl_name)
            .copied()
            .ok_or_else(|| unknown(&format!("{}.argument.{idl_name}", member.key())))?;
        if !parameters
            .iter()
            .any(|parameter| parameter.name == mirror_name)
        {
            return Err(unknown(&format!(
                "mirror.{}.parameter.{mirror_name}",
                member.key()
            )));
        }
        mapped_arguments.insert(idl_name, (mirror_name, argument));
    }
    let mut consumed_arguments = BTreeSet::new();
    let mut joined = Vec::new();
    for parameter in parameters {
        let argument = mapped_arguments
            .values()
            .find_map(|(mirror_name, argument)| {
                (mirror_name == &parameter.name).then_some(*argument)
            })
            .or_else(|| {
                arguments
                    .iter()
                    .find(|argument| argument.name == parameter.name)
                    .copied()
            })
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: format!("{}.arguments", member.key()),
                message: format!(
                    "mirror parameter `{}` has no explicit or same-named IDL argument join",
                    parameter.name
                ),
            })?;
        if !consumed_arguments.insert(argument.name.as_str()) {
            return Err(ApiPolicyError::Duplicate {
                entry: format!("{}.argument.{}", member.key(), argument.name),
            });
        }
        joined.push((argument, parameter));
    }
    if consumed_arguments.len() != arguments.len() {
        let unjoined = arguments
            .iter()
            .find(|argument| !consumed_arguments.contains(argument.name.as_str()))
            .ok_or_else(|| unknown(&format!("{}.arguments", member.key())))?;
        return Err(ApiPolicyError::Invalid {
            entry: format!("{}.argument.{}", member.key(), unjoined.name),
            message: "IDL argument has no explicit or same-named mirror parameter join".to_owned(),
        });
    }
    Ok(joined)
}

fn validate_argument_names(
    member: &IdlMember,
    arguments: &[&IdlArgument],
    parameters: &[MirrorParam],
) -> Result<(), ApiPolicyError> {
    let idl_names = arguments
        .iter()
        .map(|argument| argument.name.as_str())
        .collect::<Vec<_>>();
    let mirror_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>();
    if idl_names == mirror_names {
        Ok(())
    } else {
        Err(ApiPolicyError::Invalid {
            entry: format!("{}.arguments", member.key()),
            message: format!(
                "kept IDL argument names {idl_names:?} do not match mirror parameter names {mirror_names:?} in order"
            ),
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn method_param_from_mirror(
    mirror: &MirrorModel,
    policy: &ApiSection,
    member_key: &str,
    argument: &IdlArgument,
    parameter: &MirrorParam,
    enums: &[EnumPlan],
    allow_named_deviation: bool,
    allow_scalar_deviation: bool,
    keep_required: bool,
) -> Result<MethodParamPlan, ApiPolicyError> {
    let parameter_nullable = parameter.ty.ends_with(" | null");
    let base_type = parameter.ty.trim_end_matches(" | null");
    if let IdlType::Scalar(expected) = &argument.ty {
        if parameter_nullable || base_type != expected {
            if !allow_scalar_deviation {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{member_key}.argument.{}", argument.name),
                    message: format!(
                        "IDL scalar `{expected}` does not match mirror parameter `{}`",
                        parameter.ty
                    ),
                });
            }
        } else {
            return Ok(MethodParamPlan {
                name: argument.name.clone(),
                api_type: expected.clone(),
                expression: argument.name.clone(),
                default: (!keep_required).then(|| argument.default.clone()).flatten(),
                helper: None,
            });
        }
    }
    if matches!(argument.ty, IdlType::Boolean | IdlType::String) {
        let expected = if matches!(argument.ty, IdlType::Boolean) {
            "boolean"
        } else {
            "string"
        };
        if parameter_nullable || base_type != expected {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{member_key}.argument.{}", argument.name),
                message: format!(
                    "IDL `{expected}` does not match mirror parameter `{}`",
                    parameter.ty
                ),
            });
        }
        return Ok(MethodParamPlan {
            name: argument.name.clone(),
            api_type: expected.to_owned(),
            expression: argument.name.clone(),
            default: (!keep_required).then(|| argument.default.clone()).flatten(),
            helper: None,
        });
    }
    if let IdlType::Sequence(inner) = &argument.ty {
        if let IdlType::Scalar(element) = inner.as_ref() {
            let expected = format!("{element}[]");
            if parameter.ty != expected {
                if !allow_scalar_deviation {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{member_key}.argument.{}", argument.name),
                        message: format!(
                            "IDL scalar sequence `{expected}` does not match mirror parameter `{}`",
                            parameter.ty
                        ),
                    });
                }
            } else {
                return Ok(MethodParamPlan {
                    name: argument.name.clone(),
                    api_type: expected,
                    expression: argument.name.clone(),
                    default: (!keep_required).then(|| argument.default.clone()).flatten(),
                    helper: None,
                });
            }
        }
        if let IdlType::Named { name, nullable } = inner.as_ref() {
            let boundary_name = format!("SubscriptTypegpu{}", name.trim_start_matches("GPU"));
            if mirror.handles.contains(&boundary_name) {
                if *nullable {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{member_key}.argument.{}", argument.name),
                        message:
                            "parameter-position handle arrays cannot contain nullable elements"
                                .to_owned(),
                    });
                }
                let expected = format!("{boundary_name}[]");
                if parameter.ty != expected {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("{member_key}.argument.{}", argument.name),
                        message: format!(
                            "IDL handle sequence `{name}` joins `{expected}`, but the mirror parameter declares `{}`",
                            parameter.ty
                        ),
                    });
                }
                let api = policy
                    .interfaces
                    .iter()
                    .find(|interface| *interface == name)
                    .ok_or_else(|| ApiPolicyError::Invalid {
                        entry: format!("{member_key}.argument.{}", argument.name),
                        message: format!(
                            "mirror handle `{boundary_name}` has no selected public wrapper"
                        ),
                    })?;
                let raw_field = raw_field_for_api(api, policy);
                return Ok(MethodParamPlan {
                    name: argument.name.clone(),
                    api_type: format!("{api}[]"),
                    expression: format!("to{boundary_name}Array({})", argument.name),
                    default: (!keep_required).then(|| argument.default.clone()).flatten(),
                    helper: Some(MethodParamHelper::HandleArray {
                        boundary_name,
                        api_name: api.clone(),
                        raw_field,
                    }),
                });
            }
        }
    }
    if let IdlType::Named { name, nullable } = &argument.ty {
        if let Some(enum_plan) = enums.iter().find(|plan| plan.name == *name) {
            if base_type != enum_plan.mirror_name {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{member_key}.argument.{}", argument.name),
                    message: format!(
                        "IDL enum `{name}` joins `{}`, but the mirror declares `{}`",
                        enum_plan.mirror_name, parameter.ty
                    ),
                });
            }
            return Ok(MethodParamPlan {
                name: argument.name.clone(),
                api_type: name.clone(),
                expression: argument.name.clone(),
                default: (!keep_required).then(|| argument.default.clone()).flatten(),
                helper: None,
            });
        }
        if mirror.handles.contains(base_type) {
            let expected = format!("SubscriptTypegpu{}", name.trim_start_matches("GPU"));
            if !allow_named_deviation && base_type != expected {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{member_key}.argument.{}", argument.name),
                    message: format!(
                        "IDL named type `{name}` joins `{expected}`, but the mirror declares `{}`",
                        parameter.ty
                    ),
                });
            }
            let api = policy
                .interfaces
                .iter()
                .find(|interface| {
                    format!("SubscriptTypegpu{}", interface.trim_start_matches("GPU")) == base_type
                })
                .ok_or_else(|| ApiPolicyError::Invalid {
                    entry: format!("{member_key}.argument.{}", argument.name),
                    message: format!("mirror handle `{base_type}` has no selected public wrapper"),
                })?;
            if parameter_nullable != *nullable {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{member_key}.argument.{}", argument.name),
                    message: format!(
                        "IDL handle nullability `{nullable}` does not match mirror parameter `{}`",
                        parameter.ty
                    ),
                });
            }
            let raw_field = raw_field_for_api(api, policy);
            if parameter_nullable {
                return Ok(MethodParamPlan {
                    name: argument.name.clone(),
                    api_type: format!("{api} | null"),
                    expression: format!("toNullable{base_type}({})", argument.name),
                    default: (!keep_required).then(|| argument.default.clone()).flatten(),
                    helper: Some(MethodParamHelper::NullableHandle {
                        boundary_name: base_type.to_owned(),
                        api_name: api.clone(),
                        raw_field,
                    }),
                });
            }
            return Ok(MethodParamPlan {
                name: argument.name.clone(),
                api_type: api.clone(),
                expression: format!("{}.{}", argument.name, raw_field),
                default: (!keep_required).then(|| argument.default.clone()).flatten(),
                helper: None,
            });
        }
        if !allow_named_deviation {
            let expected = format!("SubscriptTypegpu{}", name.trim_start_matches("GPU"));
            if base_type != expected {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{member_key}.argument.{}", argument.name),
                    message: format!(
                        "IDL named type `{name}` joins `{expected}`, but the mirror declares `{}`",
                        parameter.ty
                    ),
                });
            }
        }
    }
    let (api_type, wrapper) = api_return_from_mirror(mirror, policy, base_type)?;
    let expression = wrapper.map_or_else(
        || argument.name.clone(),
        |wrapper| format!("{}.{}", argument.name, raw_field_for_api(&wrapper, policy)),
    );
    Ok(MethodParamPlan {
        name: argument.name.clone(),
        api_type,
        expression,
        default: (!keep_required).then(|| argument.default.clone()).flatten(),
        helper: None,
    })
}

fn api_return_from_mirror(
    mirror: &MirrorModel,
    policy: &ApiSection,
    ty: &str,
) -> Result<(String, Option<String>), ApiPolicyError> {
    if let Some(boundary) = ty.strip_prefix("SubscriptTypegpu") {
        if mirror.handles.contains(ty) {
            let api = policy
                .interfaces
                .iter()
                .find(|interface| {
                    if *interface == &policy.singleton_interface {
                        policy.singleton_boundary == boundary
                    } else {
                        interface.trim_start_matches("GPU") == boundary
                    }
                })
                .cloned()
                .unwrap_or_else(|| format!("GPU{boundary}"));
            return Ok((api.clone(), Some(api)));
        }
    }
    Ok((api_type_from_mirror(mirror, ty)?, None))
}

fn api_type_from_mirror(mirror: &MirrorModel, ty: &str) -> Result<String, ApiPolicyError> {
    let mut chain = Vec::new();
    let mut current = ty;
    while let Some(next) = mirror.aliases.get(current) {
        if chain.iter().any(|seen| seen == current) {
            chain.push(current.to_owned());
            return Err(ApiPolicyError::Invalid {
                entry: format!("mirror.alias.{ty}"),
                message: format!("type-alias cycle detected: {}", chain.join(" -> ")),
            });
        }
        chain.push(current.to_owned());
        current = next;
    }
    Ok(current.to_owned())
}

fn compute_instance_needs(interfaces: &mut [InterfacePlan], singleton: &str) {
    for interface in interfaces.iter_mut() {
        interface.needs_instance = interface.name == singleton
            || interface.methods.iter().any(|method| {
                matches!(
                    method,
                    MethodPlan::Async { .. } | MethodPlan::ErrorScopePop { .. }
                )
            });
    }
    loop {
        let needs: BTreeSet<String> = interfaces
            .iter()
            .filter(|interface| interface.needs_instance)
            .map(|interface| interface.name.clone())
            .collect();
        let mut changed = false;
        for interface in interfaces.iter_mut() {
            if !interface.needs_instance
                && interface.methods.iter().any(|method| {
                    method
                        .result_class()
                        .is_some_and(|result| needs.contains(result))
                })
            {
                interface.needs_instance = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

fn render(plan: &ApiPlan) -> Result<String, ApiPolicyError> {
    let default_helpers = collect_default_helpers(&plan.descriptors, &plan.policy)?;
    let mut out = String::from(
        "// GENERATED FILE — DO NOT EDIT.\n\
         // The API layer is emitted from pinned GPUWeb IDL, the subscript-typegpu.h mirror, and API policy.\n\
         // Boundary handles and future polling are implementation details.\n//\n",
    );
    out.push_str("// API policy deviations:");
    out.push('\n');
    out.push_str(&format!(
        "// - *.dispose: {}\n",
        plan.policy.manual_dispose_reason
    ));
    out.push_str(&format!(
        "// - {}: {}\n",
        plan.policy.singleton_name, plan.policy.singleton_reason
    ));
    for row in &plan.policy.deviations {
        out.push_str(&format!("// - {}: {}\n", row.member, row.reason));
    }
    for namespace in &plan.policy.namespaces {
        out.push_str(&format!(
            "// - {namespace}: {}\n",
            plan.policy.namespace_reason
        ));
    }
    for row in &plan.policy.dictionary_mappings {
        out.push_str(&format!("// - {}: {}\n", row.idl_type, row.reason));
    }
    for row in &plan.policy.dictionary_aliases {
        out.push_str(&format!("// - {}: {}\n", row.dictionary, row.reason));
    }
    for row in &plan.policy.public_only_dictionaries {
        out.push_str(&format!("// - {}: {}\n", row.dictionary, row.reason));
    }
    for row in &plan.policy.enum_exclusions {
        let enum_plan = plan
            .enums
            .iter()
            .find(|enum_plan| enum_plan.name == row.enum_name)
            .ok_or_else(|| unknown(&format!("{}.{}", row.enum_name, row.mirror)))?;
        let member = enum_plan
            .exclusions
            .iter()
            .find(|member| member.mirror_name == row.mirror)
            .ok_or_else(|| unknown(&format!("{}.{}", row.enum_name, row.mirror)))?;
        out.push_str(&format!(
            "// - {}.{:?}: the generated CEnum alias exposes boundary-only string member {:?} for facade wire value {}; the pinned WebIDL enum does not declare it\n",
            row.enum_name, member.idl_name, member.idl_name, member.wire_value
        ));
    }
    for row in &plan.policy.result_records {
        out.push_str(&format!("// - {}: {}\n", row.interface, row.reason));
    }
    for row in &plan.policy.flattened_interfaces {
        out.push_str(&format!("// - {}: {}\n", row.interface, row.reason));
    }
    out.push('\n');

    for record in &plan.result_records {
        render_result_record(&mut out, record)?;
    }
    render_record_entries(&mut out, &plan.descriptors)?;
    render_required_limits(&mut out, &plan.descriptors)?;
    for descriptor in &plan.descriptors {
        render_descriptor(&mut out, descriptor)?;
    }
    render_descriptor_helpers(&mut out, &plan.descriptors);
    render_method_param_helpers(&mut out, &plan.descriptors, &plan.interfaces);
    for descriptor in &plan.descriptors {
        if !descriptor.public_only {
            render_descriptor_conversion(&mut out, descriptor, &plan.policy)?;
        }
    }
    for namespace in &plan.namespaces {
        render_namespace(&mut out, namespace);
    }
    if !default_helpers.is_empty() {
        out.push_str(
            "// TypeScript reads Q33 optional fields as `T | undefined`, while subscript\n\
             // applies descriptor defaults before reads; default parameters bridge both.\n",
        );
        for helper in &default_helpers {
            out.push_str(&format!(
                "function default{}(value: {} = {}): {} {{\n  return value;\n}}\n\n",
                helper.name, helper.ty, helper.default, helper.ty,
            ));
        }
    }
    let needs: BTreeMap<&str, bool> = plan
        .interfaces
        .iter()
        .map(|interface| (interface.name.as_str(), interface.needs_instance))
        .collect();
    for interface in &plan.interfaces {
        render_interface(&mut out, interface, &needs)?;
    }
    out.push_str(&format!(
        "export const {}: {} = new {}(subscript_typegpu_create_{}());\n",
        plan.policy.singleton_name,
        plan.policy.singleton_interface,
        plan.policy.singleton_interface,
        naming::snake(&plan.policy.singleton_boundary),
    ));
    Ok(out)
}

fn render_result_record(out: &mut String, record: &ResultRecordPlan) -> Result<(), ApiPolicyError> {
    if let Some(enum_plan) = &record.synthetic_enum {
        out.push_str(&format!(
            "export type {} = {};\n\n",
            enum_plan.name,
            enum_plan
                .members
                .iter()
                .map(|member| format!("{:?}", member.idl_name))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    out.push_str(&format!("export class {} {{\n", record.name));
    for field in &record.fields {
        out.push_str(&format!("  readonly {}: {};\n", field.name, field.ty));
    }
    out.push_str("\n  constructor(");
    out.push_str(
        &record
            .fields
            .iter()
            .map(|field| format!("{}: {}", field.name, field.ty))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str(") {\n");
    for field in &record.fields {
        out.push_str(&format!("    this.{0} = {0};\n", field.name));
    }
    out.push_str("  }\n}\n\n");

    out.push_str(&format!(
        "function from{}(value: {}): {}{} {{\n",
        record.boundary_name,
        record.boundary_name,
        record.name,
        if record.nullable { " | null" } else { "" }
    ));
    if let Some(enum_plan) = &record.synthetic_enum {
        let synthetic_field = record
            .fields
            .iter()
            .find(|field| matches!(field.conversion, ResultRecordFieldConversion::SyntheticEnum))
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: record.name.clone(),
                message: "synthetic result enum has no synthetic result field".to_owned(),
            })?;
        out.push_str(&format!("  switch (value.{}) {{\n", synthetic_field.name));
        for member in &enum_plan.members {
            out.push_str(&format!(
                "    case {}.{}:\n      return new {}(\n",
                enum_plan.mirror_name, member.mirror_name, record.name
            ));
            for field in &record.fields {
                let expression = if field.name == synthetic_field.name {
                    format!("{:?}", member.idl_name)
                } else {
                    result_record_field_expression(field)?
                };
                out.push_str(&format!("        {expression},\n"));
            }
            out.push_str("      );\n");
        }
        for exclusion in &enum_plan.exclusions {
            out.push_str(&format!(
                "    case {}.{}:\n      return null;\n",
                enum_plan.mirror_name, exclusion
            ));
        }
        out.push_str("  }\n  unreachable();\n");
    } else {
        out.push_str(&format!("  return new {}(\n", record.name));
        for field in &record.fields {
            out.push_str(&format!(
                "    {},\n",
                result_record_field_expression(field)?
            ));
        }
        out.push_str("  );\n");
    }
    out.push_str("}\n\n");
    Ok(())
}

fn result_record_field_expression(field: &ResultRecordFieldPlan) -> Result<String, ApiPolicyError> {
    Ok(match &field.conversion {
        ResultRecordFieldConversion::Direct => format!("value.{}", field.name),
        ResultRecordFieldConversion::Enum => format!("value.{}", field.name),
        ResultRecordFieldConversion::SyntheticEnum => {
            return Err(ApiPolicyError::Invalid {
                entry: field.name.clone(),
                message: "synthetic result field requires a selected switch arm".to_owned(),
            })
        }
    })
}

fn render_method_param_helpers(
    out: &mut String,
    descriptors: &[DescriptorPlan],
    interfaces: &[InterfacePlan],
) {
    let mut emitted = BTreeSet::new();
    for field in descriptors
        .iter()
        .flat_map(|descriptor| descriptor.fields.iter())
    {
        match &field.conversion {
            DescriptorFieldConversion::HandleArray { boundary_name, .. } => {
                emitted.insert(format!("handle-array:{boundary_name}"));
            }
            DescriptorFieldConversion::NullableHandle { boundary_name, .. } => {
                emitted.insert(format!("nullable-handle:{boundary_name}"));
            }
            _ => {}
        }
    }
    for helper in interfaces
        .iter()
        .flat_map(|interface| interface.methods.iter())
        .flat_map(method_params)
        .filter_map(|parameter| parameter.helper.as_ref())
    {
        match helper {
            MethodParamHelper::HandleArray {
                boundary_name,
                api_name,
                raw_field,
            } => {
                if emitted.insert(format!("handle-array:{boundary_name}")) {
                    out.push_str(&format!(
                        "function to{boundary_name}Array(values: {api_name}[]): {boundary_name}[] {{\n  const lowered: {boundary_name}[] = [];\n  let index: i32 = 0;\n  while (index < values.length) {{\n    lowered.push(values[index].{raw_field});\n    index = index + 1;\n  }}\n  return lowered;\n}}\n\n"
                    ));
                }
            }
            MethodParamHelper::NullableHandle {
                boundary_name,
                api_name,
                raw_field,
            } => {
                if emitted.insert(format!("nullable-handle:{boundary_name}")) {
                    out.push_str(&format!(
                        "function toNullable{boundary_name}(value: {api_name} | null): {boundary_name} | null {{\n  if (value === null) {{\n    return null;\n  }}\n  return value.{raw_field};\n}}\n\n"
                    ));
                }
            }
            MethodParamHelper::NullableDescriptor { .. } => {}
        }
    }
}

fn method_params(method: &MethodPlan) -> &[MethodParamPlan] {
    match method {
        MethodPlan::Async { params, .. } | MethodPlan::Operation { params, .. } => params,
        MethodPlan::Attribute { .. }
        | MethodPlan::MappedRange { .. }
        | MethodPlan::TypedWriteF32 { .. }
        | MethodPlan::TypedReadF32 { .. }
        | MethodPlan::ErrorScopePop { .. }
        | MethodPlan::RecordDrain { .. }
        | MethodPlan::RecordFill { .. } => &[],
    }
}

fn render_record_entries(
    out: &mut String,
    descriptors: &[DescriptorPlan],
) -> Result<(), ApiPolicyError> {
    let mut entries = BTreeMap::new();
    for field in descriptors
        .iter()
        .flat_map(|descriptor| descriptor.fields.iter())
    {
        let DescriptorFieldConversion::RecordEntries {
            api_name,
            boundary_name,
        } = &field.conversion
        else {
            continue;
        };
        if let Some(previous) = entries.insert(boundary_name, api_name) {
            if previous != api_name {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("mirror.{boundary_name}"),
                    message: format!(
                        "record entry aggregate maps to both `{previous}` and `{api_name}`"
                    ),
                });
            }
        }
    }
    for api_name in entries.values() {
        out.push_str(&format!(
            "@Descriptor\nexport class {api_name} {{\n  key!: string;\n  value!: f64;\n}}\n\n"
        ));
    }
    Ok(())
}

fn render_required_limits(
    out: &mut String,
    descriptors: &[DescriptorPlan],
) -> Result<(), ApiPolicyError> {
    let mut plans = BTreeMap::new();
    for field in descriptors
        .iter()
        .flat_map(|descriptor| descriptor.fields.iter())
    {
        let DescriptorFieldConversion::RequiredLimits {
            api_name,
            boundary_name,
            fields,
            u32_unspecified,
        } = &field.conversion
        else {
            continue;
        };
        let shape = (boundary_name.clone(), fields.clone(), *u32_unspecified);
        if let Some(previous) = plans.insert(api_name.clone(), shape.clone()) {
            if previous != shape {
                return Err(ApiPolicyError::Invalid {
                    entry: api_name.clone(),
                    message: "required-limits public descriptor has conflicting plans".to_owned(),
                });
            }
        }
    }
    if plans.is_empty() {
        return Ok(());
    }
    out.push_str(
        "function defaultRequiredLimitU32(value: u32 = 0): u32 {\n  return value;\n}\n\nfunction defaultRequiredLimitU64(value: u64 = 0): u64 {\n  return value;\n}\n\nfunction toRequiredLimitU32(value: u32): u32 {\n  if (value === 0) {\n    return 4294967295;\n  }\n  return value;\n}\n\n",
    );
    for (api_name, (boundary_name, fields, _)) in plans {
        out.push_str(&format!("@Descriptor\nexport class {api_name} {{\n"));
        for field in &fields {
            out.push_str(&format!("  {}?: {} = 0;\n", field.name, field.ty));
        }
        out.push_str("}\n\n");
        out.push_str(&format!(
            "function to{boundary_name}(value: {api_name}): {boundary_name} {{\n  return new {boundary_name}(\n"
        ));
        for field in &fields {
            if field.ty == "u32" {
                out.push_str(&format!(
                    "    toRequiredLimitU32(defaultRequiredLimitU32(value.{})),\n",
                    field.name
                ));
            } else {
                out.push_str(&format!(
                    "    defaultRequiredLimitU64(value.{}),\n",
                    field.name
                ));
            }
        }
        out.push_str("  );\n}\n\n");
        out.push_str(&format!(
            "function is{api_name}Empty(value: {api_name}): boolean {{\n"
        ));
        for (index, field) in fields.iter().enumerate() {
            let default_helper = if field.ty == "u32" {
                "defaultRequiredLimitU32"
            } else {
                "defaultRequiredLimitU64"
            };
            let conjunction = if index == 0 { "  return " } else { "    && " };
            let terminator = if index + 1 == fields.len() { ";" } else { "" };
            out.push_str(&format!(
                "{conjunction}{default_helper}(value.{}) === 0{terminator}\n",
                field.name,
            ));
        }
        out.push_str("}\n\n");
    }
    Ok(())
}

fn render_wire_enum_aliases(plan: &ApiPlan) -> Result<String, ApiPolicyError> {
    let mut out = String::from(
        "// GENERATED FILE — DO NOT EDIT.\n\
         // Ambient CEnum aliases emitted from the API enum join and subscript_typegpu mirror values.\n\
         // These aliases supply function-position wire mappings to generated mirrors.\n\n",
    );
    for enum_plan in &plan.enums {
        out.push_str(&format!("type {} = CEnum<{{\n", enum_plan.name));
        for member in enum_plan.members.iter().chain(&enum_plan.exclusions) {
            out.push_str(&format!(
                "  {:?}: {},\n",
                member.idl_name, member.wire_value
            ));
        }
        out.push_str("}>;\n\n");
    }
    Ok(out)
}

fn collect_default_helpers(
    descriptors: &[DescriptorPlan],
    policy: &ApiSection,
) -> Result<Vec<DefaultHelperPlan>, ApiPolicyError> {
    let mut helpers = Vec::new();
    let mut seen = BTreeMap::new();
    for descriptor in descriptors {
        for field in &descriptor.fields {
            let Some(default) = &field.default else {
                continue;
            };
            let member = format!("{}.{}", descriptor.idl_name, field.name);
            let name = policy
                .default_helper_renames
                .iter()
                .find(|row| row.member == member)
                .map_or_else(|| naming::pascal(&field.name), |row| row.helper.clone());
            let shape = (field.ty.clone(), default.clone());
            if let Some(previous) = seen.get(&name) {
                if previous != &shape {
                    return Err(ApiPolicyError::Invalid {
                        entry: format!("default{name}"),
                        message: format!(
                            "default-helper collision: {:?} does not match {:?}",
                            shape, previous
                        ),
                    });
                }
                continue;
            }
            seen.insert(name.clone(), shape);
            helpers.push(DefaultHelperPlan {
                name,
                ty: field.ty.clone(),
                default: default.clone(),
            });
        }
    }
    if let Some(row) = policy.default_helper_renames.iter().find(|row| {
        !descriptors.iter().any(|descriptor| {
            descriptor.fields.iter().any(|field| {
                field.default.is_some()
                    && format!("{}.{}", descriptor.idl_name, field.name) == row.member
            })
        })
    }) {
        return Err(ApiPolicyError::Dead {
            entry: format!("{}.default_helper", row.member),
        });
    }
    Ok(helpers)
}

fn render_descriptor(out: &mut String, descriptor: &DescriptorPlan) -> Result<(), ApiPolicyError> {
    out.push_str("@Descriptor\n");
    out.push_str(&format!("export class {} {{\n", descriptor.name));
    for field in &descriptor.fields {
        match (&field.default, field.required, &field.conversion) {
            (Some(default), false, _) => {
                out.push_str(&format!("  {}?: {} = {};\n", field.name, field.ty, default))
            }
            (None, true, _) => out.push_str(&format!("  {}!: {};\n", field.name, field.ty)),
            (None, false, DescriptorFieldConversion::OptionalEnum { .. }) => {
                out.push_str(&format!("  {}?: {};\n", field.name, field.ty))
            }
            _ => {
                return Err(ApiPolicyError::Invalid {
                    entry: format!("{}.{}", descriptor.name, field.name),
                    message: "Q33 optional fields without defaults are restricted to absence-capable Q32 enum members"
                        .to_owned(),
                })
            }
        }
    }
    out.push_str("}\n\n");
    Ok(())
}

fn render_descriptor_conversion(
    out: &mut String,
    descriptor: &DescriptorPlan,
    policy: &ApiSection,
) -> Result<(), ApiPolicyError> {
    for field in &descriptor.fields {
        let DescriptorFieldConversion::OptionalEnum {
            public_name,
            undefined_key,
        } = &field.conversion
        else {
            continue;
        };
        let helper = optional_enum_helper_name(descriptor, field, public_name);
        out.push_str(&format!(
            "function {helper}(value: {}): {public_name} {{\n  if (value.{} !== undefined) {{\n    return value.{};\n  }}\n  return {undefined_key:?};\n}}\n\n",
            descriptor.name, field.name, field.name
        ));
    }
    out.push_str(&format!(
        "function to{}(value: {}): {} {{\n",
        descriptor.boundary_name, descriptor.name, descriptor.boundary_name,
    ));
    let mut optional_aggregate_arguments = BTreeMap::new();
    for field in &descriptor.fields {
        let local = format!("nullable{}", naming::pascal(&field.name));
        let argument = match &field.conversion {
            DescriptorFieldConversion::NullableDescriptor { boundary_name, .. } => {
                format!("{local} !== null ? to{boundary_name}({local}) : null")
            }
            DescriptorFieldConversion::RequiredLimits {
                api_name,
                boundary_name,
                ..
            } => format!("is{api_name}Empty({local}) ? null : to{boundary_name}({local})"),
            _ => continue,
        };
        out.push_str(&format!(
            "  const {local}: {} = {};\n",
            field.ty,
            descriptor_raw_expression(field, descriptor, policy)
        ));
        optional_aggregate_arguments.insert(field.name.clone(), argument);
    }
    render_descriptor_constructor(out, descriptor, policy, &optional_aggregate_arguments, "  ")?;
    out.push_str("}\n\n");
    Ok(())
}

fn descriptor_raw_expression(
    field: &DescriptorFieldPlan,
    descriptor: &DescriptorPlan,
    policy: &ApiSection,
) -> String {
    if field.default.is_some() {
        format!(
            "default{}(value.{})",
            policy
                .default_helper_renames
                .iter()
                .find(|row| row.member == format!("{}.{}", descriptor.idl_name, field.name))
                .map_or_else(|| naming::pascal(&field.name), |row| row.helper.clone()),
            field.name
        )
    } else {
        format!("value.{}", field.name)
    }
}

fn render_descriptor_constructor(
    out: &mut String,
    descriptor: &DescriptorPlan,
    policy: &ApiSection,
    overrides: &BTreeMap<String, String>,
    indent: &str,
) -> Result<(), ApiPolicyError> {
    out.push_str(&format!(
        "{indent}return new {}(\n",
        descriptor.boundary_name
    ));
    for boundary_field in &descriptor.boundary_fields {
        if let Some(value) = descriptor.boundary_defaults.get(&boundary_field.name) {
            out.push_str(&format!("{indent}  {value},\n"));
            continue;
        }
        if let Some(nested) = descriptor
            .nested_boundaries
            .iter()
            .find(|nested| nested.field_name == boundary_field.name)
        {
            out.push_str(&format!("{indent}  new {}(\n", nested.boundary_name));
            for member in &nested.members {
                let field = descriptor
                    .fields
                    .iter()
                    .find(|field| &field.name == member)
                    .ok_or_else(|| unknown(&format!("{}.{}", descriptor.name, member)))?;
                let expression = descriptor_field_expression(field, descriptor, policy, overrides)?;
                out.push_str(&format!("{indent}    {expression},\n"));
            }
            out.push_str(&format!("{indent}  ),\n"));
            continue;
        }
        let field = descriptor
            .fields
            .iter()
            .find(|field| field.name == boundary_field.name)
            .ok_or_else(|| unknown(&format!("{}.{}", descriptor.name, boundary_field.name)))?;
        let expression = descriptor_field_expression(field, descriptor, policy, overrides)?;
        out.push_str(&format!("{indent}  {expression},\n"));
    }
    out.push_str(&format!("{indent});\n"));
    Ok(())
}

fn descriptor_field_expression(
    field: &DescriptorFieldPlan,
    descriptor: &DescriptorPlan,
    policy: &ApiSection,
    overrides: &BTreeMap<String, String>,
) -> Result<String, ApiPolicyError> {
    let raw = descriptor_raw_expression(field, descriptor, policy);
    Ok(match &field.conversion {
        DescriptorFieldConversion::Direct => raw,
        DescriptorFieldConversion::Enum => raw,
        DescriptorFieldConversion::OptionalEnum { public_name, .. } => {
            format!(
                "{}(value)",
                optional_enum_helper_name(descriptor, field, public_name)
            )
        }
        DescriptorFieldConversion::RequiredOptionalBool => {
            format!("toSubscriptTypegpuOptionalBool({raw})")
        }
        DescriptorFieldConversion::EnumArray => raw,
        DescriptorFieldConversion::Descriptor(boundary_name) => {
            format!("to{boundary_name}({raw})")
        }
        DescriptorFieldConversion::DescriptorArray(boundary_name) => {
            format!("to{boundary_name}Array({raw})")
        }
        DescriptorFieldConversion::RecordEntries { boundary_name, .. } => {
            format!("to{boundary_name}Array({raw})")
        }
        DescriptorFieldConversion::RequiredLimits { .. } => overrides
            .get(&field.name)
            .cloned()
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: format!("{}.{}", descriptor.name, field.name),
                message: "required-limits optional aggregate conditional argument was not emitted"
                    .to_owned(),
            })?,
        DescriptorFieldConversion::OptionalDescriptor { boundary_name, .. } => {
            format!("toOptional{boundary_name}({raw})")
        }
        DescriptorFieldConversion::NullableDescriptor { .. } => overrides
            .get(&field.name)
            .cloned()
            .ok_or_else(|| ApiPolicyError::Invalid {
                entry: format!("{}.{}", descriptor.name, field.name),
                message:
                    "nullable descriptor optional aggregate conditional argument was not emitted"
                        .to_owned(),
            })?,
        DescriptorFieldConversion::Handle(raw_field) => format!("{raw}.{raw_field}"),
        DescriptorFieldConversion::HandleArray { boundary_name, .. } => {
            format!("to{boundary_name}Array({raw})")
        }
        DescriptorFieldConversion::NullableHandle { boundary_name, .. } => {
            format!("toNullable{boundary_name}({raw})")
        }
    })
}

fn optional_enum_helper_name(
    descriptor: &DescriptorPlan,
    field: &DescriptorFieldPlan,
    public_name: &str,
) -> String {
    format!(
        "resolve{public_name}For{}{}",
        descriptor.name,
        naming::pascal(&field.name)
    )
}

fn render_descriptor_helpers(out: &mut String, descriptors: &[DescriptorPlan]) {
    let mut emitted = BTreeSet::new();
    for descriptor in descriptors {
        for field in &descriptor.fields {
            match &field.conversion {
                DescriptorFieldConversion::RequiredOptionalBool
                    if emitted.insert("required-optional-bool".to_owned()) =>
                {
                    out.push_str(
                        "function toSubscriptTypegpuOptionalBool(value: boolean): SubscriptTypegpuOptionalBool {\n  if (value) {\n    return SubscriptTypegpuOptionalBool.SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_TRUE;\n  }\n  return SubscriptTypegpuOptionalBool.SUBSCRIPT_TYPEGPU_OPTIONAL_BOOL_FALSE;\n}\n\n",
                    );
                }
                DescriptorFieldConversion::DescriptorArray(boundary_name)
                    if emitted.insert(format!("descriptor-array:{boundary_name}")) =>
                {
                    let api_name = descriptors
                        .iter()
                        .find(|candidate| candidate.boundary_name == *boundary_name)
                        .map(|candidate| candidate.name.as_str())
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "function to{boundary_name}Array(values: {api_name}[]): {boundary_name}[] {{\n  const lowered: {boundary_name}[] = [];\n  let index: i32 = 0;\n  while (index < values.length) {{\n    lowered.push(to{boundary_name}(values[index]));\n    index = index + 1;\n  }}\n  return lowered;\n}}\n\n"
                    ));
                }
                DescriptorFieldConversion::RecordEntries {
                    api_name,
                    boundary_name,
                } if emitted.insert(format!("record-entry:{boundary_name}")) => {
                    out.push_str(&format!(
                        "function to{boundary_name}(value: {api_name}): {boundary_name} {{\n  return new {boundary_name}(value.key, value.value);\n}}\n\nfunction to{boundary_name}Array(values: {api_name}[]): {boundary_name}[] {{\n  const lowered: {boundary_name}[] = [];\n  let index: i32 = 0;\n  while (index < values.length) {{\n    lowered.push(to{boundary_name}(values[index]));\n    index = index + 1;\n  }}\n  return lowered;\n}}\n\n"
                    ));
                }
                DescriptorFieldConversion::HandleArray {
                    boundary_name,
                    api_name,
                    raw_field,
                } if emitted.insert(format!("handle-array:{boundary_name}")) => {
                    out.push_str(&format!(
                        "function to{boundary_name}Array(values: {api_name}[]): {boundary_name}[] {{\n  const lowered: {boundary_name}[] = [];\n  let index: i32 = 0;\n  while (index < values.length) {{\n    lowered.push(values[index].{raw_field});\n    index = index + 1;\n  }}\n  return lowered;\n}}\n\n"
                    ));
                }
                DescriptorFieldConversion::OptionalDescriptor {
                    boundary_name,
                    absent_values,
                } if emitted.insert(format!("optional:{boundary_name}")) => {
                    let api_name = descriptors
                        .iter()
                        .find(|candidate| candidate.boundary_name == *boundary_name)
                        .map(|candidate| candidate.name.as_str())
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "function toOptional{boundary_name}(value: {api_name} | null): {boundary_name} {{\n  if (value === null) {{\n    return new {boundary_name}({});\n  }}\n  return to{boundary_name}(value);\n}}\n\n",
                        absent_values.join(", ")
                    ));
                }
                DescriptorFieldConversion::NullableHandle {
                    boundary_name,
                    api_name,
                    raw_field,
                } if emitted.insert(format!("nullable-handle:{boundary_name}")) => {
                    out.push_str(&format!(
                        "function toNullable{boundary_name}(value: {api_name} | null): {boundary_name} | null {{\n  if (value === null) {{\n    return null;\n  }}\n  return value.{raw_field};\n}}\n\n"
                    ));
                }
                _ => {}
            }
        }
    }
}

fn render_namespace(out: &mut String, namespace: &NamespacePlan) {
    out.push_str(&format!("class {}Namespace {{\n", namespace.name));
    for (name, _) in &namespace.constants {
        out.push_str(&format!("  {name}: {};\n", namespace.value_type));
    }
    out.push_str("\n  constructor() {\n");
    for (name, value) in &namespace.constants {
        out.push_str(&format!("    this.{name} = 0x{value:04X};\n"));
    }
    out.push_str("  }\n");
    out.push_str("}\n\n");
    out.push_str(&format!(
        "export const {0}: {0}Namespace = new {0}Namespace();\n\n",
        namespace.name
    ));
}

fn render_interface(
    out: &mut String,
    interface: &InterfacePlan,
    needs: &BTreeMap<&str, bool>,
) -> Result<(), ApiPolicyError> {
    render_interface_class(out, interface, needs, false)?;
    if interface.host_owned {
        render_interface_class(out, interface, needs, true)?;
        out.push_str(&format!(
            "export function hostOwned{}(instance: SubscriptTypegpuInstance, {}: SubscriptTypegpu{}): GPUHostOwnedDevice {{\n  return new GPUHostOwnedDevice(instance, {});\n}}\n\n",
            interface.name,
            interface.raw_field,
            interface.boundary,
            interface.raw_field,
        ));
    }
    Ok(())
}

fn render_interface_class(
    out: &mut String,
    interface: &InterfacePlan,
    needs: &BTreeMap<&str, bool>,
    host_owned: bool,
) -> Result<(), ApiPolicyError> {
    let class_name = if host_owned {
        "GPUHostOwnedDevice"
    } else {
        interface.name.as_str()
    };
    out.push_str(&format!("export class {class_name} {{\n"));
    let shared_instance_field = interface.needs_instance && interface.raw_field == "instance";
    if interface.needs_instance {
        if interface.host_owned {
            out.push_str("  private instance: SubscriptTypegpuInstance;\n");
        } else {
            out.push_str("  instance: SubscriptTypegpuInstance;\n");
        }
    }
    if !shared_instance_field {
        out.push_str(&format!(
            "  {}{}: SubscriptTypegpu{};\n",
            if interface.host_owned { "private " } else { "" },
            interface.raw_field,
            interface.boundary
        ));
    }
    if interface.idempotent_dispose {
        out.push_str("  private disposed: boolean;\n");
    }
    for method in &interface.methods {
        if let MethodPlan::Attribute {
            name,
            result_class: Some(result),
            ..
        } = method
        {
            if host_owned && interface.name == "GPUDevice" && name == "queue" {
                continue;
            }
            out.push_str(&format!("  {name}Value: {result};\n"));
        }
    }
    out.push('\n');
    let mut constructor_params = Vec::new();
    if interface.needs_instance {
        constructor_params.push("instance: SubscriptTypegpuInstance".to_owned());
    }
    if !shared_instance_field {
        constructor_params.push(format!(
            "{}: SubscriptTypegpu{}",
            interface.raw_field, interface.boundary
        ));
    }
    out.push_str(&format!(
        "  constructor({}) {{\n",
        constructor_params.join(", ")
    ));
    if interface.needs_instance {
        out.push_str("    this.instance = instance;\n");
    }
    if !shared_instance_field {
        out.push_str(&format!("    this.{0} = {0};\n", interface.raw_field));
    }
    if interface.idempotent_dispose {
        out.push_str("    this.disposed = false;\n");
    }
    for method in &interface.methods {
        if let MethodPlan::Attribute {
            name,
            getter,
            result_class: Some(result),
            ..
        } = method
        {
            if host_owned && interface.name == "GPUDevice" && name == "queue" {
                continue;
            }
            let constructor = if needs.get(result.as_str()).copied().unwrap_or(false) {
                format!(
                    "new {result}(this.instance, {getter}(this.{}))",
                    interface.raw_field
                )
            } else {
                format!("new {result}({getter}(this.{}))", interface.raw_field)
            };
            out.push_str(&format!("    this.{name}Value = {constructor};\n"));
        }
    }
    out.push_str("  }\n\n");

    for method in &interface.methods {
        if host_owned && matches!(method, MethodPlan::Operation { name, .. } if name == "destroy") {
            continue;
        }
        match method {
            MethodPlan::Async {
                name,
                begin,
                params,
                begin_args,
                take,
                result_class,
                nullable,
                boolean_result,
            } => render_async_method(
                out,
                name,
                begin,
                params,
                begin_args,
                take.as_deref(),
                result_class.as_deref(),
                *nullable,
                *boolean_result,
                needs,
            ),
            MethodPlan::Attribute {
                name,
                getter,
                return_type,
                result_class,
                enum_conversion,
            } => {
                out.push_str(&format!("  {name}(): {return_type} {{\n"));
                if host_owned && interface.name == "GPUDevice" && name == "queue" {
                    out.push_str(&format!(
                        "    return new GPUQueue(this.instance, {getter}(this.{}));\n",
                        interface.raw_field
                    ));
                } else if result_class.is_some() {
                    out.push_str(&format!("    return this.{name}Value;\n"));
                } else {
                    let value = format!("{getter}(this.{})", interface.raw_field);
                    if let Some(conversion) = enum_conversion {
                        out.push_str(&format!("    return {conversion}({value});\n"));
                    } else {
                        out.push_str(&format!("    return {value};\n"));
                    }
                }
                out.push_str("  }\n\n");
            }
            MethodPlan::Operation {
                name,
                function,
                params,
                return_type,
                result_class,
                call_args,
                default_variant,
            } => render_operation_method(
                out,
                interface,
                name,
                function,
                params,
                return_type,
                result_class.as_deref(),
                call_args,
                default_variant.as_ref(),
                needs,
            )?,
            MethodPlan::MappedRange { read, write } => {
                render_mapped_range_methods(out, interface, read, write)
            }
            MethodPlan::TypedWriteF32 { function } => {
                render_typed_write_f32(out, interface, function)
            }
            MethodPlan::TypedReadF32 { function } => {
                render_typed_read_f32(out, interface, function)
            }
            MethodPlan::ErrorScopePop {
                name,
                begin,
                take,
                record,
                result_class,
                nullable,
                conversion,
                seed_values,
            } => render_error_scope_pop_method(
                out,
                interface,
                name,
                begin,
                take,
                record,
                result_class,
                *nullable,
                conversion,
                seed_values,
            ),
            MethodPlan::RecordDrain {
                name,
                function,
                record,
                result_class,
                conversion,
                seed_values,
                pump,
            } => render_record_drain_method(
                out,
                interface,
                name,
                function,
                record,
                result_class,
                conversion,
                seed_values,
                *pump,
            ),
            MethodPlan::RecordFill {
                name,
                function,
                record,
                result_class,
                conversion,
                seed_values,
                success,
            } => render_result_record_fill_method(
                out,
                interface,
                name,
                function,
                record,
                result_class,
                conversion,
                seed_values,
                success,
            ),
        }
    }
    if host_owned {
    } else if interface.host_owned {
        out.push_str(&format!(
            "  dispose(): void {{\n    this.queueValue.dispose();\n    subscript_typegpu_{}_release(this.{});\n  }}\n",
            naming::snake(&interface.boundary), interface.raw_field
        ));
    } else if interface.idempotent_dispose {
        out.push_str(&format!(
            "  dispose(): void {{\n    if (!this.disposed) {{\n      subscript_typegpu_{}_release(this.{});\n      this.disposed = true;\n    }}\n  }}\n",
            naming::snake(&interface.boundary), interface.raw_field
        ));
    } else {
        out.push_str(&format!(
            "  dispose(): void {{\n    subscript_typegpu_{}_release(this.{});\n  }}\n",
            naming::snake(&interface.boundary),
            interface.raw_field
        ));
    }
    if !host_owned {
        out.push_str("\n  [Symbol.dispose](): void {\n    this.dispose();\n  }\n");
    }
    out.push_str("}\n\n");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_async_method(
    out: &mut String,
    name: &str,
    begin: &str,
    params: &[MethodParamPlan],
    begin_args: &[String],
    take: Option<&str>,
    result_class: Option<&str>,
    nullable: bool,
    boolean_result: bool,
    needs: &BTreeMap<&str, bool>,
) {
    let return_type = if boolean_result {
        "boolean".to_owned()
    } else {
        let result = result_class.unwrap_or("void");
        if nullable {
            format!("{result} | null")
        } else {
            result.to_owned()
        }
    };
    out.push_str(&format!(
        "  async {name}({}): Promise<{return_type}> {{\n",
        params
            .iter()
            .map(render_method_param)
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str(&format!(
        "    const future: SubscriptTypegpuFutureId = {begin}({});\n",
        begin_args.join(", ")
    ));
    out.push_str("    let status: i32 = subscript_typegpu_future_status(this.instance, future);\n");
    out.push_str("    while (status === 0) {\n");
    out.push_str("      subscript_typegpu_instance_process_events(this.instance);\n");
    out.push_str("      status = subscript_typegpu_future_status(this.instance, future);\n");
    out.push_str("      await Context.suspend();\n");
    out.push_str("    }\n");
    if boolean_result {
        out.push_str(
            "    subscript_typegpu_future_drop(this.instance, future);\n    return status === 1;\n",
        );
    } else {
        out.push_str(
            "    if (status !== 1) {\n      subscript_typegpu_future_drop(this.instance, future);\n      return null;\n    }\n",
        );
        let result = result_class.unwrap_or("void");
        let take = take.unwrap_or_default();
        let handle = format!("{take}(this.instance, future)");
        let constructed = if needs.get(result).copied().unwrap_or(false) {
            format!("new {result}(this.instance, {handle})")
        } else {
            format!("new {result}({handle})")
        };
        out.push_str(&format!("    return {constructed};\n"));
    }
    out.push_str("  }\n\n");
}

fn render_mapped_range_methods(
    out: &mut String,
    interface: &InterfacePlan,
    read: &str,
    write: &str,
) {
    out.push_str("  readMappedRange(offset: u64, size: u64): u8[] {\n");
    out.push_str("    const bytes: u8[] = [];\n");
    out.push_str("    let index: u64 = 0;\n");
    out.push_str("    while (index < size) {\n");
    out.push_str("      bytes.push(0);\n");
    out.push_str("      index = index + 1;\n");
    out.push_str("    }\n");
    out.push_str(&format!(
        "    if ({read}(this.{}, offset, bytes) !== 1) {{\n",
        interface.raw_field
    ));
    out.push_str("      return [];\n");
    out.push_str("    }\n");
    out.push_str("    return bytes;\n");
    out.push_str("  }\n\n");
    out.push_str("  writeMappedRange(offset: u64, data: u8[]): boolean {\n");
    out.push_str(&format!(
        "    return {write}(this.{}, offset, data) === 1;\n",
        interface.raw_field
    ));
    out.push_str("  }\n\n");
}

fn render_typed_write_f32(out: &mut String, interface: &InterfacePlan, function: &str) {
    out.push_str("  // bufferOffset counts bytes; data length counts f32 elements.\n");
    out.push_str("  writeBufferF32(buffer: GPUBuffer, bufferOffset: u64, data: f32[]): void {\n");
    out.push_str(&format!(
        "    {function}(this.{}, buffer.buffer, bufferOffset, data);\n",
        interface.raw_field
    ));
    out.push_str("  }\n\n");
}

fn render_typed_read_f32(out: &mut String, interface: &InterfacePlan, function: &str) {
    out.push_str("  // offset counts bytes; count counts f32 elements.\n");
    out.push_str("  readMappedRangeF32(offset: u64, count: u64): f32[] {\n");
    out.push_str("    const values: f32[] = [];\n");
    out.push_str("    let index: u64 = 0;\n");
    out.push_str("    while (index < count) {\n");
    out.push_str("      values.push(0);\n");
    out.push_str("      index = index + 1;\n");
    out.push_str("    }\n");
    out.push_str(&format!(
        "    if ({function}(this.{}, offset, values) !== 1) {{\n",
        interface.raw_field
    ));
    out.push_str("      return [];\n");
    out.push_str("    }\n");
    out.push_str("    return values;\n");
    out.push_str("  }\n\n");
}

#[allow(clippy::too_many_arguments)]
fn render_error_scope_pop_method(
    out: &mut String,
    interface: &InterfacePlan,
    name: &str,
    begin: &str,
    take: &str,
    record: &str,
    result_class: &str,
    nullable: bool,
    conversion: &str,
    seed_values: &[String],
) {
    out.push_str(&format!(
        "  async {name}(): Promise<{result_class}{}> {{\n",
        if nullable { " | null" } else { "" }
    ));
    out.push_str(&format!(
        "    const future: SubscriptTypegpuFutureId = {begin}(this.{});\n",
        interface.raw_field
    ));
    out.push_str("    let status: i32 = subscript_typegpu_future_status(this.instance, future);\n");
    out.push_str("    while (status === 0) {\n");
    out.push_str("      subscript_typegpu_instance_process_events(this.instance);\n");
    out.push_str("      status = subscript_typegpu_future_status(this.instance, future);\n");
    out.push_str("      await Context.suspend();\n");
    out.push_str("    }\n");
    out.push_str(
        "    if (status !== 1) {\n      subscript_typegpu_future_drop(this.instance, future);\n      return null;\n    }\n",
    );
    out.push_str(&format!(
        "    const record: {record} = new {record}({});\n",
        seed_values.join(", ")
    ));
    out.push_str(&format!(
        "    if (!{take}(this.instance, future, record)) {{\n      subscript_typegpu_future_drop(this.instance, future);\n      return null;\n    }}\n"
    ));
    out.push_str(&format!("    return {conversion}(record);\n"));
    out.push_str("  }\n\n");
}

#[allow(clippy::too_many_arguments)]
fn render_record_drain_method(
    out: &mut String,
    interface: &InterfacePlan,
    name: &str,
    function: &str,
    record: &str,
    result_class: &str,
    conversion: &str,
    seed_values: &[String],
    pump: bool,
) {
    out.push_str(&format!("  {name}(): {result_class} | null {{\n"));
    if pump {
        out.push_str("    subscript_typegpu_instance_process_events(this.instance);\n");
    }
    out.push_str(&format!(
        "    const record: {record} = new {record}({});\n",
        seed_values.join(", ")
    ));
    out.push_str(&format!(
        "    if (!{function}(this.{}, record)) {{\n      return null;\n    }}\n",
        interface.raw_field
    ));
    out.push_str(&format!("    return {conversion}(record);\n"));
    out.push_str("  }\n\n");
}

#[allow(clippy::too_many_arguments)]
fn render_result_record_fill_method(
    out: &mut String,
    interface: &InterfacePlan,
    name: &str,
    function: &str,
    record: &str,
    result_class: &str,
    conversion: &str,
    seed_values: &[String],
    success: &RecordFillSuccess,
) {
    out.push_str(&format!("  {name}(): {result_class} | null {{\n"));
    out.push_str(&format!(
        "    const record: {record} = new {record}({});\n",
        seed_values.join(", ")
    ));
    let call = format!("{function}(this.{}, record)", interface.raw_field);
    match success {
        RecordFillSuccess::Boolean => {
            out.push_str(&format!(
                "    if (!{call}) {{\n      return null;\n    }}\n"
            ));
        }
        RecordFillSuccess::StatusOne => {
            out.push_str(&format!(
                "    if ({call} !== 1) {{\n      return null;\n    }}\n"
            ));
        }
    }
    out.push_str(&format!("    return {conversion}(record);\n"));
    out.push_str("  }\n\n");
}

#[allow(clippy::too_many_arguments)]
fn render_operation_method(
    out: &mut String,
    interface: &InterfacePlan,
    name: &str,
    function: &str,
    params: &[MethodParamPlan],
    return_type: &str,
    result_class: Option<&str>,
    boundary_args: &[String],
    default_variant: Option<&DefaultVariantPlan>,
    needs: &BTreeMap<&str, bool>,
) -> Result<(), ApiPolicyError> {
    out.push_str(&format!(
        "  {name}({}): {return_type} {{\n",
        params
            .iter()
            .map(render_method_param)
            .collect::<Vec<_>>()
            .join(", ")
    ));
    let nullable_descriptors = params
        .iter()
        .filter_map(|parameter| {
            let MethodParamHelper::NullableDescriptor {
                boundary_name,
                api_name,
            } = parameter.helper.as_ref()?
            else {
                return None;
            };
            Some((
                parameter.name.clone(),
                parameter.expression.clone(),
                boundary_name.clone(),
                api_name.clone(),
            ))
        })
        .collect::<Vec<_>>();
    let mut lowered_args = boundary_args.to_vec();
    render_operation_nullable_descriptor_branches(
        out,
        interface,
        function,
        return_type,
        result_class,
        needs,
        &nullable_descriptors,
        0,
        &mut lowered_args,
        "    ",
    )?;
    out.push_str("  }\n\n");
    if let Some(default_variant) = default_variant {
        let mut call_args = vec![format!("this.{}", interface.raw_field)];
        call_args.extend(boundary_args.iter().cloned());
        let null_call_args = call_args
            .iter()
            .map(|argument| {
                if argument == &default_variant.descriptor_expression {
                    "null".to_owned()
                } else {
                    argument.clone()
                }
            })
            .collect::<Vec<_>>();
        let null_call = format!("{function}({})", null_call_args.join(", "));
        out.push_str(&format!("  {}(): {return_type} {{\n", default_variant.name));
        render_operation_result(out, &null_call, return_type, result_class, needs, "    ");
        out.push_str("  }\n\n");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_operation_nullable_descriptor_branches(
    out: &mut String,
    interface: &InterfacePlan,
    function: &str,
    return_type: &str,
    result_class: Option<&str>,
    needs: &BTreeMap<&str, bool>,
    nullable_descriptors: &[(String, String, String, String)],
    index: usize,
    boundary_args: &mut [String],
    indent: &str,
) -> Result<(), ApiPolicyError> {
    if let Some((name, expression, boundary_name, api_name)) = nullable_descriptors.get(index) {
        let matching = boundary_args
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| (argument == expression).then_some(index))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(ApiPolicyError::Invalid {
                entry: format!("{}.argument.{name}", interface.name),
                message: format!(
                    "nullable descriptor `{api_name}` lowering to `{boundary_name}` matched {} mirror call arguments, expected one",
                    matching.len()
                ),
            });
        }
        let argument_index = matching[0];
        out.push_str(&format!("{indent}if ({name} === null) {{\n"));
        boundary_args[argument_index] = "null".to_owned();
        render_operation_nullable_descriptor_branches(
            out,
            interface,
            function,
            return_type,
            result_class,
            needs,
            nullable_descriptors,
            index + 1,
            boundary_args,
            &format!("{indent}  "),
        )?;
        out.push_str(&format!("{indent}}}\n"));
        boundary_args[argument_index] = expression.clone();
        return render_operation_nullable_descriptor_branches(
            out,
            interface,
            function,
            return_type,
            result_class,
            needs,
            nullable_descriptors,
            index + 1,
            boundary_args,
            indent,
        );
    }
    let mut call_args = vec![format!("this.{}", interface.raw_field)];
    call_args.extend(boundary_args.iter().cloned());
    let call = format!("{function}({})", call_args.join(", "));
    render_operation_result(out, &call, return_type, result_class, needs, indent);
    Ok(())
}

fn render_operation_result(
    out: &mut String,
    call: &str,
    return_type: &str,
    result_class: Option<&str>,
    needs: &BTreeMap<&str, bool>,
    indent: &str,
) {
    if let Some(result) = result_class {
        let constructed = if needs.get(result).copied().unwrap_or(false) {
            format!("new {result}(this.instance, {call})")
        } else {
            format!("new {result}({call})")
        };
        out.push_str(&format!("{indent}return {constructed};\n"));
    } else if return_type == "void" {
        out.push_str(&format!("{indent}{call};\n"));
        if indent != "    " {
            out.push_str(&format!("{indent}return;\n"));
        }
    } else {
        out.push_str(&format!("{indent}return {call};\n"));
    }
}

fn render_method_param(param: &MethodParamPlan) -> String {
    match &param.default {
        Some(default) => format!("{}: {} = {default}", param.name, param.api_type),
        None => format!("{}: {}", param.name, param.api_type),
    }
}

fn raw_field_for_api(api: &str, policy: &ApiSection) -> String {
    if api == policy.singleton_interface {
        lower_first(&policy.singleton_boundary)
    } else {
        lower_first(api.trim_start_matches("GPU"))
    }
}

fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
    }
}
