use crate::Module;

pub trait VerificationPass {
    fn run(&mut self, module: &Module) -> Result<(), String>;
}

pub trait GenerationPass {
    fn run(&mut self, module: &mut Module);
}

pub struct PassManager {
    verifiers: Vec<Box<dyn VerificationPass>>,
    optimizers: Vec<Box<dyn GenerationPass>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            verifiers: Vec::new(),
            optimizers: Vec::new(),
        }
    }

    pub fn add_verifier(&mut self, verifier: Box<dyn VerificationPass>) {
        self.verifiers.push(verifier);
    }

    pub fn add_optiizer(&mut self, optiizer: Box<dyn GenerationPass>) {
        self.optimizers.push(optiizer);
    }

    pub fn run(&mut self, module: &mut Module) -> Result<(), String> {
        for verifier in &mut self.verifiers {
            verifier.run(module)?;
        }
        for optimizer in &mut self.optimizers {
            optimizer.run(module);
        }
        Ok(())
    }
}
