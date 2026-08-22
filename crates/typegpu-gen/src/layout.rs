//! Pure WGSL and C layout arithmetic.

/// One scalar schema type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
    /// A 32-bit float.
    F32,
    /// A 32-bit signed integer.
    I32,
    /// A 32-bit unsigned integer.
    U32,
    /// A 16-bit float.
    F16,
}

impl Scalar {
    fn size(self) -> u32 {
        match self {
            Self::F16 => 2,
            Self::F32 | Self::I32 | Self::U32 => 4,
        }
    }

    /// Returns the WGSL spelling.
    #[must_use]
    pub fn wgsl(self) -> &'static str {
        match self {
            Self::F32 => "f32",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::F16 => "f16",
        }
    }
}

/// A vector schema type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector {
    /// The component scalar.
    pub scalar: Scalar,
    /// The component count.
    pub lanes: u8,
    /// The class alignment override for the C layout.
    pub c_alignment: Option<u32>,
}

/// A column-major matrix schema type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    /// The column count.
    pub columns: u8,
    /// The row count.
    pub rows: u8,
    /// The class alignment override for the C layout.
    pub c_alignment: Option<u32>,
    /// The column class alignment override for the C layout.
    pub column_alignment: Option<u32>,
}

/// One named struct member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The source member name.
    pub name: String,
    /// The member type.
    pub ty: TypeTree,
}

/// A named struct schema type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    /// The source class name.
    pub name: String,
    /// Members in declaration order.
    pub members: Vec<Member>,
    /// The class alignment override for the C layout.
    pub c_alignment: Option<u32>,
}

/// A pure schema type tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTree {
    /// A scalar.
    Scalar(Scalar),
    /// A vector.
    Vector(Vector),
    /// A matrix.
    Matrix(Matrix),
    /// A 32-bit signed or unsigned atomic value.
    Atomic(Scalar),
    /// A fixed-size array.
    Array(Box<TypeTree>, u32),
    /// A struct.
    Struct(Struct),
}

/// One laid-out struct member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberLayout {
    /// The member name.
    pub name: String,
    /// The member byte offset.
    pub offset: u32,
    /// The member layout.
    pub layout: Layout,
}

/// The byte layout of one type tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The byte alignment.
    pub align: u32,
    /// The byte size.
    pub size: u32,
    /// The array stride when this layout is an array.
    pub stride: Option<u32>,
    /// Struct member layouts in declaration order.
    pub members: Vec<MemberLayout>,
}

/// Rounds `value` up to the next multiple of `modulo`.
#[must_use]
pub fn round_up(value: u32, modulo: u32) -> u32 {
    debug_assert!(modulo.is_power_of_two());
    (value + modulo - 1) & !(modulo - 1)
}

fn scalar_layout(scalar: Scalar) -> Layout {
    let size = scalar.size();
    Layout {
        align: size,
        size,
        stride: None,
        members: Vec::new(),
    }
}

fn wgsl_vector_layout(vector: &Vector) -> Layout {
    let component = vector.scalar.size();
    let align = match vector.lanes {
        2 => component * 2,
        3 | 4 => component * 4,
        _ => 1,
    };
    Layout {
        align,
        size: component * u32::from(vector.lanes),
        stride: None,
        members: Vec::new(),
    }
}

fn c_vector_layout(vector: &Vector) -> Layout {
    let component = vector.scalar.size();
    let natural_align = component;
    let align = vector
        .c_alignment
        .unwrap_or(natural_align)
        .max(natural_align);
    Layout {
        align,
        size: round_up(component * u32::from(vector.lanes), align),
        stride: None,
        members: Vec::new(),
    }
}

fn struct_layout(
    structure: &Struct,
    member_layout: impl Fn(&TypeTree) -> Layout,
    honor_override: bool,
) -> Layout {
    let mut cursor = 0;
    let mut align = 1;
    let mut members = Vec::with_capacity(structure.members.len());
    for member in &structure.members {
        let layout = member_layout(&member.ty);
        cursor = round_up(cursor, layout.align);
        members.push(MemberLayout {
            name: member.name.clone(),
            offset: cursor,
            layout: layout.clone(),
        });
        cursor += layout.size;
        align = align.max(layout.align);
    }
    if honor_override {
        align = align.max(structure.c_alignment.unwrap_or(1));
    }
    Layout {
        align,
        size: round_up(cursor, align),
        stride: None,
        members,
    }
}

/// Computes the WGSL default layout.
#[must_use]
pub fn wgsl_layout(tree: &TypeTree) -> Layout {
    match tree {
        TypeTree::Scalar(scalar) => scalar_layout(*scalar),
        TypeTree::Atomic(scalar) => scalar_layout(*scalar),
        TypeTree::Vector(vector) => wgsl_vector_layout(vector),
        TypeTree::Matrix(matrix) => {
            let column = wgsl_vector_layout(&Vector {
                scalar: Scalar::F32,
                lanes: matrix.rows,
                c_alignment: None,
            });
            let stride = round_up(column.size, column.align);
            Layout {
                align: column.align,
                size: stride * u32::from(matrix.columns),
                stride: Some(stride),
                members: Vec::new(),
            }
        }
        TypeTree::Array(element, length) => {
            let element = wgsl_layout(element);
            let stride = round_up(element.size, element.align);
            Layout {
                align: element.align,
                size: stride * length,
                stride: Some(stride),
                members: Vec::new(),
            }
        }
        TypeTree::Struct(structure) => struct_layout(structure, wgsl_layout, false),
    }
}

/// Computes the subscript C layout with class alignment overrides.
#[must_use]
pub fn c_layout(tree: &TypeTree) -> Layout {
    match tree {
        TypeTree::Scalar(scalar) => scalar_layout(*scalar),
        TypeTree::Atomic(scalar) => scalar_layout(*scalar),
        TypeTree::Vector(vector) => c_vector_layout(vector),
        TypeTree::Matrix(matrix) => {
            let column = c_vector_layout(&Vector {
                scalar: Scalar::F32,
                lanes: matrix.rows,
                c_alignment: matrix.column_alignment,
            });
            let natural_align = column.align;
            let align = matrix
                .c_alignment
                .unwrap_or(natural_align)
                .max(natural_align);
            let size = round_up(column.size * u32::from(matrix.columns), align);
            Layout {
                align,
                size,
                stride: Some(column.size),
                members: Vec::new(),
            }
        }
        TypeTree::Array(element, length) => {
            let element = c_layout(element);
            Layout {
                align: element.align,
                size: element.size * length,
                stride: Some(element.size),
                members: Vec::new(),
            }
        }
        TypeTree::Struct(structure) => struct_layout(structure, c_layout, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vector(scalar: Scalar, lanes: u8, c_alignment: u32) -> TypeTree {
        TypeTree::Vector(Vector {
            scalar,
            lanes,
            c_alignment: Some(c_alignment),
        })
    }

    #[test]
    fn ly3_vector_table() {
        let cases = [
            (Scalar::F32, 2, 8, 8),
            (Scalar::F32, 3, 16, 12),
            (Scalar::F32, 4, 16, 16),
            (Scalar::I32, 2, 8, 8),
            (Scalar::I32, 3, 16, 12),
            (Scalar::I32, 4, 16, 16),
            (Scalar::U32, 2, 8, 8),
            (Scalar::U32, 3, 16, 12),
            (Scalar::U32, 4, 16, 16),
            (Scalar::F16, 2, 4, 4),
            (Scalar::F16, 3, 8, 6),
            (Scalar::F16, 4, 8, 8),
        ];
        for (scalar, lanes, align, size) in cases {
            let layout = wgsl_layout(&vector(scalar, lanes, align));
            assert_eq!((layout.align, layout.size), (align, size));
        }
    }

    #[test]
    fn ly4_matrix_table() {
        for (columns, rows, align, size) in [(2, 2, 8, 16), (3, 3, 16, 48), (4, 4, 16, 64)] {
            let layout = wgsl_layout(&TypeTree::Matrix(Matrix {
                columns,
                rows,
                c_alignment: Some(align),
                column_alignment: Some(align),
            }));
            assert_eq!((layout.align, layout.size), (align, size));
        }
    }

    #[test]
    fn nested_golden_shapes() {
        let one = TypeTree::Struct(Struct {
            name: "One".into(),
            members: vec![
                Member {
                    name: "a".into(),
                    ty: TypeTree::Scalar(Scalar::U32),
                },
                Member {
                    name: "b".into(),
                    ty: vector(Scalar::F32, 3, 16),
                },
            ],
            c_alignment: None,
        });
        let one_layout = wgsl_layout(&one);
        assert_eq!((one_layout.align, one_layout.size), (16, 32));
        assert_eq!(one_layout.members[1].offset, 16);
        let array = TypeTree::Array(Box::new(one.clone()), 3);
        assert_eq!(wgsl_layout(&array).stride, Some(32));
        let two = TypeTree::Struct(Struct {
            name: "Two".into(),
            members: vec![
                Member {
                    name: "c".into(),
                    ty: array,
                },
                Member {
                    name: "d".into(),
                    ty: vector(Scalar::U32, 4, 16),
                },
            ],
            c_alignment: None,
        });
        let two_layout = wgsl_layout(&two);
        assert_eq!((two_layout.align, two_layout.size), (16, 112));
        assert_eq!(two_layout.members[1].offset, 96);
    }

    #[test]
    fn r33_c_layout_cases() {
        let vec3 = vector(Scalar::F32, 3, 16);
        assert_eq!((c_layout(&vec3).size, c_layout(&vec3).align), (16, 16));
        let mixed = TypeTree::Struct(Struct {
            name: "Mixed".into(),
            members: vec![
                Member {
                    name: "a".into(),
                    ty: TypeTree::Scalar(Scalar::F32),
                },
                Member {
                    name: "p".into(),
                    ty: vec3,
                },
            ],
            c_alignment: None,
        });
        assert_eq!((c_layout(&mixed).size, c_layout(&mixed).align), (32, 16));
        let mat3 = TypeTree::Matrix(Matrix {
            columns: 3,
            rows: 3,
            c_alignment: Some(16),
            column_alignment: Some(16),
        });
        assert_eq!((c_layout(&mat3).size, c_layout(&mat3).align), (48, 16));
        let vec2 = vector(Scalar::F32, 2, 8);
        assert_eq!((c_layout(&vec2).size, c_layout(&vec2).align), (8, 8));
    }
}
