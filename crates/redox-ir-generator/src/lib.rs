use redox_ast::{Expr, ExprKind, TopLevel, TopLevelKind, Type as AstType};
use rxir::{Function, Module, ModuleBuilder, Operand};

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

                    // TODO: This is just a placeholder, maybe we should add the type to the return
                    // as well?
                    builder.build_instruction(
                        entry,
                        rxir::Instruction::Return {
                            value: Some(Operand::Immediate(42)),
                        },
                    );
                }
                _ => todo!(),
            },
        }
    }

    fn rxir_type(ty: &AstType) -> rxir::Type {
        use redox_ast::NumberKind;
        match ty {
            AstType::Tuple(types) => {
                if types.is_empty() {
                    rxir::Type::Void
                } else {
                    unimplemented!("Tuple types are not supported in the IR yet")
                }
            }
            AstType::Number(ty) => match ty.kind {
                NumberKind::Signed => {
                    if ty.bits == 32 {
                        rxir::Type::Signed32
                    } else {
                        unimplemented!("Only signed 32-bit integers are supported in the IR yet")
                    }
                }
                _ => unimplemented!("Only signed integers are supported in the IR yet"),
            },
        }
    }
}
