//! Two-way validation (F18) and plan construction: resolves
//! `policy.toml` against `webgpu.yml` into a typed emission plan, or
//! fails with one of the named policy error classes.

use std::collections::BTreeSet;

use crate::model::{Callback, Function, Yml};
use crate::naming;
use crate::policy::{Policy, PolicyError, TypedPairRow};

/// A scalar C type crossing the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scalar {
    U32,
    U64,
    I32,
    Usize,
    F32,
}

impl Scalar {
    pub fn c_name(self) -> &'static str {
        match self {
            Scalar::U32 => "uint32_t",
            Scalar::U64 => "uint64_t",
            Scalar::I32 => "int32_t",
            Scalar::Usize => "size_t",
            Scalar::F32 => "float",
        }
    }

    pub fn rust_name(self) -> &'static str {
        match self {
            Scalar::U32 => "u32",
            Scalar::U64 => "u64",
            Scalar::I32 => "i32",
            Scalar::Usize => "usize",
            Scalar::F32 => "f32",
        }
    }
}

/// One ordinary method argument supported by the shared sync and
/// count-first array patterns.
#[derive(Debug, Clone)]
pub(crate) enum MethodArg {
    Scalar(String, Scalar),
    Bitflag(String, String),
    Enum(String, String),
    Object {
        name: String,
        object: String,
        nullable: bool,
    },
    StructPointer {
        name: String,
        shape: String,
        nullable: bool,
        owns_storage: bool,
    },
}

/// A sync method's return shape.
#[derive(Debug)]
pub(crate) enum SyncRet {
    Void,
    Handle(String),
    Scalar(Scalar),
    Bitflag(String),
    Enum(String),
}

/// One plain sync method (pattern `sync`).
#[derive(Debug)]
pub(crate) struct SyncOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub args: Vec<MethodArg>,
    pub ret: SyncRet,
}

/// One freestanding create function (pattern `create`).
#[derive(Debug)]
pub(crate) struct CreateOp {
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub returns_object: String,
    /// Dropped optional descriptor: (yml arg name, opaque WGPU type).
    pub dropped_arg: Option<(String, String)>,
    pub doc: Option<String>,
}

/// The callback side of a future-poll op.
#[derive(Debug)]
pub(crate) struct CallbackPlan {
    /// Generated `unsafe extern "C" fn` name.
    pub rust_fn: String,
    /// `WGPUXxxCallback` typedef name.
    pub cb_type: String,
    /// `WGPUXxxCallbackInfo` struct name.
    pub cb_info: String,
    /// Success constant name (`WGPUXxxStatus_Success`).
    pub status_const: String,
    pub status_value: u32,
    /// The object the callback delivers.
    pub handle_object: Option<String>,
}

/// One async op (pattern `future-poll`, F6 triple).
#[derive(Debug)]
pub(crate) struct AsyncOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    /// Dropped optional descriptor: (yml arg name, opaque WGPU type).
    pub dropped_arg: Option<(String, String)>,
    pub cb: CallbackPlan,
    /// `subscript_typegpu_xxx_take` export name.
    pub take_fn: Option<String>,
    /// `SLOT_KIND_*` constant name and value.
    pub kind_const: String,
    pub kind_value: u32,
    /// The first async op carries the protocol comments in subscript-typegpu.h.
    pub first: bool,
    /// `adapter.request_device` exposes the area-7 public descriptor while
    /// retaining the area-6 callback fields internally.
    pub device_descriptor: bool,
}

/// One chain-free facade descriptor field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorFieldKind {
    StringView,
    Bitflag,
    Enum,
    U32,
    U64,
    Usize,
    U16,
    F32,
    F64,
    I32,
    Bool,
    Struct,
    StructPointer,
    Object,
    EnumArray,
    StructArray,
    ObjectArray,
}

/// One descriptor field, preserving its yml/C spelling.
#[derive(Debug, Clone)]
pub(crate) struct DescriptorField {
    pub name: String,
    pub kind: DescriptorFieldKind,
    pub named_type: Option<String>,
    /// Whether an object member accepts/preserves NULL.
    pub nullable: bool,
    /// Backend yml/C count field for an array member.
    pub backend_count_name: Option<String>,
    /// Exact public C count field required by the pair audit.
    pub public_count_name: Option<String>,
    /// F15 internal constant substituted when this public field is zero.
    pub zero_constant: Option<String>,
    /// A nested struct conversion owns backing storage.
    pub nested_owns_storage: bool,
}

/// One plain chain-free struct crossing the facade boundary.
#[derive(Debug, Clone)]
pub(crate) struct StructPlan {
    pub source: String,
    pub wgpu_struct: String,
    pub subscript_typegpu_struct: String,
    pub extensible: bool,
    pub fields: Vec<DescriptorField>,
    /// Conversion owns arrays or nullable struct pointees.
    pub owns_storage: bool,
    /// The backend value is copied out of an owning conversion holder.
    pub backend_copy: bool,
}

/// One internal F15 constant used by a zero-rule conversion.
#[derive(Debug, Clone)]
pub(crate) struct SentinelConst {
    pub source: String,
    pub rust_name: String,
    pub rust_value: String,
}

/// One F12 descriptor operation.
#[derive(Debug)]
pub(crate) struct DescriptorOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub descriptor: String,
    pub optional: bool,
    pub returns_object: String,
}

/// One descriptor-carrying async creation operation.
#[derive(Debug)]
pub(crate) struct DescriptorAsyncOp {
    pub async_op: AsyncOp,
    pub descriptor: String,
}

/// WGSL-only shader creation with an internal extension chain.
#[derive(Debug)]
pub(crate) struct ShaderWgslOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub returns_object: String,
    pub descriptor_wgpu: String,
    pub descriptor_subscript_typegpu: String,
    pub extension_wgpu: String,
    pub s_type_const: String,
    pub s_type_value: u32,
}

/// Queue texture upload with its three struct pointers and F20 bytes.
#[derive(Debug)]
pub(crate) struct WriteTextureOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub destination: String,
    pub layout: String,
    pub extent: String,
}

/// Error-scope future plus facade-owned device event drains (F11/F14).
#[derive(Debug)]
pub(crate) struct DeviceEventsOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub cb_type: String,
    pub cb_info: String,
    pub cb_fn: String,
    pub status_const: String,
    pub status_value: u32,
    pub kind_const: String,
    pub kind_value: u32,
    pub take_fn: String,
}

/// One F13 limits out-fill, method or freestanding.
#[derive(Debug)]
pub(crate) struct LimitsOp {
    pub receiver: Option<String>,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub shape: String,
}

/// One F11 Rev 1 adapter-info fill.
#[derive(Debug)]
pub(crate) struct AdapterInfoOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub success_const: String,
    pub success_value: u32,
}

/// One scalar feature-presence probe, method or freestanding.
#[derive(Debug)]
pub(crate) struct FeatureOp {
    pub receiver: Option<String>,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub enum_name: String,
}

/// One input string-view method (F10).
#[derive(Debug)]
pub(crate) struct LabelOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub param: String,
}

/// One ordinary argument before the byte pair.
#[derive(Debug, Clone)]
pub(crate) enum ByteArg {
    Object(String, String),
    Scalar(String, Scalar),
}

/// One F20 byte-pair reshape.
#[derive(Debug)]
pub(crate) struct BytePairOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub args: Vec<ByteArg>,
    pub mutable: bool,
    pub returns_status: bool,
    pub error_status: i32,
}

/// One S3 float sibling derived from an F20 byte-pair method.
#[derive(Debug)]
pub(crate) struct TypedPairOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub args: Vec<ByteArg>,
    pub mutable: bool,
    pub returns_status: bool,
    pub error_status: i32,
    pub offset_param: String,
}

/// One count-first method array element.
#[derive(Debug, Clone)]
pub(crate) enum ArrayElement {
    Scalar(Scalar),
    Object(String),
}

/// One count-first array operation.
#[derive(Debug)]
pub(crate) struct ArrayOp {
    pub receiver: String,
    pub wgpu_fn: String,
    pub subscript_typegpu_fn: String,
    pub args: Vec<MethodArg>,
    pub param: String,
    pub backend_count: String,
    pub public_count: String,
    pub element: ArrayElement,
}

/// The explicit range + whole-resource mapAsync variants (A3/F15).
#[derive(Debug)]
pub(crate) struct MapAsyncOp {
    pub async_op: AsyncOp,
    pub whole_subscript_typegpu_fn: String,
}

/// One emitted constant set (`[[constants]]` row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstKind {
    Bitflag,
    Enum,
}

/// One emitted constant set (`[[constants]]` row).
#[derive(Debug)]
pub(crate) struct ConstSet {
    /// yml source name (for the doc comment).
    pub source: String,
    /// (constant name, rust type, formatted value) rows.
    pub rows: Vec<(String, &'static str, String)>,
    pub kind: ConstKind,
    pub name: String,
}

/// A middle chunk of the emitted surface, in policy order.
#[derive(Debug)]
pub(crate) enum Chunk {
    Async(AsyncOp),
    Sync(SyncOp),
    Descriptor(DescriptorOp),
    DescriptorAsync(DescriptorAsyncOp),
    ShaderWgsl(ShaderWgslOp),
    Label(LabelOp),
    BytePair(BytePairOp),
    TypedPair(TypedPairOp),
    Array(ArrayOp),
    MapAsync(MapAsyncOp),
    WriteTexture(WriteTextureOp),
    DeviceEvents(DeviceEventsOp),
    Limits(LimitsOp),
    AdapterInfo(AdapterInfoOp),
    Feature(FeatureOp),
}

/// The resolved emission plan.
#[derive(Debug)]
pub(crate) struct Plan {
    /// Subset objects in typedef order (yml names).
    pub objects: Vec<String>,
    /// The future-anchor object (yml name).
    pub anchor: String,
    /// Creation chunk: freestanding creates then anchor sync methods
    /// (the anchor release is implicit in this chunk).
    pub creates: Vec<CreateOp>,
    pub anchor_syncs: Vec<SyncOp>,
    /// Middle chunks in policy order.
    pub chunks: Vec<Chunk>,
    /// Constant sets in policy order.
    pub const_sets: Vec<ConstSet>,
    /// Internal sentinel constants, in policy order.
    pub sentinel_consts: Vec<SentinelConst>,
    /// Public boundary structs, dependency-first and deduplicated.
    pub structs: Vec<StructPlan>,
    /// `callback_mode.allow_process_events` constant and value.
    /// Async operations require this value.
    pub mode_const: Option<(String, u32)>,
    /// Whether the string-view machinery is needed.
    pub needs_string_view: bool,
    /// Opaque pointer-only WGPU types (dropped descriptors), in first
    /// use order.
    pub pointer_only: Vec<String>,
    /// Handle types referenced by signatures but not yet active area
    /// objects (for example command buffers accepted by queue submit).
    pub referenced_objects: Vec<String>,
    /// Whether request-device must install the F14 creation callbacks.
    pub device_events: bool,
    /// Whether the request-device facade descriptor is public.
    pub device_descriptor: bool,
}

/// Tracks consumption of policy rows for the `dead` check.
struct Ledger {
    /// (key, consumed) in policy order.
    rows: Vec<(String, bool)>,
}

impl Ledger {
    fn new() -> Self {
        Ledger { rows: Vec::new() }
    }

    fn add(&mut self, key: String) {
        self.rows.push((key, false));
    }

    fn consume(&mut self, key: &str) {
        for (k, consumed) in &mut self.rows {
            if k == key {
                *consumed = true;
            }
        }
    }

    fn first_dead(&self) -> Option<&str> {
        self.rows
            .iter()
            .find(|(_, consumed)| !consumed)
            .map(|(k, _)| k.as_str())
    }
}

/// Splits `object.method`. A bare name yields no method part.
fn split_construct(construct: &str) -> (&str, Option<&str>) {
    match construct.split_once('.') {
        Some((o, m)) => (o, Some(m)),
        None => (construct, None),
    }
}

/// webgpu.h's generated count spelling singularizes a trailing plural
/// Derives the webgpu.h count field for an array member.
fn backend_array_count(member: &str) -> String {
    let singular = member
        .strip_suffix("ies")
        .map(|stem| format!("{stem}y"))
        .or_else(|| member.strip_suffix('s').map(str::to_string))
        .unwrap_or_else(|| member.to_string());
    format!("{singular}_count")
}

fn public_array_count(member: &str) -> String {
    format!("{}Count", naming::camel(member))
}

fn validate_instance_descriptor(yml: &Yml) -> Result<(), PolicyError> {
    let source = yml
        .struct_("instance_descriptor")
        .ok_or_else(|| PolicyError::Unknown {
            entry: "struct.instance_descriptor".into(),
        })?;
    if source.kind != "extensible" {
        return Err(PolicyError::Invalid {
            entry: "struct.instance_descriptor".into(),
            message: format!(
                "instance creation requires an extensible instance descriptor, found `{}`",
                source.kind
            ),
        });
    }
    for member in &source.members {
        if let Some(inner) = member
            .ty
            .strip_prefix("array<")
            .and_then(|value| value.strip_suffix('>'))
        {
            if member.pointer.as_deref() != Some("immutable") {
                return Err(PolicyError::Invalid {
                    entry: "struct.instance_descriptor".into(),
                    message: format!(
                        "instance descriptor array `{}` must be immutable",
                        member.name
                    ),
                });
            }
            if let Some(name) = inner.strip_prefix("enum.") {
                if yml.enum_(name).is_none() {
                    return Err(PolicyError::Unknown {
                        entry: format!("enum.{name}"),
                    });
                }
            } else {
                return Err(PolicyError::Invalid {
                    entry: "struct.instance_descriptor".into(),
                    message: format!(
                        "unsupported instance descriptor array element type `{inner}`"
                    ),
                });
            }
            continue;
        }
        if let Some(name) = member.ty.strip_prefix("struct.") {
            if member.pointer.as_deref() != Some("immutable") {
                return Err(PolicyError::Invalid {
                    entry: "struct.instance_descriptor".into(),
                    message: format!(
                        "instance descriptor struct member `{}` must be an immutable pointer",
                        member.name
                    ),
                });
            }
            if yml.struct_(name).is_none() {
                return Err(PolicyError::Unknown {
                    entry: format!("struct.{name}"),
                });
            }
            continue;
        }
        return Err(PolicyError::Invalid {
            entry: "struct.instance_descriptor".into(),
            message: format!(
                "unsupported instance descriptor member `{}` type `{}`",
                member.name, member.ty
            ),
        });
    }
    Ok(())
}

/// Resolves the policy against the yml or fails with the first policy
/// error in deterministic order: duplicate, unknown, invalid,
/// unpoliced, dead.
pub(crate) fn build(yml: &Yml, policy: &Policy) -> Result<Plan, PolicyError> {
    check_duplicates(policy)?;
    check_unknown(yml, policy)?;

    // Reasons are load-bearing policy content (F18): reject empties.
    for (construct, reason) in policy
        .exclude
        .iter()
        .map(|r| (&r.construct, &r.reason))
        .chain(policy.constants.iter().map(|r| (&r.source, &r.reason)))
        .chain(policy.renames.iter().map(|r| (&r.construct, &r.reason)))
        .chain(policy.sentinels.iter().map(|r| (&r.construct, &r.reason)))
        .chain(policy.typed_pairs.iter().map(|r| (&r.source, &r.reason)))
        .chain(
            policy
                .chain_flattenings
                .iter()
                .map(|r| (&r.construct, &r.reason)),
        )
    {
        if reason.trim().is_empty() {
            return Err(PolicyError::Invalid {
                entry: construct.clone(),
                message: "empty reason".into(),
            });
        }
    }

    let mut ledger = Ledger::new();
    for name in &policy.slice.objects {
        ledger.add(format!("object:{name}"));
    }
    for row in &policy.functions {
        ledger.add(format!("function:{}", row.name));
    }
    for row in &policy.map {
        ledger.add(format!("map:{}", row.method));
    }
    for row in &policy.exclude {
        ledger.add(format!("exclude:{}", row.construct));
    }
    for row in &policy.constants {
        ledger.add(format!("constant:{}", row.source));
    }
    for row in &policy.renames {
        ledger.add(format!("rename:{}", row.construct));
    }
    for row in &policy.sentinels {
        ledger.add(format!("sentinel:{}", row.construct));
    }
    for row in &policy.chain_flattenings {
        ledger.add(format!("chain-flattening:{}", row.construct));
    }
    for row in &policy.typed_pairs {
        ledger.add(format!("typed-pair:{}", row.source));
    }

    if !policy.slice.objects.contains(&policy.slice.future_anchor) {
        return Err(PolicyError::Invalid {
            entry: policy.slice.future_anchor.clone(),
            message: "future_anchor must be a subset object".into(),
        });
    }

    // Handle pattern: subset objects become opaque typedefs plus
    // Release wrappers; the implicit AddRef family needs its policy
    // exclusion row (F4) to be policed.
    let addref_key = "exclude:addref".to_string();
    let addref_present = ledger.rows.iter().any(|(k, _)| *k == addref_key);
    if !addref_present {
        return Err(PolicyError::Unpoliced {
            construct: "addref".into(),
        });
    }
    ledger.consume(&addref_key);
    for name in &policy.slice.objects {
        ledger.consume(&format!("object:{name}"));
    }

    let mut structs = Vec::new();
    let mut sentinel_consts = Vec::new();

    // Freestanding functions: creates retain the opening chunk; fills and
    // feature probes enter the ordinary generated stream.
    let mut creates = Vec::new();
    let mut function_chunks = Vec::new();
    for row in &policy.functions {
        let function = yml.function(&row.name).expect("checked by check_unknown");
        match row.pattern.as_str() {
            "create" => creates.push(build_create(yml, policy, function, row.doc.clone())?),
            "limits-fill" => function_chunks.push(Chunk::Limits(build_limits(
                yml,
                policy,
                None,
                function,
                &mut structs,
                &mut sentinel_consts,
                &mut ledger,
            )?)),
            "feature-probe" => function_chunks.push(Chunk::Feature(build_feature(None, function)?)),
            other => {
                return Err(PolicyError::Invalid {
                    entry: row.name.clone(),
                    message: format!("unsupported function pattern `{other}`"),
                });
            }
        }
        ledger.consume(&format!("function:{}", row.name));
    }

    // Method rows: anchor sync methods join the creation chunk;
    // everything else becomes a middle chunk in policy order.
    let mut anchor_syncs = Vec::new();
    let mut chunks = function_chunks;
    let mut async_count = 0u32;
    let mut needs_string_view = false;
    let mut pointer_only: Vec<String> = Vec::new();
    let mut referenced_objects: Vec<String> = Vec::new();
    let mut device_events = false;
    let mut device_descriptor = false;
    for create in &creates {
        if let Some((_, ty)) = &create.dropped_arg {
            push_unique(&mut pointer_only, ty.clone());
        }
    }
    let mut mode_const = None;

    for row in &policy.map {
        let (object, method) = split_construct(&row.method);
        let method = method.expect("checked by check_unknown");
        if !policy.slice.objects.iter().any(|o| o == object) {
            return Err(PolicyError::Invalid {
                entry: row.method.clone(),
                message: "method's object is not in the subset".into(),
            });
        }
        let function = yml
            .object(object)
            .and_then(|o| o.methods.iter().find(|m| m.name == method))
            .expect("checked by check_unknown");
        match row.pattern.as_str() {
            "sync" => {
                let op = build_sync(
                    yml,
                    policy,
                    object,
                    function,
                    false,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?;
                if object == policy.slice.future_anchor {
                    anchor_syncs.push(op);
                } else {
                    chunks.push(Chunk::Sync(op));
                }
            }
            "sync-scalar" => {
                chunks.push(Chunk::Sync(build_sync(
                    yml,
                    policy,
                    object,
                    function,
                    true,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?));
            }
            "sync-args" => {
                chunks.push(Chunk::Sync(build_sync(
                    yml,
                    policy,
                    object,
                    function,
                    true,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?));
            }
            "limits-fill" => {
                chunks.push(Chunk::Limits(build_limits(
                    yml,
                    policy,
                    Some(object),
                    function,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?));
            }
            "adapter-info-fill" => {
                chunks.push(Chunk::AdapterInfo(build_adapter_info(
                    yml, object, function,
                )?));
                needs_string_view = true;
            }
            "feature-probe" => {
                chunks.push(Chunk::Feature(build_feature(Some(object), function)?));
            }
            "future-poll" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let mut op = build_async(yml, policy, object, function, async_count)?;
                if object == "adapter" && function.name == "request_device" {
                    register_device_descriptor_support(
                        yml,
                        policy,
                        &row.method,
                        &mut structs,
                        &mut sentinel_consts,
                        &mut ledger,
                    )?;
                    op.device_descriptor = true;
                    device_descriptor = true;
                }
                if let Some((_, ty)) = &op.dropped_arg {
                    push_unique(&mut pointer_only, ty.clone());
                }
                if mode_const.is_none() {
                    let mode_enum = yml.enum_("callback_mode").ok_or(PolicyError::Unknown {
                        entry: "enum.callback_mode".into(),
                    })?;
                    let value =
                        mode_enum
                            .value_of("allow_process_events")
                            .ok_or(PolicyError::Unknown {
                                entry: "callback_mode.allow_process_events".into(),
                            })?;
                    mode_const = Some((
                        naming::wgpu_enum_member("callback_mode", "allow_process_events"),
                        value,
                    ));
                }
                needs_string_view = true;
                async_count += 1;
                chunks.push(Chunk::Async(op));
            }
            "descriptor" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let op = build_descriptor(
                    yml,
                    policy,
                    object,
                    function,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?;
                let descriptor = structs
                    .iter()
                    .find(|shape| shape.source == op.descriptor)
                    .expect("descriptor shape was registered");
                needs_string_view |= descriptor
                    .fields
                    .iter()
                    .any(|field| field.kind == DescriptorFieldKind::StringView);
                chunks.push(Chunk::Descriptor(op));
            }
            "descriptor-future-poll" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let op = build_descriptor_async(
                    yml,
                    policy,
                    object,
                    function,
                    async_count,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?;
                ensure_async_support(yml, &mut mode_const)?;
                needs_string_view = true;
                async_count += 1;
                chunks.push(Chunk::DescriptorAsync(op));
            }
            "shader-wgsl" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                chunks.push(Chunk::ShaderWgsl(build_shader_wgsl(
                    yml,
                    policy,
                    object,
                    function,
                    &mut ledger,
                )?));
                needs_string_view = true;
            }
            "label" => {
                needs_string_view = true;
                chunks.push(Chunk::Label(build_label(object, function)?));
            }
            "byte-pair" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let byte_pair = build_byte_pair(yml, policy, object, function)?;
                chunks.push(Chunk::BytePair(byte_pair));
                if let Some(typed) = policy
                    .typed_pairs
                    .iter()
                    .find(|typed| typed.source == row.method)
                {
                    chunks.push(Chunk::TypedPair(build_typed_pair(
                        yml, policy, object, function, typed,
                    )?));
                    ledger.consume(&format!("typed-pair:{}", typed.source));
                }
            }
            "array" | "empty-array" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let op = build_array(
                    yml,
                    policy,
                    object,
                    function,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?;
                if let ArrayElement::Object(element) = &op.element {
                    push_unique(&mut referenced_objects, element.clone());
                }
                chunks.push(Chunk::Array(op));
            }
            "map-async" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let op = build_map_async(yml, policy, object, function, async_count)?;
                if mode_const.is_none() {
                    let mode_enum = yml.enum_("callback_mode").ok_or(PolicyError::Unknown {
                        entry: "enum.callback_mode".into(),
                    })?;
                    let value =
                        mode_enum
                            .value_of("allow_process_events")
                            .ok_or(PolicyError::Unknown {
                                entry: "callback_mode.allow_process_events".into(),
                            })?;
                    mode_const = Some((
                        naming::wgpu_enum_member("callback_mode", "allow_process_events"),
                        value,
                    ));
                }
                needs_string_view = true;
                async_count += 1;
                chunks.push(Chunk::MapAsync(op));
            }
            "write-texture" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                let op = build_write_texture(
                    yml,
                    policy,
                    object,
                    function,
                    &mut structs,
                    &mut sentinel_consts,
                    &mut ledger,
                )?;
                chunks.push(Chunk::WriteTexture(op));
            }
            "device-events" => {
                if row.reason.is_none() {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "reshape rows require a reason".into(),
                    });
                }
                if device_events {
                    return Err(PolicyError::Invalid {
                        entry: row.method.clone(),
                        message: "device-events pattern may appear only once".into(),
                    });
                }
                let op = build_device_events(yml, policy, object, function, async_count)?;
                ensure_async_support(yml, &mut mode_const)?;
                needs_string_view = true;
                async_count += 1;
                device_events = true;
                chunks.push(Chunk::DeviceEvents(op));
            }
            other => {
                return Err(PolicyError::Invalid {
                    entry: row.method.clone(),
                    message: format!("unsupported pattern `{other}`"),
                });
            }
        }
        ledger.consume(&format!("map:{}", row.method));
        if policy
            .renames
            .iter()
            .any(|rename| rename.construct == row.method)
        {
            ledger.consume(&format!("rename:{}", row.method));
        }
    }

    needs_string_view |= structs.iter().any(|shape| {
        shape
            .fields
            .iter()
            .any(|field| field.kind == DescriptorFieldKind::StringView)
    });

    // The string-view machinery leans on the yml `strlen` constant
    // being the usize sentinel; verify rather than assume.
    if needs_string_view {
        let strlen_ok = yml
            .constant("strlen")
            .is_some_and(|c| matches!(&c.value, serde_yaml::Value::String(s) if s == "usize_max"));
        if !strlen_ok {
            return Err(PolicyError::Unknown {
                entry: "constant.strlen".into(),
            });
        }
    }

    // Constant sets.
    let mut const_sets = Vec::new();
    for row in &policy.constants {
        const_sets.push(build_const_set(yml, &row.source)?);
        ledger.consume(&format!("constant:{}", row.source));
    }

    // Unpoliced sweep: every freestanding function and every method
    // of every subset object must be policed by a pattern or exclude
    // row (Release/AddRef are covered by the handle pattern above).
    let function_keys: BTreeSet<&str> = policy.functions.iter().map(|r| r.name.as_str()).collect();
    let map_keys: BTreeSet<&str> = policy.map.iter().map(|r| r.method.as_str()).collect();
    let exclude_keys: BTreeSet<&str> = policy
        .exclude
        .iter()
        .map(|r| r.construct.as_str())
        .collect();
    for function in &yml.functions {
        if function_keys.contains(function.name.as_str()) {
            continue;
        }
        if exclude_keys.contains(function.name.as_str()) {
            ledger.consume(&format!("exclude:{}", function.name));
            continue;
        }
        return Err(PolicyError::Unpoliced {
            construct: function.name.clone(),
        });
    }
    for object_name in &policy.slice.objects {
        let object = yml.object(object_name).expect("checked by check_unknown");
        for method in &object.methods {
            let key = format!("{object_name}.{}", method.name);
            if key == "instance.create_surface" {
                continue;
            }
            if map_keys.contains(key.as_str()) {
                continue;
            }
            if exclude_keys.contains(key.as_str()) {
                ledger.consume(&format!("exclude:{key}"));
                continue;
            }
            return Err(PolicyError::Unpoliced { construct: key });
        }
    }

    if let Some(dead) = ledger.first_dead() {
        let entry = dead.split_once(':').map(|(_, e)| e).unwrap_or(dead);
        return Err(PolicyError::Dead {
            entry: entry.to_string(),
        });
    }

    if device_events {
        pointer_only.retain(|name| name != "WGPUDeviceDescriptor");
    }

    if creates
        .iter()
        .any(|create| create.returns_object == "instance")
    {
        validate_instance_descriptor(yml)?;
    }

    Ok(Plan {
        objects: policy.slice.objects.clone(),
        anchor: policy.slice.future_anchor.clone(),
        creates,
        anchor_syncs,
        chunks,
        const_sets,
        sentinel_consts,
        structs,
        mode_const,
        needs_string_view,
        pointer_only,
        referenced_objects,
        device_events,
        device_descriptor,
    })
}

fn push_unique(list: &mut Vec<String>, item: String) {
    if !list.contains(&item) {
        list.push(item);
    }
}

fn subscript_typegpu_policy_method(policy: &Policy, object: &str, method: &str) -> String {
    let construct = format!("{object}.{method}");
    policy
        .renames
        .iter()
        .find(|row| row.construct == construct)
        .map(|row| row.to.clone())
        .unwrap_or_else(|| naming::subscript_typegpu_method(object, method))
}

/// Duplicate scan over every policy section, in policy order.
fn check_duplicates(policy: &Policy) -> Result<(), PolicyError> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut check = |name: &str| -> Result<(), PolicyError> {
        if !seen.insert(name.to_string()) {
            return Err(PolicyError::Duplicate {
                entry: name.to_string(),
            });
        }
        Ok(())
    };
    for name in &policy.slice.objects {
        check(name)?;
    }
    for row in &policy.functions {
        check(&row.name)?;
    }
    for row in &policy.map {
        check(&row.method)?;
    }
    // Exclusions have their own duplicate set. If a construct is both mapped
    // and excluded, the mapped step consumes its rule while the stale
    // exclusion remains for the later `dead` check to diagnose precisely.
    let mut excluded = BTreeSet::new();
    for row in &policy.exclude {
        if !excluded.insert(row.construct.clone()) {
            return Err(PolicyError::Duplicate {
                entry: row.construct.clone(),
            });
        }
    }
    for row in &policy.constants {
        check(&row.source)?;
    }
    let mut rename_seen = BTreeSet::new();
    for row in &policy.renames {
        if !rename_seen.insert(row.construct.clone()) {
            return Err(PolicyError::Duplicate {
                entry: row.construct.clone(),
            });
        }
    }
    for row in &policy.sentinels {
        check(&row.construct)?;
    }
    for row in &policy.chain_flattenings {
        check(&row.construct)?;
    }
    let mut typed_pairs = BTreeSet::new();
    for row in &policy.typed_pairs {
        if !typed_pairs.insert(&row.source) {
            return Err(PolicyError::Duplicate {
                entry: row.source.clone(),
            });
        }
    }
    Ok(())
}

/// Unknown scan: every policy name must exist at the pin.
fn check_unknown(yml: &Yml, policy: &Policy) -> Result<(), PolicyError> {
    let unknown = |entry: &str| PolicyError::Unknown {
        entry: entry.to_string(),
    };
    for name in &policy.slice.objects {
        if yml.object(name).is_none() {
            return Err(unknown(name));
        }
    }
    for row in &policy.functions {
        if yml.function(&row.name).is_none() {
            return Err(unknown(&row.name));
        }
    }
    let method_exists = |construct: &str| -> bool {
        let (object, method) = split_construct(construct);
        match method {
            None => false,
            Some(method) => yml
                .object(object)
                .is_some_and(|o| o.methods.iter().any(|m| m.name == method)),
        }
    };
    for row in &policy.map {
        if !method_exists(&row.method) {
            return Err(unknown(&row.method));
        }
    }
    for row in &policy.typed_pairs {
        if !method_exists(&row.source) {
            return Err(unknown(&row.source));
        }
    }
    for row in &policy.exclude {
        if row.construct != "addref"
            && !method_exists(&row.construct)
            && yml.function(&row.construct).is_none()
        {
            return Err(unknown(&row.construct));
        }
    }
    for row in &policy.constants {
        let exists = match split_construct(&row.source) {
            ("bitflag", Some(name)) => yml.bitflag(name).is_some(),
            ("enum", Some(name)) => yml.enum_(name).is_some(),
            ("constant", Some(name)) => yml.constant(name).is_some(),
            _ => false,
        };
        if !exists {
            return Err(unknown(&row.source));
        }
    }
    for row in &policy.renames {
        let (owner, field) = split_construct(&row.construct);
        let struct_count_exists = field.is_some_and(|field| {
            yml.struct_(owner).is_some_and(|shape| {
                shape.members.iter().any(|member| {
                    member.ty.starts_with("array<") && backend_array_count(&member.name) == field
                })
            })
        });
        let parts: Vec<&str> = row.construct.split('.').collect();
        let method_count_exists = match parts.as_slice() {
            [object, method, field] => yml
                .object(object)
                .and_then(|owner| owner.methods.iter().find(|item| item.name == *method))
                .is_some_and(|function| {
                    function.args.iter().any(|arg| {
                        arg.ty.starts_with("array<") && backend_array_count(&arg.name) == *field
                    })
                }),
            _ => false,
        };
        if !method_exists(&row.construct) && !struct_count_exists && !method_count_exists {
            return Err(unknown(&row.construct));
        }
    }
    for row in &policy.sentinels {
        let (owner, field) = split_construct(&row.construct);
        let member_exists = field.is_some_and(|field| {
            yml.struct_(owner)
                .is_some_and(|shape| shape.members.iter().any(|member| member.name == field))
        });
        let target_exists = split_construct(&row.zero_maps_to).1.is_some_and(|name| {
            row.zero_maps_to.starts_with("constant.") && yml.constant(name).is_some()
        });
        if !member_exists {
            return Err(unknown(&row.construct));
        }
        if !target_exists {
            return Err(unknown(&row.zero_maps_to));
        }
    }
    for row in &policy.chain_flattenings {
        let (base, extension) = split_construct(&row.construct);
        let Some(extension) = extension else {
            return Err(unknown(&row.construct));
        };
        let fields_exist = yml.struct_(extension).is_some_and(|shape| {
            shape.kind == "extension"
                && shape.extends.iter().any(|candidate| candidate == base)
                && row
                    .fields
                    .iter()
                    .all(|field| shape.members.iter().any(|member| member.name == *field))
        });
        if yml.struct_(base).is_none() || !fields_exist {
            return Err(unknown(&row.construct));
        }
    }
    Ok(())
}

/// Classifies a dropped-descriptor argument list: either empty or a
/// single optional immutable struct pointer.
fn dropped_descriptor(
    entry: &str,
    args: &[crate::model::Arg],
) -> Result<Option<(String, String)>, PolicyError> {
    match args {
        [] => Ok(None),
        [arg]
            if arg.optional
                && arg.pointer.as_deref() == Some("immutable")
                && arg.ty.starts_with("struct.") =>
        {
            let struct_name = arg.ty.trim_start_matches("struct.");
            Ok(Some((arg.name.clone(), naming::wgpu_type(struct_name))))
        }
        _ => Err(PolicyError::Invalid {
            entry: entry.to_string(),
            message: "the generator supports only an optional descriptor argument for this pattern"
                .into(),
        }),
    }
}

fn build_create(
    _yml: &Yml,
    policy: &Policy,
    function: &Function,
    doc: Option<String>,
) -> Result<CreateOp, PolicyError> {
    let returns_object = function
        .returns
        .as_ref()
        .map(|r| r.ty.trim_start_matches("object.").to_string())
        .filter(|_| {
            function
                .returns
                .as_ref()
                .is_some_and(|r| r.ty.starts_with("object."))
        })
        .ok_or(PolicyError::Invalid {
            entry: function.name.clone(),
            message: "create pattern requires an object return".into(),
        })?;
    if !policy.slice.objects.contains(&returns_object) {
        return Err(PolicyError::Invalid {
            entry: function.name.clone(),
            message: "created object is not in the subset".into(),
        });
    }
    Ok(CreateOp {
        wgpu_fn: format!("wgpu{}", naming::pascal(&function.name)),
        subscript_typegpu_fn: format!("subscript_typegpu_{}", function.name),
        returns_object,
        dropped_arg: dropped_descriptor(&function.name, &function.args)?,
        doc,
    })
}

fn build_limits(
    yml: &Yml,
    policy: &Policy,
    receiver: Option<&str>,
    function: &Function,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<LimitsOp, PolicyError> {
    let entry = receiver.map_or_else(
        || function.name.clone(),
        |object| format!("{object}.{}", function.name),
    );
    let [arg] = function.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "limits-fill requires one out struct".into(),
        });
    };
    if arg.pointer.as_deref() != Some("mutable") || !arg.ty.starts_with("struct.") {
        return Err(PolicyError::Invalid {
            entry,
            message: "limits-fill requires one mutable struct pointer".into(),
        });
    }
    if function.returns.as_ref().map(|value| value.ty.as_str()) != Some("enum.status") {
        return Err(PolicyError::Invalid {
            entry,
            message: "limits-fill requires enum.status return".into(),
        });
    }
    let shape = arg.ty.trim_start_matches("struct.");
    register_struct(yml, policy, &entry, shape, structs, sentinel_consts, ledger)?;
    let wgpu_fn = receiver.map_or_else(
        || format!("wgpu{}", naming::pascal(&function.name)),
        |object| naming::wgpu_method(object, &function.name),
    );
    let subscript_typegpu_fn = receiver.map_or_else(
        || format!("subscript_typegpu_{}", function.name),
        |object| subscript_typegpu_policy_method(policy, object, &function.name),
    );
    Ok(LimitsOp {
        receiver: receiver.map(str::to_owned),
        wgpu_fn,
        subscript_typegpu_fn,
        shape: shape.to_owned(),
    })
}

fn build_feature(receiver: Option<&str>, function: &Function) -> Result<FeatureOp, PolicyError> {
    let entry = receiver.map_or_else(
        || function.name.clone(),
        |object| format!("{object}.{}", function.name),
    );
    let [arg] = function.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "feature-probe requires one enum argument".into(),
        });
    };
    let Some(enum_name) = arg.ty.strip_prefix("enum.") else {
        return Err(PolicyError::Invalid {
            entry,
            message: "feature-probe requires one enum argument".into(),
        });
    };
    if arg.pointer.is_some()
        || function.returns.as_ref().map(|value| value.ty.as_str()) != Some("bool")
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "feature-probe requires scalar enum -> bool".into(),
        });
    }
    let wgpu_fn = receiver.map_or_else(
        || format!("wgpu{}", naming::pascal(&function.name)),
        |object| naming::wgpu_method(object, &function.name),
    );
    let subscript_typegpu_fn = receiver.map_or_else(
        || format!("subscript_typegpu_{}", function.name),
        |object| naming::subscript_typegpu_method(object, &function.name),
    );
    Ok(FeatureOp {
        receiver: receiver.map(str::to_owned),
        wgpu_fn,
        subscript_typegpu_fn,
        enum_name: enum_name.to_owned(),
    })
}

fn build_adapter_info(
    yml: &Yml,
    receiver: &str,
    function: &Function,
) -> Result<AdapterInfoOp, PolicyError> {
    let entry = format!("{receiver}.{}", function.name);
    let [arg] = function.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "adapter-info-fill requires one out struct".into(),
        });
    };
    if arg.ty != "struct.adapter_info"
        || arg.pointer.as_deref() != Some("mutable")
        || function.returns.as_ref().map(|value| value.ty.as_str()) != Some("enum.status")
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "adapter-info-fill requires mutable adapter_info and enum.status".into(),
        });
    }
    let info = yml.struct_("adapter_info").ok_or(PolicyError::Unknown {
        entry: "struct.adapter_info".into(),
    })?;
    let expected = [
        ("vendor", "out_string"),
        ("architecture", "out_string"),
        ("device", "out_string"),
        ("description", "out_string"),
        ("backend_type", "enum.backend_type"),
        ("adapter_type", "enum.adapter_type"),
        ("vendor_ID", "uint32"),
        ("device_ID", "uint32"),
        ("subgroup_min_size", "uint32"),
        ("subgroup_max_size", "uint32"),
    ];
    if !info.free_members
        || info.members.len() != expected.len()
        || info
            .members
            .iter()
            .zip(expected)
            .any(|(member, (name, ty))| member.name != name || member.ty != ty)
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "adapter_info width/free-members shape changed".into(),
        });
    }
    let success_value = yml
        .enum_("status")
        .and_then(|status| status.value_of("success"))
        .ok_or(PolicyError::Unknown {
            entry: "status.success".into(),
        })?;
    Ok(AdapterInfoOp {
        receiver: receiver.to_owned(),
        wgpu_fn: naming::wgpu_method(receiver, &function.name),
        subscript_typegpu_fn: naming::subscript_typegpu_method(receiver, &function.name),
        success_const: naming::wgpu_enum_member("status", "success"),
        success_value,
    })
}

fn register_device_descriptor_support(
    yml: &Yml,
    policy: &Policy,
    entry: &str,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<(), PolicyError> {
    let descriptor = yml
        .struct_("device_descriptor")
        .ok_or(PolicyError::Unknown {
            entry: "struct.device_descriptor".into(),
        })?;
    let expected = [
        ("label", "string_with_default_empty"),
        ("required_features", "array<enum.feature_name>"),
        ("required_limits", "struct.limits"),
        ("default_queue", "struct.queue_descriptor"),
        ("device_lost_callback_info", "callback.device_lost"),
        (
            "uncaptured_error_callback_info",
            "callback.uncaptured_error",
        ),
    ];
    if descriptor.kind != "extensible"
        || descriptor.members.len() != expected.len()
        || descriptor
            .members
            .iter()
            .zip(expected)
            .any(|(member, (name, ty))| member.name != name || member.ty != ty)
        || descriptor.members[1].pointer.as_deref() != Some("immutable")
        || descriptor.members[2].pointer.as_deref() != Some("immutable")
        || !descriptor.members[2].optional
    {
        return Err(PolicyError::Invalid {
            entry: entry.to_owned(),
            message: "device_descriptor input composite changed".into(),
        });
    }
    register_struct(
        yml,
        policy,
        entry,
        "limits",
        structs,
        sentinel_consts,
        ledger,
    )?;
    register_struct(
        yml,
        policy,
        entry,
        "queue_descriptor",
        structs,
        sentinel_consts,
        ledger,
    )?;
    let key = "device_descriptor.required_feature_count";
    let rename = policy
        .renames
        .iter()
        .find(|row| row.construct == key)
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.to_owned(),
            message: "required_features requires requiredFeaturesCount rename".into(),
        })?;
    if rename.to != "requiredFeaturesCount" {
        return Err(PolicyError::Invalid {
            entry: key.into(),
            message:
                "pair count must be pointer-field name + Count exactly (`requiredFeaturesCount`)"
                    .into(),
        });
    }
    ledger.consume(&format!("rename:{key}"));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_sync(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    allow_args: bool,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<SyncOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    if method.callback.is_some() {
        return Err(PolicyError::Invalid {
            entry,
            message: "async method mapped with pattern `sync`; use `future-poll`".into(),
        });
    }
    if !allow_args && !method.args.is_empty() {
        return Err(PolicyError::Invalid {
            entry,
            message: "the generator's sync pattern takes no arguments; use a reshape".into(),
        });
    }
    let mut args = Vec::new();
    for arg in &method.args {
        args.push(build_method_arg(
            yml,
            policy,
            &entry,
            arg,
            structs,
            sentinel_consts,
            ledger,
        )?);
    }
    let ret = match &method.returns {
        None => SyncRet::Void,
        Some(r) if r.ty.starts_with("object.") => {
            let target = r.ty.trim_start_matches("object.").to_string();
            if !policy.slice.objects.contains(&target) {
                return Err(PolicyError::Invalid {
                    entry,
                    message: "returned object is not in the subset".into(),
                });
            }
            SyncRet::Handle(target)
        }
        Some(r) if r.ty.starts_with("bitflag.") => {
            SyncRet::Bitflag(r.ty.trim_start_matches("bitflag.").to_string())
        }
        Some(r) if r.ty.starts_with("enum.") => {
            SyncRet::Enum(r.ty.trim_start_matches("enum.").to_string())
        }
        Some(r) => SyncRet::Scalar(scalar_of(&entry, &r.ty)?),
    };
    Ok(SyncOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: subscript_typegpu_policy_method(policy, object, &method.name),
        args,
        ret,
    })
}

fn build_method_arg(
    yml: &Yml,
    policy: &Policy,
    entry: &str,
    arg: &crate::model::Arg,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<MethodArg, PolicyError> {
    if arg.pointer.is_none() {
        if let Ok(scalar) = scalar_of(entry, &arg.ty) {
            return Ok(MethodArg::Scalar(arg.name.clone(), scalar));
        }
        if let Some(name) = arg.ty.strip_prefix("bitflag.") {
            if yml.bitflag(name).is_none() {
                return Err(PolicyError::Unknown {
                    entry: format!("bitflag.{name}"),
                });
            }
            return Ok(MethodArg::Bitflag(arg.name.clone(), name.to_string()));
        }
        if let Some(name) = arg.ty.strip_prefix("enum.") {
            if yml.enum_(name).is_none() {
                return Err(PolicyError::Unknown {
                    entry: format!("enum.{name}"),
                });
            }
            return Ok(MethodArg::Enum(arg.name.clone(), name.to_string()));
        }
        if let Some(name) = arg.ty.strip_prefix("object.") {
            if !policy.slice.objects.iter().any(|object| object == name) {
                return Err(PolicyError::Invalid {
                    entry: entry.to_string(),
                    message: format!("argument object `{name}` is not in the subset"),
                });
            }
            return Ok(MethodArg::Object {
                name: arg.name.clone(),
                object: name.to_string(),
                nullable: arg.optional,
            });
        }
    }
    if arg.pointer.as_deref() == Some("immutable") {
        if let Some(name) = arg.ty.strip_prefix("struct.") {
            register_struct(yml, policy, entry, name, structs, sentinel_consts, ledger)?;
            let owns_storage = structs
                .iter()
                .find(|shape| shape.source == name)
                .expect("method argument shape was registered")
                .owns_storage;
            return Ok(MethodArg::StructPointer {
                name: arg.name.clone(),
                shape: name.to_string(),
                nullable: arg.optional,
                owns_storage,
            });
        }
    }
    Err(PolicyError::Invalid {
        entry: entry.to_string(),
        message: format!("unsupported sync argument type `{}`", arg.ty),
    })
}

fn build_async(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    index: u32,
) -> Result<AsyncOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let dropped_arg = dropped_descriptor(&entry, &method.args)?;
    build_async_core(yml, policy, object, method, index, dropped_arg)
}

fn build_async_core(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    index: u32,
    dropped_arg: Option<(String, String)>,
) -> Result<AsyncOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let callback_name = method
        .callback
        .as_deref()
        .and_then(|c| c.strip_prefix("callback."))
        .ok_or(PolicyError::Invalid {
            entry: entry.clone(),
            message: "future-poll pattern requires a callback method".into(),
        })?;
    let callback = yml.callback(callback_name).ok_or(PolicyError::Unknown {
        entry: format!("callback.{callback_name}"),
    })?;
    let cb = callback_plan(yml, policy, &entry, callback)?;
    let take_fn = cb
        .handle_object
        .as_ref()
        .map(|_| format!("subscript_typegpu_{}_take", callback_name));
    Ok(AsyncOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: subscript_typegpu_policy_method(policy, object, &method.name),
        dropped_arg,
        cb,
        take_fn,
        kind_const: format!("SLOT_KIND_{}", naming::upper_snake(callback_name)),
        kind_value: index,
        first: index == 0,
        device_descriptor: false,
    })
}

fn ensure_async_support(
    yml: &Yml,
    mode_const: &mut Option<(String, u32)>,
) -> Result<(), PolicyError> {
    if mode_const.is_none() {
        let mode_enum = yml.enum_("callback_mode").ok_or(PolicyError::Unknown {
            entry: "enum.callback_mode".into(),
        })?;
        let value = mode_enum
            .value_of("allow_process_events")
            .ok_or(PolicyError::Unknown {
                entry: "callback_mode.allow_process_events".into(),
            })?;
        *mode_const = Some((
            naming::wgpu_enum_member("callback_mode", "allow_process_events"),
            value,
        ));
    }
    Ok(())
}

/// A `callback_mode`-style callback with a status, optional owned
/// object, and borrowed out-string.
fn callback_plan(
    yml: &Yml,
    policy: &Policy,
    entry: &str,
    callback: &Callback,
) -> Result<CallbackPlan, PolicyError> {
    if callback.style != "callback_mode" {
        return Err(PolicyError::Invalid {
            entry: entry.to_string(),
            message: "future-poll supports only callback_mode-style callbacks".into(),
        });
    }
    let (status, handle, message) = match callback.args.as_slice() {
        [status, message] => (status, None, message),
        [status, handle, message] => (status, Some(handle), message),
        _ => {
            return Err(PolicyError::Invalid {
                entry: entry.to_string(),
                message:
                    "future-poll requires (status, message) or (status, handle, message) callbacks"
                        .into(),
            });
        }
    };
    let status_enum = status
        .ty
        .strip_prefix("enum.")
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.to_string(),
            message: "callback status argument must be an enum".into(),
        })?;
    let handle_object = handle
        .map(|arg| {
            arg.ty
                .strip_prefix("object.")
                .ok_or_else(|| PolicyError::Invalid {
                    entry: entry.to_string(),
                    message: "callback handle argument must be an object".into(),
                })
        })
        .transpose()?;
    if message.ty != "out_string" {
        return Err(PolicyError::Invalid {
            entry: entry.to_string(),
            message: "callback message argument must be an out_string".into(),
        });
    }
    if handle_object.is_some_and(|object| !policy.slice.objects.iter().any(|o| o == object)) {
        return Err(PolicyError::Invalid {
            entry: entry.to_string(),
            message: "callback handle object is not in the subset".into(),
        });
    }
    let status_value = yml
        .enum_(status_enum)
        .and_then(|e| e.value_of("success"))
        .ok_or(PolicyError::Unknown {
            entry: format!("{status_enum}.success"),
        })?;
    Ok(CallbackPlan {
        rust_fn: format!("{}_callback", callback.name),
        cb_type: format!("WGPU{}Callback", naming::pascal(&callback.name)),
        cb_info: format!("WGPU{}CallbackInfo", naming::pascal(&callback.name)),
        status_const: naming::wgpu_enum_member(status_enum, "success"),
        status_value,
        handle_object: handle_object.map(str::to_string),
    })
}

fn build_descriptor(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<DescriptorOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let [arg] = method.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "descriptor pattern requires one argument".into(),
        });
    };
    if arg.pointer.as_deref() != Some("immutable") || !arg.ty.starts_with("struct.") {
        return Err(PolicyError::Invalid {
            entry,
            message: "descriptor pattern requires one immutable struct pointer".into(),
        });
    }
    let struct_name = arg.ty.trim_start_matches("struct.");
    register_struct(
        yml,
        policy,
        &entry,
        struct_name,
        structs,
        sentinel_consts,
        ledger,
    )?;
    let returns_object = method
        .returns
        .as_ref()
        .and_then(|ret| ret.ty.strip_prefix("object."))
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "descriptor pattern requires an object return".into(),
        })?
        .to_string();
    if !policy
        .slice
        .objects
        .iter()
        .any(|candidate| candidate == &returns_object)
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "returned object is not in the subset".into(),
        });
    }
    Ok(DescriptorOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: subscript_typegpu_policy_method(policy, object, &method.name),
        descriptor: struct_name.to_string(),
        optional: arg.optional,
        returns_object,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_descriptor_async(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    index: u32,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<DescriptorAsyncOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let [arg] = method.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "descriptor-future-poll requires one descriptor argument".into(),
        });
    };
    if arg.optional || arg.pointer.as_deref() != Some("immutable") || !arg.ty.starts_with("struct.")
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "descriptor-future-poll requires one non-null immutable struct pointer".into(),
        });
    }
    let descriptor = arg.ty.trim_start_matches("struct.");
    register_struct(
        yml,
        policy,
        &entry,
        descriptor,
        structs,
        sentinel_consts,
        ledger,
    )?;
    Ok(DescriptorAsyncOp {
        async_op: build_async_core(yml, policy, object, method, index, None)?,
        descriptor: descriptor.to_string(),
    })
}

fn build_shader_wgsl(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    ledger: &mut Ledger,
) -> Result<ShaderWgslOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let [arg] = method.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "shader-wgsl requires one descriptor argument".into(),
        });
    };
    if arg.optional
        || arg.pointer.as_deref() != Some("immutable")
        || arg.ty != "struct.shader_module_descriptor"
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "shader-wgsl requires shader_module_descriptor by immutable pointer".into(),
        });
    }
    let base = yml
        .struct_("shader_module_descriptor")
        .expect("checked by method shape");
    if base.members.len() != 1
        || base.members[0].name != "label"
        || base.members[0].ty != "string_with_default_empty"
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "shader module base descriptor must contain only label".into(),
        });
    }
    let row = policy
        .chain_flattenings
        .iter()
        .find(|row| row.construct == "shader_module_descriptor.shader_source_WGSL")
        .ok_or_else(|| PolicyError::Unpoliced {
            construct: "shader_module_descriptor.shader_source_WGSL".into(),
        })?;
    if row.fields.as_slice() != ["code"] {
        return Err(PolicyError::Invalid {
            entry: row.construct.clone(),
            message: "WGSL flattening must expose exactly the `code` field".into(),
        });
    }
    let extension = yml
        .struct_("shader_source_WGSL")
        .expect("checked by check_unknown");
    if extension.members.len() != 1
        || extension.members[0].name != "code"
        || extension.members[0].ty != "string_with_default_empty"
    {
        return Err(PolicyError::Invalid {
            entry: row.construct.clone(),
            message: "WGSL extension must contain one string code field".into(),
        });
    }
    let returns_object = method
        .returns
        .as_ref()
        .and_then(|ret| ret.ty.strip_prefix("object."))
        .filter(|target| policy.slice.objects.iter().any(|item| item == *target))
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "shader-wgsl requires a subset object return".into(),
        })?;
    let s_type_value = yml
        .enum_("s_type")
        .and_then(|enum_| enum_.value_of("shader_source_WGSL"))
        .ok_or_else(|| PolicyError::Unknown {
            entry: "s_type.shader_source_WGSL".into(),
        })?;
    ledger.consume("chain-flattening:shader_module_descriptor.shader_source_WGSL");
    Ok(ShaderWgslOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: subscript_typegpu_policy_method(policy, object, &method.name),
        returns_object: returns_object.to_string(),
        descriptor_wgpu: naming::wgpu_type("shader_module_descriptor"),
        descriptor_subscript_typegpu: naming::subscript_typegpu_type("shader_module_descriptor"),
        extension_wgpu: naming::wgpu_type("shader_source_WGSL"),
        s_type_const: naming::wgpu_enum_member("s_type", "shader_source_WGSL"),
        s_type_value,
    })
}

fn register_struct(
    yml: &Yml,
    policy: &Policy,
    entry: &str,
    struct_name: &str,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<(), PolicyError> {
    if structs.iter().any(|shape| shape.source == struct_name) {
        return Ok(());
    }
    let source = yml
        .struct_(struct_name)
        .ok_or_else(|| PolicyError::Unknown {
            entry: format!("struct.{struct_name}"),
        })?;
    if source.kind == "extension" {
        return Err(PolicyError::Invalid {
            entry: entry.to_string(),
            message: format!("extension struct `{struct_name}` cannot cross the chain-free facade"),
        });
    }

    // Dependencies precede users in C and Rust output. Recursive
    // ownership metadata retains pair storage and nullable pointees
    // through the backend call.
    for member in &source.members {
        let direct = member.ty.strip_prefix("struct.");
        let array = member
            .ty
            .strip_prefix("array<struct.")
            .and_then(|value| value.strip_suffix('>'));
        if let Some(nested) = direct.or(array) {
            register_struct(yml, policy, entry, nested, structs, sentinel_consts, ledger)?;
            if structs
                .iter()
                .find(|shape| shape.source == nested)
                .is_some_and(|shape| shape.owns_storage)
            {
                structs
                    .iter_mut()
                    .find(|shape| shape.source == nested)
                    .expect("nested struct was registered")
                    .backend_copy = true;
            }
        }
    }

    let mut fields = Vec::new();
    for member in &source.members {
        let mut backend_count_name = None;
        let mut public_count_name = None;
        let mut zero_constant = None;
        let mut nested_owns_storage = false;
        let (kind, named_type) = if matches!(
            member.ty.as_str(),
            "string_with_default_empty" | "nullable_string"
        ) {
            (DescriptorFieldKind::StringView, None)
        } else if let Some(name) = member.ty.strip_prefix("bitflag.") {
            (DescriptorFieldKind::Bitflag, Some(name.to_string()))
        } else if let Some(name) = member.ty.strip_prefix("enum.") {
            (DescriptorFieldKind::Enum, Some(name.to_string()))
        } else if let Some(name) = member.ty.strip_prefix("struct.") {
            nested_owns_storage = structs
                .iter()
                .find(|shape| shape.source == name)
                .expect("nested struct was registered")
                .owns_storage;
            if member.pointer.is_some() {
                if member.pointer.as_deref() != Some("immutable") {
                    return Err(PolicyError::Invalid {
                        entry: entry.to_string(),
                        message: format!("struct pointer `{}` must be immutable", member.name),
                    });
                }
                (DescriptorFieldKind::StructPointer, Some(name.to_string()))
            } else {
                (DescriptorFieldKind::Struct, Some(name.to_string()))
            }
        } else if let Some(name) = member.ty.strip_prefix("object.") {
            if !policy.slice.objects.iter().any(|object| object == name) {
                return Err(PolicyError::Invalid {
                    entry: entry.to_string(),
                    message: format!("struct member object `{name}` is not in the subset"),
                });
            }
            (DescriptorFieldKind::Object, Some(name.to_string()))
        } else if let Some(inner) = member
            .ty
            .strip_prefix("array<")
            .and_then(|value| value.strip_suffix('>'))
        {
            if member.pointer.as_deref() != Some("immutable") {
                return Err(PolicyError::Invalid {
                    entry: entry.to_string(),
                    message: format!("array `{}` must be immutable", member.name),
                });
            }
            let (array_kind, element) = if let Some(name) = inner.strip_prefix("enum.") {
                if yml.enum_(name).is_none() {
                    return Err(PolicyError::Unknown {
                        entry: format!("enum.{name}"),
                    });
                }
                (DescriptorFieldKind::EnumArray, name)
            } else if let Some(name) = inner.strip_prefix("struct.") {
                if yml.struct_(name).is_none() {
                    return Err(PolicyError::Unknown {
                        entry: format!("struct.{name}"),
                    });
                }
                nested_owns_storage = structs
                    .iter()
                    .find(|shape| shape.source == name)
                    .expect("array element struct was registered")
                    .owns_storage;
                (DescriptorFieldKind::StructArray, name)
            } else if let Some(name) = inner.strip_prefix("object.") {
                if !policy.slice.objects.iter().any(|object| object == name) {
                    return Err(PolicyError::Invalid {
                        entry: entry.to_string(),
                        message: format!("array element object `{name}` is not in the subset"),
                    });
                }
                (DescriptorFieldKind::ObjectArray, name)
            } else {
                return Err(PolicyError::Invalid {
                    entry: entry.to_string(),
                    message: format!("unsupported descriptor array element type `{inner}`"),
                });
            };
            let backend = backend_array_count(&member.name);
            let rename_key = format!("{struct_name}.{backend}");
            let expected = public_array_count(&member.name);
            let rename = policy
                .renames
                .iter()
                .find(|row| row.construct == rename_key)
                .ok_or_else(|| PolicyError::Invalid {
                    entry: entry.to_string(),
                    message: format!(
                        "array `{}` requires rename `{rename_key}` to `{expected}`",
                        member.name
                    ),
                })?;
            if rename.to != expected {
                return Err(PolicyError::Invalid {
                    entry: rename.construct.clone(),
                    message: format!(
                        "pair count must be pointer-field name + Count exactly (`{expected}`)"
                    ),
                });
            }
            ledger.consume(&format!("rename:{rename_key}"));
            backend_count_name = Some(backend);
            public_count_name = Some(rename.to.clone());
            (array_kind, Some(element.to_string()))
        } else {
            let kind = match member.ty.as_str() {
                "uint32" => DescriptorFieldKind::U32,
                "uint64" => DescriptorFieldKind::U64,
                "usize" => DescriptorFieldKind::Usize,
                "uint16" => DescriptorFieldKind::U16,
                "float32" => DescriptorFieldKind::F32,
                "nullable_float32" => DescriptorFieldKind::F32,
                "float64_supertype" => DescriptorFieldKind::F64,
                "int32" => DescriptorFieldKind::I32,
                "bool" => DescriptorFieldKind::Bool,
                _ => {
                    return Err(PolicyError::Invalid {
                        entry: entry.to_string(),
                        message: format!("unsupported descriptor member type `{}`", member.ty),
                    });
                }
            };
            (kind, None)
        };

        if member.optional
            && !matches!(
                kind,
                DescriptorFieldKind::Object | DescriptorFieldKind::StructPointer
            )
        {
            return Err(PolicyError::Invalid {
                entry: entry.to_string(),
                message: format!(
                    "optional descriptor member `{}` is supported only for handles",
                    member.name
                ),
            });
        }

        let sentinel_key = format!("{struct_name}.{}", member.name);
        if let Some(row) = policy
            .sentinels
            .iter()
            .find(|row| row.construct == sentinel_key)
        {
            if member.ty != "uint64" {
                return Err(PolicyError::Invalid {
                    entry: row.construct.clone(),
                    message: "zero-rule sentinel requires a uint64 field".into(),
                });
            }
            let default = member.default.as_ref().and_then(serde_yaml::Value::as_str);
            if default != Some(row.zero_maps_to.as_str()) {
                return Err(PolicyError::Invalid {
                    entry: row.construct.clone(),
                    message: format!(
                        "zero-rule target `{}` must match the pinned yml default",
                        row.zero_maps_to
                    ),
                });
            }
            let constant_name = row
                .zero_maps_to
                .strip_prefix("constant.")
                .expect("checked by check_unknown");
            let constant = yml
                .constant(constant_name)
                .expect("checked by check_unknown");
            let value = constant.value.as_str();
            if value != Some("uint64_max") {
                return Err(PolicyError::Invalid {
                    entry: row.construct.clone(),
                    message: format!(
                        "unsupported uint64 sentinel value `{}`",
                        value.unwrap_or("<non-string>")
                    ),
                });
            }
            let rust_name = format!("WGPU_{}", naming::upper_snake(constant_name));
            if !sentinel_consts
                .iter()
                .any(|sentinel| sentinel.source == row.zero_maps_to)
            {
                sentinel_consts.push(SentinelConst {
                    source: row.zero_maps_to.clone(),
                    rust_name: rust_name.clone(),
                    rust_value: "u64::MAX".into(),
                });
            }
            zero_constant = Some(rust_name);
            ledger.consume(&format!("sentinel:{sentinel_key}"));
        } else if member
            .default
            .as_ref()
            .and_then(serde_yaml::Value::as_str)
            .and_then(|default| default.strip_prefix("constant."))
            .and_then(|name| yml.constant(name))
            .and_then(|constant| constant.value.as_str())
            .is_some_and(|value| matches!(value, "uint64_max" | "usize_max"))
        {
            return Err(PolicyError::Unpoliced {
                construct: sentinel_key,
            });
        }
        fields.push(DescriptorField {
            name: member.name.clone(),
            kind,
            named_type,
            nullable: member.optional,
            backend_count_name,
            public_count_name,
            zero_constant,
            nested_owns_storage,
        });
    }
    let owns_storage = fields.iter().any(|field| {
        matches!(
            field.kind,
            DescriptorFieldKind::StructArray | DescriptorFieldKind::StructPointer
        ) || (field.kind == DescriptorFieldKind::Struct && field.nested_owns_storage)
    });
    structs.push(StructPlan {
        source: struct_name.to_string(),
        wgpu_struct: naming::wgpu_type(struct_name),
        subscript_typegpu_struct: naming::subscript_typegpu_type(struct_name),
        extensible: source.kind == "extensible",
        fields,
        owns_storage,
        backend_copy: false,
    });
    Ok(())
}

fn build_write_texture(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<WriteTextureOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let [destination, data, data_size, layout, extent] = method.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "write-texture requires destination, bytes, layout, and extent".into(),
        });
    };
    let immutable_struct = |arg: &crate::model::Arg| {
        arg.pointer.as_deref() == Some("immutable") && arg.ty.starts_with("struct.")
    };
    if !immutable_struct(destination)
        || data.ty != "c_void"
        || data.pointer.as_deref() != Some("immutable")
        || data_size.ty != "usize"
        || !immutable_struct(layout)
        || !immutable_struct(extent)
        || method.returns.is_some()
        || method.callback.is_some()
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "write-texture requires immutable struct pointers and a size-last byte pair"
                .into(),
        });
    }
    let destination = destination.ty.trim_start_matches("struct.");
    let layout = layout.ty.trim_start_matches("struct.");
    let extent = extent.ty.trim_start_matches("struct.");
    for name in [destination, layout, extent] {
        register_struct(yml, policy, &entry, name, structs, sentinel_consts, ledger)?;
    }
    Ok(WriteTextureOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: naming::subscript_typegpu_method(object, &method.name),
        destination: destination.to_string(),
        layout: layout.to_string(),
        extent: extent.to_string(),
    })
}

fn build_label(object: &str, method: &Function) -> Result<LabelOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let [arg] = method.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "label pattern requires one string argument".into(),
        });
    };
    if arg.ty != "string_with_default_empty" {
        return Err(PolicyError::Invalid {
            entry,
            message: "label pattern requires string_with_default_empty".into(),
        });
    }
    Ok(LabelOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: naming::subscript_typegpu_method(object, &method.name),
        param: naming::camel(&arg.name),
    })
}

fn build_byte_pair(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
) -> Result<BytePairOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    if method.callback.is_some() {
        return Err(PolicyError::Invalid {
            entry,
            message: "byte-pair pattern cannot map an async method".into(),
        });
    }
    let pair_index = method
        .args
        .iter()
        .position(|arg| arg.ty == "c_void" && arg.pointer.is_some())
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "byte-pair pattern requires a void pointer".into(),
        })?;
    let data = &method.args[pair_index];
    if method
        .args
        .get(pair_index + 1)
        .is_none_or(|arg| arg.ty != "usize")
        || pair_index + 2 != method.args.len()
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "byte-pair pattern requires a final adjacent (void*, size_t) pair".into(),
        });
    }
    let mutable = match data.pointer.as_deref() {
        Some("mutable") => true,
        Some("immutable") => false,
        _ => {
            return Err(PolicyError::Invalid {
                entry,
                message: "byte-pair data pointer must be mutable or immutable".into(),
            });
        }
    };
    let mut args = Vec::new();
    for arg in &method.args[..pair_index] {
        if let Some(target) = arg.ty.strip_prefix("object.") {
            if !policy
                .slice
                .objects
                .iter()
                .any(|candidate| candidate == target)
            {
                return Err(PolicyError::Invalid {
                    entry: entry.clone(),
                    message: format!("argument object `{target}` is not in the subset"),
                });
            }
            args.push(ByteArg::Object(arg.name.clone(), target.to_string()));
        } else {
            args.push(ByteArg::Scalar(
                arg.name.clone(),
                scalar_of(&entry, &arg.ty)?,
            ));
        }
    }
    let returns_status = match &method.returns {
        None => false,
        Some(ret) if ret.ty == "enum.status" => true,
        Some(ret) => {
            return Err(PolicyError::Invalid {
                entry,
                message: format!("unsupported byte-pair return `{}`", ret.ty),
            });
        }
    };
    let error_status = if returns_status {
        let status = yml.enum_("status").ok_or_else(|| PolicyError::Unknown {
            entry: "enum.status".into(),
        })?;
        status
            .value_of("error")
            .ok_or_else(|| PolicyError::Unknown {
                entry: "status.error".into(),
            })? as i32
    } else {
        0
    };
    Ok(BytePairOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: naming::subscript_typegpu_method(object, &method.name),
        args,
        mutable,
        returns_status,
        error_status,
    })
}

fn build_typed_pair(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    row: &TypedPairRow,
) -> Result<TypedPairOp, PolicyError> {
    let entry = format!("typed-pair.{}", row.source);
    if row.element != "float" {
        return Err(PolicyError::Invalid {
            entry,
            message: format!(
                "typed-pair element must be `float`, found `{}`",
                row.element
            ),
        });
    }
    let api_member =
        crate::api::typed_pair_api_member(&row.source).ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "typed-pair source cannot derive an API member".into(),
        })?;
    if !crate::api::has_synthetic_typed_anchor(policy, &api_member) {
        return Err(PolicyError::Invalid {
            entry,
            message: format!("typed-pair source has no synthetic API anchor `{api_member}`"),
        });
    }
    if !policy
        .map
        .iter()
        .any(|mapped| mapped.method == row.source && mapped.pattern == "byte-pair")
    {
        return Err(PolicyError::Invalid {
            entry,
            message: "typed-pair source must use the `byte-pair` pattern".into(),
        });
    }
    let source = build_byte_pair(yml, policy, object, method)?;
    let offsets = source
        .args
        .iter()
        .filter_map(|arg| match arg {
            ByteArg::Scalar(name, Scalar::U64 | Scalar::Usize) if name.ends_with("offset") => {
                Some(name.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let [offset_param] = offsets.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "typed-pair source must expose one byte-offset scalar".into(),
        });
    };
    Ok(TypedPairOp {
        receiver: source.receiver,
        wgpu_fn: source.wgpu_fn,
        subscript_typegpu_fn: format!("{}_f32", source.subscript_typegpu_fn),
        args: source.args,
        mutable: source.mutable,
        returns_status: source.returns_status,
        error_status: source.error_status,
        offset_param: offset_param.clone(),
    })
}

fn build_array(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    structs: &mut Vec<StructPlan>,
    sentinel_consts: &mut Vec<SentinelConst>,
    ledger: &mut Ledger,
) -> Result<ArrayOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let Some((arg, prefix)) = method.args.split_last() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "array pattern requires a final array argument".into(),
        });
    };
    if arg.pointer.as_deref() != Some("immutable") {
        return Err(PolicyError::Invalid {
            entry,
            message: "array pattern requires an immutable final array".into(),
        });
    }
    let inner = arg
        .ty
        .strip_prefix("array<")
        .and_then(|value| value.strip_suffix('>'))
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "array pattern requires an immutable final array".into(),
        })?;
    let element = if let Some(object) = inner.strip_prefix("object.") {
        if !policy
            .slice
            .objects
            .iter()
            .any(|candidate| candidate == object)
        {
            return Err(PolicyError::Invalid {
                entry: entry.clone(),
                message: format!("array element object `{object}` is not in the subset"),
            });
        }
        ArrayElement::Object(object.to_string())
    } else {
        ArrayElement::Scalar(scalar_of(&entry, inner)?)
    };
    if method.returns.is_some() || method.callback.is_some() {
        return Err(PolicyError::Invalid {
            entry,
            message: "array pattern requires a synchronous void method".into(),
        });
    }
    let mut args = Vec::new();
    for prefix_arg in prefix {
        let built = build_method_arg(
            yml,
            policy,
            &entry,
            prefix_arg,
            structs,
            sentinel_consts,
            ledger,
        )?;
        if matches!(built, MethodArg::StructPointer { .. }) {
            return Err(PolicyError::Invalid {
                entry: entry.clone(),
                message: "array pattern does not support struct-pointer prefixes".into(),
            });
        }
        args.push(built);
    }
    let backend = backend_array_count(&arg.name);
    let expected = public_array_count(&arg.name);
    let rename_key = format!("{object}.{}.{backend}", method.name);
    let rename = policy
        .renames
        .iter()
        .find(|row| row.construct == rename_key)
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: format!(
                "array `{}` requires rename `{rename_key}` to `{expected}`",
                arg.name
            ),
        })?;
    if rename.to != expected {
        return Err(PolicyError::Invalid {
            entry: rename.construct.clone(),
            message: format!(
                "pair count must be pointer-field name + Count exactly (`{expected}`)"
            ),
        });
    }
    ledger.consume(&format!("rename:{rename_key}"));
    Ok(ArrayOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: subscript_typegpu_policy_method(policy, object, &method.name),
        args,
        param: naming::camel(&arg.name),
        backend_count: naming::camel(&backend),
        public_count: rename.to.clone(),
        element,
    })
}

fn build_map_async(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    index: u32,
) -> Result<MapAsyncOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    let [mode, offset, size] = method.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "map-async requires mode, offset, and size arguments".into(),
        });
    };
    if mode.ty != "bitflag.map_mode" || offset.ty != "usize" || size.ty != "usize" {
        return Err(PolicyError::Invalid {
            entry,
            message: "map-async requires (bitflag.map_mode, usize, usize)".into(),
        });
    }
    let callback_name = method
        .callback
        .as_deref()
        .and_then(|name| name.strip_prefix("callback."))
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "map-async requires a callback".into(),
        })?;
    let callback = yml
        .callback(callback_name)
        .ok_or_else(|| PolicyError::Unknown {
            entry: format!("callback.{callback_name}"),
        })?;
    let cb = callback_plan(yml, policy, &entry, callback)?;
    if cb.handle_object.is_some() {
        return Err(PolicyError::Invalid {
            entry,
            message: "map-async callback must not carry a handle".into(),
        });
    }
    let subscript_typegpu_fn = naming::subscript_typegpu_method(object, &method.name);
    Ok(MapAsyncOp {
        whole_subscript_typegpu_fn: subscript_typegpu_fn.replace("map_async", "map_whole_async"),
        async_op: AsyncOp {
            receiver: object.to_string(),
            wgpu_fn: naming::wgpu_method(object, &method.name),
            subscript_typegpu_fn,
            dropped_arg: None,
            cb,
            take_fn: None,
            kind_const: format!("SLOT_KIND_{}", naming::upper_snake(callback_name)),
            kind_value: index,
            first: index == 0,
            device_descriptor: false,
        },
    })
}

fn build_device_events(
    yml: &Yml,
    policy: &Policy,
    object: &str,
    method: &Function,
    index: u32,
) -> Result<DeviceEventsOp, PolicyError> {
    let entry = format!("{object}.{}", method.name);
    if object != "device" || method.name != "pop_error_scope" || !method.args.is_empty() {
        return Err(PolicyError::Invalid {
            entry,
            message: "device-events requires device.pop_error_scope with no ordinary arguments"
                .into(),
        });
    }
    let callback_name = method
        .callback
        .as_deref()
        .and_then(|name| name.strip_prefix("callback."))
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "device-events requires a callback".into(),
        })?;
    let callback = yml
        .callback(callback_name)
        .ok_or_else(|| PolicyError::Unknown {
            entry: format!("callback.{callback_name}"),
        })?;
    let [status, error_type, message] = callback.args.as_slice() else {
        return Err(PolicyError::Invalid {
            entry,
            message: "device-events callback requires (status, error type, message)".into(),
        });
    };
    if callback.style != "callback_mode"
        || error_type.ty != "enum.error_type"
        || message.ty != "out_string"
    {
        return Err(PolicyError::Invalid {
            entry,
            message:
                "device-events callback must be callback_mode (status, enum.error_type, out_string)"
                    .into(),
        });
    }
    let status_enum = status
        .ty
        .strip_prefix("enum.")
        .ok_or_else(|| PolicyError::Invalid {
            entry: entry.clone(),
            message: "device-events status must be an enum".into(),
        })?;
    let status_value = yml
        .enum_(status_enum)
        .and_then(|values| values.value_of("success"))
        .ok_or_else(|| PolicyError::Unknown {
            entry: format!("{status_enum}.success"),
        })?;
    Ok(DeviceEventsOp {
        receiver: object.to_string(),
        wgpu_fn: naming::wgpu_method(object, &method.name),
        subscript_typegpu_fn: subscript_typegpu_policy_method(policy, object, &method.name),
        cb_type: format!("WGPU{}Callback", naming::pascal(callback_name)),
        cb_info: format!("WGPU{}CallbackInfo", naming::pascal(callback_name)),
        cb_fn: format!("{}_callback", callback_name),
        status_const: naming::wgpu_enum_member(status_enum, "success"),
        status_value,
        kind_const: format!("SLOT_KIND_{}", naming::upper_snake(callback_name)),
        kind_value: index,
        take_fn: "subscript_typegpu_pop_error_scope_take".into(),
    })
}

fn scalar_of(entry: &str, ty: &str) -> Result<Scalar, PolicyError> {
    match ty {
        "uint32" => Ok(Scalar::U32),
        "uint64" => Ok(Scalar::U64),
        "int32" => Ok(Scalar::I32),
        "usize" => Ok(Scalar::Usize),
        "float32" => Ok(Scalar::F32),
        other => Err(PolicyError::Invalid {
            entry: entry.to_string(),
            message: format!("type `{other}` is not a generator scalar"),
        }),
    }
}

fn build_const_set(yml: &Yml, source: &str) -> Result<ConstSet, PolicyError> {
    let unknown = || PolicyError::Unknown {
        entry: source.to_string(),
    };
    match split_construct(source) {
        ("bitflag", Some(name)) => {
            let bitflag = yml.bitflag(name).ok_or_else(unknown)?;
            let rows = bitflag
                .entries
                .iter()
                .map(|e| {
                    let value = bitflag.value_of(&e.name).expect("entry exists");
                    (
                        naming::wgpu_enum_member(name, &e.name),
                        "u64",
                        naming::hex_flag(value),
                    )
                })
                .collect();
            Ok(ConstSet {
                source: source.to_string(),
                rows,
                kind: ConstKind::Bitflag,
                name: name.to_string(),
            })
        }
        ("enum", Some(name)) => {
            let enum_ = yml.enum_(name).ok_or_else(unknown)?;
            let rows = enum_
                .entries
                .iter()
                .flatten()
                .map(|e| {
                    let value = enum_.value_of(&e.name).expect("entry exists");
                    (
                        naming::wgpu_enum_member(name, &e.name),
                        "i32",
                        naming::hex_enum(value),
                    )
                })
                .collect();
            Ok(ConstSet {
                source: source.to_string(),
                rows,
                kind: ConstKind::Enum,
                name: name.to_string(),
            })
        }
        _ => Err(unknown()),
    }
}
