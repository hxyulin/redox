use redox_ast::{ExprKind, TopLevel, TopLevelKind};

#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {

}

pub struct TypeChecker {
   // We don't take ownership of the AST 
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn type_check(&mut self, ast: &mut Vec<TopLevel>) -> Result<(), TypeCheckError> {
        Ok(())
    }
}
