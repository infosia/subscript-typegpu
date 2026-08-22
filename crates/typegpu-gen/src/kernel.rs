//! Typed-HIR to WGSL kernel emission.

use std::collections::{BTreeMap, BTreeSet};

use subscript_compiler::hir::{
    BinOp, Callee, Expr, ExprKind, ForOfKind, Function, Module, Stmt, UnOp,
};
use subscript_compiler::{Diagnostic, Pos, RuleCode, Type};

use crate::mapping::{self, MethodEmission};
use crate::pipeline::{BindingKind, Pipeline};

#[derive(Debug, Clone)]
pub(crate) struct Snippet {
    pub(crate) text: String,
}

fn diagnostic(rule: &str, message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("{rule}: {} (author)", message.into()),
        pos,
    )
}

fn generator_diagnostic(message: impl Into<String>, pos: Pos) -> Diagnostic {
    Diagnostic::new(
        RuleCode::S100,
        format!("K15: {} (generator)", message.into()),
        pos,
    )
}

fn function<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
}

fn class_name<'a>(module: &'a Module, ty: &Type) -> Option<&'a str> {
    let Type::Class(id) = ty else { return None };
    module.classes.get(id.0).map(|class| class.name.as_str())
}

fn wgsl_type(module: &Module, ty: &Type, pos: &Pos) -> Result<String, Diagnostic> {
    Ok(match ty {
        Type::F32 => "f32".to_owned(),
        Type::I32 => "i32".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::F16 => {
            return Err(diagnostic(
                "K4",
                "f16 is storage-only in kernels",
                pos.clone(),
            ))
        }
        Type::FixedArray(item, length) => {
            format!("array<{}, {length}>", wgsl_type(module, item, pos)?)
        }
        Type::Class(id) => {
            let class = &module.classes[id.0];
            if !class.is_value {
                return Err(diagnostic(
                    "K5",
                    format!(
                        "reference class `{}` is not a kernel value type",
                        class.name
                    ),
                    pos.clone(),
                ));
            }
            if class.pos.file != "typegpu-types.ts" {
                return Ok(mapping::ident(&class.name));
            }
            match class.name.as_str() {
                "Vec2f" => "vec2<f32>".to_owned(),
                "Vec3f" => "vec3<f32>".to_owned(),
                "Vec4f" => "vec4<f32>".to_owned(),
                "Vec2i" => "vec2<i32>".to_owned(),
                "Vec3i" => "vec3<i32>".to_owned(),
                "Vec4i" => "vec4<i32>".to_owned(),
                "Vec2u" => "vec2<u32>".to_owned(),
                "Vec3u" => "vec3<u32>".to_owned(),
                "Vec4u" => "vec4<u32>".to_owned(),
                "Vec2h" => "vec2<f16>".to_owned(),
                "Vec3h" => "vec3<f16>".to_owned(),
                "Vec4h" => "vec4<f16>".to_owned(),
                "Mat2x2f" => "mat2x2<f32>".to_owned(),
                "Mat3x3f" => "mat3x3<f32>".to_owned(),
                "Mat4x4f" => "mat4x4<f32>".to_owned(),
                name => mapping::ident(name),
            }
        }
        _ => {
            return Err(diagnostic(
                "K5",
                format!(
                    "type `{}` is not allowed in a kernel",
                    type_name(module, ty)
                ),
                pos.clone(),
            ))
        }
    })
}

fn type_name(module: &Module, ty: &Type) -> String {
    subscript_compiler::types::display_type(
        ty,
        &|id| module.classes[id.0].name.clone(),
        &|id| module.enums[id.0].name.clone(),
        &|id| module.string_aliases[id.0].name.clone(),
    )
}

fn binop(op: BinOp) -> Option<&'static str> {
    Some(match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        // Measured at ba6aa2e: a u32 / u32 expression has Type::U32 in typed HIR.
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        _ => return None,
    })
}

fn binary_precedence(op: BinOp) -> u8 {
    match op {
        BinOp::Or => 1,
        BinOp::And => 2,
        BinOp::BitOr => 3,
        BinOp::BitAnd => 4,
        BinOp::Eq | BinOp::Ne => 5,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 6,
        BinOp::Add | BinOp::Sub => 7,
        BinOp::Mul | BinOp::Div | BinOp::Rem => 8,
        _ => 0,
    }
}

fn binary_operand(expr: &Expr, text: String, parent: BinOp, right: bool) -> String {
    let ExprKind::Binary { op, .. } = &expr.kind else {
        return text;
    };
    let child = binary_precedence(*op);
    let parent = binary_precedence(parent);
    if child < parent || (right && child == parent) {
        format!("({text})")
    } else {
        text
    }
}

fn literal(expr: &Expr) -> Result<String, Diagnostic> {
    match (&expr.kind, &expr.ty) {
        (ExprKind::Int(value), Type::U32) => Ok(format!("{value}u")),
        (ExprKind::Int(value), Type::I32) => Ok(format!("{value}i")),
        (ExprKind::Float(value), Type::F32) => Ok(f32_literal(*value)),
        (ExprKind::Float(_), Type::F64) => Err(diagnostic(
            "K5",
            "f64 is outside the kernel value types",
            expr.pos.clone(),
        )),
        (ExprKind::Bool(value), Type::Bool) => Ok(value.to_string()),
        _ => Err(diagnostic(
            "K6",
            "literal has no WGSL spelling for its HIR type",
            expr.pos.clone(),
        )),
    }
}

fn f32_literal(value: f64) -> String {
    let mut text = value.to_string();
    if !text.contains('.') && !text.contains('e') && !text.contains('E') {
        text.push_str(".0");
    }
    format!("{text}f")
}

#[derive(Debug, Clone)]
struct BindingRef {
    name: String,
    kind: BindingKind,
}

struct Emitter<'a> {
    module: &'a Module,
    layout_params: BTreeMap<String, usize>,
    layout_names: BTreeSet<String>,
    invocation_param: String,
    bindings: BTreeMap<(usize, String), BindingRef>,
    used_builtins: BTreeSet<String>,
    pending: Vec<(usize, String)>,
    conditional_index: u32,
}

impl<'a> Emitter<'a> {
    fn new(module: &'a Module, pipeline: &'a Pipeline, kernel: &Function) -> Self {
        let mut layout_params = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        for (group, layout) in pipeline.layouts.iter().enumerate() {
            layout_params.insert(kernel.params[group].name.clone(), group);
            for binding in &layout.bindings {
                bindings.insert(
                    (group, binding.name.clone()),
                    BindingRef {
                        name: mapping::ident(&binding.name),
                        kind: binding.kind,
                    },
                );
            }
        }
        Self {
            module,
            layout_params,
            layout_names: pipeline
                .layouts
                .iter()
                .map(|layout| layout.name.clone())
                .collect(),
            invocation_param: kernel.params[pipeline.layouts.len()].name.clone(),
            bindings,
            used_builtins: BTreeSet::new(),
            pending: Vec::new(),
            conditional_index: 0,
        }
    }

    fn helper(module: &'a Module) -> Self {
        Self {
            module,
            layout_params: BTreeMap::new(),
            layout_names: BTreeSet::new(),
            invocation_param: String::new(),
            bindings: BTreeMap::new(),
            used_builtins: BTreeSet::new(),
            pending: Vec::new(),
            conditional_index: 0,
        }
    }

    fn binding_ref(&self, expr: &Expr) -> Option<BindingRef> {
        let ExprKind::Field { obj, name } = &expr.kind else {
            return None;
        };
        let ExprKind::Local(param) = &obj.kind else {
            return None;
        };
        let group = *self.layout_params.get(param)?;
        self.bindings.get(&(group, name.clone())).cloned()
    }

    fn snippet(&mut self, expr: &Expr) -> Result<Snippet, Diagnostic> {
        if let ExprKind::Cast(value) = &expr.kind {
            if !matches!(expr.ty, Type::F32 | Type::I32 | Type::U32)
                || !matches!(value.ty, Type::F32 | Type::I32 | Type::U32)
            {
                return Err(diagnostic(
                    "K12",
                    "cast is not among f32, i32, and u32",
                    expr.pos.clone(),
                ));
            }
        }
        let ty = if matches!(
            expr.kind,
            ExprKind::Cast(_) | ExprKind::New { .. } | ExprKind::Cond { .. }
        ) {
            wgsl_type(self.module, &expr.ty, &expr.pos)?
        } else {
            String::new()
        };
        let local = |text: String, _ty: String| Snippet { text };
        match &expr.kind {
            ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) => {
                Ok(local(literal(expr)?, ty))
            }
            ExprKind::Str(_) => Err(diagnostic(
                "K5",
                "string local or expression in kernel",
                expr.pos.clone(),
            )),
            ExprKind::Local(name) => Ok(Snippet {
                text: mapping::ident(name),
            }),
            ExprKind::Unary { op, operand } => {
                let value = self.snippet(operand)?;
                let spelling = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                    UnOp::BitNot => "~",
                    _ => {
                        return Err(diagnostic(
                            "K9",
                            "unary operator is outside K9",
                            expr.pos.clone(),
                        ))
                    }
                };
                let value = if matches!(
                    operand.kind,
                    ExprKind::Unary { .. } | ExprKind::Binary { .. } | ExprKind::Assign { .. }
                ) {
                    format!("({})", value.text)
                } else {
                    value.text
                };
                Ok(local(format!("{spelling}{value}"), ty))
            }
            ExprKind::Binary { op, left, right } => {
                let Some(spelling) = binop(*op) else {
                    return Err(diagnostic(
                        "K9",
                        "this binary operator is outside K9",
                        expr.pos.clone(),
                    ));
                };
                let left_text = self.snippet(left)?.text;
                let right_text = self.snippet(right)?.text;
                let left = binary_operand(left, left_text, *op, false);
                let right = binary_operand(right, right_text, *op, true);
                Ok(local(format!("{left} {spelling} {right}"), ty))
            }
            ExprKind::Assign { op, target, value } => {
                let target = self.snippet(target)?;
                let value = self.snippet(value)?;
                let spelling = match op {
                    None => "=".to_owned(),
                    Some(op) => format!(
                        "{}=",
                        binop(*op).ok_or_else(|| diagnostic(
                            "K9",
                            "assignment operator is outside K9",
                            expr.pos.clone()
                        ))?
                    ),
                };
                Ok(local(
                    format!("{} {spelling} {}", target.text, value.text),
                    ty,
                ))
            }
            ExprKind::Cast(value) => {
                let value = self.snippet(value)?;
                Ok(local(format!("{}({})", ty, value.text), ty))
            }
            ExprKind::Field { obj, name } => {
                if let Some(binding) = self.binding_ref(expr) {
                    return Ok(Snippet { text: binding.name });
                }
                if matches!(&obj.kind, ExprKind::Local(name) if name == &self.invocation_param) {
                    let builtin = match name.as_str() {
                        "globalId" => "globalId",
                        "localId" => "localId",
                        "workgroupId" => "workgroupId",
                        "numWorkgroups" => "numWorkgroups",
                        "localIndex" => "localIndex",
                        _ => {
                            return Err(diagnostic(
                                "PI4",
                                format!("unknown ComputeInvocation field `{name}`"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    self.used_builtins.insert(builtin.to_owned());
                    return Ok(Snippet {
                        text: builtin.to_owned(),
                    });
                }
                if obj.ty == Type::F16 || expr.ty == Type::F16 {
                    return Err(diagnostic(
                        "K4",
                        "f16 field access is not admitted in a kernel",
                        expr.pos.clone(),
                    ));
                }
                let object = self.snippet(obj)?;
                Ok(local(
                    format!("{}.{}", object.text, mapping::ident(name)),
                    ty,
                ))
            }
            ExprKind::Index { obj, index, .. } => {
                let object = self.snippet(obj)?;
                let index = self.snippet(index)?;
                Ok(Snippet {
                    text: format!("{}[{}]", object.text, index.text),
                })
            }
            ExprKind::Call { callee, args } => self.call(expr, callee, args),
            ExprKind::New { class, args } => {
                let class = &self.module.classes[class.0];
                if !class.is_value {
                    return Err(diagnostic(
                        "K5",
                        format!("new reference class `{}` in kernel", class.name),
                        expr.pos.clone(),
                    ));
                }
                let args = args
                    .iter()
                    .map(|arg| self.snippet(arg).map(|value| value.text))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(local(format!("{}({})", ty, args.join(", ")), ty))
            }
            ExprKind::Cond { cond, then, els } => {
                let cond = self.snippet(cond)?;
                let then = self.snippet(then)?;
                let els = self.snippet(els)?;
                let result = format!("_conditional_{}", self.conditional_index);
                self.conditional_index += 1;
                self.pending.extend([
                    (0, format!("var {result}: {ty};")),
                    (0, format!("if ({}) {{", cond.text)),
                    (1, format!("{result} = {};", then.text)),
                    (0, "} else {".to_owned()),
                    (1, format!("{result} = {};", els.text)),
                    (0, "}".to_owned()),
                ]);
                Ok(Snippet { text: result })
            }
            ExprKind::Template(_) => Err(diagnostic(
                "K9",
                "template string in kernel",
                expr.pos.clone(),
            )),
            ExprKind::Lambda { .. } => {
                Err(diagnostic("K9", "a lambda is outside K9", expr.pos.clone()))
            }
            ExprKind::AsyncSuspend | ExprKind::AsyncCall { .. } => {
                Err(diagnostic("K9", "await is outside K9", expr.pos.clone()))
            }
            ExprKind::Length(_) => Err(diagnostic(
                "K9",
                "Length of a T[] is outside the kernel subset",
                expr.pos.clone(),
            )),
            ExprKind::ArrayLit(_) => Err(diagnostic(
                "K9",
                "an array literal is outside K9",
                expr.pos.clone(),
            )),
            _ => Err(diagnostic(
                "K9",
                "this expression construct is outside K9",
                expr.pos.clone(),
            )),
        }
    }

    fn call(&mut self, expr: &Expr, callee: &Callee, args: &[Expr]) -> Result<Snippet, Diagnostic> {
        match callee {
            Callee::Math(function) => {
                let Some(name) = mapping::math(*function) else {
                    return Err(diagnostic(
                        "K11",
                        format!("Math.{} is outside K11", function.name()),
                        expr.pos.clone(),
                    ));
                };
                let args = args
                    .iter()
                    .map(|arg| match (&arg.kind, &arg.ty) {
                        // JavaScript Math arguments are typed as f64 by the checker; K11 maps
                        // those literal arguments to WGSL f32 without admitting f64 elsewhere.
                        (ExprKind::Float(value), Type::F64) => Ok(f32_literal(*value)),
                        _ => self.snippet(arg).map(|value| value.text),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let text = if name.is_empty() {
                    args.first().cloned().ok_or_else(|| {
                        diagnostic("K11", "Math.fround requires one argument", expr.pos.clone())
                    })?
                } else {
                    format!("{name}({})", args.join(", "))
                };
                Ok(Snippet { text })
            }
            Callee::Func(name) => {
                let is_library = function(self.module, name)
                    .is_some_and(|function| function.pos.file == "typegpu-types.ts");
                let mapped = is_library.then(|| mapping::free_function(name)).flatten();
                let called = mapped.unwrap_or_else(|| name.split('<').next().unwrap_or(name));
                let called = if mapped.is_some() {
                    called.to_owned()
                } else {
                    mapping::ident(called)
                };
                let args = args
                    .iter()
                    .map(|arg| self.snippet(arg).map(|value| value.text))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Snippet {
                    text: format!("{called}({})", args.join(", ")),
                })
            }
            Callee::Method { recv, name } => {
                if let Some(binding) = self.binding_ref(recv) {
                    let args = args
                        .iter()
                        .map(|arg| self.snippet(arg).map(|value| value.text))
                        .collect::<Result<Vec<_>, _>>()?;
                    let text = match (binding.kind, name.as_str(), args.as_slice()) {
                        (BindingKind::Uniform, "get", []) => binding.name,
                        (BindingKind::Storage | BindingKind::MutStorage, "get", [index]) => {
                            format!("{}[{index}]", binding.name)
                        }
                        (BindingKind::MutStorage, "set", [index, value]) => {
                            format!("{}[{index}] = {value}", binding.name)
                        }
                        (BindingKind::Storage | BindingKind::MutStorage, "length", []) => {
                            format!("arrayLength(&{})", binding.name)
                        }
                        _ => {
                            return Err(diagnostic(
                                "PI6",
                                format!("binding method `{name}` is not valid for this wrapper"),
                                expr.pos.clone(),
                            ))
                        }
                    };
                    return Ok(Snippet { text });
                }
                let recv_value = self.snippet(recv)?;
                let receiver = class_name(self.module, &recv.ty).ok_or_else(|| {
                    diagnostic(
                        "K10",
                        "a method receiver is not a library class",
                        expr.pos.clone(),
                    )
                })?;
                let library = matches!(&recv.ty, Type::Class(id) if self.module.classes[id.0].pos.file == "typegpu-types.ts");
                if !library {
                    return Err(diagnostic(
                        "K10",
                        format!("method `{receiver}.{name}` is outside K10"),
                        expr.pos.clone(),
                    ));
                }
                let Some(emission) = mapping::method(receiver, name) else {
                    return Err(diagnostic(
                        "K10",
                        format!("method `{receiver}.{name}` is outside K10"),
                        expr.pos.clone(),
                    ));
                };
                let args = args
                    .iter()
                    .map(|arg| self.snippet(arg).map(|value| value.text))
                    .collect::<Result<Vec<_>, _>>()?;
                let text = match emission {
                    MethodEmission::Binary(op) if args.len() == 1 => {
                        format!("({} {op} {})", recv_value.text, args[0])
                    }
                    MethodEmission::Builtin(builtin) if args.is_empty() => {
                        format!("{builtin}({})", recv_value.text)
                    }
                    MethodEmission::Builtin(builtin) => {
                        format!("{builtin}({}, {})", recv_value.text, args.join(", "))
                    }
                    _ => {
                        return Err(diagnostic(
                            "K10",
                            format!("wrong arity for `{receiver}.{name}`"),
                            expr.pos.clone(),
                        ))
                    }
                };
                Ok(Snippet { text })
            }
            Callee::Value(_) => Err(diagnostic(
                "K9",
                "function value or lambda call in kernel",
                expr.pos.clone(),
            )),
            _ => Err(diagnostic(
                "K9",
                "call target is outside the kernel subset",
                expr.pos.clone(),
            )),
        }
    }

    fn statements(
        &mut self,
        statements: &[Stmt],
        indent: usize,
        out: &mut String,
    ) -> Result<(), Diagnostic> {
        for statement in statements {
            self.statement(statement, indent, out)?;
        }
        Ok(())
    }

    fn line(out: &mut String, indent: usize, text: &str) {
        out.push_str(&"  ".repeat(indent));
        out.push_str(text);
        out.push('\n');
    }

    fn flush_pending(&mut self, out: &mut String, indent: usize) {
        for (relative, text) in self.pending.drain(..) {
            Self::line(out, indent + relative, &text);
        }
    }

    fn statement(
        &mut self,
        statement: &Stmt,
        indent: usize,
        out: &mut String,
    ) -> Result<(), Diagnostic> {
        match statement {
            Stmt::Let {
                name,
                ty,
                mutable,
                init,
                pos,
            } => {
                if class_name(self.module, ty)
                    .is_some_and(|class| self.layout_names.contains(class))
                {
                    return Err(diagnostic(
                        "PI13",
                        "a layout class is used as a kernel local",
                        pos.clone(),
                    ));
                }
                if !matches!(
                    init.kind,
                    ExprKind::Cast(_)
                        | ExprKind::Call { .. }
                        | ExprKind::Lambda { .. }
                        | ExprKind::Template(_)
                        | ExprKind::AsyncSuspend
                        | ExprKind::AsyncCall { .. }
                        | ExprKind::ArrayLit(_)
                        | ExprKind::Length(_)
                ) {
                    let _ = wgsl_type(self.module, ty, pos)?;
                }
                let value = self.snippet(init)?;
                let _ = wgsl_type(self.module, ty, pos)?;
                self.flush_pending(out, indent);
                let value_class =
                    matches!(ty, Type::Class(id) if self.module.classes[id.0].is_value);
                let declaration = if *mutable || value_class {
                    "var"
                } else {
                    "let"
                };
                Self::line(
                    out,
                    indent,
                    &format!("{declaration} {} = {};", mapping::ident(name), value.text),
                );
            }
            Stmt::Expr(expr) => {
                let value = self.snippet(expr)?;
                self.flush_pending(out, indent);
                Self::line(out, indent, &format!("{};", value.text));
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    let value = self.snippet(value)?;
                    self.flush_pending(out, indent);
                    Self::line(out, indent, &format!("return {};", value.text));
                } else {
                    Self::line(out, indent, "return;");
                }
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                let cond = self.snippet(cond)?;
                self.flush_pending(out, indent);
                Self::line(out, indent, &format!("if ({}) {{", cond.text));
                self.statements(then, indent + 1, out)?;
                if let Some(els) = els {
                    Self::line(out, indent, "} else {");
                    self.statements(els, indent + 1, out)?;
                }
                Self::line(out, indent, "}");
            }
            Stmt::While { cond, body, .. } => {
                let cond = self.snippet(cond)?;
                self.flush_pending(out, indent);
                Self::line(out, indent, &format!("while ({}) {{", cond.text));
                self.statements(body, indent + 1, out)?;
                Self::line(out, indent, "}");
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
                pos,
            } => {
                let init = match init.as_deref() {
                    Some(Stmt::Let {
                        name,
                        ty,
                        mutable,
                        init,
                        pos,
                    }) => {
                        if class_name(self.module, ty)
                            .is_some_and(|class| self.layout_names.contains(class))
                        {
                            return Err(diagnostic(
                                "PI13",
                                "a layout class is used as a for-loop local",
                                pos.clone(),
                            ));
                        }
                        let value = self.snippet(init)?;
                        let value_class =
                            matches!(ty, Type::Class(id) if self.module.classes[id.0].is_value);
                        let declaration = if *mutable || value_class {
                            "var"
                        } else {
                            "let"
                        };
                        let _ = wgsl_type(self.module, ty, pos)?;
                        format!("{} {} = {}", declaration, mapping::ident(name), value.text)
                    }
                    Some(Stmt::Expr(expr)) => self.snippet(expr)?.text,
                    None => String::new(),
                    _ => {
                        return Err(diagnostic(
                            "K7",
                            "unsupported for-loop initializer",
                            pos.clone(),
                        ))
                    }
                };
                let cond = cond
                    .as_ref()
                    .map(|value| self.snippet(value).map(|value| value.text))
                    .transpose()?
                    .unwrap_or_default();
                let step = step
                    .as_ref()
                    .map(|value| self.snippet(value).map(|value| value.text))
                    .transpose()?
                    .unwrap_or_default();
                self.flush_pending(out, indent);
                Self::line(out, indent, &format!("for ({init}; {cond}; {step}) {{"));
                self.statements(body, indent + 1, out)?;
                Self::line(out, indent, "}");
            }
            Stmt::ForOf {
                name,
                ty,
                subject,
                kind: ForOfKind::FixedArrayValues,
                body,
                pos,
            } => {
                let Type::FixedArray(_, length) = subject.ty else {
                    return Err(diagnostic(
                        "K7",
                        "for...of subject is not a FixedArray",
                        pos.clone(),
                    ));
                };
                let subject = self.snippet(subject)?;
                let index = format!("{}_index", mapping::ident(name));
                Self::line(
                    out,
                    indent,
                    &format!(
                        "for (var {index} = 0u; {index} < {length}u; {index} = {index} + 1u) {{"
                    ),
                );
                Self::line(
                    out,
                    indent + 1,
                    &format!("let {} = {}[{index}];", mapping::ident(name), subject.text),
                );
                let _ = wgsl_type(self.module, ty, pos)?;
                self.statements(body, indent + 1, out)?;
                Self::line(out, indent, "}");
            }
            Stmt::Block(body) => {
                Self::line(out, indent, "{");
                self.statements(body, indent + 1, out)?;
                Self::line(out, indent, "}");
            }
            Stmt::ForOf { pos, .. } | Stmt::Switch { pos, .. } => {
                return Err(diagnostic(
                    "K7",
                    "statement is outside the current kernel subset",
                    pos.clone(),
                ));
            }
            Stmt::Break(pos) | Stmt::Continue(pos) => {
                return Err(diagnostic(
                    "K7",
                    "break and continue are outside K7",
                    pos.clone(),
                ));
            }
            _ => {
                return Err(diagnostic(
                    "K7",
                    "statement is outside the current kernel subset",
                    Pos::new("", 1, 1),
                ))
            }
        }
        Ok(())
    }
}

fn called_functions_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            if let Callee::Func(name) = callee {
                out.insert(name.clone());
            }
            if let Callee::Value(value) = callee {
                called_functions_expr(value, out);
            }
            if let Callee::Method { recv, .. } = callee {
                called_functions_expr(recv, out);
            }
            for arg in args {
                called_functions_expr(arg, out);
            }
        }
        ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => called_functions_expr(operand, out),
        ExprKind::Binary { left, right, .. } => {
            called_functions_expr(left, out);
            called_functions_expr(right, out);
        }
        ExprKind::Assign { target, value, .. } => {
            called_functions_expr(target, out);
            called_functions_expr(value, out);
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                called_functions_expr(arg, out);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                called_functions_expr(field, out);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            called_functions_expr(obj, out);
            called_functions_expr(index, out);
        }
        ExprKind::Cond { cond, then, els } => {
            called_functions_expr(cond, out);
            called_functions_expr(then, out);
            called_functions_expr(els, out);
        }
        _ => {}
    }
}

fn called_functions_stmt(stmt: &Stmt, out: &mut BTreeSet<String>) {
    match stmt {
        Stmt::Let { init, .. } | Stmt::Expr(init) => called_functions_expr(init, out),
        Stmt::Return {
            value: Some(value), ..
        } => called_functions_expr(value, out),
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            called_functions_expr(cond, out);
            for stmt in then {
                called_functions_stmt(stmt, out);
            }
            if let Some(els) = els {
                for stmt in els {
                    called_functions_stmt(stmt, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            called_functions_expr(cond, out);
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            if let Some(init) = init {
                called_functions_stmt(init, out);
            }
            if let Some(cond) = cond {
                called_functions_expr(cond, out);
            }
            if let Some(step) = step {
                called_functions_expr(step, out);
            }
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::ForOf { subject, body, .. } => {
            called_functions_expr(subject, out);
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            called_functions_expr(disc, out);
            for stmt in cases.iter().flat_map(|case| &case.body) {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::Block(body) => {
            for stmt in body {
                called_functions_stmt(stmt, out);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
        _ => {}
    }
}

fn collect_schema_type(
    module: &Module,
    ty: &Type,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    match ty {
        Type::Class(id) => {
            let class = &module.classes[id.0];
            if class.is_value
                && class.pos.file != "typegpu-types.ts"
                && class.pos.file != "typegpu.ts"
                && seen.insert(class.name.clone())
            {
                out.push(class.name.clone());
            }
        }
        Type::FixedArray(item, _) => collect_schema_type(module, item, seen, out),
        _ => {}
    }
}

fn collect_schema_expr(
    module: &Module,
    expr: &Expr,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    collect_schema_type(module, &expr.ty, seen, out);
    match &expr.kind {
        ExprKind::Unary { operand, .. }
        | ExprKind::Cast(operand)
        | ExprKind::Length(operand)
        | ExprKind::Field { obj: operand, .. }
        | ExprKind::JsonResultValue(operand) => {
            collect_schema_expr(module, operand, seen, out);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_schema_expr(module, left, seen, out);
            collect_schema_expr(module, right, seen, out);
        }
        ExprKind::Assign { target, value, .. } => {
            collect_schema_expr(module, target, seen, out);
            collect_schema_expr(module, value, seen, out);
        }
        ExprKind::Call { callee, args } => {
            if let Callee::Value(value) = callee {
                collect_schema_expr(module, value, seen, out);
            }
            if let Callee::Method { recv, .. } = callee {
                collect_schema_expr(module, recv, seen, out);
            }
            for arg in args {
                collect_schema_expr(module, arg, seen, out);
            }
        }
        ExprKind::New { args, .. } | ExprKind::ArrayLit(args) => {
            for arg in args {
                collect_schema_expr(module, arg, seen, out);
            }
        }
        ExprKind::DescriptorLit { fields, .. } => {
            for field in fields.iter().flatten() {
                collect_schema_expr(module, field, seen, out);
            }
        }
        ExprKind::Index { obj, index, .. } => {
            collect_schema_expr(module, obj, seen, out);
            collect_schema_expr(module, index, seen, out);
        }
        ExprKind::Cond { cond, then, els } => {
            collect_schema_expr(module, cond, seen, out);
            collect_schema_expr(module, then, seen, out);
            collect_schema_expr(module, els, seen, out);
        }
        ExprKind::Template(parts) => {
            for part in parts {
                if let subscript_compiler::hir::TplPart::Expr(value) = part {
                    collect_schema_expr(module, value, seen, out);
                }
            }
        }
        ExprKind::Lambda { body, .. } => {
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        _ => {}
    }
}

fn collect_schema_stmt(
    module: &Module,
    stmt: &Stmt,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    match stmt {
        Stmt::Let { ty, init, .. } => {
            collect_schema_type(module, ty, seen, out);
            collect_schema_expr(module, init, seen, out);
        }
        Stmt::Expr(expr) => collect_schema_expr(module, expr, seen, out),
        Stmt::Return {
            value: Some(value), ..
        } => collect_schema_expr(module, value, seen, out),
        Stmt::Return { value: None, .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            collect_schema_expr(module, cond, seen, out);
            for stmt in then {
                collect_schema_stmt(module, stmt, seen, out);
            }
            if let Some(els) = els {
                for stmt in els {
                    collect_schema_stmt(module, stmt, seen, out);
                }
            }
        }
        Stmt::While { cond, body, .. } => {
            collect_schema_expr(module, cond, seen, out);
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::For {
            init,
            cond,
            step,
            body,
            ..
        } => {
            if let Some(init) = init {
                collect_schema_stmt(module, init, seen, out);
            }
            if let Some(cond) = cond {
                collect_schema_expr(module, cond, seen, out);
            }
            if let Some(step) = step {
                collect_schema_expr(module, step, seen, out);
            }
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::ForOf {
            ty, subject, body, ..
        } => {
            collect_schema_type(module, ty, seen, out);
            collect_schema_expr(module, subject, seen, out);
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::Switch { disc, cases, .. } => {
            collect_schema_expr(module, disc, seen, out);
            for stmt in cases.iter().flat_map(|case| &case.body) {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::Block(body) => {
            for stmt in body {
                collect_schema_stmt(module, stmt, seen, out);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
        _ => {}
    }
}

pub(crate) fn referenced_schema_names(
    module: &Module,
    pipeline: &Pipeline,
) -> Result<Vec<String>, Diagnostic> {
    let kernel = function(module, &pipeline.entry).ok_or_else(|| {
        generator_diagnostic(
            "the kernel disappeared from typed HIR",
            pipeline.pos.clone(),
        )
    })?;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            collect_schema_type(module, &binding.item_ty, &mut seen, &mut out);
        }
    }
    for name in dependencies(module, kernel)? {
        let helper = function(module, &name).ok_or_else(|| {
            generator_diagnostic("a helper disappeared from typed HIR", pipeline.pos.clone())
        })?;
        for param in &helper.params {
            collect_schema_type(module, &param.ty, &mut seen, &mut out);
        }
        collect_schema_type(module, &helper.ret, &mut seen, &mut out);
        for stmt in &helper.body {
            collect_schema_stmt(module, stmt, &mut seen, &mut out);
        }
    }
    for stmt in &kernel.body {
        collect_schema_stmt(module, stmt, &mut seen, &mut out);
    }
    Ok(out)
}

fn dependencies(module: &Module, kernel: &Function) -> Result<Vec<String>, Diagnostic> {
    fn visit(
        module: &Module,
        name: &str,
        stack: &mut Vec<String>,
        done: &mut BTreeSet<String>,
        order: &mut Vec<String>,
        pos: &Pos,
    ) -> Result<(), Diagnostic> {
        if done.contains(name) {
            return Ok(());
        }
        if let Some(start) = stack.iter().position(|item| item == name) {
            let mut cycle = stack[start..].to_vec();
            cycle.push(name.to_owned());
            return Err(diagnostic(
                "K2",
                format!("recursive helper cycle: {}", cycle.join(" -> ")),
                pos.clone(),
            ));
        }
        let Some(function) = function(module, name) else {
            return Ok(());
        };
        if function.pos.file == "typegpu-types.ts" {
            return Ok(());
        }
        stack.push(name.to_owned());
        let mut calls = BTreeSet::new();
        for stmt in &function.body {
            called_functions_stmt(stmt, &mut calls);
        }
        for called in calls {
            visit(module, &called, stack, done, order, &function.pos)?;
        }
        stack.pop();
        done.insert(name.to_owned());
        order.push(name.to_owned());
        Ok(())
    }
    let mut calls = BTreeSet::new();
    for stmt in &kernel.body {
        called_functions_stmt(stmt, &mut calls);
    }
    let mut order = Vec::new();
    let mut done = BTreeSet::new();
    for called in calls {
        visit(
            module,
            &called,
            &mut Vec::new(),
            &mut done,
            &mut order,
            &kernel.pos,
        )?;
    }
    Ok(order)
}

fn builtin_parameter(name: &str) -> &'static str {
    match name {
        "globalId" => "@builtin(global_invocation_id) globalId: vec3<u32>",
        "localId" => "@builtin(local_invocation_id) localId: vec3<u32>",
        "workgroupId" => "@builtin(workgroup_id) workgroupId: vec3<u32>",
        "numWorkgroups" => "@builtin(num_workgroups) numWorkgroups: vec3<u32>",
        "localIndex" => "@builtin(local_invocation_index) localIndex: u32",
        _ => "",
    }
}

pub(crate) fn emit(
    module: &Module,
    pipeline: &Pipeline,
    structs: &[(String, String)],
    uses_f16: bool,
) -> Result<String, Diagnostic> {
    let kernel = function(module, &pipeline.entry)
        .ok_or_else(|| diagnostic("K1", "kernel disappeared from HIR", pipeline.pos.clone()))?;
    let mut binding_pairs = BTreeSet::new();
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            if !binding_pairs.insert((layout.group, binding.index)) {
                return Err(generator_diagnostic(
                    "two bindings use one group and binding pair",
                    binding.pos.clone(),
                ));
            }
        }
    }
    let helpers = dependencies(module, kernel)?;
    let mut emitter = Emitter::new(module, pipeline, kernel);
    let mut helper_text = String::new();
    for name in helpers {
        let helper = function(module, &name).ok_or_else(|| {
            generator_diagnostic(
                format!("helper `{name}` disappeared from typed HIR"),
                pipeline.pos.clone(),
            )
        })?;
        if helper.is_async || helper.is_generator {
            return Err(diagnostic(
                "K2",
                format!("helper `{name}` is async or a generator"),
                helper.pos.clone(),
            ));
        }
        for param in &helper.params {
            let takes_layout = class_name(module, &param.ty)
                .is_some_and(|name| pipeline.layouts.iter().any(|layout| layout.name == name));
            if takes_layout || class_name(module, &param.ty) == Some("ComputeInvocation") {
                return Err(diagnostic(
                    "K2",
                    format!("helper `{name}` takes a layout class or ComputeInvocation"),
                    param.pos.clone(),
                ));
            }
            let _ = wgsl_type(module, &param.ty, &param.pos)?;
        }
        let params = helper
            .params
            .iter()
            .map(|param| {
                Ok(format!(
                    "{}: {}",
                    mapping::ident(&param.name),
                    wgsl_type(module, &param.ty, &param.pos)?
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let result = if helper.ret == Type::Void {
            String::new()
        } else {
            format!(" -> {}", wgsl_type(module, &helper.ret, &helper.pos)?)
        };
        helper_text.push_str(&format!(
            "fn {}({}){result} {{\n",
            mapping::ident(name.split('<').next().unwrap_or(&name)),
            params.join(", ")
        ));
        let mut helper_emitter = Emitter::helper(module);
        helper_emitter.statements(&helper.body, 1, &mut helper_text)?;
        helper_text.push_str("}\n\n");
    }
    let mut entry_body = String::new();
    emitter.statements(&kernel.body, 1, &mut entry_body)?;

    let mut out = String::new();
    if uses_f16
        || pipeline
            .layouts
            .iter()
            .flat_map(|layout| &layout.bindings)
            .any(|binding| {
                class_name(module, &binding.item_ty)
                    .is_some_and(|name| matches!(name, "Vec2h" | "Vec3h" | "Vec4h"))
            })
    {
        out.push_str("enable f16;\n\n");
    }
    for (_, structure) in structs {
        out.push_str(structure);
        out.push('\n');
    }
    for layout in &pipeline.layouts {
        for binding in &layout.bindings {
            let item = wgsl_type(module, &binding.item_ty, &binding.pos)?;
            let declaration_ty = if binding.kind == BindingKind::Uniform {
                item
            } else {
                format!("array<{item}>")
            };
            out.push_str(&format!(
                "@group({}) @binding({}) var<{}> {}: {};\n",
                layout.group,
                binding.index,
                binding.kind.wgsl(),
                mapping::ident(&binding.name),
                declaration_ty
            ));
        }
    }
    out.push('\n');
    out.push_str(&helper_text);
    let parameters = [
        "globalId",
        "localId",
        "workgroupId",
        "numWorkgroups",
        "localIndex",
    ]
    .into_iter()
    .filter(|name| emitter.used_builtins.contains(*name))
    .map(builtin_parameter)
    .collect::<Vec<_>>();
    out.push_str(&format!(
        "@compute @workgroup_size({}, {}, {})\n",
        pipeline.workgroup[0], pipeline.workgroup[1], pipeline.workgroup[2]
    ));
    out.push_str(&format!(
        "fn {}({}) {{\n",
        mapping::ident(&pipeline.entry),
        parameters.join(", ")
    ));
    out.push_str(&entry_body);
    out.push_str("}\n");
    Ok(out)
}
