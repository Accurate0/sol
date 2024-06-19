use std::borrow::Cow;
use std::usize;
use thiserror::Error;

use crate::{
    ast,
    compiler::{self, CompiledProgram},
    instructions::Instruction,
};

#[derive(Error, Debug)]
pub enum ExecutionError {}

#[derive(Default, Debug)]
pub enum RegisterValue<'a> {
    #[default]
    Empty,
    Literal(Cow<'a, ast::Literal>),
    Function(&'a compiler::Function),
}

pub struct VM {
    functions: Vec<compiler::Function>,
    global_code: Vec<Instruction>,
    literals: Vec<ast::Literal>,
}

macro_rules! impl_binary_op {
    ($registers:expr, $dest: expr, $lhs:expr, $x:tt, $rhs:expr) => {
        match (&$registers[*$lhs as usize], &$registers[*$rhs as usize]) {
            (RegisterValue::Literal(lhs), RegisterValue::Literal(rhs)) => {
                let lhs = lhs.as_ref();
                let rhs = rhs.as_ref();

                match (lhs, rhs) {
                    (ast::Literal::Float(lhs), ast::Literal::Float(rhs)) => {
                        $registers[*$dest as usize] =
                            RegisterValue::Literal(Cow::Owned(ast::Literal::Float(lhs $x rhs)))
                    }
                    (ast::Literal::Float(lhs), ast::Literal::Integer(rhs)) => {
                        $registers[*$dest as usize] = RegisterValue::Literal(Cow::Owned(
                            ast::Literal::Float(*lhs $x *rhs as f64),
                        ))
                    }
                    (ast::Literal::Integer(lhs), ast::Literal::Float(rhs)) => {
                        $registers[*$dest as usize] = RegisterValue::Literal(Cow::Owned(
                            ast::Literal::Float(*lhs as f64 $x *rhs),
                        ))
                    }
                    (ast::Literal::Integer(lhs), ast::Literal::Integer(rhs)) => {
                        $registers[*$dest as usize] =
                            RegisterValue::Literal(Cow::Owned(ast::Literal::Integer(lhs $x rhs)))
                    }

                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    };
}

impl VM {
    pub fn new(compiled_program: CompiledProgram) -> Self {
        Self {
            functions: compiled_program.functions,
            global_code: compiled_program.global_code,
            literals: compiled_program.literals,
        }
    }

    pub fn run(&self) -> Result<(), ExecutionError> {
        let mut registers = Vec::<RegisterValue>::with_capacity(u8::MAX as usize);
        registers.resize_with(u8::MAX as usize, Default::default);
        let mut saved_ip_stack = vec![];
        let mut saved_code_stack = vec![];

        let mut current_code = &self.global_code;

        let mut ip = 0;
        loop {
            if ip >= current_code.len() {
                if !saved_ip_stack.is_empty() {
                    ip = saved_ip_stack.pop().unwrap();
                    current_code = saved_code_stack.pop().unwrap();
                    continue;
                }

                break;
            }

            let current_instruction = &current_code[ip];
            // tracing::info!("executing: {:?}", current_instruction);
            // tracing::info!("executing: {:?}", ip);
            // tracing::info!("executing: {:?}", current_code);

            match current_instruction {
                Instruction::LoadFunction { dest, src } => {
                    let func = &self.functions[*src as usize];
                    registers[*dest as usize] = RegisterValue::Function(func);
                }
                Instruction::CallFunction { src } => {
                    let func = &registers[*src as usize];
                    let func = match func {
                        RegisterValue::Function(f) => f,
                        _ => unreachable!(),
                    };

                    // FIXME: REGISTER WINDOW
                    saved_ip_stack.push(ip + 1);
                    saved_code_stack.push(current_code);

                    current_code = match func {
                        compiler::Function::Defined { code, .. } => code,
                        _ => unreachable!(),
                    };
                    ip = 0;
                    continue;
                }
                Instruction::LoadLiteral { dest, src } => {
                    let literal = &self.literals[*src as usize];
                    registers[*dest as usize] = RegisterValue::Literal(Cow::Borrowed(literal));
                }
                Instruction::Add { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, +, rhs)
                }
                Instruction::Sub { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, -, rhs)
                }
                Instruction::Mul { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, *, rhs)
                }
                Instruction::Div { dest, lhs, rhs } => {
                    impl_binary_op!(registers, dest, lhs, /, rhs)
                }
            }

            ip += 1;
        }

        dbg!(registers);

        Ok(())
    }
}
