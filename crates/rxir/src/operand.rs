use crate::{TempVarId, Type};

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate { ty: Type, value: u64 },
    TempVar { ty: Type, id: TempVarId },
}

impl Operand {
    pub fn ty(&self) -> Type {
        match self {
            Self::Immediate { ty, .. } => ty.clone(),
            Self::TempVar { ty, .. } => ty.clone(),
        }
    }
}

impl ToString for Operand {
    fn to_string(&self) -> String {
        match self {
            Operand::Immediate { ty, value } => format!("{value}{ty}"),
            Operand::TempVar { ty: _, id } => id.to_string(),
        }
    }
}
