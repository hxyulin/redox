use crate::{NumberType, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberLiteral {
    pub kind: NumberType,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Number(NumberLiteral),
}

impl Literal {
    pub fn ty(&self) -> Type {
        match self {
            Self::Number(number) => Type::Number(number.kind.clone()),
        }
    }
}

impl NumberLiteral {
    pub const fn new(kind: NumberType, value: u64) -> Self {
        Self { kind, value }
    }

    pub const fn int32(value: u64) -> Self {
        Self::new(NumberType::I32, value)
    }
}
