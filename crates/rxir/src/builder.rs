use crate::{Block, BlockId, Function, Instruction, Module, TempVar, TempVarId, Type};
use ascii::AsciiString;
use std::collections::HashMap;

pub struct ModuleBuilder {
    counter: usize,

    variables: HashMap<BlockId, HashMap<TempVarId, TempVar>>,
    blocks: HashMap<BlockId, Block>,
    functions: Vec<Function>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            counter: 0,
            variables: HashMap::new(),
            blocks: HashMap::new(),
            functions: Vec::new(),
        }
    }

    pub fn create_block(&mut self, name: Option<AsciiString>) -> BlockId {
        let block = Block::default();
        let id = match name {
            Some(name) => BlockId::Named(name),
            None => {
                self.counter += 1;
                BlockId::Generated(self.counter - 1)
            }
        };
        self.blocks.insert(id.clone(), block);
        self.variables.insert(id.clone(), HashMap::new());
        id
    }

    /// Creates a value, that is considered to be 'loaded', this can come from:
    /// - Function arguments
    /// - Function return values
    /// - Results of operations (e.g. addition)
    /// - Immediates
    /// - Loaded variables from the 'load' instruction
    pub fn create_value(
        &mut self,
        block: &BlockId,
        ty: Type,
        name: Option<AsciiString>,
    ) -> TempVarId {
        let id = match name {
            Some(name) => TempVarId::Named(name),
            None => {
                self.counter += 1;
                TempVarId::Generated(self.counter - 1)
            }
        };
        self.variables
            .get_mut(&block)
            .unwrap()
            .insert(id.clone(), TempVar { ty });
        id
    }

    /// This returns a pointer to a value, which is allocated on the stack.
    pub fn build_alloca(
        &mut self,
        block: &BlockId,
        ty: Type,
        name: Option<AsciiString>,
    ) -> TempVarId {
        let ptr_ty = Type::pointer(ty.clone());
        let id = self.create_value(block, ptr_ty, name);
        self.get_block_mut(block)
            .instructions
            .push(Instruction::Alloca {
                dest: id.clone(),
                ty,
            });
        id
    }

    pub fn get_block(&self, block: &BlockId) -> &Block {
        self.blocks.get(block).unwrap()
    }

    pub fn get_block_mut(&mut self, block: &BlockId) -> &mut Block {
        self.blocks.get_mut(block).unwrap()
    }

    pub fn get_var_type(&self, block: &BlockId, id: &TempVarId) -> Type {
        self.variables.get(block).unwrap()[id].ty.clone()
    }

    pub fn build_function(
        &mut self,
        signature: AsciiString,
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

    pub fn build_instruction(&mut self, block: &BlockId, instruction: Instruction) {
        self.get_block_mut(block).instructions.push(instruction);
    }

    pub fn build(self, name: AsciiString) -> Module {
        Module {
            name,
            blocks: self.blocks.into_iter().map(|(i, b)| (i, b)).collect(),
            functions: self.functions,
        }
    }
}
