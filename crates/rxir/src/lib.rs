use std::collections::HashMap;

mod builder;
pub use builder::*;
mod pass;
pub use pass::*;

#[derive(Debug, Clone)]
pub enum IRBuildType {
    DynamicLibrary,
    StaticLibrary,
    Executable { entry: BlockId },
}

#[derive(Debug, Clone)]
pub struct TempVar {
    ty: Type,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate { ty: Type, value: u64 },
    TempVar { ty: Type, id: TempVarId },
}

impl ToString for Operand {
    fn to_string(&self) -> String {
        match self {
            Operand::Immediate { ty, value } => format!("{} {}", ty.to_string(), value),
            Operand::TempVar { ty, id } => format!("{} %{}", ty.to_string(), id),
        }
    }
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

impl Module {}

impl ToString for Module {
    fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("Module {}\n", self.name));
        for function in &self.functions {
            let arguments = function
                .arguments
                .iter()
                .map(|(name, ty)| format!("%{}: {}", name, ty.to_string()))
                .collect::<Vec<String>>()
                .join(", ");
            let associated_blocks = utils::get_related_blocks(self, function);
            result.push_str(&format!(
                "Function {} {} ({}) {{\n",
                function.return_ty.to_string(),
                function.signature,
                arguments
            ));

            for block in associated_blocks {
                result.push_str(&format!("  Block {}\n", block));
                for instruction in &self.blocks[&block].instructions {
                    result.push_str(&format!("    {}\n", instruction.to_string()));
                }
            }

            result.push_str("}\n");
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    /// This string can only be ascii, and is the mangled name of the function.
    pub signature: String,
    pub arguments: Vec<(TempVarId, Type)>,
    pub entry: BlockId,
    pub return_ty: Type,
}

#[derive(Default, Debug, Clone)]
pub struct Block {
    var_count: TempVarId,
    pub temporaries: Vec<TempVar>,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            var_count: 0,
            temporaries: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn create_var(&mut self, ty: Type) -> TempVarId {
        self.temporaries.push(TempVar { ty });
        self.var_count += 1;
        self.var_count - 1
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Signed32,
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Void => "void".to_string(),
            Type::Signed32 => "s32".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloca { dest: TempVarId, ty: Type },
    Return { value: Option<Operand> },
    Load { dest: TempVarId, src: TempVarId },
    Store { dest: TempVarId, src: Operand },
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Alloca { dest, ty } => format!("Alloca {} {}", dest, ty.to_string()),
            Instruction::Return { value } => format!(
                "Return {}",
                value.clone().map_or("void".to_string(), |v| v.to_string())
            ),
            Instruction::Load { dest, src } => format!("Load %{} %{}", dest, src),
            Instruction::Store { dest, src } => format!("Store %{} {}", dest, src.to_string()),
        }
    }
}

// Example: store 42 in a stack variable (pseudo-code, not actually how the IR will look)
// let a = alloca i32
// store 42 in a
// return a
// // or
// // When loading, we dont need to alloca
// let b = load a
// return b

pub mod utils {
    use crate::{BlockId, Function, Module};
    use std::collections::HashSet;

    /// Gets all the blocks that are related to the given function, ordered in the order of execution
    pub fn get_related_blocks(module: &Module, function: &Function) -> Vec<BlockId> {
        let mut blocks = Vec::new();
        blocks.push(function.entry);
        let mut visited: HashSet<BlockId> = HashSet::new();
        let mut temp = vec![function.entry];
        // Now we iterate through the blocks, always starting with the first block, and prepending
        // to the blocks that have already visited, so the blocks are ordered in the correct order
        // when executing

        while let Some(block) = temp.pop() {
            let connected_blocks = get_connected_blocks(module, &block);
            let connected_blocks = connected_blocks.iter().filter(|blk| {
                if visited.contains(blk) {
                    return false;
                }
                visited.insert(**blk);
                true
            });
            // We reverse the order so that the blocks are ordered in the correct order
            temp.extend(connected_blocks.rev());
            // TODO: We need to add it to the blocks, but we need to make sure that we add it in
            // the correct order, this is difficult because we need to put it in the blocks in
            // normal order, but reversed order for the stack (because the stack is top to bottom)
        }

        blocks
    }

    /// Gets all the blocks that are directly connected to the given block, so for a jump instruction
    /// this will return the block that the jump will jump to, for a branch instruction this will
    /// return the two blocks that the branch will jump to.
    pub fn get_connected_blocks(module: &Module, block: &BlockId) -> Vec<BlockId> {
        // TODO: Implement this, but we currently dont have control flow
        Vec::new()
    }
}
