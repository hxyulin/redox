use crate::{Block, BlockId, Function, Instruction, Module, TempVarId, Type};

pub struct ModuleBuilder {
    blocks: Vec<Block>,
    functions: Vec<Function>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn create_block(&mut self) -> BlockId {
        let block = Block::default();
        self.blocks.push(block);
        self.blocks.len() - 1
    }

    pub fn get_block(&self, block: BlockId) -> &Block {
        &self.blocks[block]
    }

    pub fn get_block_mut(&mut self, block: BlockId) -> &mut Block {
        &mut self.blocks[block]
    }

    pub fn get_var_type(&self, block: BlockId, id: TempVarId) -> Type {
        self.get_block(block).temporaries[id].ty.clone()
    }

    pub fn build_function(
        &mut self,
        signature: String,
        arguments: Vec<(TempVarId, Type)>,
        return_ty: Type,
        entry: BlockId,
    ) {
        let function = Function {
            signature,
            arguments,
            return_ty,
            entry,
        };
        self.functions.push(function);
    }

    pub fn build_instruction(&mut self, block: BlockId, instruction: Instruction) {
        self.blocks[block].instructions.push(instruction);
    }

    pub fn build(self, name: String) -> Module {
        Module {
            name,
            blocks: self
                .blocks
                .into_iter()
                .enumerate()
                .map(|(i, b)| (i, b))
                .collect(),
            functions: self.functions,
        }
    }
}
