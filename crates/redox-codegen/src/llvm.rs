use crate::CodegenBackend;
use std::{collections::HashMap, path::PathBuf};

use inkwell::{
    builder::Builder,
    context::Context,
    llvm_sys::LLVMCallConv,
    module::Module,
    targets::{Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
    AddressSpace,
};
use rxir::Operand;

pub struct LLVMContext {
    context: Context,
}

impl Default for LLVMContext {
    fn default() -> Self {
        Self {
            context: Context::create(),
        }
    }
}

pub struct LLVMCodegenBackend<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
}

struct BlockMeta<'ctx> {
    variables: HashMap<rxir::TempVarId, BasicValueEnum<'ctx>>,
}

impl<'ctx> BlockMeta<'ctx> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl<'ctx> LLVMCodegenBackend<'ctx> {
    pub fn new(context: &'ctx LLVMContext) -> Self {
        let builder = context.context.create_builder();
        // TODO: This will be the name of the executable / library
        let module = context.context.create_module("main");
        Self {
            context: &context.context,
            builder,
            module,
        }
    }
}

impl<'ctx> LLVMCodegenBackend<'ctx> {
    fn compile_function(
        &self,
        module: &rxir::Module,
        function: &rxir::Function,
    ) -> Result<(), String> {
        // FIXME: This is a hack to get the correct signature for the main function, we should
        // probbaly make our own entrypoint and avoid libc stuff
        let args: Vec<BasicMetadataTypeEnum> = function
            .arguments
            .iter()
            .map(|(_id, ty)| self.llvm_type(ty).unwrap().into())
            .collect();
        let fn_type = if let Some(ty) = self.llvm_type(&function.return_ty) {
            ty.fn_type(&args, false)
        } else {
            self.context.void_type().fn_type(&args, false)
        };
        let llvm_fn = self
            .module
            .add_function(function.signature.as_str(), fn_type, None);
        // For now we just C calling convention beacuse we are using clang to link
        llvm_fn.set_linkage(inkwell::module::Linkage::External);
        llvm_fn.set_call_conventions(LLVMCallConv::LLVMCCallConv as u32);
        let entry = self.context.append_basic_block(llvm_fn, "entry");
        let mut meta = BlockMeta::new();
        for (idx, (id, _ty)) in function.arguments.iter().enumerate() {
            let value = llvm_fn.get_nth_param(idx as u32).unwrap();
            meta.variables.insert(id.clone(), value);
        }
        self.builder.position_at_end(entry);
        let block = module.blocks.get(&function.entry).unwrap();
        self.compile_block(block, &meta)?;
        Ok(())
    }

    fn compile_block(&self, block: &rxir::Block, meta: &BlockMeta) -> Result<(), String> {
        for instruction in &block.instructions {
            self.compile_instruction(block, instruction, meta)?;
        }
        Ok(())
    }

    fn compile_instruction(
        &self,
        block: &rxir::Block,
        instruction: &rxir::Instruction,
        meta: &BlockMeta,
    ) -> Result<(), String> {
        match instruction {
            rxir::Instruction::Alloca { dest, ty } => unimplemented!(),
            rxir::Instruction::Return { value } => match value {
                None => self.builder.build_return(None).unwrap(),
                Some(Operand::Immediate { ty, value }) => self
                    .builder
                    .build_return(Some(&self.llvm_value(ty, *value)?))
                    .unwrap(),
                Some(Operand::TempVar { ty: _, id }) => {
                    let value = meta.variables.get(id).unwrap();
                    self.builder.build_return(Some(value)).unwrap()
                }
            },

            rxir::Instruction::Load { dest, src } => unimplemented!(),
            rxir::Instruction::Store { dest, src } => unimplemented!(),
        };
        Ok(())
    }

    fn llvm_value(&self, ty: &rxir::Type, value: u64) -> Result<BasicValueEnum<'ctx>, String> {
        match ty {
            rxir::Type::Void | rxir::Type::Pointer(_) => unreachable!(),
            rxir::Type::Signed32 => {
                // We need to bitcast the value to i32
                Ok(self.context.i32_type().const_int(value, false).into())
            }
        }
    }

    fn llvm_type(&self, ty: &rxir::Type) -> Option<BasicTypeEnum<'ctx>> {
        match ty {
            rxir::Type::Void => None,
            rxir::Type::Signed32 => Some(self.context.i32_type().into()),
            rxir::Type::Pointer(_) => Some(self.context.ptr_type(AddressSpace::default()).into()),
        }
    }
}

impl CodegenBackend for LLVMCodegenBackend<'_> {
    fn gen_module(&mut self, module: &rxir::Module) -> Result<(), String> {
        let llvm_module = self.context.create_module(module.name.as_str());
        for function in &module.functions {
            self.compile_function(module, function)?;
        }

        // Verification
        llvm_module.verify().map_err(|err| err.to_string())?;
        for function in llvm_module.get_functions() {
            if !function.verify(false) {
                return Err("Function verification failed".to_string());
            }
        }

        // Linking into the module
        self.module
            .link_in_module(llvm_module)
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    fn write_intermediate(&mut self, path: PathBuf) -> Result<(), String> {
        let string = self.module.print_to_string().to_string();
        std::fs::write(path, string).map_err(|err| err.to_string())
    }

    fn write_object(&mut self, path: PathBuf) -> Result<(), String> {
        // TODO: Set target, optimization level, etc...
        let cpu = "generic";
        let features = "";
        let optimization = inkwell::OptimizationLevel::Default;
        let triple = TargetMachine::get_default_triple();
        #[cfg(target_arch = "aarch64")]
        {
            Target::initialize_aarch64(&inkwell::targets::InitializationConfig::default());
        }
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &triple,
                &cpu,
                features,
                optimization,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .unwrap();

        target_machine
            .write_to_file(
                &self.module,
                inkwell::targets::FileType::Object,
                path.as_path(),
            )
            .map_err(|err| err.to_string())
    }
}
