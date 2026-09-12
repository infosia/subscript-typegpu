//! Pipeline declarations, layouts, and binding-wrapper discovery.

use std::collections::BTreeSet;

use subscript_compiler::hir::{Callee, ClassDef, Expr, ExprKind, Function, Module, Stmt};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

/// The address space and the resource kind of one layout binding (PI5, TX1).
///
/// `Guard` is the hidden uniform binding of a guarded declaration (PI15). It reaches the layout
/// spec and no resources class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingKind {
    Uniform,
    Storage,
    MutStorage,
    Texture(TextureSampleType, TextureViewDimension),
    StorageTexture(
        StorageTextureFormat,
        StorageTextureAccess,
        TextureViewDimension,
    ),
    Sampler,
    Guard,
}

/// The sampled type of a texture binding (TX1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureSampleType {
    Float,
}

/// The view dimension of a texture binding (TX1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureViewDimension {
    TwoD,
    TwoDArray,
}

impl TextureViewDimension {
    /// Returns the dimension of the emitted WGSL texture type, such as `2d_array`.
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::TwoD => "2d",
            Self::TwoDArray => "2d_array",
        }
    }

    /// Returns the `GPUTextureViewDimension` value that the generated layout spec carries.
    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::TwoD => "2d",
            Self::TwoDArray => "2d-array",
        }
    }
}

impl TextureSampleType {
    /// Returns the sampled type argument of the emitted WGSL `texture_2d` type.
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Float => "f32",
        }
    }

    /// Returns the `GPUTextureSampleType` value that the generated layout spec carries.
    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::Float => "float",
        }
    }
}

/// The format of a storage texture binding (TX1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageTextureFormat {
    Rgba8unorm,
    Rgba16float,
    R32float,
    Rgba32float,
}

/// The access mode of a storage texture binding (TX1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageTextureAccess {
    Write,
    Read,
    ReadWrite,
}

impl StorageTextureAccess {
    /// Returns the access argument of the emitted WGSL `texture_storage_2d` type.
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::Read => "read",
            Self::ReadWrite => "read_write",
        }
    }

    /// Returns the `GPUStorageTextureAccess` value that the generated layout spec carries.
    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::Write => "write-only",
            Self::Read => "read-only",
            Self::ReadWrite => "read-write",
        }
    }

    /// Reports whether a kernel can call `dimensions` and `load` on a binding with this access.
    pub(crate) fn can_read(self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite)
    }

    /// Reports whether a kernel can call `store` on a binding with this access.
    pub(crate) fn can_write(self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite)
    }
}

impl StorageTextureFormat {
    /// Returns the format argument of the emitted WGSL `texture_storage_2d` type.
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Rgba8unorm => "rgba8unorm",
            Self::Rgba16float => "rgba16float",
            Self::R32float => "r32float",
            Self::Rgba32float => "rgba32float",
        }
    }

    /// Returns the `GPUTextureFormat` value that the generated layout spec carries.
    pub(crate) fn webgpu(self) -> &'static str {
        self.wgsl()
    }
}

impl BindingKind {
    /// Returns the address space of the emitted WGSL `var` declaration.
    ///
    /// A texture and a sampler declare no address space, so both give the empty string.
    pub(crate) fn wgsl(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Storage => "storage, read",
            Self::MutStorage => "storage, read_write",
            Self::Guard => "uniform",
            Self::Texture(_, _) | Self::StorageTexture(_, _, _) | Self::Sampler => "",
        }
    }

    /// Returns the `kind` value that the generated layout spec entry carries.
    pub(crate) fn webgpu(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Storage => "read-only-storage",
            Self::MutStorage => "storage",
            Self::Texture(_, _) => "texture",
            Self::StorageTexture(_, _, _) => "storageTexture",
            Self::Sampler => "sampler",
            Self::Guard => "guard",
        }
    }

    /// Reports whether a `GPUBuffer` fills the binding, which every guard binding also does.
    pub(crate) fn is_buffer(self) -> bool {
        matches!(
            self,
            Self::Uniform | Self::Storage | Self::MutStorage | Self::Guard
        )
    }
}

/// One binding of a layout class.
#[derive(Debug, Clone)]
pub(crate) struct Binding {
    /// The layout field name, which is also the emitted WGSL variable name.
    pub(crate) name: String,
    /// The binding index, which is the field's declaration position from 0 (PI3).
    pub(crate) index: u32,
    /// The address space and the resource kind.
    pub(crate) kind: BindingKind,
    /// The wrapper's item type `T`, which sizes the binding.
    pub(crate) item_ty: Type,
    /// The field declaration position.
    pub(crate) pos: Pos,
}

/// One layout class of a pipeline declaration (PI3).
#[derive(Debug, Clone)]
pub(crate) struct Layout {
    /// The layout class name.
    pub(crate) name: String,
    /// The bind group index, which is the kernel's parameter order (PI2).
    pub(crate) group: u32,
    /// The bindings, in field declaration order. A guarded declaration appends its guard last.
    pub(crate) bindings: Vec<Binding>,
}

/// One compute pipeline declaration (PI1).
#[derive(Debug, Clone)]
pub(crate) struct Pipeline {
    /// The module-level `const` name that carries the declaration.
    pub(crate) declaration: String,
    /// The kernel function name, which becomes the WGSL entry point.
    pub(crate) entry: String,
    /// The workgroup size, from the descriptor literal.
    pub(crate) workgroup: [u32; 3],
    /// Whether sequential host simulation keeps the kernel's behavior (CL2).
    pub(crate) host_runnable: bool,
    /// Whether the declaration owns a hidden guard binding (PI15).
    pub(crate) guarded: bool,
    /// The layout classes, in group order from 0.
    pub(crate) layouts: Vec<Layout>,
    /// The declaration position.
    pub(crate) pos: Pos,
}

/// Builds one author-facing diagnostic that names `rule`, the single rule it enforces (PI13).
fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

/// Builds a diagnostic that names the generator as its source (K15).
///
/// The checker already typed the declaration against the library signature, so a shape the
/// generator cannot read here is a generator defect, never an author mistake.
fn generator_diagnostic(message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("K15: {} (generator)", message.into()),
        pos,
    )
}

/// Returns the class name of a class type, and `None` for every other type.
pub(crate) fn class_name<'a>(module: &'a Module, ty: &Type) -> Option<&'a str> {
    let Type::Class(id) = ty else { return None };
    module.classes.get(id.0).map(|class| class.name.as_str())
}

/// Renders one type the way the checker prints it, for a diagnostic message.
pub(crate) fn type_name(module: &Module, ty: &Type, pos: &Pos) -> Result<String, Diagnostic> {
    // Type display requires infallible callbacks. A missing name invalidates the complete result.
    let error = std::cell::RefCell::new(None);
    let name = |value: Option<&str>, table: &str, index: usize| {
        value.map(str::to_owned).unwrap_or_else(|| {
            *error.borrow_mut() = Some(crate::internal(
                "pipeline::type_name",
                format!("missing {table} {index}"),
                pos,
            ));
            String::new()
        })
    };
    let text = subscript_compiler::types::display_type(
        ty,
        &|id| {
            name(
                module.classes.get(id.0).map(|item| item.name.as_str()),
                "class",
                id.0,
            )
        },
        &|id| {
            name(
                module.enums.get(id.0).map(|item| item.name.as_str()),
                "enum",
                id.0,
            )
        },
        &|id| {
            name(
                module
                    .string_aliases
                    .get(id.0)
                    .map(|item| item.name.as_str()),
                "string alias",
                id.0,
            )
        },
    );
    match error.into_inner() {
        Some(error) => Err(error),
        None => Ok(text),
    }
}

/// Returns the class definition when the type is a class that `typegpu.ts` declares.
///
/// The file that declares a class identifies a library class. The class name alone never does.
pub(crate) fn library_class<'a>(
    module: &'a Module,
    ty: &Type,
) -> Option<&'a subscript_compiler::hir::ClassDef> {
    let Type::Class(id) = ty else { return None };
    module
        .classes
        .get(id.0)
        .filter(|class| class.pos.file == "typegpu.ts")
}

/// Classifies one layout field type as a binding wrapper and returns its item type (PI5, TX1).
///
/// The result is `None` when the type is not a library wrapper, which the caller reports as a PI3
/// violation. A buffer wrapper carries its `T` in the host body's `values` array, and a storage
/// texture carries its format marker in `formats`.
///
/// # Errors
///
/// Returns a TX1 diagnostic for a comparison sampler, a texture sample type outside `f32`, and a
/// storage texture format outside the marker set. A library wrapper that lost a marker field gives
/// a generator diagnostic.
fn wrapper(
    module: &Module,
    ty: &Type,
    pos: &Pos,
) -> Result<Option<(BindingKind, Type)>, Diagnostic> {
    // A comparison sampler needs a typed comparison operation set that this revision does not
    // define, so the check precedes the wrapper match (TX1).
    if class_name(module, ty).is_some_and(|name| name == "ComparisonSampler") {
        return Err(diagnostic(
            "TX1",
            "comparison samplers are not supported in this revision",
            pos.clone(),
        ));
    }
    let Some(class) = library_class(module, ty) else {
        return Ok(None);
    };
    let kind = if class.name.starts_with("Uniform<") {
        BindingKind::Uniform
    } else if class.name.starts_with("Storage<") {
        BindingKind::Storage
    } else if class.name.starts_with("MutStorage<") {
        BindingKind::MutStorage
    } else if class.name.starts_with("Texture2d<") || class.name.starts_with("Texture2dArray<") {
        let Some(values) = class.fields.iter().find(|field| field.name == "values") else {
            return Err(generator_diagnostic(
                "library texture wrapper lost its typed marker field",
                pos.clone(),
            ));
        };
        let Type::Array(item) = &values.ty else {
            return Err(generator_diagnostic(
                "library texture wrapper typed marker is not an array",
                pos.clone(),
            ));
        };
        if item.as_ref() != &Type::F32 {
            return Err(diagnostic(
                "TX1",
                "texture sample type must be f32",
                pos.clone(),
            ));
        }
        return Ok(Some((
            BindingKind::Texture(
                TextureSampleType::Float,
                if class.name.starts_with("Texture2dArray<") {
                    TextureViewDimension::TwoDArray
                } else {
                    TextureViewDimension::TwoD
                },
            ),
            Type::F32,
        )));
    } else if class.name.starts_with("StorageTexture2d<")
        || class.name.starts_with("ReadStorageTexture2d<")
        || class.name.starts_with("ReadWriteStorageTexture2d<")
        || class.name.starts_with("ReadStorageTexture2dArray<")
        || class.name.starts_with("WriteStorageTexture2dArray<")
    {
        let access = if class.name.starts_with("ReadWriteStorageTexture2d<") {
            StorageTextureAccess::ReadWrite
        } else if class.name.starts_with("ReadStorageTexture2d<")
            || class.name.starts_with("ReadStorageTexture2dArray<")
        {
            StorageTextureAccess::Read
        } else {
            StorageTextureAccess::Write
        };
        let Some(formats) = class.fields.iter().find(|field| field.name == "formats") else {
            return Err(generator_diagnostic(
                "library storage texture wrapper lost its format marker field",
                pos.clone(),
            ));
        };
        let Type::Array(item) = &formats.ty else {
            return Err(generator_diagnostic(
                "library storage texture wrapper format marker is not an array",
                pos.clone(),
            ));
        };
        let marker = library_class(module, item).and_then(|marker| match marker.name.as_str() {
            "Rgba8unorm" => Some(StorageTextureFormat::Rgba8unorm),
            "Rgba16float" => Some(StorageTextureFormat::Rgba16float),
            "R32float" => Some(StorageTextureFormat::R32float),
            "Rgba32float" => Some(StorageTextureFormat::Rgba32float),
            _ => None,
        });
        let Some(format) = marker else {
            return Err(diagnostic(
                "TX1",
                "storage texture format must be a float-channel library marker",
                pos.clone(),
            ));
        };
        return Ok(Some((
            BindingKind::StorageTexture(
                format,
                access,
                if class.name.contains("Texture2dArray<") {
                    TextureViewDimension::TwoDArray
                } else {
                    TextureViewDimension::TwoD
                },
            ),
            (**item).clone(),
        )));
    } else if class.name == "Sampler" {
        return Ok(Some((BindingKind::Sampler, ty.clone())));
    } else {
        return Ok(None);
    };
    let Some(values) = class.fields.iter().find(|field| field.name == "values") else {
        return Ok(None);
    };
    let Type::Array(item) = &values.ty else {
        return Ok(None);
    };
    Ok(Some((kind, (**item).clone())))
}

/// Reports whether `ty` is a legal item type for a buffer binding wrapper (PI5).
///
/// The set is `f32`, `i32`, `u32`, a schema class, and a library vector or matrix. A library
/// runtime class and a bool vector are outside it.
///
/// # Errors
///
/// If the type names a class the module does not hold, returns an internal diagnostic.
fn allowed_binding_item(module: &Module, ty: &Type, pos: &Pos) -> Result<bool, Diagnostic> {
    Ok(match ty {
        Type::F32 | Type::I32 | Type::U32 => true,
        Type::Class(id) => {
            let class = crate::class(module, id.0, "pipeline::allowed_binding_item", pos)?;
            class.is_value
                && (class.pos.file != "typegpu.ts")
                && (class.pos.file != "webgpu.ts")
                && !crate::schema::is_bool_vector(module, ty, pos)?
        }
        _ => false,
    })
}

/// Reports whether `statement` is the assignment `this.<field> = <parameter>` (PI3).
fn assigns_field(statement: &Stmt, field: &str, parameter: &str) -> bool {
    let Stmt::Expr(expr) = statement else {
        return false;
    };
    let ExprKind::Assign {
        op: None,
        target,
        value,
        ..
    } = &expr.kind
    else {
        return false;
    };
    let ExprKind::Field { obj, name, .. } = &target.kind else {
        return false;
    };
    if !matches!(obj.kind, ExprKind::This) || name != field {
        return false;
    }
    matches!(&value.kind, ExprKind::Local(local) if local == parameter)
}

/// Returns the position of `statement` when the statement carries one.
fn statement_pos(statement: &Stmt) -> Option<Pos> {
    match statement {
        Stmt::Expr(expr) => Some(expr.pos.clone()),
        Stmt::Let { pos, .. }
        | Stmt::Return { pos, .. }
        | Stmt::If { pos, .. }
        | Stmt::While { pos, .. }
        | Stmt::For { pos, .. }
        | Stmt::ForOf { pos, .. }
        | Stmt::Switch { pos, .. } => Some(pos.clone()),
        Stmt::Break(pos) | Stmt::Continue(pos) => Some(pos.clone()),
        _ => None,
    }
}

/// Checks the constructor of a layout class against PI3.
///
/// The constructor takes one parameter per field, in declaration order, each typed as its field.
/// The body is the assignments `this.<field> = <parameter>` in the same order and nothing else.
/// The generator reads the field list, so one spelling serves every layout class.
///
/// # Errors
///
/// A class with no constructor, and a constructor of any other form, give a PI3 diagnostic that
/// names the class and the first departure.
fn layout_constructor(class: &ClassDef) -> Result<(), Diagnostic> {
    let Some(ctor) = &class.ctor else {
        return Err(diagnostic(
            "PI3",
            format!("layout class `{}` declares no constructor", class.name),
            class.pos.clone(),
        ));
    };
    for (index, field) in class.fields.iter().enumerate() {
        let Some(parameter) = ctor.params.get(index) else {
            return Err(diagnostic(
                "PI3",
                format!(
                    "layout constructor of `{}` declares no parameter for field `{}`",
                    class.name, field.name
                ),
                ctor.pos.clone(),
            ));
        };
        if parameter.ty != field.ty {
            return Err(diagnostic(
                "PI3",
                format!(
                    "layout constructor parameter `{}` of `{}` is not the type of field `{}`",
                    parameter.name, class.name, field.name
                ),
                parameter.pos.clone(),
            ));
        }
        let Some(statement) = ctor.body.get(index) else {
            return Err(diagnostic(
                "PI3",
                format!(
                    "layout constructor of `{}` omits the assignment `this.{} = {}`",
                    class.name, field.name, parameter.name
                ),
                ctor.pos.clone(),
            ));
        };
        if !assigns_field(statement, &field.name, &parameter.name) {
            return Err(diagnostic(
                "PI3",
                format!(
                    "layout constructor of `{}` holds a statement that is not `this.{} = {}`",
                    class.name, field.name, parameter.name
                ),
                statement_pos(statement).unwrap_or_else(|| ctor.pos.clone()),
            ));
        }
    }
    if let Some(parameter) = ctor.params.get(class.fields.len()) {
        return Err(diagnostic(
            "PI3",
            format!(
                "layout constructor of `{}` declares the parameter `{}` that names no field",
                class.name, parameter.name
            ),
            parameter.pos.clone(),
        ));
    }
    if let Some(statement) = ctor.body.get(class.fields.len()) {
        return Err(diagnostic(
            "PI3",
            format!(
                "layout constructor of `{}` holds a statement after the field assignments",
                class.name
            ),
            statement_pos(statement).unwrap_or_else(|| ctor.pos.clone()),
        ));
    }
    Ok(())
}

/// Reads one layout class into its bindings (PI3).
///
/// `group` becomes the bind group index. Binding indices follow field declaration order from 0.
///
/// # Errors
///
/// A class that is not a plain class of binding wrappers gives a PI3 diagnostic, and so does a
/// constructor outside the PI3 form. A class with no field gives a TX2 diagnostic. A buffer item
/// type outside PI5 gives a PI5 diagnostic.
pub(crate) fn layout(
    module: &Module,
    ty: &Type,
    group: u32,
    pos: &Pos,
) -> Result<Layout, Diagnostic> {
    let Type::Class(id) = ty else {
        return Err(diagnostic(
            "PI3",
            "pipeline layout is not a class",
            Pos::new("", 1, 1),
        ));
    };
    let class = crate::class(module, id.0, "pipeline::layout", pos)?;
    // A layout class is a plain class: not `@CStruct`, not `@Descriptor`, and not a library class
    // (PI3). The author never instantiates it.
    if class.is_value || class.is_descriptor || class.pos.file == "typegpu.ts" {
        return Err(diagnostic(
            "PI3",
            format!("`{}` is not a plain layout class", class.name),
            class.pos.clone(),
        ));
    }
    if !class.methods.is_empty() || class.index_signature.is_some() {
        return Err(diagnostic(
            "PI3",
            format!("layout class `{}` contains a non-field member", class.name),
            class.pos.clone(),
        ));
    }
    if class.fields.is_empty() {
        return Err(diagnostic(
            "TX2",
            format!("layout class `{}` is empty", class.name),
            class.pos.clone(),
        ));
    }
    layout_constructor(class)?;
    // The binding index is the field's declaration position from 0 (PI3). A guarded declaration
    // appends its hidden binding after this loop.
    let mut bindings = Vec::new();
    for (index, field) in class.fields.iter().enumerate() {
        let Some((kind, item_ty)) = wrapper(module, &field.ty, &field.pos)? else {
            return Err(diagnostic(
                "PI3",
                format!(
                    "layout field `{}.{}` is not a Uniform, Storage, or MutStorage binding wrapper",
                    class.name, field.name
                ),
                field.pos.clone(),
            ));
        };
        if kind.is_buffer() && !allowed_binding_item(module, &item_ty, &field.pos)? {
            return Err(diagnostic(
                "PI5",
                format!(
                    "layout field `{}.{}` has a binding item type outside PI5",
                    class.name, field.name
                ),
                field.pos.clone(),
            ));
        }
        bindings.push(Binding {
            name: field.name.clone(),
            index: index as u32,
            kind,
            item_ty,
            pos: field.pos.clone(),
        });
    }
    Ok(Layout {
        name: class.name.clone(),
        group,
        bindings,
    })
}

/// Returns the value of an integer literal that fits a `u32`, and `None` for every other
/// expression.
fn literal_u32(expr: &Expr) -> Option<u32> {
    let ExprKind::Int(value) = expr.kind else {
        return None;
    };
    u32::try_from(value).ok()
}

/// Reads the `workgroupSize` member of a `ComputePipelineSpec` literal (PI1).
///
/// The size reaches the WGSL `@workgroup_size` attribute and the generated `_WORKGROUP_*`
/// constants, so it must be known before emission.
///
/// # Errors
///
/// Returns a PI1 diagnostic in three cases. The options are not a descriptor literal. The member
/// is absent, or it is not an array of three integer literals. An axis is zero.
fn workgroup(module: &Module, expr: &Expr) -> Result<[u32; 3], Diagnostic> {
    let ExprKind::DescriptorLit { .. } = &expr.kind else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let Some(field) = crate::descriptor_field(module, expr, "workgroupSize")? else {
        return Err(diagnostic(
            "PI1",
            "pipeline options omit workgroupSize",
            expr.pos.clone(),
        ));
    };
    let Some(value) = field else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            expr.pos.clone(),
        ));
    };
    let ExprKind::ArrayLit(values) = &value.kind else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            value.pos.clone(),
        ));
    };
    let [x_value, y_value, z_value] = values.as_slice() else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size requires three literals",
            value.pos.clone(),
        ));
    };
    let Some(x) = literal_u32(x_value) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            x_value.pos.clone(),
        ));
    };
    let Some(y) = literal_u32(y_value) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            y_value.pos.clone(),
        ));
    };
    let Some(z) = literal_u32(z_value) else {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup size is not literal",
            z_value.pos.clone(),
        ));
    };
    if x == 0 || y == 0 || z == 0 {
        return Err(diagnostic(
            "PI1",
            "pipeline workgroup dimensions must be nonzero",
            value.pos.clone(),
        ));
    }
    Ok([x, y, z])
}

/// Checks that the spec's `name` member equals the declaration's `const` name.
///
/// The runtime names the pipeline from `spec.name` in its CL2 trap, and the generated constants
/// carry the declaration name. The check keeps one name for one pipeline.
///
/// # Errors
///
/// Returns a PI1 diagnostic when the member is absent, is not a string literal, or differs from
/// `declaration`.
fn validate_pipeline_name(
    module: &Module,
    expr: &Expr,
    declaration: &str,
) -> Result<(), Diagnostic> {
    let ExprKind::DescriptorLit { .. } = &expr.kind else {
        return Err(diagnostic(
            "PI1",
            "pipeline options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let Some(field) = crate::descriptor_field(module, expr, "name")? else {
        return Err(generator_diagnostic(
            "library ComputePipelineSpec lost its name field",
            expr.pos.clone(),
        ));
    };
    let Some(value) = field else {
        return Err(diagnostic(
            "PI1",
            format!("pipeline `{declaration}` options omit name"),
            expr.pos.clone(),
        ));
    };
    let ExprKind::Str(name) = &value.kind else {
        return Err(diagnostic(
            "PI1",
            format!("pipeline `{declaration}` name is not a string literal"),
            value.pos.clone(),
        ));
    };
    if name != declaration {
        return Err(diagnostic(
            "PI1",
            format!("pipeline `{declaration}` options name is `{name}`"),
            value.pos.clone(),
        ));
    }
    Ok(())
}

/// Reads the `guarded` member of a `ComputePipelineSpec` literal, which defaults to false (PI15).
///
/// # Errors
///
/// Returns a PI15 diagnostic when the options are not a descriptor literal or when the member is
/// not a boolean literal. The generator emits the guard `if` at compile time, so a computed value
/// has no meaning.
fn guarded_option(module: &Module, expr: &Expr) -> Result<bool, Diagnostic> {
    let ExprKind::DescriptorLit { .. } = &expr.kind else {
        return Err(diagnostic(
            "PI15",
            "guarded pipeline options must be a descriptor literal",
            expr.pos.clone(),
        ));
    };
    let Some(field) = crate::descriptor_field(module, expr, "guarded")? else {
        return Err(generator_diagnostic(
            "library ComputePipelineSpec lost its guarded field",
            expr.pos.clone(),
        ));
    };
    match field {
        None => Ok(false),
        Some(Expr {
            kind: ExprKind::Bool(value),
            ..
        }) => Ok(*value),
        Some(value) => Err(diagnostic(
            "PI15",
            "pipeline guarded option must be a boolean literal",
            value.pos.clone(),
        )),
    }
}

/// Returns the module-level function of this name, and `None` when the module declares none.
pub(crate) fn function<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
}

/// Returns the layout count of a `computePipeline` declaration function, and `None` for every
/// other call.
///
/// The layout count is the group count (PI2). The declaring file identifies the library function,
/// so a program's own `computePipeline` never matches (PI1).
fn compute_arity(module: &Module, name: &str) -> Option<usize> {
    let base = crate::base_name(name);
    let declaration = function(module, name)?;
    if declaration.params.first()?.pos.file != "typegpu.ts" {
        return None;
    }
    Some(match base {
        "computePipeline" => 1,
        "computePipeline2" => 2,
        "computePipeline3" => 3,
        "computePipeline4" => 4,
        _ => return None,
    })
}

/// Reports whether `expr` holds a `computePipeline` call anywhere inside it.
///
/// The caller runs the walk over function bodies alone, where a declaration is a PI1 violation.
fn call_in_expr(module: &Module, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { callee: Callee::Func(name), .. } if compute_arity(module, name).is_some() => true,
        ExprKind::AbsenceTest { value: operand, .. } | ExprKind::Unary { operand, .. } | ExprKind::Cast(operand) | ExprKind::Length(operand) => call_in_expr(module, operand),
        ExprKind::Binary { left, right, .. } => call_in_expr(module, left) || call_in_expr(module, right),
        ExprKind::Assign { target, value, .. } => call_in_expr(module, target) || call_in_expr(module, value),
        ExprKind::Call { callee, args } => {
            matches!(callee, Callee::Value(value) if call_in_expr(module, value))
                || matches!(callee, Callee::Method { recv, .. } if call_in_expr(module, recv))
                || args.iter().any(|arg| call_in_expr(module, arg))
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => args.iter().any(|arg| call_in_expr(module, arg)),
        ExprKind::DescriptorLit { fields, .. } => fields.iter().flatten().any(|value| call_in_expr(module, value)),
        ExprKind::Field { obj, .. } | ExprKind::JsonResultValue(obj) => call_in_expr(module, obj),
        ExprKind::Index { obj, index, .. } => call_in_expr(module, obj) || call_in_expr(module, index),
        ExprKind::Template(parts) => parts.iter().any(|part| matches!(part, subscript_compiler::hir::TplPart::Expr(value) if call_in_expr(module, value))),
        ExprKind::Lambda { body, .. } => body.iter().any(|stmt| stmt_has_compute(module, stmt)),
        ExprKind::Cond { cond, then, els } => call_in_expr(module, cond) || call_in_expr(module, then) || call_in_expr(module, els),
        _ => false,
    }
}

/// Reports whether `stmt` holds a `computePipeline` call anywhere inside it, nested bodies
/// included.
fn stmt_has_compute(module: &Module, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => call_in_expr(module, init),
        Stmt::Return { value, .. } => value
            .as_ref()
            .is_some_and(|value| call_in_expr(module, value)),
        Stmt::If {
            cond, then, els, ..
        } => {
            call_in_expr(module, cond)
                || then.iter().any(|stmt| stmt_has_compute(module, stmt))
                || els
                    .as_ref()
                    .is_some_and(|items| items.iter().any(|stmt| stmt_has_compute(module, stmt)))
        }
        Stmt::While { cond, body, .. } => {
            call_in_expr(module, cond) || body.iter().any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            init.as_deref()
                .is_some_and(|stmt| stmt_has_compute(module, stmt))
                || cond
                    .as_ref()
                    .is_some_and(|value| call_in_expr(module, value))
                || step
                    .as_ref()
                    .is_some_and(|value| call_in_expr(module, value))
                || body.iter().any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::ForOf { subject, body, .. } => {
            call_in_expr(module, subject) || body.iter().any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::Switch { disc, cases, .. } => {
            call_in_expr(module, disc)
                || cases
                    .iter()
                    .flat_map(|case| &case.body)
                    .any(|stmt| stmt_has_compute(module, stmt))
        }
        Stmt::Block(body) => body.iter().any(|stmt| stmt_has_compute(module, stmt)),
        Stmt::Break(_) | Stmt::Continue(_) => false,
    }
}

/// Collects every module-level compute pipeline declaration of one program (PI1).
///
/// `shells` names the functions whose bodies are author WGSL, which the barrier check and the
/// host-runnable check both skip. The result follows the module's global declaration order.
///
/// # Errors
///
/// Returns every PI1, PI3, PI5, and PI15 violation. A declaration inside a function, a mutable
/// declaration, a non-literal workgroup size, and a guarded kernel that reaches a barrier are
/// the cases.
pub(crate) fn discover(
    module: &Module,
    shells: &crate::shell::ShellProgram,
) -> Result<Vec<Pipeline>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    // A declaration is a module-level `const`, because the generator reads it at compile time. A
    // call inside a function body reaches no generated constant (PI1).
    for function in &module.functions {
        if function.pos.file != "typegpu.ts"
            && function
                .body
                .iter()
                .any(|stmt| stmt_has_compute(module, stmt))
        {
            diagnostics.push(diagnostic(
                "PI1",
                "a pipeline declaration appears inside a function",
                function.pos.clone(),
            ));
        }
    }
    let mut pipelines = Vec::new();
    for global in &module.globals {
        let ExprKind::Call {
            callee: Callee::Func(callee),
            args,
        } = &global.init.kind
        else {
            continue;
        };
        let Some(arity) = compute_arity(module, callee) else {
            continue;
        };
        if global.mutable {
            diagnostics.push(diagnostic(
                "PI1",
                "a pipeline declaration must be const",
                global.pos.clone(),
            ));
            continue;
        }
        let Some(Expr {
            kind: ExprKind::FuncRef(entry),
            ..
        }) = args.first()
        else {
            diagnostics.push(diagnostic(
                "K1",
                "pipeline kernel is not a named function",
                global.init.pos.clone(),
            ));
            continue;
        };
        // The checker typed the call against the library signature, so the kernel exists and its
        // parameter count matches the form. Either failure here is a generator defect.
        let Some(kernel) = function(module, entry) else {
            diagnostics.push(generator_diagnostic(
                format!("kernel `{entry}` disappeared from typed HIR"),
                global.init.pos.clone(),
            ));
            continue;
        };
        if kernel.params.len() != arity + 1 {
            diagnostics.push(generator_diagnostic(
                format!("kernel `{entry}` has an impossible parameter count"),
                kernel.pos.clone(),
            ));
            continue;
        }
        let mut layouts = Vec::new();
        for (group, param) in kernel.params.iter().take(arity).enumerate() {
            match layout(module, &param.ty, group as u32, &param.pos) {
                Ok(layout) => layouts.push(layout),
                Err(error) => diagnostics.push(error),
            }
        }
        // The parameter count check includes the invocation after all layouts.
        let invocation = kernel.params.get(arity).ok_or_else(|| {
            vec![crate::internal(
                "pipeline::discover",
                "missing invocation parameter",
                &kernel.pos,
            )]
        })?;
        let invocation_ok = class_name(module, &invocation.ty)
            .is_some_and(|name| name == "ComputeInvocation")
            && library_class(module, &invocation.ty).is_some();
        if !invocation_ok {
            diagnostics.push(generator_diagnostic(
                format!("kernel `{entry}` lost its ComputeInvocation parameter"),
                invocation.pos.clone(),
            ));
            continue;
        }
        let Some(options) = args.get(1) else {
            diagnostics.push(diagnostic(
                "PI1",
                "pipeline declaration omits options",
                global.init.pos.clone(),
            ));
            continue;
        };
        if let Err(error) = validate_pipeline_name(module, options, &global.name) {
            diagnostics.push(error);
            continue;
        }
        let guarded = match guarded_option(module, options) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        // The guard occupies one binding of the last layout, and the one-layout form is the only
        // form the rule defines (PI15).
        if guarded && arity != 1 {
            diagnostics.push(diagnostic(
                "PI15",
                "guarded is legal on the one-layout computePipeline form only",
                options.pos.clone(),
            ));
            continue;
        }
        // The guard wraps the body in an `if` over the global id, which is non-uniform control
        // flow. A barrier inside it is illegal, so the two do not combine (PI15, K22).
        if guarded {
            match crate::kernel::reaches_barrier(module, kernel, shells) {
                Ok(true) => {
                    diagnostics.push(diagnostic(
                        "PI15",
                        format!("guarded pipeline `{}` reaches a barrier", global.name),
                        kernel.pos.clone(),
                    ));
                    continue;
                }
                Ok(false) => {}
                Err(error) => {
                    diagnostics.push(error);
                    continue;
                }
            }
        }
        let host_runnable = match crate::kernel::host_runnable(module, kernel, shells) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        match workgroup(module, options) {
            Ok(workgroup) if layouts.len() == arity => {
                if guarded {
                    let Some(last) = layouts.last_mut() else {
                        diagnostics.push(generator_diagnostic(
                            "guarded pipeline has no last layout",
                            global.pos.clone(),
                        ));
                        continue;
                    };
                    // The guard takes the last layout's highest binding index plus one, so the
                    // author's own bindings keep their declaration-order indices (PI15).
                    let binding = last
                        .bindings
                        .iter()
                        .map(|binding| binding.index)
                        .max()
                        .map_or(0, |binding| binding + 1);
                    last.bindings.push(Binding {
                        name: format!("{}_guard", global.name),
                        index: binding,
                        kind: BindingKind::Guard,
                        item_ty: Type::U32,
                        pos: options.pos.clone(),
                    });
                }
                pipelines.push(Pipeline {
                    declaration: global.name.clone(),
                    entry: entry.clone(),
                    workgroup,
                    host_runnable,
                    guarded,
                    layouts,
                    pos: global.pos.clone(),
                });
            }
            // A layout failed and pushed its own diagnostic. The run ends in an error, so this
            // declaration produces nothing.
            Ok(_) => {}
            Err(error) => diagnostics.push(error),
        }
    }
    if diagnostics.is_empty() {
        Ok(pipelines)
    } else {
        Err(diagnostics)
    }
}

/// Returns the author schema names that the pipelines' binding item types reach.
///
/// The result excludes the library vector, matrix, and atomic classes, which carry no generated
/// layout constants.
pub(crate) fn schema_names(module: &Module, pipelines: &[Pipeline]) -> BTreeSet<String> {
    pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.layouts)
        .flat_map(|layout| &layout.bindings)
        .filter_map(|binding| class_name(module, &binding.item_ty))
        .filter(|name| {
            module.classes.iter().any(|class| {
                class.name == **name && class.is_value && class.pos.file != "typegpu-types.ts"
            })
        })
        .map(str::to_owned)
        .collect()
}
