use redox_ast::{ExprKind, TopLevel, TopLevelKind};

#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {
    UnableToInferType,
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnableToInferType => write!(f, "Unable to infer type"),
        }
    }
}

pub struct TypeChecker {
    // We don't take ownership of the AST
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn type_check(&mut self, ast: &mut Vec<TopLevel>) -> Result<(), TypeCheckError> {
        for node in ast {
            match &node.kind {
                TopLevelKind::Expr(expr) => match &expr.kind {
                    ExprKind::FunctionDef(function) => {
                        if function.return_ty.is_none() {
                            return Err(TypeCheckError::UnableToInferType);
                        }
                        // TODO: Ensure the main function returns '()'
                    }
                    ExprKind::Literal(_) => unreachable!(),
                },
            }
        }
        Ok(())
    }
}
