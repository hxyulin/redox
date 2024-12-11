use crate::CodegenBackend;
use std::path::PathBuf;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    llvm_sys::{target_machine, LLVMCallConv},
    module::Module,
    targets::{Target, TargetMachine, TargetTriple},
    types::BasicMetadataTypeEnum,
};

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
        let llvm_return_type = self.context.void_type();
        let args: Vec<BasicMetadataTypeEnum> = Vec::new();
        let fn_type = llvm_return_type.fn_type(&args, false);
        let llvm_fn = self
            .module
            .add_function(function.signature.as_str(), fn_type, None);
        // For now we just C calling convention beacuse we are using clang to link
        llvm_fn.set_linkage(inkwell::module::Linkage::External);
        llvm_fn.set_call_conventions(LLVMCallConv::LLVMCCallConv as u32);
        let entry = self.context.append_basic_block(llvm_fn, "entry");
        self.builder.position_at_end(entry);
        self.builder.build_return(None).unwrap();
        Ok(())
    }
}

impl CodegenBackend for LLVMCodegenBackend<'_> {
    fn gen_module(&mut self, module: &rxir::Module) -> Result<(), String> {
        let llvm_module = self.context.create_module(module.name.as_str());
        for function in &module.functions {
            self.compile_function(module, function)?;
        }

        // Verification
        llvm_module.verify().map_err(|err| format!("{}", err))?;
        for function in llvm_module.get_functions() {
            if !function.verify(false) {
                return Err("Function verification failed".to_string());
            }
        }

        // Linking into the module
        self.module
            .link_in_module(llvm_module)
            .map_err(|err| format!("{}", err))?;

        Ok(())
    }

    fn write_intermediate(&mut self, path: PathBuf) -> Result<(), String> {
        let string = self.module.print_to_string().to_string();
        std::fs::write(path, string).map_err(|err| format!("{}", err))
    }

    fn write_object(&mut self, path: PathBuf) -> Result<(), String> {
        // TODO: Set target, optimization level, etc...
        let cpu = "generic";
        let features = "";
        let optiization = inkwell::OptimizationLevel::Default;
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
                optiization,
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
            .map_err(|err| format!("{}", err))
    }
}
