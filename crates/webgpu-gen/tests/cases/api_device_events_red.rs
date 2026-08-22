//! Device-event fill-record and hierarchy claims fail by the exact joined construct.

fn document() -> String {
    r#"<script type=idl>
interface EventTarget {};
interface GPUDevice : EventTarget {
  readonly attribute Promise<GPUDeviceLostInfo> lost;
  undefined pushErrorScope(GPUErrorFilter filter);
  Promise<GPUError?> popErrorScope();
  attribute EventHandler onuncapturederror;
};
interface GPUError {
  readonly attribute DOMString message;
};
interface GPUValidationError : GPUError {
  constructor(DOMString message);
};
interface GPUOutOfMemoryError : GPUError {
  constructor(DOMString message);
};
interface GPUInternalError : GPUError {
  constructor(DOMString message);
};
interface GPUDeviceLostInfo {
  readonly attribute GPUDeviceLostReason reason;
  readonly attribute DOMString message;
};
enum GPUErrorFilter { "validation", "out-of-memory", "internal" };
enum GPUDeviceLostReason { "unknown", "destroyed" };
</script>
"#
    .to_owned()
}

fn mirror() -> String {
    r#"interface SubscriptTypegpuDevice {
  readonly brand: never;
}
declare enum SubscriptTypegpuErrorFilter {
  SUBSCRIPT_TYPEGPU_ERROR_FILTER_VALIDATION = 1,
  SUBSCRIPT_TYPEGPU_ERROR_FILTER_OUT_OF_MEMORY = 2,
  SUBSCRIPT_TYPEGPU_ERROR_FILTER_INTERNAL = 3,
}
declare enum SubscriptTypegpuErrorType {
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR = 1,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION = 2,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_OUT_OF_MEMORY = 3,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_INTERNAL = 4,
  SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN = 5,
}
declare enum SubscriptTypegpuDeviceLostReason {
  SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_UNKNOWN = 1,
  SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_DESTROYED = 2,
  SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_CALLBACK_CANCELLED = 3,
  SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_FAILED_CREATION = 4,
}
declare class SubscriptTypegpuErrorRecord {
  type: SubscriptTypegpuErrorType;
  message: string;
  constructor(type: SubscriptTypegpuErrorType, message: string);
}
declare class SubscriptTypegpuLostRecord {
  reason: SubscriptTypegpuDeviceLostReason;
  message: string;
  constructor(reason: SubscriptTypegpuDeviceLostReason, message: string);
}
declare function subscript_typegpu_create_instance(): SubscriptTypegpuInstance;
declare function subscript_typegpu_instance_process_events(instance: SubscriptTypegpuInstance): void;
declare function subscript_typegpu_future_status(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): i32;
declare function subscript_typegpu_future_drop(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId): void;
declare function subscript_typegpu_device_release(device: SubscriptTypegpuDevice): void;
declare function subscript_typegpu_device_push_error_scope(device: SubscriptTypegpuDevice, filter: SubscriptTypegpuErrorFilter): void;
declare function subscript_typegpu_device_pop_error_scope(device: SubscriptTypegpuDevice): SubscriptTypegpuFutureId;
declare function subscript_typegpu_pop_error_scope_take(instance: SubscriptTypegpuInstance, future: SubscriptTypegpuFutureId, out: SubscriptTypegpuErrorRecord | null): boolean;
declare function subscript_typegpu_device_next_uncaptured_error(device: SubscriptTypegpuDevice, out: SubscriptTypegpuErrorRecord | null): boolean;
declare function subscript_typegpu_device_lost_info(device: SubscriptTypegpuDevice, out: SubscriptTypegpuLostRecord | null): boolean;
"#
    .to_owned()
}

fn policy() -> String {
    r#"[slice]
future_anchor = "unused"
objects = []

[api]
interfaces = ["GPUDevice"]
dictionaries = []
namespaces = []
enums = ["GPUErrorFilter", "GPUDeviceLostReason"]
namespace_reason = "fixture namespace reshape"
singleton_interface = "GPUDevice"
singleton_name = "fixture"
singleton_boundary = "Device"
singleton_reason = "fixture singleton"
manual_dispose_reason = "fixture disposal"

[[api.interface_parents]]
interface = "GPUDevice"
parent = "EventTarget"
reason = "fixture DOM exclusion"

[[api.interface_parents]]
interface = "GPUValidationError"
parent = "GPUError"
reason = "fixture flatten"

[[api.interface_parents]]
interface = "GPUOutOfMemoryError"
parent = "GPUError"
reason = "fixture flatten"

[[api.interface_parents]]
interface = "GPUInternalError"
parent = "GPUError"
reason = "fixture flatten"

[[api.result_records]]
interface = "GPUError"
boundary = "SubscriptTypegpuErrorRecord"
nullable = true
synthetic_field = "type"
synthetic_enum = "GPUErrorType"
synthetic_boundary_enum = "SubscriptTypegpuErrorType"
synthetic_enum_mappings = ["validation=SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION", "out-of-memory=SUBSCRIPT_TYPEGPU_ERROR_TYPE_OUT_OF_MEMORY", "internal=SUBSCRIPT_TYPEGPU_ERROR_TYPE_INTERNAL", "unknown=SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN"]
synthetic_enum_exclusions = ["SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR"]
reason = "fixture error fill"

[[api.result_records]]
interface = "GPUDeviceLostInfo"
boundary = "SubscriptTypegpuLostRecord"
nullable = false
reason = "fixture lost fill"

[[api.flattened_interfaces]]
interface = "GPUValidationError"
target = "GPUError"
reason = "fixture flatten"

[[api.flattened_interfaces]]
interface = "GPUOutOfMemoryError"
target = "GPUError"
reason = "fixture flatten"

[[api.flattened_interfaces]]
interface = "GPUInternalError"
target = "GPUError"
reason = "fixture flatten"

[[api.deviations]]
member = "GPUDevice.lost"
pattern = "device-lost-poll"
reason = "fixture poll"

[[api.generate]]
member = "GPUDevice.pushErrorScope"
pattern = "operation"

[[api.deviations]]
member = "GPUDevice.popErrorScope"
pattern = "error-scope-pop"
reason = "fixture pop fill"

[[api.deviations]]
member = "GPUDevice.onuncapturederror"
pattern = "uncaptured-error-drain"
reason = "fixture drain"

[[api.deviations]]
member = "GPUError.message"
pattern = "result-record-field"
reason = "fixture flatten loss"

[[api.generate]]
member = "GPUDeviceLostInfo.reason"
pattern = "result-record-field"

[[api.generate]]
member = "GPUDeviceLostInfo.message"
pattern = "result-record-field"

[[api.exclude]]
member = "GPUValidationError.@constructor"
reason = "fixture flatten"

[[api.exclude]]
member = "GPUOutOfMemoryError.@constructor"
reason = "fixture flatten"

[[api.exclude]]
member = "GPUInternalError.@constructor"
reason = "fixture flatten"

[[api.generate]]
member = "GPUErrorFilter.validation"
pattern = "enum-value"

[[api.generate]]
member = "GPUErrorFilter.out-of-memory"
pattern = "enum-value"

[[api.generate]]
member = "GPUErrorFilter.internal"
pattern = "enum-value"

[[api.generate]]
member = "GPUDeviceLostReason.unknown"
pattern = "enum-value"

[[api.generate]]
member = "GPUDeviceLostReason.destroyed"
pattern = "enum-value"

[[api.enum_exclusions]]
enum_name = "GPUDeviceLostReason"
mirror = "SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_CALLBACK_CANCELLED"
reason = "fixture boundary-only value"

[[api.enum_exclusions]]
enum_name = "GPUDeviceLostReason"
mirror = "SUBSCRIPT_TYPEGPU_DEVICE_LOST_REASON_FAILED_CREATION"
reason = "fixture boundary-only value"
"#
    .to_owned()
}

fn red(idl: &str, mirror: &str, policy: &str, expected: &str) {
    let error = subscript_typegpu_webgpu_gen::generate_api(idl, mirror, policy)
        .expect_err("invalid device-event fixture unexpectedly generated")
        .to_string();
    eprintln!("{error}");
    assert_eq!(error, expected);
}

#[test]
fn flattened_subclasses_require_a_selected_result_target() {
    let policy = policy().replace(
        "interface = \"GPUValidationError\"\ntarget = \"GPUError\"",
        "interface = \"GPUValidationError\"\ntarget = \"GPUWrongError\"",
    );
    red(
        &document(),
        &mirror(),
        &policy,
        "api policy error (invalid): `GPUValidationError`: flattened interface target `GPUWrongError` is not a selected result record",
    );
}

#[test]
fn flattened_subclass_constructors_must_be_excluded() {
    let policy = policy().replace(
        "[[api.exclude]]\nmember = \"GPUValidationError.@constructor\"\nreason = \"fixture flatten\"\n\n",
        "",
    );
    red(
        &document(),
        &mirror(),
        &policy,
        "api policy error (invalid): `GPUValidationError.@constructor`: a flattened error-subclass constructor must be excluded",
    );
}

#[test]
fn result_record_fields_join_the_exact_mirror_type() {
    let mirror = mirror().replace("message: string;", "message: u32;");
    red(
        &document(),
        &mirror,
        &policy(),
        "api policy error (invalid): `GPUError.message`: unsupported result-record field join from IDL `String` to mirror `u32`",
    );
}

#[test]
fn synthetic_result_enums_account_for_every_mirror_constant() {
    let mirror = mirror().replace(
        "  SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN = 5,",
        "  SUBSCRIPT_TYPEGPU_ERROR_TYPE_UNKNOWN = 5,\n  SUBSCRIPT_TYPEGPU_ERROR_TYPE_EXTRA = 6,",
    );
    red(
        &document(),
        &mirror,
        &policy(),
        "api policy error (invalid): `mirror.SubscriptTypegpuErrorType.SUBSCRIPT_TYPEGPU_ERROR_TYPE_EXTRA`: mirror enum constant is unaccounted by synthetic result enum `GPUErrorType`",
    );
}

#[test]
fn synthetic_result_enum_exclusions_cannot_name_a_mapped_constant() {
    let policy = policy().replace(
        "synthetic_enum_exclusions = [\"SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR\"]",
        "synthetic_enum_exclusions = [\"SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION\", \"SUBSCRIPT_TYPEGPU_ERROR_TYPE_NO_ERROR\"]",
    );
    red(
        &document(),
        &mirror(),
        &policy,
        "api policy error (invalid): `GPUError.synthetic_enum.SUBSCRIPT_TYPEGPU_ERROR_TYPE_VALIDATION`: synthetic enum exclusion names an already-mapped constant",
    );
}

#[test]
fn result_record_nullability_must_cover_null_producing_exclusions() {
    let policy = policy().replace(
        "boundary = \"SubscriptTypegpuErrorRecord\"\nnullable = true",
        "boundary = \"SubscriptTypegpuErrorRecord\"\nnullable = false",
    );
    red(
        &document(),
        &mirror(),
        &policy,
        "api policy error (invalid): `GPUError`: result-record nullable must exactly match its null-producing synthetic enum exclusions",
    );
}

#[test]
fn result_record_mirror_fields_require_selected_idl_attributes() {
    let mirror = mirror().replace(
        "  reason: SubscriptTypegpuDeviceLostReason;\n  message: string;",
        "  reason: SubscriptTypegpuDeviceLostReason;\n  extra: u32;\n  message: string;",
    );
    red(
        &document(),
        &mirror,
        &policy(),
        "api policy error (invalid): `mirror.SubscriptTypegpuLostRecord.extra`: result record field has no selected IDL attribute on `GPUDeviceLostInfo`",
    );
}

#[test]
fn result_record_idl_attributes_require_mirror_fields() {
    let idl = document().replace(
        "  readonly attribute DOMString message;\n};\nenum GPUErrorFilter",
        "  readonly attribute DOMString message;\n  readonly attribute DOMString detail;\n};\nenum GPUErrorFilter",
    );
    let policy = policy().replace(
        "member = \"GPUDeviceLostInfo.message\"\npattern = \"result-record-field\"",
        "member = \"GPUDeviceLostInfo.message\"\npattern = \"result-record-field\"\n\n[[api.generate]]\nmember = \"GPUDeviceLostInfo.detail\"\npattern = \"result-record-field\"",
    );
    red(
        &idl,
        &mirror(),
        &policy,
        "api policy error (invalid): `GPUDeviceLostInfo`: selected result-record attributes do not exactly cover mirror fields",
    );
}

#[test]
fn error_scope_pop_requires_the_exact_fill_take_signature() {
    let mirror = mirror().replace(
        "out: SubscriptTypegpuErrorRecord | null): boolean;",
        "out: SubscriptTypegpuLostRecord | null): boolean;",
    );
    red(
        &document(),
        &mirror,
        &policy(),
        "api policy error (invalid): `mirror.subscript_typegpu_pop_error_scope_take`: parameter types are [\"SubscriptTypegpuInstance\", \"SubscriptTypegpuFutureId\", \"SubscriptTypegpuLostRecord | null\"], expected [\"SubscriptTypegpuInstance\", \"SubscriptTypegpuFutureId\", \"SubscriptTypegpuErrorRecord | null\"]",
    );
}

#[test]
fn device_lost_poll_requires_the_exact_fill_record() {
    let mirror = mirror().replace(
        "subscript_typegpu_device_lost_info(device: SubscriptTypegpuDevice, out: SubscriptTypegpuLostRecord | null)",
        "subscript_typegpu_device_lost_info(device: SubscriptTypegpuDevice, out: SubscriptTypegpuErrorRecord | null)",
    );
    red(
        &document(),
        &mirror,
        &policy(),
        "api policy error (invalid): `mirror.subscript_typegpu_device_lost_info`: parameter types are [\"SubscriptTypegpuDevice\", \"SubscriptTypegpuErrorRecord | null\"], expected [\"SubscriptTypegpuDevice\", \"SubscriptTypegpuLostRecord | null\"]",
    );
}

#[test]
fn result_record_attributes_require_the_claiming_pattern() {
    let policy = policy().replace(
        "member = \"GPUDeviceLostInfo.message\"\npattern = \"result-record-field\"",
        "member = \"GPUDeviceLostInfo.message\"\npattern = \"attribute-method\"",
    );
    red(
        &document(),
        &mirror(),
        &policy,
        "api policy error (invalid): `GPUDeviceLostInfo.message`: pattern `attribute-method` does not match the IDL member kind",
    );
}
