use redox_ast::{Block, Expr, ExprKind, TopLevel, TopLevelKind, Type};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {
    UnableToInferType,
    IncompatibleTypes { expected: Type, found: Type },
    UnknownVariable(String),
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnableToInferType => write!(f, "Unable to infer type"),
            Self::IncompatibleTypes { expected, found } => {
                write!(f, "Expected type {expected}, found type {found}")
            }
            Self::UnknownVariable(name) => write!(f, "Unknown variable {name}"),
        }
    }
}

pub struct TypeChecker {
    // We don't take ownership of the AST
}

struct FunctionContext {
    arguments: Vec<(String, Type)>,
    return_ty: Option<Type>,
}

struct BlockContext {
    variables: HashMap<String, Type>,
}

impl BlockContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    #[instrument(skip(self, ast))]
    pub fn type_check(&mut self, ast: &mut Vec<TopLevel>) -> Result<(), TypeCheckError> {
        for node in &mut *ast {
            tracing::trace!("Type checking node");
            match &mut node.kind {
                TopLevelKind::Expr(expr) => match &mut expr.kind {
                    ExprKind::FunctionDef(function) => {
                        let mut ctx = FunctionContext {
                            arguments: function.arguments.clone(),
                            return_ty: function.return_ty.clone(),
                        };
                        if !self.evaluate_block(&mut function.body, &mut ctx)? {
                            return Err(TypeCheckError::IncompatibleTypes {
                                expected: ctx.return_ty.clone().unwrap(),
                                found: Type::empty(),
                            });
                        }
                        if function.return_ty.is_none() {
                            return Err(TypeCheckError::UnableToInferType);
                        }
                    }
                    ExprKind::Literal(_) | ExprKind::Return(_) | ExprKind::Variable(_) => {
                        unreachable!()
                    }
                },
            }
        }

        Ok(())
    }

    #[instrument(skip(self, block, ctx))]
    fn evaluate_block(
        &mut self,
        block: &mut Block,
        ctx: &mut FunctionContext,
    ) -> Result<bool, TypeCheckError> {
        tracing::trace!("Evaluating block");
        let mut block_ctx = BlockContext::new();
        for arg in &ctx.arguments {
            block_ctx.variables.insert(arg.0.clone(), arg.1.clone());
        }
        for (_idx, statement) in &mut block.statements.iter_mut().enumerate() {
            tracing::trace!("Evaluating statement {_idx}");
            if self.evaluate_expr(statement, ctx, &mut block_ctx)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns a type, and whether of not it was not a return statement
    // TODO: This architecture is bad, since return statements should return '()', and not an
    // actual type
    #[instrument(skip(self, statement, ctx, block_ctx))]
    fn evaluate_expr(
        &mut self,
        statement: &mut Expr,
        ctx: &mut FunctionContext,
        block_ctx: &mut BlockContext,
    ) -> Result<bool, TypeCheckError> {
        tracing::trace!("Evaluating expression");
        // TODO: Control Flow evaluation
        match &mut statement.kind {
            ExprKind::Return(expr) => match expr {
                Some(ref mut expr) => {
                    // We need it to evluate the type first
                    self.evaluate_expr(expr, ctx, block_ctx)?;
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
            ExprKind::Variable(name) => {
                if let Some(ty) = block_ctx.variables.get(name) {
                    statement.ty.replace(ty.clone());
                    Ok(false)
                } else {
                    Err(TypeCheckError::UnknownVariable(name.clone()))
                }
            }
            ExprKind::FunctionDef(..) => unimplemented!(),
        }
    }
}
