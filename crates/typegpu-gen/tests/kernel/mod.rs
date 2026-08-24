use subscript_compiler::SourceFile;

use crate::support;

fn generate(source: &str) -> subscript_typegpu_gen::Generated {
    let mut files = support::b01_files();
    files.pop();
    files.push(SourceFile::new("kernel-test.ts", source));
    subscript_typegpu_gen::generate(&files).unwrap_or_else(|diagnostics| {
        panic!(
            "kernel test generation failed: {}",
            diagnostics
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}

fn reject(source: &str) -> subscript_compiler::Diagnostic {
    let mut files = support::b01_files();
    files.pop();
    files.push(SourceFile::new("kernel-test.ts", source));
    let diagnostics = subscript_typegpu_gen::generate(&files)
        .expect_err("kernel uniformity fixture unexpectedly generated");
    assert_eq!(
        diagnostics.len(),
        1,
        "uniformity fixture must produce one diagnostic: {diagnostics:?}"
    );
    diagnostics.into_iter().next().expect("one diagnostic")
}

fn validate(wgsl: &str) {
    let module = naga::front::wgsl::parse_str(wgsl)
        .unwrap_or_else(|error| panic!("WGSL parse failed:\n{}", error.emit_to_string(wgsl)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap_or_else(|error| panic!("WGSL validation failed: {error:?}\n{wgsl}"));
}

fn assert_host_runnable(source: &str, expected: bool) {
    let generated = generate(source);
    let pipeline = generated
        .compute_pipelines
        .first()
        .expect("one compute pipeline");
    assert_eq!(pipeline.host_runnable, expected, "{source}");
    assert!(
        generated.support_module.contains(&format!(
            "export const pipeline_HOST_RUNNABLE: boolean = {expected};"
        )),
        "{}",
        generated.support_module,
    );
}

#[test]
fn cl6_storage_barrier_alone_is_not_host_runnable() {
    assert_host_runnable(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline, storageBarrier } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { storageBarrier(); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
        false,
    );
}

#[test]
fn cl6_storage_atomic_alone_is_not_host_runnable() {
    assert_host_runnable(
        r#"
import { AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
@CStruct class Counter { value: AtomicU32; constructor(value: AtomicU32) { this.value = value; } }
class Layout { counters!: MutStorage<Counter>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.counters.get(0).value.add(1); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
        false,
    );
}

#[test]
fn cl6_written_private_variable_alone_is_not_host_runnable() {
    assert_host_runnable(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, PrivateVar, computePipeline, privateVar } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
const state: PrivateVar<u32> = privateVar<u32>(1);
function kernel(res: Layout, ctx: ComputeInvocation): void { state.set(state.get() + 1); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
        false,
    );
}

#[test]
fn cl6_workgroup_variable_alone_is_not_host_runnable() {
    assert_host_runnable(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, WorkgroupVar, computePipeline, workgroupVar } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
const shared: WorkgroupVar<u32> = workgroupVar<u32>();
function kernel(res: Layout, ctx: ComputeInvocation): void { shared.set(1); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
        false,
    );
}

#[test]
fn cl6_read_private_variable_alone_is_host_runnable() {
    assert_host_runnable(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, PrivateVar, computePipeline, privateVar } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
const state: PrivateVar<u32> = privateVar<u32>(1);
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output.set(0, state.get()); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
        true,
    );
}

#[test]
fn pipeline_name_must_match_its_declaration() {
    for (source, expected) in [
        (
            r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output.set(0, 1); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { workgroupSize: [1, 1, 1] });
"#,
            "options omit name",
        ),
        (
            r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output.set(0, 1); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "other", workgroupSize: [1, 1, 1] });
"#,
            "options name is `other`",
        ),
    ] {
        let diagnostic = reject(source);
        assert!(diagnostic.message.contains("PI1"), "{diagnostic:?}");
        assert!(diagnostic.message.contains(expected), "{diagnostic:?}");
    }
}

#[test]
fn local_shadow_of_a_binding_is_renamed_for_every_reference() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, Uniform, computePipeline } from "./typegpu";
@CStruct class Params { value: u32; constructor(value: u32) { this.value = value; } }
@CStruct class Result { local: u32; reread: u32; constructor(local: u32, reread: u32) { this.local = local; this.reread = reread; } }
class Layout { params!: Uniform<Params>; output!: MutStorage<Result>; }
function shadow(res: Layout, ctx: ComputeInvocation): void {
  let params: Params = res.params.get();
  params.value = params.value + 1;
  const reread: Params = res.params.get();
  res.output.set(0, new Result(params.value, reread.value));
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(shadow, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("var params_ = params;"), "{wgsl}");
    assert!(
        wgsl.contains("params_.value = params_.value + 1u;"),
        "{wgsl}"
    );
    assert!(wgsl.contains("var reread = params;"), "{wgsl}");
    assert!(!wgsl.contains("var reread = params_;"), "{wgsl}");
    assert!(
        wgsl.contains("output[0u] = Result(params_.value, reread.value);"),
        "{wgsl}"
    );
    validate(wgsl);
}

#[test]
fn uniform_stride_loop_with_conditional_binding_load_emits() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage, WorkgroupArray, workgroupArray, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: f32; constructor(value: f32) { this.value = value; } }
class Layout { input!: Storage<Item>; }
const partials: WorkgroupArray<f32> = workgroupArray<f32>(4);
function reduction(res: Layout, ctx: ComputeInvocation): void {
  const global: u32 = ctx.globalId.x;
  const local: u32 = ctx.localIndex;
  partials[local] = global < 4 ? res.input[global].value : 0.0;
  workgroupBarrier();
  let stride: u32 = 2;
  while (stride > 0) {
    if (local < stride) { partials[local] = partials[local] + partials[local + stride]; }
    workgroupBarrier();
    stride = stride / 2;
  }
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(reduction, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert_eq!(wgsl.matches("workgroupBarrier();").count(), 2);
    assert!(wgsl.contains("while (stride > 0u)"));
    validate(wgsl);
}

#[test]
fn k22_rejects_non_uniform_continue_from_a_barrier_loop() {
    let loop_exit = reject(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  let running: boolean = true;
  while (running) {
    if (ctx.localIndex === 0) { continue; }
    workgroupBarrier();
    running = false;
  }
  res.output[0] = new Item(1);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    assert!(loop_exit.message.contains("`continue` statement"));
    assert!(loop_exit.message.contains("builtin `ctx.localIndex`"));
}

#[test]
fn k22_taints_loop_writes_after_non_uniform_exits_and_steps() {
    let loop_exit = reject(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  let count: u32 = 0;
  for (let index: u32 = 0; index < 4; index += 1) {
    if (ctx.localIndex === 0) { break; }
    count += 1;
  }
  if (count === 4) { workgroupBarrier(); }
  res.output[0] = new Item(count);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    assert!(loop_exit.message.contains("barrier statement"));
    assert!(loop_exit.message.contains("builtin `ctx.localIndex`"));

    let continue_exit = reject(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  let count: u32 = 0;
  for (let index: u32 = 0; index < 4; index += 1) {
    count += 1;
    if (ctx.localIndex === 0) { continue; }
  }
  if (count === 4) { workgroupBarrier(); }
  res.output[0] = new Item(count);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    assert!(continue_exit.message.contains("barrier statement"));
    assert!(continue_exit.message.contains("builtin `ctx.localIndex`"));

    let non_uniform_step = reject(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  let index: u32 = 0;
  for (index = 0; index < ctx.localIndex; index += 1) {}
  if (index < 4) { workgroupBarrier(); }
  res.output[0] = new Item(index);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    assert!(non_uniform_step.message.contains("barrier statement"));
    assert!(non_uniform_step
        .message
        .contains("builtin `ctx.localIndex`"));
}

#[test]
fn switch_grouping_module_constants_and_nested_control_flow_emit() {
    let generated = generate(
        r#"
import { Mat2x2f, Vec2u, v2f, v2u } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const LIMIT: u32 = 4;
const OFFSET: Vec2u = v2u(1, 2);
const BASIS: Mat2x2f = new Mat2x2f(v2f(1.0, 0.0), v2f(0.0, 1.0));
function depth(res: Layout, ctx: ComputeInvocation): void {
  let i: u32 = 0;
  let result: u32 = 0;
  while (i < LIMIT) {
    switch (i) {
      case 0:
      case 1: result += OFFSET.x; break;
      case 2: { i += 1; continue; }
      default: return;
    }
    { result += 1; }
    i += 1;
  }
  switch (result) {
    default:
    case 0: result += 1; break;
  }
  switch (result) {
    case 0:
    default: result += 1; break;
  }
  res.output[0] = new Item(result + (BASIS.mulVec(v2f(1.0, 0.0)).x as u32));
}
export const depthPipeline: ComputePipelineSpec = computePipeline<Layout>(depth, { name: "depthPipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "const LIMIT: u32 = 4u;",
        "const OFFSET: vec2<u32> = vec2<u32>(1u, 2u);",
        "const BASIS: mat2x2<f32> = mat2x2<f32>(vec2<f32>(1.0f, 0.0f), vec2<f32>(0.0f, 1.0f));",
        "case 0u, 1u: {",
        "case 2u: {",
        "continue;",
        "default: {",
        "{\n      result += 1u;\n    }",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    assert_eq!(wgsl.matches("case 0u, default: {").count(), 2);
    validate(wgsl);
}

#[test]
fn private_workgroup_variables_barriers_and_builtins_emit() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, PrivateVar, privateVar, WorkgroupArray, workgroupArray, workgroupBarrier, WorkgroupVar, workgroupVar } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
@CStruct class Initial { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const BASE: u32 = 3;
const privateState: PrivateVar<Initial> = privateVar<Initial>(new Initial(BASE));
const sharedValue: WorkgroupVar<u32> = workgroupVar<u32>();
const sharedValues: WorkgroupArray<u32> = workgroupArray<u32>(4);
function variables(res: Layout, ctx: ComputeInvocation): void {
  privateState.set(new Initial(privateState.get().value + ctx.workgroupId.x + ctx.numWorkgroups.x));
  sharedValue.set(ctx.localId.x);
  sharedValues[ctx.localIndex] = sharedValue.get();
  workgroupBarrier();
  res.output[ctx.globalId.x] = new Item(sharedValues[(sharedValues.length() - 1)] + privateState.get().value);
}
export const variablePipeline: ComputePipelineSpec = computePipeline<Layout>(variables, { name: "variablePipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "var<private> privateState: Initial = Initial(BASE);",
        "var<workgroup> sharedValue: u32;",
        "var<workgroup> sharedValues: array<u32, 4u>;",
        "@builtin(global_invocation_id) globalId: vec3<u32>",
        "@builtin(local_invocation_id) localId: vec3<u32>",
        "@builtin(workgroup_id) workgroupId: vec3<u32>",
        "@builtin(num_workgroups) numWorkgroups: vec3<u32>",
        "@builtin(local_invocation_index) localIndex: u32",
        "workgroupBarrier();",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    validate(wgsl);
}

#[test]
fn uniform_reads_binding_lengths_and_workgroup_get_follow_k22() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, Storage, Uniform, WorkgroupArray, workgroupArray, workgroupBarrier } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { params!: Uniform<Item>; input!: Storage<Item>; output!: MutStorage<Item>; }
const hist: WorkgroupArray<u32> = workgroupArray<u32>(4);
function kernel(res: Layout, ctx: ComputeInvocation): void {
  if (res.params.get().value > 0 && res.input.length() > 0) { workgroupBarrier(); }
  const fromGet: u32 = hist.get(ctx.localIndex);
  const fromIndex: u32 = hist[ctx.localIndex];
  res.output[0] = new Item(fromGet + fromIndex);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("if (params.value > 0u && arrayLength(&input) > 0u)"));
    assert_eq!(wgsl.matches("hist[localIndex]").count(), 2);
    validate(wgsl);
}

#[test]
fn k19_folds_checked_scalar_constant_expressions() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const SUM: u32 = 4 + 5 * 2;
const NEXT: u32 = SUM + 1;
const SCALE: f32 = 1.5 + 2.25;
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[0] = new Item(NEXT + (SCALE as u32)); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("const SUM: u32 = 14u;"));
    assert!(wgsl.contains("const NEXT: u32 = 15u;"));
    assert!(wgsl.contains("const SCALE: f32 = 3.75f;"));
    validate(wgsl);
}

#[test]
fn k14_suffixes_a_folded_u32_above_i32_max() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Item { value: u32; constructor(value: u32) { this.value = value; } }
class Layout { output!: MutStorage<Item>; }
const LARGE: u32 = 2147483647 + 1;
const MINIMUM: i32 = -2147483648;
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[ctx.globalId.x] = new Item(LARGE + (MINIMUM as u32)); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("const LARGE: u32 = 2147483648u;"));
    assert!(wgsl.contains("const MINIMUM: i32 = (-2147483647i - 1i);"));
    validate(wgsl);
}

#[test]
fn atomic_storage_and_workgroup_places_emit_every_operation() {
    let generated = generate(
        r#"
import { AtomicI32, AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, storageBarrier, WorkgroupVar, workgroupVar } from "./typegpu";
@CStruct class Counters { unsigned: AtomicU32; signed: AtomicI32; constructor(unsigned: AtomicU32, signed: AtomicI32) { this.unsigned = unsigned; this.signed = signed; } }
class Layout { counters!: MutStorage<Counters>; }
const localCounter: WorkgroupVar<AtomicU32> = workgroupVar<AtomicU32>();
function atomics(res: Layout, ctx: ComputeInvocation): void {
  const a: u32 = res.counters[0].unsigned.load();
  res.counters[0].unsigned.store(a);
  res.counters[0].unsigned.add(1);
  res.counters[0].unsigned.sub(1);
  res.counters[0].unsigned.min(2);
  res.counters[0].unsigned.max(3);
  res.counters[0].unsigned.exchange(4);
  localCounter.get().store(ctx.localIndex);
  localCounter.get().add(1);
  res.counters[0].signed.add(-1);
  res.counters[0].signed.store(-2147483648);
  storageBarrier();
}
export const atomicPipeline: ComputePipelineSpec = computePipeline<Layout>(atomics, { name: "atomicPipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "unsigned: atomic<u32>",
        "signed: atomic<i32>",
        "var<workgroup> localCounter: atomic<u32>;",
        "atomicLoad(&counters[0u].unsigned)",
        "atomicStore(&counters[0u].unsigned, a)",
        "atomicAdd(&counters[0u].unsigned, 1u)",
        "atomicSub(&counters[0u].unsigned, 1u)",
        "atomicMin(&counters[0u].unsigned, 2u)",
        "atomicMax(&counters[0u].unsigned, 3u)",
        "atomicExchange(&counters[0u].unsigned, 4u)",
        "atomicStore(&localCounter, localIndex)",
        "atomicAdd(&localCounter, 1u)",
        "atomicAdd(&counters[0u].signed, -1i)",
        "atomicStore(&counters[0u].signed, (-2147483647i - 1i))",
        "storageBarrier();",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
    validate(wgsl);
}

#[test]
fn atomic_receiver_emits_a_conditional_index_prelude() {
    let generated = generate(
        r#"
import { AtomicU32 } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage } from "./typegpu";
@CStruct class Counter { value: AtomicU32; constructor(value: AtomicU32) { this.value = value; } }
class Layout { counters!: MutStorage<Counter>; }
function kernel(res: Layout, ctx: ComputeInvocation): void {
  res.counters[ctx.localIndex === 0 ? 1 : 2].value.add(1);
}
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    let prelude = wgsl
        .find("var _g_conditional_0: u32;")
        .expect("conditional prelude");
    let atomic = wgsl
        .find("atomicAdd(&counters[_g_conditional_0].value, 1u);")
        .expect("atomic statement");
    assert!(
        prelude < atomic,
        "receiver prelude must precede atomic statement:\n{wgsl}"
    );
    validate(wgsl);
}

#[test]
fn conditional_uses_control_flow_and_all_identifiers_use_one_mangler() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Word { let: f32; constructor(value: f32) { this.let = value; } }
class Layout { new!: Storage<Word>; }
function loop(res: Layout, ctx: ComputeInvocation): void {
  const item: Word = res.new[0];
  const value: f32 = ctx.globalId.x > 0 ? item.let : 1.0;
}
export const names: ComputePipelineSpec = computePipeline<Layout>(loop, { name: "names", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("struct Word {\n  let_: f32,\n}"));
    assert!(wgsl.contains("var<storage, read> new_: array<Word>;"));
    assert!(wgsl.contains("var _g_conditional_0: f32;\n  if ("));
    assert!(wgsl.contains("fn loop_("));
    assert!(generated
        .support_module
        .contains("names_ENTRY: string = \"loop_\""));
}

#[test]
fn half_vectors_enable_the_capability_and_binding_sizes_use_wgsl_layout() {
    let generated = generate(
        r#"
import { Vec2h } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class HalfLayout { values!: Storage<Vec2h>; }
function half(res: HalfLayout, ctx: ComputeInvocation): void { const value: Vec2h = res.values[0]; }
export const halfPipeline: ComputePipelineSpec = computePipeline<HalfLayout>(half, { name: "halfPipeline", workgroupSize: [2, 3, 4] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.starts_with("enable f16;\n\n"));
    assert!(wgsl.contains("array<vec2<f16>>"));
    assert!(generated.support_module.contains("minBindingSize: 4"));
}

#[test]
fn uniform_and_two_layout_bindings_keep_group_order() {
    let generated = generate(
        r#"
import { Vec4f } from "./typegpu-types";
import { ComputeInvocation, computePipeline2, ComputePipelineSpec, Storage, Uniform } from "./typegpu";
class First { params!: Uniform<Vec4f>; }
class Second { values!: Storage<Vec4f>; }
function groups(a: First, b: Second, ctx: ComputeInvocation): void {
  const params: Vec4f = a.params.get();
  const value: Vec4f = b.values[0];
}

export const grouped: ComputePipelineSpec = computePipeline2<First, Second>(groups, { name: "grouped", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(wgsl.contains("@group(0u) @binding(0u) var<uniform> params: vec4<f32>;"));
    assert!(wgsl.contains("@group(1u) @binding(0u) var<storage, read> values: array<vec4<f32>>;"));
    assert!(
        generated
            .support_module
            .matches("minBindingSize: 16")
            .count()
            == 2
    );
}

#[test]
fn control_flow_operators_casts_helpers_and_builtins_emit_as_wgsl() {
    let generated = generate(
        r#"
import { clamp, fract, sign, Vec4f } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, MutStorage, Storage } from "./typegpu";
@CStruct class Pack { values: FixedArray<Vec4f, 2>; constructor(values: FixedArray<Vec4f, 2>) { this.values = values; } }
class Layout { input!: Storage<Pack>; output!: MutStorage<Pack>; }
function helper(value: f32): f32 { return clamp(value, 0.0, 1.0); }
function operations(res: Layout, ctx: ComputeInvocation): void {
  let total: f32 = 0.0;
  let index: u32 = 0;
  while (index < 1) { total += helper(-1.0); index += 1; }
  for (let i: u32 = 0; i < 2; i += 1) { total += i as f32; }
  const pack: Pack = res.input[0];
  for (const value of pack.values) { total += value.length(); }
  const inverted: u32 = ~index;
  const flag: boolean = !false;
  const choice: f32 = flag ? fract(total) : sign(total);
  const integer: i32 = choice as i32;
  const unsigned: u32 = integer as u32;
  res.output[0] = pack;
}

export const operationPipeline: ComputePipelineSpec = computePipeline<Layout>(operations, { name: "operationPipeline", workgroupSize: [4, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "fn helper(value: f32) -> f32 {",
        "while (index < 1u) {",
        "for (var i = 0u; i < 2u; i += 1u) {",
        "for (var _g_value_index = 0u; _g_value_index < 2u;",
        "total += length(value);",
        "let inverted = ~index;",
        "let flag = !false;",
        "var _g_conditional_0: f32;",
        "let integer = i32(choice);",
        "let unsigned = u32(integer);",
        "output[0u] = pack;",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
}

#[test]
fn lowered_operators_preserve_emitted_wgsl_precedence() {
    let generated = generate(
        r#"
import { Vec3f } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class Layout { values!: Storage<Vec3f>; }
function precedence(res: Layout, ctx: ComputeInvocation): void {
  const a: Vec3f = res.values[0];
  const b: Vec3f = res.values[1];
  const s: f32 = 2.0;
  const first: Vec3f = a.add(b).scale(s);
  const second: Vec3f = a.scale(s).add(b);
  const third: f32 = a.dot(b) * 2.0;
  const fourth: f32 = -(a.x + b.x);
  const rounded: f32 = (Math.fround(1.0 + 2.0) as f32) * s;
}

export const precedencePipeline: ComputePipelineSpec = computePipeline<Layout>(precedence, { name: "precedencePipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "var first = (a + b) * s;",
        "var second = a * s + b;",
        "let third = dot(a, b) * 2.0f;",
        "let fourth = -(a.x + b.x);",
        "let rounded = (1.0f + 2.0f) * s;",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
}

#[test]
fn every_k10_and_k11_mapping_reaches_the_emitter() {
    let generated = generate(
        r#"
import { clamp, fract, mix, sign, smoothstep, step, Mat4x4f, Vec3f, Vec3i, Vec3u, Vec4f } from "./typegpu-types";
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
class MappingLayout {
  floats!: Storage<Vec3f>;
  ints!: Storage<Vec3i>;
  uints!: Storage<Vec3u>;
  matrices!: Storage<Mat4x4f>;
  vectors4!: Storage<Vec4f>;
}
function mappings(res: MappingLayout, ctx: ComputeInvocation): void {
  const a: Vec3f = res.floats[0]; const b: Vec3f = res.floats[1];
  const add: Vec3f = a.add(b); const sub: Vec3f = a.sub(b); const mul: Vec3f = a.mul(b);
  const scale: Vec3f = a.scale(2.0); const dot: f32 = a.dot(b); const cross: Vec3f = a.cross(b);
  const length: f32 = a.length(); const normalized: Vec3f = a.normalize();
  const ia: Vec3i = res.ints[0]; const ib: Vec3i = res.ints[1];
  const iadd: Vec3i = ia.add(ib); const isub: Vec3i = ia.sub(ib); const imul: Vec3i = ia.mul(ib); const iscale: Vec3i = ia.scale(2);
  const ua: Vec3u = res.uints[0]; const ub: Vec3u = res.uints[1];
  const uadd: Vec3u = ua.add(ub); const usub: Vec3u = ua.sub(ub); const umul: Vec3u = ua.mul(ub); const uscale: Vec3u = ua.scale(2);
  const matrix: Mat4x4f = res.matrices[0]; const other: Mat4x4f = res.matrices[1]; const vector: Vec4f = res.vectors4[0];
  const mm: Mat4x4f = matrix.mul(other); const mv: Vec4f = matrix.mulVec(vector); const mt: Mat4x4f = matrix.transpose();
  Math.abs(-1.0); Math.min(1.0, 2.0); Math.max(1.0, 2.0); Math.floor(1.5);
  Math.ceil(1.5); Math.sqrt(4.0); Math.pow(2.0, 3.0); Math.exp(1.0);
  Math.log(1.0); Math.sin(1.0); Math.cos(1.0); Math.tan(1.0); Math.fround(1.25);
  const scalar: f32 = 0.5;
  const clamped: f32 = clamp(scalar, 0.0, 1.0); const mixed: f32 = mix(scalar, scalar, 0.5);
  const stepped: f32 = step(0.5, scalar); const smoothed: f32 = smoothstep(0.0, 1.0, scalar);
  const fractional: f32 = fract(scalar); const signed: f32 = sign(scalar);
}

export const mappingPipeline: ComputePipelineSpec = computePipeline<MappingLayout>(mappings, { name: "mappingPipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    for expected in [
        "a + b",
        "a - b",
        "a * b",
        "a * 2.0f",
        "dot(a, b)",
        "cross(a, b)",
        "length(a)",
        "normalize(a)",
        "ia + ib",
        "ua + ub",
        "matrix * other",
        "matrix * vector",
        "transpose(matrix)",
        "abs(-1.0f)",
        "min(1.0f, 2.0f)",
        "max(1.0f, 2.0f)",
        "floor(1.5f)",
        "ceil(1.5f)",
        "sqrt(4.0f)",
        "pow(2.0f, 3.0f)",
        "exp(1.0f)",
        "log(1.0f)",
        "sin(1.0f)",
        "cos(1.0f)",
        "tan(1.0f)",
        "1.25f;",
        "clamp(scalar, 0.0f, 1.0f)",
        "mix(scalar, scalar, 0.5f)",
        "step(0.5f, scalar)",
        "smoothstep(0.0f, 1.0f, scalar)",
        "fract(scalar)",
        "sign(scalar)",
    ] {
        assert!(wgsl.contains(expected), "missing `{expected}` in:\n{wgsl}");
    }
}

#[test]
fn conditional_preludes_stay_at_their_runtime_evaluation_site() {
    let generated = generate(
        r#"
import { ComputeInvocation, computePipeline, ComputePipelineSpec, Storage } from "./typegpu";
@CStruct class Pack {
  left: FixedArray<f32, 2>;
  right: FixedArray<f32, 2>;
  constructor(left: FixedArray<f32, 2>, right: FixedArray<f32, 2>) { this.left = left; this.right = right; }
}

class Layout { packs!: Storage<Pack>; }
function control(res: Layout, ctx: ComputeInvocation): void {
  let i: u32 = 0;
  const flag: boolean = ctx.globalId.x === (0 as u32);
  while (i < (2 as u32) ? true : false) {
    const chosen: u32 = flag ? (i > (0 as u32) ? (2 as u32) : (1 as u32)) : (0 as u32);
    i += chosen;
  }
  for (let j: u32 = 0; j < (flag ? (2 as u32) : (1 as u32)); j += 1) { i += j; }
  const pack: Pack = res.packs[0];
  for (const value of (flag ? pack.left : pack.right)) { i += value as u32; }
}
export const controlPipeline: ComputePipelineSpec = computePipeline<Layout>(control, { name: "controlPipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    let loop_start = wgsl
        .find("loop {")
        .expect("conditional while becomes a loop");
    let while_cond = wgsl
        .find("var _g_conditional_0: bool;")
        .expect("while conditional prelude exists");
    assert!(
        while_cond > loop_start,
        "while prelude escaped its loop:\n{wgsl}"
    );
    assert!(
        wgsl.contains("if (flag) {\n      var _g_conditional_1: u32;\n      if (i > u32(0i)) {"),
        "nested branch prelude escaped its branch:\n{wgsl}"
    );
    let for_loop = wgsl
        .find("var _g_conditional_3: u32;")
        .expect("for condition prelude exists");
    assert!(for_loop > wgsl.find("var j = 0u;").expect("for initializer"));
    let for_of_prelude = wgsl
        .find("var _g_conditional_4: array<f32, 2u>;")
        .expect("for-of subject prelude exists");
    let for_of = wgsl
        .find("for (var _g_value_index")
        .expect("for-of loop exists");
    assert!(
        for_of_prelude < for_of,
        "for-of prelude was not flushed:\n{wgsl}"
    );
}

#[test]
fn wgsl_shell_uses_the_typed_signature_and_literal_body() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, WgslShellSpec, computePipeline, wgslDeclarations, wgslShell } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
wgslDeclarations("const SHELL_BIAS: u32 = 7u;");
function addBias(input: u32): u32 { print(`host=${input}`); return input + 100; }
export const shell: WgslShellSpec = wgslShell<(input: u32) => u32>(addBias, { body: "return input + SHELL_BIAS;" });
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[0] = addBias(5); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(
        wgsl.starts_with("const SHELL_BIAS: u32 = 7u;\n\n"),
        "{wgsl}"
    );
    assert!(
        wgsl.contains("fn addBias(input: u32) -> u32 {\n  return input + SHELL_BIAS;\n}"),
        "{wgsl}"
    );
    assert!(!wgsl.contains("input + 100"), "{wgsl}");
    assert!(!wgsl.contains("host="), "{wgsl}");
    assert!(wgsl.find("fn addBias").unwrap() < wgsl.find("@group").unwrap());
    let span = generated
        .wgsl_spans
        .iter()
        .find(|span| span.label == "shell addBias")
        .expect("shell span");
    assert_eq!(span.start_line, span.end_line);
    validate(wgsl);
}

#[test]
fn wgsl_shell_preserves_relative_indentation_and_empty_lines() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, WgslShellSpec, computePipeline, wgslShell } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
function choose(input: u32): u32 { return input; }
const shell: WgslShellSpec = wgslShell<(input: u32) => u32>(choose, {
  body: "    if (input > 0u) {\n      return input;\n\n    }\n    return 0u;",
});
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output.set(0, choose(1)); }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [1, 1, 1] });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(
        wgsl.contains(
            "fn choose(input: u32) -> u32 {\n  if (input > 0u) {\n    return input;\n\n  }\n  return 0u;\n}"
        ),
        "{wgsl}"
    );
    assert!(!wgsl.lines().any(|line| line.ends_with(' ')), "{wgsl}");
    validate(wgsl);
}

#[test]
fn guarded_pipeline_emits_the_hidden_last_binding_and_three_axis_fence() {
    let generated = generate(
        r#"
import { ComputeInvocation, ComputePipelineSpec, MutStorage, computePipeline } from "./typegpu";
class Layout { output!: MutStorage<u32>; }
function kernel(res: Layout, ctx: ComputeInvocation): void { res.output[ctx.globalId.x] = 9; }
export const pipeline: ComputePipelineSpec = computePipeline<Layout>(kernel, { name: "pipeline", workgroupSize: [4, 2, 1], guarded: true });
"#,
    );
    let wgsl = &generated.pipelines[0].1;
    assert!(
        wgsl.contains("@group(0u) @binding(1u) var<uniform> pipeline_guard: vec3<u32>;"),
        "{wgsl}"
    );
    assert!(
        wgsl.contains("if (globalId.x < pipeline_guard.x && globalId.y < pipeline_guard.y && globalId.z < pipeline_guard.z)"),
        "{wgsl}"
    );
    assert!(
        generated.support_module.contains(
            "binding: 1, visibility: COMPUTE_VISIBILITY, kind: \"guard\", minBindingSize: 16"
        ),
        "{}",
        generated.support_module
    );
    assert!(!generated.support_module.contains("pipeline_guard!:"));
    assert!(
        generated
            .support_module
            .contains("  ], pipeline.guardBuffer(0));"),
        "{}",
        generated.support_module
    );
    validate(wgsl);
}
