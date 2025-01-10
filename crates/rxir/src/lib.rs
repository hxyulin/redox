use std::{collections::HashMap, fmt::Write};

/// RXIR Representation:
/// - variables prefixed with '%' are temporary variables
/// - variables prefixed with '@' are block labels
mod builder;
mod operand;
mod pass;
pub use crate::{builder::*, operand::*, pass::*};
pub use ascii::AsciiString;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TempVarId {
    Generated(usize),
    Named(AsciiString),
}

impl ToString for TempVarId {
    fn to_string(&self) -> String {
        match self {
            Self::Generated(size) => format!("%{size}"),
            Self::Named(name) => format!("%{name}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlockId {
    Generated(usize),
    Named(AsciiString),
}

impl ToString for BlockId {
    fn to_string(&self) -> String {
        match self {
            Self::Generated(size) => format!("@{size}"),
            Self::Named(name) => format!("@{name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TempVar {
    ty: Type,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: AsciiString,
    pub blocks: HashMap<BlockId, Block>,
    pub functions: Vec<Function>,
}

impl ToString for Module {
    fn to_string(&self) -> String {
        let mut result = format!("module {}\n", self.name);
        for function in &self.functions {
            result.push_str(&function.to_string(self));
        }
        result
    }
}

impl Module {}

#[derive(Debug, Clone)]
pub struct Function {
    pub signature: AsciiString,
    pub arguments: Vec<(TempVarId, Type)>,
    pub entry: BlockId,
    pub return_ty: Type,
}

impl Function {
    fn to_string(&self, module: &Module) -> String {
        let arguments = self
            .arguments
            .iter()
            .map(|(id, ty)| format!("{}: {ty}", id.to_string()))
            .collect::<Vec<String>>()
            .join(", ");
        let mut result = format!(
            "fn {} {} ({}) {{\n",
            self.return_ty, self.signature, arguments
        );

        let associated_blocks = utils::get_related_blocks(module, self);
        for id in associated_blocks {
            let block = module.blocks.get(&id).unwrap();
            result.push_str(&format!("{}:\n", id.to_string()));
            result.push_str(&block.to_string());
        }

        result.push_str("}\n");
        result
    }
}

#[derive(Default, Debug, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
}

impl ToString for Block {
    fn to_string(&self) -> String {
        let mut result = String::new();
        for instruction in &self.instructions {
            result.push_str("\t");
            result.push_str(instruction.to_string().as_str());
            result.push('\n');
        }
        result
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Signed32,
    Pointer(Box<Type>),
}

impl Type {
    pub fn pointer(ty: Type) -> Self {
        Type::Pointer(Box::new(ty))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => f.write_str("void"),
            Type::Signed32 => f.write_str("i32"),
            Type::Pointer(ty) => f.write_fmt(format_args!("{}*", *ty)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    /// Allocates a new variable on the stack, returns it into the variable at the given id, which
    /// is a pointer to the type, which needs to be loaded into a register
    Alloca {
        dest: TempVarId,
        ty: Type,
    },
    Return {
        value: Option<Operand>,
    },
    Load {
        dest: TempVarId,
        src: TempVarId,
    },
    Store {
        dest: TempVarId,
        src: Operand,
    },
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Self::Alloca { dest, ty } => format!("{} = alloca {}", dest.to_string(), ty),
            Self::Return { value } => match value {
                None => "return void".to_string(),
                Some(value) => format!("return {} {}", value.ty(), value.to_string()),
            },
            Self::Load { .. } | Self::Store { .. } => unimplemented!(),
        }
    }
}

// Example: store 42 in a stack variable (pseudo-code, not actually how the IR will look)
// let a = alloca i32 // 'a' have type (*i32)
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
        blocks.push(function.entry.clone());
        let mut visited: HashSet<BlockId> = HashSet::new();
        let mut temp = vec![function.entry.clone()];
        // Now we iterate through the blocks, always starting with the first block, and prepending
        // to the blocks that have already visited, so the blocks are ordered in the correct order
        // when executing

        while let Some(block) = temp.pop() {
            let connected_blocks = get_connected_blocks(module, &block);
            let connected_blocks = connected_blocks
                .iter()
                .filter(|blk| {
                    let blk = (*blk).clone();
                    if visited.contains(&blk) {
                        return false;
                    }
                    visited.insert(blk);
                    true
                })
                .cloned();
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
