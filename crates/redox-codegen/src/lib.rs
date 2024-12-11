use std::path::PathBuf;

/// This is the trait that codegen backends must implement
/// It is intended to have one per module
pub trait CodegenBackend {
    fn gen_module(&mut self, module: &rxir::Module) -> Result<(), String>;

    fn write_intermediate(&mut self, path: PathBuf) -> Result<(), String>;
    fn write_object(&mut self, path: PathBuf) -> Result<(), String>;
}

pub mod llvm;
