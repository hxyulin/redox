use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberKind {
    Signed,
    Unsigned,
    Float,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberType {
    pub kind: NumberKind,
    pub bits: u32,
}

impl NumberType {
    pub const I32: Self = Self {
        kind: NumberKind::Signed,
        bits: 32,
    };

    pub const I64: Self = Self {
        kind: NumberKind::Signed,
        bits: 64,
    };

    pub const U32: Self = Self {
        kind: NumberKind::Unsigned,
        bits: 32,
    };

    pub const U64: Self = Self {
        kind: NumberKind::Unsigned,
        bits: 64,
    };

    pub const F32: Self = Self {
        kind: NumberKind::Float,
        bits: 32,
    };

    pub const F64: Self = Self {
        kind: NumberKind::Float,
        bits: 64,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Tuple(Vec<Type>),
    Number(NumberType),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Do proper type display
        write!(f, "{:?}", self)
    }
}

impl Type {
    pub fn empty() -> Self {
        Type::Tuple(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Type::Tuple(types) => types.is_empty(),
            _ => false,
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i32" => Ok(Type::Number(NumberType::I32)),
            "i64" => Ok(Type::Number(NumberType::I64)),
            "u32" => Ok(Type::Number(NumberType::U32)),
            "u64" => Ok(Type::Number(NumberType::U64)),
            "f32" => Ok(Type::Number(NumberType::F32)),
            "f64" => Ok(Type::Number(NumberType::F64)),
            _ => Err(format!("Struct types are not yet supported")),
        }
    }
}
