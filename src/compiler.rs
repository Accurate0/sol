use crate::ast::{self, Statement};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {}

#[derive(Default, Debug)]
pub struct CompiledProgram {}

#[derive(Default, Debug)]
pub struct Environment {
    pub parent: Box<Environment>,
    // name -> stack pointer
    // actually, should search up the stack
    // could optimise for functions by making a map with name -> [stack pointers]
    // multiple allow redef of same variable, shadowing it
    // literal strings go into constant map with String -> String
}

pub struct Compiler<'a> {
    program: &'a ast::ParsedProgram,
}

impl<'a> Compiler<'a> {
    pub fn new(program: &'a ast::ParsedProgram) -> Self {
        Self { program }
    }

    pub fn compile(&self) -> Result<CompiledProgram, CompilerError> {
        let mut compiled_program = CompiledProgram {
            ..Default::default()
        };

        for statement in &self.program.statements {
            Self::compile_statement(&statement, &mut compiled_program)
        }

        Ok(compiled_program)
    }

    pub fn compile_statement(statement: &Statement, program: &mut CompiledProgram) {
        match statement {
            Statement::Const { name, value } => todo!(),
            Statement::Let { name, value } => todo!(),
            Statement::If {
                condition,
                body,
                else_statement,
            } => todo!(),
            Statement::Block { body } => todo!(),
            Statement::Function(_) => todo!(),
            Statement::Expression(_) => todo!(),
        }
    }
}
