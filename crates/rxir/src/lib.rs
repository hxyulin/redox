use std::collections::HashMap;

mod builder;
pub use builder::*;
// Redefines basic types for RXIR passes
mod pass;
pub use pass::*;

#[derive(Debug, Clone)]
pub enum IRBuildType {
    DynamicLibrary,
    StaticLibrary,
    Executable { entry: BlockId },
}

#[derive(Debug, Clone)]
pub struct TempVar {}

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate(i64),
    TempVar(TempVarId),
}

#[derive(Debug, Clone)]
pub struct RedoxIR {
    pub build_type: IRBuildType,
    pub modules: Vec<Module>,
}

/// THis is just the index, which is managed in the state of the IR
/// builder.
pub type BlockId = usize;
pub type TempVarId = usize;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub blocks: HashMap<BlockId, Block>,
    // TODO: We probably want to make FunctionIds to functions
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    /// This string can only be ascii, and is the mangled name of the function.
    pub signature: String,
    pub entry: BlockId,
}

#[derive(Default, Debug, Clone)]
pub struct Block {
    // The id is just the index in the blocks vector.
    pub temporaries: Vec<TempVar>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Void,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloca { dest: TempVarId, ty: Type },
    Return { value: Option<Operand> },
    Load { dest: TempVarId, src: TempVarId },
    Store { dest: TempVarId, src: Operand },
}

// Example: store 42 in a stack variable (pseudo-code, not actually how the IR will look)
// let a = alloca i32
// store 42 in a
// return a
// // or
// // When loading, we dont need to alloca
// let b = load a
// return b

