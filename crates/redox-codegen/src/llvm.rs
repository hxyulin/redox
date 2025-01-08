use crate::CodegenBackend;
use std::path::PathBuf;

use inkwell::{
    builder::Builder,
    context::Context,
    llvm_sys::LLVMCallConv,
    module::Module,
    targets::{Target, TargetMachine},
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
        // FIXME: This is a hack to get the correct signature for the main function, we should
        // probbaly make our own entrypoint and avoid libc stuff
        let is_main = function.signature == "main";
        let args: Vec<BasicMetadataTypeEnum> = Vec::new();
        let fn_type = if is_main {
            self.context.i32_type().fn_type(&args, false)
        } else {
            // TODO: Conversion from rxir::Type to LLVMType
            self.context.void_type().fn_type(&args, false)
        };
        let llvm_fn = self
            .module
            .add_function(function.signature.as_str(), fn_type, None);
        // For now we just C calling convention beacuse we are using clang to link
        llvm_fn.set_linkage(inkwell::module::Linkage::External);
        llvm_fn.set_call_conventions(LLVMCallConv::LLVMCCallConv as u32);
        let entry = self.context.append_basic_block(llvm_fn, "entry");
        self.builder.position_at_end(entry);
        if is_main {
            // TODO: We will probably need to do some sort of 'find and replace' here, since the
            // expression codegen will automatically generate a return statement for us.
            self.builder
                .build_return(Some(&self.context.i32_type().const_int(0, false)))
                .unwrap();
        } else {
            self.builder.build_return(None).unwrap();
        }
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
