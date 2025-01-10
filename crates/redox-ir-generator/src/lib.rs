use redox_ast::{Block, Expr, ExprKind, Literal, TopLevel, TopLevelKind, Type as AstType};
use rxir::{BlockId, Module, ModuleBuilder, Operand, TempVarId};
use std::collections::HashMap;

// Now it has been type checked, any additional errors are panics
pub struct IrGenerator {}

pub struct ModuleOps {
    pub name: String,
}

pub struct BlockMeta {
    pub variables: HashMap<String, TempVarId>,
}

impl BlockMeta {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
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

        module_builder.build(ops.name.parse().expect("Invalid module name"))
    }

    fn generate_top_level(&mut self, builder: &mut ModuleBuilder, node: TopLevel) {
        match &node.kind {
            // We need a seperate top level expr generator
            TopLevelKind::Expr(expr) => match &expr.kind {
                ExprKind::FunctionDef(function) => {
                    let entry = builder.create_block(None);
                    let mut block_meta = BlockMeta::new();
                    let arguments = function
                        .arguments
                        .iter()
                        .map(|(name, ty)| {
                            let ty = Self::rxir_type(ty);
                            let id = builder.create_value(&entry, ty.clone(), None);
                            block_meta.variables.insert(name.clone(), id.clone());
                            (id, ty)
                        })
                        .collect();
                    builder.build_function(
                        function.name.parse().expect("Invalid function name"),
                        arguments,
                        Self::rxir_type(function.return_ty.as_ref().unwrap()),
                        entry.clone(),
                    );

                    self.generate_block(builder, &entry, &function.body, &mut block_meta);
                }
                _ => todo!(),
            },
        }
    }

    fn generate_block(
        &mut self,
        builder: &mut ModuleBuilder,
        block: &BlockId,
        body: &Block,
        meta: &mut BlockMeta,
    ) {
        for statement in &body.statements {
            self.generate_instruction(builder, &block, statement, meta);
        }
    }

    fn generate_instruction(
        &mut self,
        builder: &mut ModuleBuilder,
        block: &BlockId,
        expr: &Expr,
        meta: &mut BlockMeta,
    ) {
        match &expr.kind {
            ExprKind::Return(expr) => {
                let value = if let Some(expr) = expr {
                    let value = match &expr.kind {
                        ExprKind::Literal(literal) => match literal {
                            Literal::Number(number) => Operand::Immediate {
                                ty: Self::rxir_type(&number.kind.clone().into()),
                                value: number.value,
                            },
                        },
                        ExprKind::Variable(name) => {
                            let id = meta.variables.get(name).unwrap();
                            Operand::TempVar {
                                ty: builder.get_var_type(block, id),
                                id: id.clone(),
                            }
                        }
                        _ => todo!(),
                    };
                    Some(value)
                } else {
                    None
                };
                builder.build_instruction(block, rxir::Instruction::Return { value });
            }
            _ => todo!(),
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
