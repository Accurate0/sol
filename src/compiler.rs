use crate::{
    ast::{self, Expression, Statement},
    instructions::Instruction,
    vm::Register,
};
use std::cell::RefCell;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {}

#[derive(Default, Debug)]
pub struct CompiledProgram {}

#[derive(Default, Debug, Clone)]
pub enum Value {
    #[default]
    Nil,
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
}

pub struct Compiler<'a> {
    // FIXME: get iterator instead
    program: &'a ast::ParsedProgram,
    first_available_register: RefCell<Register>,
}

impl<'a> Compiler<'a> {
    pub fn new(program: &'a ast::ParsedProgram) -> Self {
        Self {
            program,
            first_available_register: RefCell::new(0),
        }
    }

    pub fn compile(&self) -> Result<CompiledProgram, CompilerError> {
        for statement in &self.program.statements {
            self.compile_statement(statement);
        }

        Ok(CompiledProgram {})
    }

    fn compile_const(&self, name: &String, value: &Expression) {
        match value {
            Expression::Literal(lit) => {
                let value = match lit {
                    ast::Literal::String(s) => Value::String(s.to_owned()),
                    ast::Literal::Float(f) => Value::Float(*f),
                    ast::Literal::Integer(i) => Value::Integer(*i),
                    ast::Literal::Boolean(b) => Value::Boolean(*b),
                };
            }
            _ => unreachable!(),
        }
    }

    fn get_register(&self) -> Register {
        // let mut current_block = self.current_block.borrow_mut();
        // let reg = current_block.first_available_register;
        //
        // current_block.first_available_register += 1;
        let mut first_available_register = self.first_available_register.borrow_mut();
        let reg = *first_available_register;
        *first_available_register += 1;

        reg
    }

    fn compile_expression(&self, expression: &Expression) -> Register {
        match expression {
            Expression::Prefix { op, expr } => todo!(),
            Expression::FunctionCall { name, args } => {
                let registers = args
                    .iter()
                    .map(|e| self.compile_expression(e))
                    .collect::<Vec<_>>();

                let instruction = Instruction::FunctionCall {
                    name: name.to_owned(),
                    args: registers,
                };

                println!("{:?}", instruction);

                // need a calling convention, special register?
                99
            }
            Expression::Infix { op, lhs, rhs } => match op {
                ast::Operator::Plus => {
                    let dest = self.get_register();
                    let lhs = self.compile_expression(lhs);
                    let rhs = self.compile_expression(rhs);

                    let instruction = Instruction::Add { dest, lhs, rhs };
                    println!("{:?}", instruction);

                    dest
                }
                ast::Operator::Minus => todo!(),
                ast::Operator::Multiply => todo!(),
                ast::Operator::Not => todo!(),
                ast::Operator::Divide => todo!(),
            },
            Expression::Literal(lit) => {
                let dest = self.get_register();
                let instruction = Instruction::Load {
                    dest,
                    value: match lit {
                        ast::Literal::String(s) => Value::String(s.to_owned()),
                        ast::Literal::Float(f) => Value::Float(*f),
                        ast::Literal::Integer(i) => Value::Integer(*i),
                        ast::Literal::Boolean(b) => Value::Boolean(*b),
                    },
                };

                println!("{:?}", instruction);
                dest
            }
            Expression::Variable(name) => {
                let dest = self.get_register();
                let instruction = Instruction::GetVariable {
                    dest,
                    src: name.to_owned(),
                };

                println!("{:?}", instruction);
                dest
            }
        }
    }

    fn compile_let(&self, name: &String, expression: &Expression) {
        let rhs = self.compile_expression(expression);

        let instruction = Instruction::SetVariable {
            src: rhs,
            dest: name.to_owned(),
        };

        println!("{:?}", instruction);
    }

    fn compile_block(&self, statements: &Vec<Statement>) {
        // scope change
        for statement in statements {
            self.compile_statement(statement);
        }
    }

    fn compile_function(&self, function: &ast::Function) {
        // new scope

        match *function.body {
            Statement::Block { ref body } => self.compile_block(body),
            _ => unreachable!(),
        }
    }

    pub fn compile_statement(&self, statement: &Statement) {
        match statement {
            Statement::Const { name, value } => self.compile_const(name, value),
            Statement::Let { name, value } => self.compile_let(name, value),
            Statement::Block { body } => self.compile_block(body),
            Statement::If {
                condition,
                body,
                else_statement,
            } => todo!(),
            Statement::Function(f) => self.compile_function(f),
            Statement::Expression(e) => {
                self.compile_expression(e);
            }
        }
    }
}
