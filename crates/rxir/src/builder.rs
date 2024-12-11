use crate::{Block, BlockId, Function, Module};

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

    pub fn build_function(&mut self, name: String, entry: BlockId) {
        let function = Function {
            signature: name,
            entry,
        };
        self.functions.push(function);
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