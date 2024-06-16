use crate::{
    ast::{self, Expression, Statement},
    instructions::Instruction,
    lexer::Lexer,
    parser::Parser,
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
    parser: &'a mut Parser<'a, &'a mut Lexer<'a>>,
    first_available_register: RefCell<Register>,
}

// we know all the registers that are required as args in order
// we can copy them to the function new's frame containing its registers
// and it can access them from there, first available register will be + n
// we can store the details of the args -> register mapping at compile time
// to ensure the right register is accessed
// where n is number of args
// return register can be 0 always for all values
// when a function call ends we'll copy its register 0 to the variable or whatever
// it was assigned to
// let x = function_call(arg1, arg2);
// function call's registers/stack -> [return_value, arg1, arg2]
// our stack had [arg2, arg1] in random spots that we copied over
// and we'll copy to next register from reg0 of function call
// and then SetVariable x next_available

impl<'a> Compiler<'a> {
    pub fn new(parser: &'a mut Parser<'a, &'a mut Lexer<'a>>) -> Self {
        Self {
            parser,
            first_available_register: RefCell::new(0),
        }
    }

    pub fn compile(&mut self) -> Result<CompiledProgram, CompilerError> {
        while let Some(statement) = self.parser.next() {
            self.compile_statement(&statement);
        }

        Ok(CompiledProgram {})
    }

    fn compile_const(&self, name: &str, value: &Expression) {
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
