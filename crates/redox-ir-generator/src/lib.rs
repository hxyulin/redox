use redox_ast::{Expr, ExprKind, TopLevel, TopLevelKind, Type as AstType};
use rxir::{Function, Module, ModuleBuilder};

// Now it has been type checked, any additional errors are panics
pub struct IrGenerator {}

pub struct ModuleOps {
    pub name: String,
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_module(&mut self, ops: ModuleOps, ast: Vec<TopLevel>) -> Module {
        let mut module_builder = ModuleBuilder::new();

        for node in ast {
            self.generate_top_level(&mut module_builder, node);
        }

        module_builder.build(ops.name)
    }

    fn generate_top_level(&mut self, builder: &mut ModuleBuilder, node: TopLevel) {
        match &node.kind {
            // We need a seperate top level expr generator
            TopLevelKind::Expr(expr) => match &expr.kind {
                ExprKind::FunctionDef(function) => {
                    let entry = builder.create_block();
                    builder.build_function(
                        function.name.clone(),
                        Self::rxir_type(function.return_ty.as_ref().unwrap()),
                        entry,
                    );
                }
                _ => todo!(),
            },
        }
    }

    fn rxir_type(ty: &AstType) -> rxir::Type {
        match ty {
            AstType::Tuple(types) => {
                if types.is_empty() {
                    rxir::Type::Void
                } else {
                    unimplemented!("Tuple types are not supported in the IR yet")
                }
            }
        }
    }
}
