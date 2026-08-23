//! The single source of GPU meanings for library calls.

use subscript_compiler::hir::MathFn;

// Synced with naga 29.0.1 keywords::wgsl.
pub(crate) const RESERVED: &[&str] = &[
    // Keywords
    "alias",
    "break",
    "case",
    "const",
    "const_assert",
    "continue",
    "continuing",
    "default",
    "diagnostic",
    "discard",
    "else",
    "enable",
    "false",
    "fn",
    "for",
    "if",
    "let",
    "loop",
    "override",
    "requires",
    "return",
    "struct",
    "switch",
    "true",
    "var",
    "while",
    // Reserved
    "NULL",
    "Self",
    "abstract",
    "active",
    "alignas",
    "alignof",
    "as",
    "asm",
    "asm_fragment",
    "async",
    "attribute",
    "auto",
    "await",
    "become",
    "cast",
    "catch",
    "class",
    "co_await",
    "co_return",
    "co_yield",
    "coherent",
    "column_major",
    "common",
    "compile",
    "compile_fragment",
    "concept",
    "const_cast",
    "consteval",
    "constexpr",
    "constinit",
    "crate",
    "debugger",
    "decltype",
    "delete",
    "demote",
    "demote_to_helper",
    "do",
    "dynamic_cast",
    "enum",
    "explicit",
    "export",
    "extends",
    "extern",
    "external",
    "fallthrough",
    "filter",
    "final",
    "finally",
    "friend",
    "from",
    "fxgroup",
    "get",
    "goto",
    "groupshared",
    "highp",
    "impl",
    "implements",
    "import",
    "inline",
    "instanceof",
    "interface",
    "layout",
    "lowp",
    "macro",
    "macro_rules",
    "match",
    "mediump",
    "meta",
    "mod",
    "module",
    "move",
    "mut",
    "mutable",
    "namespace",
    "new",
    "nil",
    "noexcept",
    "noinline",
    "nointerpolation",
    "non_coherent",
    "noncoherent",
    "noperspective",
    "null",
    "nullptr",
    "of",
    "operator",
    "package",
    "packoffset",
    "partition",
    "pass",
    "patch",
    "pixelfragment",
    "precise",
    "precision",
    "premerge",
    "priv",
    "protected",
    "pub",
    "public",
    "readonly",
    "ref",
    "regardless",
    "register",
    "reinterpret_cast",
    "require",
    "resource",
    "restrict",
    "self",
    "set",
    "shared",
    "sizeof",
    "smooth",
    "snorm",
    "static",
    "static_assert",
    "static_cast",
    "std",
    "subroutine",
    "super",
    "target",
    "template",
    "this",
    "thread_local",
    "throw",
    "trait",
    "try",
    "type",
    "typedef",
    "typeid",
    "typename",
    "typeof",
    "union",
    "unless",
    "unorm",
    "unsafe",
    "unsized",
    "use",
    "using",
    "varying",
    "virtual",
    "volatile",
    "wgsl",
    "where",
    "with",
    "writeonly",
    "yield",
];
pub(crate) const BUILTIN_IDENTIFIERS: &[&str] = &[
    // types
    "bool",
    "i32",
    "u32",
    "f32",
    "f16",
    "array",
    "atomic",
    "vec2",
    "vec3",
    "vec4",
    "mat2x2",
    "mat2x3",
    "mat2x4",
    "mat3x2",
    "mat3x3",
    "mat3x4",
    "mat4x2",
    "mat4x3",
    "mat4x4",
    "ptr",
    "sampler",
    "sampler_comparison",
    "texture_1d",
    "texture_2d",
    "texture_2d_array",
    "texture_3d",
    "texture_cube",
    "texture_cube_array",
    "texture_multisampled_2d",
    "texture_depth_multisampled_2d",
    "texture_external",
    "texture_storage_1d",
    "texture_storage_2d",
    "texture_storage_2d_array",
    "texture_storage_3d",
    "texture_depth_2d",
    "texture_depth_2d_array",
    "texture_depth_cube",
    "texture_depth_cube_array",
    // enumerants
    "read",
    "write",
    "read_write",
    "function",
    "private",
    "workgroup",
    "uniform",
    "storage",
    "rgba8unorm",
    "rgba8snorm",
    "rgba8uint",
    "rgba8sint",
    "rgba16unorm",
    "rgba16snorm",
    "rgba16uint",
    "rgba16sint",
    "rgba16float",
    "rg8unorm",
    "rg8snorm",
    "rg8uint",
    "rg8sint",
    "rg16unorm",
    "rg16snorm",
    "rg16uint",
    "rg16sint",
    "rg16float",
    "r32uint",
    "r32sint",
    "r32float",
    "rg32uint",
    "rg32sint",
    "rg32float",
    "rgba32uint",
    "rgba32sint",
    "rgba32float",
    "bgra8unorm",
    "r8unorm",
    "r8snorm",
    "r8uint",
    "r8sint",
    "r16unorm",
    "r16snorm",
    "r16uint",
    "r16sint",
    "r16float",
    "rgb10a2unorm",
    "rgb10a2uint",
    "rg11b10ufloat",
    // functions
    "bitcast",
    "all",
    "any",
    "select",
    "arrayLength",
    "abs",
    "acos",
    "acosh",
    "asin",
    "asinh",
    "atan",
    "atanh",
    "atan2",
    "ceil",
    "clamp",
    "cos",
    "cosh",
    "countLeadingZeros",
    "countOneBits",
    "countTrailingZeros",
    "cross",
    "degrees",
    "determinant",
    "distance",
    "dot",
    "dot4U8Packed",
    "dot4I8Packed",
    "exp",
    "exp2",
    "extractBits",
    "faceForward",
    "firstLeadingBit",
    "firstTrailingBit",
    "floor",
    "fma",
    "fract",
    "frexp",
    "insertBits",
    "inverseSqrt",
    "ldexp",
    "length",
    "log",
    "log2",
    "max",
    "min",
    "mix",
    "modf",
    "normalize",
    "pow",
    "quantizeToF16",
    "radians",
    "reflect",
    "refract",
    "reverseBits",
    "round",
    "saturate",
    "sign",
    "sin",
    "sinh",
    "smoothstep",
    "sqrt",
    "step",
    "tan",
    "tanh",
    "transpose",
    "trunc",
    "dpdx",
    "dpdxCoarse",
    "dpdxFine",
    "dpdy",
    "dpdyCoarse",
    "dpdyFine",
    "fwidth",
    "fwidthCoarse",
    "fwidthFine",
    "textureDimensions",
    "textureGather",
    "textureGatherCompare",
    "textureLoad",
    "textureNumLayers",
    "textureNumLevels",
    "textureNumSamples",
    "textureSample",
    "textureSampleBias",
    "textureSampleCompare",
    "textureSampleCompareLevel",
    "textureSampleGrad",
    "textureSampleLevel",
    "textureSampleBaseClampToEdge",
    "textureStore",
    "atomicLoad",
    "atomicStore",
    "atomicAdd",
    "atomicSub",
    "atomicMax",
    "atomicMin",
    "atomicAnd",
    "atomicOr",
    "atomicXor",
    "atomicExchange",
    "atomicCompareExchangeWeak",
    "pack4x8snorm",
    "pack4x8unorm",
    "pack4xI8",
    "pack4xU8",
    "pack4xI8Clamp",
    "pack4xU8Clamp",
    "pack2x16snorm",
    "pack2x16unorm",
    "pack2x16float",
    "unpack4x8snorm",
    "unpack4x8unorm",
    "unpack4xI8",
    "unpack4xU8",
    "unpack2x16snorm",
    "unpack2x16unorm",
    "unpack2x16float",
    "storageBarrier",
    "textureBarrier",
    "workgroupBarrier",
    "workgroupUniformLoad",
    "subgroupAdd",
    "subgroupExclusiveAdd",
    "subgroupInclusiveAdd",
    "subgroupAll",
    "subgroupAnd",
    "subgroupAny",
    "subgroupBallot",
    "subgroupBroadcast",
    "subgroupBroadcastFirst",
    "subgroupElect",
    "subgroupMax",
    "subgroupMin",
    "subgroupMul",
    "subgroupExclusiveMul",
    "subgroupInclusiveMul",
    "subgroupOr",
    "subgroupShuffle",
    "subgroupShuffleDown",
    "subgroupShuffleUp",
    "subgroupShuffleXor",
    "subgroupXor",
    "quadBroadcast",
    "quadSwapDiagonal",
    "quadSwapX",
    "quadSwapY",
    // not in the WGSL spec
    "i64",
    "u64",
    "f64",
    "push_constant",
    "r64uint",
];

pub(crate) fn ident(name: &str) -> String {
    let mut result = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if name.starts_with("_g_")
        || RESERVED.contains(&result.as_str())
        || BUILTIN_IDENTIFIERS.contains(&result.as_str())
    {
        result.push('_');
    } else if result.ends_with('_') {
        // A legal author identifier that is the result of mangling another name
        // moves one step farther away, so the mapping stays injective.
        result.push('_');
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MethodEmission {
    Binary(&'static str),
    Builtin(&'static str),
    Unary(&'static str),
    Swizzle(&'static str),
}

#[derive(Debug)]
struct MethodGroup {
    receivers: &'static [&'static str],
    methods: &'static [(&'static str, MethodEmission)],
}

const FLOAT_VECTORS: &[&str] = &["Vec2f", "Vec3f", "Vec4f"];
const SIGNED_VECTORS: &[&str] = &["Vec2i", "Vec3i", "Vec4i"];
const UNSIGNED_VECTORS: &[&str] = &["Vec2u", "Vec3u", "Vec4u"];
const BOOL_VECTORS: &[&str] = &["Vec2b", "Vec3b", "Vec4b"];
const MATRICES: &[&str] = &["Mat2x2f", "Mat3x3f", "Mat4x4f"];

const VECTOR_BASE: &[(&str, MethodEmission)] = &[
    ("add", MethodEmission::Binary("+")),
    ("sub", MethodEmission::Binary("-")),
    ("mul", MethodEmission::Binary("*")),
    ("scale", MethodEmission::Binary("*")),
    ("dot", MethodEmission::Builtin("dot")),
];
const FLOAT_METHODS: &[(&str, MethodEmission)] = &[
    ("length", MethodEmission::Builtin("length")),
    ("normalize", MethodEmission::Builtin("normalize")),
    ("abs", MethodEmission::Builtin("abs")),
    ("floor", MethodEmission::Builtin("floor")),
    ("ceil", MethodEmission::Builtin("ceil")),
    ("fract", MethodEmission::Builtin("fract")),
    ("sqrt", MethodEmission::Builtin("sqrt")),
    ("exp", MethodEmission::Builtin("exp")),
    ("log", MethodEmission::Builtin("log")),
    ("sin", MethodEmission::Builtin("sin")),
    ("cos", MethodEmission::Builtin("cos")),
    ("tan", MethodEmission::Builtin("tan")),
    ("sign", MethodEmission::Builtin("sign")),
    ("min", MethodEmission::Builtin("min")),
    ("max", MethodEmission::Builtin("max")),
    ("clamp", MethodEmission::Builtin("clamp")),
    ("pow", MethodEmission::Builtin("pow")),
    ("mix", MethodEmission::Builtin("mix")),
    ("step", MethodEmission::Builtin("step")),
    ("smoothstep", MethodEmission::Builtin("smoothstep")),
    ("distance", MethodEmission::Builtin("distance")),
    ("reflect", MethodEmission::Builtin("reflect")),
    ("refract", MethodEmission::Builtin("refract")),
    ("faceForward", MethodEmission::Builtin("faceForward")),
];
const SIGNED_METHODS: &[(&str, MethodEmission)] = &[
    ("abs", MethodEmission::Builtin("abs")),
    ("min", MethodEmission::Builtin("min")),
    ("max", MethodEmission::Builtin("max")),
    ("clamp", MethodEmission::Builtin("clamp")),
];
const UNSIGNED_METHODS: &[(&str, MethodEmission)] = &[
    ("min", MethodEmission::Builtin("min")),
    ("max", MethodEmission::Builtin("max")),
    ("clamp", MethodEmission::Builtin("clamp")),
];
const COMPARISON_METHODS: &[(&str, MethodEmission)] = &[
    ("lt", MethodEmission::Binary("<")),
    ("le", MethodEmission::Binary("<=")),
    ("gt", MethodEmission::Binary(">")),
    ("ge", MethodEmission::Binary(">=")),
    ("eq", MethodEmission::Binary("==")),
    ("ne", MethodEmission::Binary("!=")),
    ("select", MethodEmission::Builtin("select")),
];
const VEC3_SWIZZLES: &[(&str, MethodEmission)] = &[
    ("xy", MethodEmission::Swizzle("xy")),
    ("xz", MethodEmission::Swizzle("xz")),
    ("yz", MethodEmission::Swizzle("yz")),
];
const VEC4_SWIZZLES: &[(&str, MethodEmission)] = &[
    ("xy", MethodEmission::Swizzle("xy")),
    ("xz", MethodEmission::Swizzle("xz")),
    ("xw", MethodEmission::Swizzle("xw")),
    ("yz", MethodEmission::Swizzle("yz")),
    ("yw", MethodEmission::Swizzle("yw")),
    ("zw", MethodEmission::Swizzle("zw")),
    ("xyz", MethodEmission::Swizzle("xyz")),
    ("xyw", MethodEmission::Swizzle("xyw")),
    ("xzw", MethodEmission::Swizzle("xzw")),
    ("yzw", MethodEmission::Swizzle("yzw")),
];
const METHOD_GROUPS: &[MethodGroup] = &[
    MethodGroup {
        receivers: FLOAT_VECTORS,
        methods: VECTOR_BASE,
    },
    MethodGroup {
        receivers: SIGNED_VECTORS,
        methods: VECTOR_BASE,
    },
    MethodGroup {
        receivers: UNSIGNED_VECTORS,
        methods: VECTOR_BASE,
    },
    MethodGroup {
        receivers: FLOAT_VECTORS,
        methods: FLOAT_METHODS,
    },
    MethodGroup {
        receivers: SIGNED_VECTORS,
        methods: SIGNED_METHODS,
    },
    MethodGroup {
        receivers: UNSIGNED_VECTORS,
        methods: UNSIGNED_METHODS,
    },
    MethodGroup {
        receivers: FLOAT_VECTORS,
        methods: COMPARISON_METHODS,
    },
    MethodGroup {
        receivers: SIGNED_VECTORS,
        methods: COMPARISON_METHODS,
    },
    MethodGroup {
        receivers: UNSIGNED_VECTORS,
        methods: COMPARISON_METHODS,
    },
    MethodGroup {
        receivers: BOOL_VECTORS,
        methods: &[
            ("any", MethodEmission::Builtin("any")),
            ("all", MethodEmission::Builtin("all")),
            ("not", MethodEmission::Unary("!")),
        ],
    },
    MethodGroup {
        receivers: &["Vec3f"],
        methods: &[("cross", MethodEmission::Builtin("cross"))],
    },
    MethodGroup {
        receivers: &["Vec3f", "Vec3i", "Vec3u"],
        methods: VEC3_SWIZZLES,
    },
    MethodGroup {
        receivers: &["Vec4f", "Vec4i", "Vec4u"],
        methods: VEC4_SWIZZLES,
    },
    MethodGroup {
        receivers: MATRICES,
        methods: &[
            ("mul", MethodEmission::Binary("*")),
            ("mulVec", MethodEmission::Binary("*")),
            ("transpose", MethodEmission::Builtin("transpose")),
        ],
    },
];

pub(crate) fn method(receiver: &str, name: &str) -> Option<MethodEmission> {
    METHOD_GROUPS
        .iter()
        .filter(|group| group.receivers.contains(&receiver))
        .flat_map(|group| group.methods)
        .find_map(|(method, emission)| (*method == name).then_some(*emission))
}

pub(crate) fn math(function: MathFn) -> Option<&'static str> {
    Some(match function {
        MathFn::Abs => "abs",
        MathFn::Min => "min",
        MathFn::Max => "max",
        MathFn::Floor => "floor",
        MathFn::Ceil => "ceil",
        MathFn::Sqrt => "sqrt",
        MathFn::Pow => "pow",
        MathFn::Exp => "exp",
        MathFn::Log => "log",
        MathFn::Sin => "sin",
        MathFn::Cos => "cos",
        MathFn::Tan => "tan",
        MathFn::Fround => "",
        _ => return None,
    })
}

pub(crate) fn free_function(name: &str) -> Option<&'static str> {
    let base = name.split('<').next().unwrap_or(name);
    Some(match base {
        "clamp" => "clamp",
        "mix" => "mix",
        "step" => "step",
        "smoothstep" => "smoothstep",
        "fract" => "fract",
        "sign" => "sign",
        "v2f" => "vec2<f32>",
        "v3f" => "vec3<f32>",
        "v4f" => "vec4<f32>",
        "v2i" => "vec2<i32>",
        "v3i" => "vec3<i32>",
        "v4i" => "vec4<i32>",
        "v2u" => "vec2<u32>",
        "v3u" => "vec3<u32>",
        "v4u" => "vec4<u32>",
        "v3fFrom2" => "vec3<f32>",
        "v4fFrom2" | "v4fFrom3" => "vec4<f32>",
        "v2fSplat" => "vec2<f32>",
        "v3fSplat" => "vec3<f32>",
        "v4fSplat" => "vec4<f32>",
        "v3iFrom2" => "vec3<i32>",
        "v4iFrom2" | "v4iFrom3" => "vec4<i32>",
        "v2iSplat" => "vec2<i32>",
        "v3iSplat" => "vec3<i32>",
        "v4iSplat" => "vec4<i32>",
        "v3uFrom2" => "vec3<u32>",
        "v4uFrom2" | "v4uFrom3" => "vec4<u32>",
        "v2uSplat" => "vec2<u32>",
        "v3uSplat" => "vec3<u32>",
        "v4uSplat" => "vec4<u32>",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use subscript_compiler::SourceFile;

    use super::*;

    #[test]
    fn tables_pin_the_decided_surface() {
        assert_eq!(method("Vec3f", "dot"), Some(MethodEmission::Builtin("dot")));
        assert_eq!(
            method("Mat4x4f", "mulVec"),
            Some(MethodEmission::Binary("*"))
        );
        assert_eq!(math(MathFn::Fround), Some(""));
        assert_eq!(free_function("smoothstep"), Some("smoothstep"));
        assert_eq!(
            method("Vec3f", "reflect"),
            Some(MethodEmission::Builtin("reflect"))
        );
        assert_eq!(method("Vec3h", "abs"), None);
    }

    #[test]
    fn k10_table_matches_vector_and_matrix_hir_methods() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("generator crate is under the repository root")
            .to_path_buf();
        let path = root.join("lib/typegpu-types.ts");
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let module =
            subscript_compiler::check_program(&[SourceFile::new("typegpu-types.ts", source)])
                .expect("check typegpu-types.ts");
        let hir = module
            .classes
            .iter()
            .filter(|class| {
                class.pos.file == "typegpu-types.ts"
                    && (class.name.starts_with("Vec") || class.name.starts_with("Mat"))
            })
            .flat_map(|class| {
                class
                    .methods
                    .iter()
                    .map(|function| (class.name.clone(), function.name.clone()))
            })
            .collect::<BTreeSet<_>>();
        let table = METHOD_GROUPS
            .iter()
            .flat_map(|group| {
                group.receivers.iter().flat_map(|receiver| {
                    group
                        .methods
                        .iter()
                        .map(|(name, _)| ((*receiver).to_owned(), (*name).to_owned()))
                })
            })
            .collect::<BTreeSet<_>>();
        let extra = table.difference(&hir).cloned().collect::<Vec<_>>();
        assert!(
            extra.is_empty(),
            "K10 table has rows without HIR methods: {extra:?}"
        );
        let missing = hir.difference(&table).cloned().collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "K10 table lacks HIR methods: {missing:?}"
        );
    }

    #[test]
    fn identifier_lists_match_naga_29() {
        assert_eq!(RESERVED, naga::keywords::wgsl::RESERVED);
        assert_eq!(
            BUILTIN_IDENTIFIERS,
            naga::keywords::wgsl::BUILTIN_IDENTIFIERS
        );
        for name in RESERVED.iter().chain(BUILTIN_IDENTIFIERS) {
            assert_eq!(ident(name), format!("{name}_"));
        }
        assert_eq!(ident("userName"), "userName");
        assert_eq!(ident("let"), "let_");
        assert_eq!(ident("let_"), "let__");
        assert_eq!(ident("_g_conditional_0"), "_g_conditional_0_");
    }
}
