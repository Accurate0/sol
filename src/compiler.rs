use crate::ast;

pub struct Bytecode {
    pub instructions: Vec<u8>,
}

pub struct Compiler {
    program: ast::Program,
}

impl Compiler {
    pub fn new(program: ast::Program) -> Self {
        Self { program }
    }

    // pub fn compile(&self) -> Bytecode {}
}
