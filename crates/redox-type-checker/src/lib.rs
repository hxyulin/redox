use redox_ast::{Block, Expr, ExprKind, TopLevel, TopLevelKind, Type};

#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {
    UnableToInferType,
    IncompatibleTypes { expected: Type, found: Type },
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnableToInferType => write!(f, "Unable to infer type"),
            Self::IncompatibleTypes { expected, found } => {
                write!(f, "Expected type {expected}, found type {found}")
            }
        }
    }
}

pub struct TypeChecker {
    // We don't take ownership of the AST
}

struct FunctionContext {
    return_ty: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn type_check(&mut self, ast: &mut Vec<TopLevel>) -> Result<(), TypeCheckError> {
        for node in &mut *ast {
            match &mut node.kind {
                TopLevelKind::Expr(expr) => match &mut expr.kind {
                    ExprKind::FunctionDef(function) => {
                        let mut ctx = FunctionContext {
                            return_ty: function.return_ty.clone(),
                        };
                        self.evaluate_block(&mut function.body, &mut ctx)?;
                        if function.return_ty.is_none() {
                            return Err(TypeCheckError::UnableToInferType);
                        }
                    }
                    ExprKind::Literal(_) | ExprKind::Return(_) => unreachable!(),
                },
            }
        }

        Ok(())
    }

    fn evaluate_block(
        &mut self,
        block: &mut Block,
        ctx: &mut FunctionContext,
    ) -> Result<(), TypeCheckError> {
        for (idx, statement) in &mut block.statements.iter_mut().enumerate() {
            if self.evaluate_expr(statement, ctx)? {
                break;
            }
        }
        Ok(())
    }

    /// Returns a type, and whether of not it was not a return statement
    // TODO: This architecture is bad, since return statements should return '()', and not an
    // actual type
    fn evaluate_expr(
        &mut self,
        statement: &mut Expr,
        ctx: &mut FunctionContext,
    ) -> Result<bool, TypeCheckError> {
        // TODO: Control Flow evaluation
        match &mut statement.kind {
            ExprKind::Return(expr) => match expr {
                Some(ref mut expr) => {
                    // We need it to evluate the type first
                    self.evaluate_expr(expr, ctx)?;
                    statement.ty.replace(Type::empty());
                    Ok(true)
                }
                None => {
                    statement.ty.replace(Type::empty());
                    Ok(true)
                }
            },
            ExprKind::Literal(lit) => {
                statement.ty.replace(lit.ty());
                Ok(false)
            }
            ExprKind::FunctionDef(..) => unimplemented!(),
        }
    }
}
