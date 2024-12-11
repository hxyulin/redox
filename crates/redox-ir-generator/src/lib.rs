use redox_ast::{Expr, ExprKind, TopLevel, TopLevelKind};
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
                ExprKind::FunctionDef { name } => {
                    let entry = builder.create_block();
                    builder.build_function(name.clone(), entry);
                }
                _ => todo!(),
            },
        }
    }
}
