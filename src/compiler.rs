use crate::{
    ast::{self, Literal, Statement},
    instructions::{FunctionId, Instruction, LiteralId, Register},
    parser::ParserError,
    scope::{Scope, ScopeType},
};
use std::{cell::RefCell, rc::Rc};
use thiserror::Error;

impl From<ParserError> for CompilerError {
    fn from(value: ParserError) -> Self {
        CompilerError::ParserError { source: value }
    }
}

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("parser error: {source}")]
    ParserError { source: ParserError },
    #[error("Unknown error")]
    UnknownError,
    #[error("variable '{0}' not found in scope", variable)]
    VariableNotFound { variable: String },
}

#[derive(Default, Debug, PartialEq)]
pub struct CompiledProgram {
    pub functions: Vec<Function>,
    pub global_code: Vec<Instruction>,
    pub global_register_count: u8,
    pub literals: Vec<Literal>,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub code: Vec<Instruction>,
    pub register_count: u8,
}

#[derive(Debug)]
pub struct Compiler<I>
where
    I: Iterator<Item = Result<Statement, ParserError>>,
{
    parser: I,
    scope_stack: Vec<Scope>,
    next_available_register: Register,
    functions: Vec<Function>,
    literals: Vec<Literal>,
    global_code: Rc<RefCell<Vec<Instruction>>>,
    current_code: Rc<RefCell<Vec<Instruction>>>,
}

impl<I> Compiler<I>
where
    I: Iterator<Item = Result<Statement, ParserError>>,
{
    // wtf
    pub fn new(parser: I) -> Self {
        let global_code = Rc::new(Vec::new().into());
        Self {
            parser,
            scope_stack: vec![Scope::new(ScopeType::Global)],
            literals: vec![],
            next_available_register: 1,
            functions: Default::default(),
            global_code: Rc::clone(&global_code),
            current_code: global_code,
        }
    }

    pub fn compile(mut self) -> Result<CompiledProgram, CompilerError> {
        while let Some(statement) = self.parser.next() {
            self.compile_statement(&statement?)?;
        }

        drop(self.current_code);

        let global_register_count = self.next_available_register;

        Ok(CompiledProgram {
            functions: self.functions,
            global_code: Rc::into_inner(self.global_code)
                .ok_or(CompilerError::UnknownError)?
                .into_inner(),
            global_register_count,
            literals: self.literals,
        })
    }

    fn add_scope(&mut self) {
        self.scope_stack.push(Scope::new(ScopeType::Local));
    }

    fn define_current_scope(&mut self, name: &str, register: Register) {
        let current_scope = self.scope_stack.last_mut().unwrap();
        current_scope.define(name, register);
    }

    fn resolve(&mut self, name: &str) -> Option<Register> {
        let scope_stack = &mut self.scope_stack.iter().rev();
        for v in scope_stack {
            if let Some(reg) = v.contains(name) {
                return Some(reg);
            }
        }

        None
    }

    fn remove_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn get_register(&mut self) -> Register {
        let reg = self.next_available_register;
        self.next_available_register += 1;
        reg
    }

    fn compile_function(&mut self, func: &ast::Function) -> Result<(), CompilerError> {
        let prev_register_count = self.next_available_register;
        self.next_available_register = 1;

        self.add_scope();
        let prev_code = self.current_code.replace(Vec::new());

        for arg_name in &func.parameters {
            self.define_current_scope(arg_name, self.next_available_register);
            self.next_available_register += 1;
        }

        match *func.body {
            Statement::Block { ref body } => {
                for statement in body {
                    self.compile_statement(statement)?;
                }
            }
            _ => unreachable!(),
        }

        self.current_code.borrow_mut().push(Instruction::Return);

        let function_code = self.current_code.replace(prev_code);
        let used_registers = self.next_available_register;

        self.functions.push(Function {
            name: func.name.to_owned(),
            code: function_code,
            register_count: used_registers,
        });

        self.remove_scope();
        self.next_available_register = prev_register_count;

        Ok(())
    }

    fn compile_let(&mut self, name: &str, value: &ast::Expression) -> Result<(), CompilerError> {
        let expression_value_register = self.compile_expression(value)?;
        self.define_current_scope(name, expression_value_register);

        Ok(())
    }

    fn compile_expression(&mut self, expr: &ast::Expression) -> Result<Register, CompilerError> {
        // FIXME: potentially wasting registers
        match expr {
            #[allow(unused)]
            ast::Expression::Prefix { op, expr } => todo!(),
            ast::Expression::Infix { op, lhs, rhs } => {
                let lhs = self.compile_expression(lhs)?;
                let rhs = self.compile_expression(rhs)?;

                let dest = self.get_register();

                let instruction = match op {
                    ast::Operator::Plus => Instruction::Add { dest, lhs, rhs },
                    ast::Operator::Minus => Instruction::Sub { dest, lhs, rhs },
                    ast::Operator::Divide => Instruction::Div { dest, lhs, rhs },
                    ast::Operator::Multiply => Instruction::Mul { dest, lhs, rhs },
                    _ => unreachable!(),
                };

                self.current_code.borrow_mut().push(instruction);

                Ok(dest)
            }
            ast::Expression::Literal(lit) => {
                let reg = self.get_register();
                let literal_list = self.literals.iter().enumerate();
                let mut found_id = None;
                for (index, literal) in literal_list {
                    if literal == lit {
                        found_id = Some(index as LiteralId);
                        break;
                    }
                }

                let literal_id = if let Some(found_id) = found_id {
                    found_id
                } else {
                    // FIXME:
                    self.literals.push(lit.clone());
                    // FIXME:
                    (self.literals.len() - 1) as LiteralId
                };

                let instruction = Instruction::LoadLiteral {
                    dest: reg,
                    src: literal_id,
                };

                self.current_code.borrow_mut().push(instruction);

                Ok(reg)
            }
            ast::Expression::Variable(name) => {
                self.resolve(name).ok_or(CompilerError::VariableNotFound {
                    variable: name.to_owned(),
                })
            }
            ast::Expression::FunctionCall {
                name: function_to_call,
                args,
            } => {
                let function_list = self.functions.iter().rev().enumerate();
                let mut found_id = None;
                for (index, function) in function_list {
                    if *function.name == *function_to_call {
                        found_id = Some(index);
                    }
                }

                let found_id = match found_id {
                    Some(f) => (self.functions.len() - f - 1) as FunctionId,
                    _ => unreachable!(),
                };

                let mut regs = vec![];
                for arg in args {
                    regs.push(self.compile_expression(arg)?);
                }

                let start_reg = self.next_available_register;
                for reg in regs {
                    let dest = self.get_register();
                    let mut current_code = self.current_code.borrow_mut();
                    current_code.push(Instruction::Copy { dest, src: reg });
                }

                let last_reg = self.next_available_register;

                let reg = self.get_register();
                let instruction = Instruction::LoadFunction {
                    dest: reg,
                    src: found_id,
                };

                self.current_code.borrow_mut().push(instruction);

                let instruction = Instruction::CallFunction {
                    src: reg,
                    args: start_reg..last_reg,
                };

                self.current_code.borrow_mut().push(instruction);

                Ok(reg)
            }
        }
    }

    pub fn compile_block(&mut self, body: &Vec<Statement>) -> Result<(), CompilerError> {
        self.add_scope();

        for statement in body {
            self.compile_statement(statement)?
        }

        self.remove_scope();

        Ok(())
    }

    #[allow(unused)]
    pub fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Const { name, value } => todo!(),
            Statement::Let { name, value } => self.compile_let(name, value)?,
            Statement::If {
                condition,
                body,
                else_statement,
            } => todo!(),
            Statement::Block { body } => self.compile_block(body)?,
            Statement::Function(func) => self.compile_function(func)?,
            Statement::Expression(expr) => {
                self.compile_expression(expr);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::*;
    use crate::parser::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn small_input() {
        let input = r#"
let x = 3;
let y = 4;
let z = x + y;

fn print() {}

fn test(a) {
    let y = 1.3 + a;
    {
        let z = y + 3;
    }

    let z = y + 2;
}

fn main() {
    let x = 1.3 + 3;
    {
	print("Hello");
	print(x);
	let y = test(4);
    }

    print(x);
}


main();
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let mut parser = Parser::new(&mut lexer, &input);
        let compiler = Compiler::new(&mut parser);

        let output = compiler.compile().unwrap();

        let mut expected = CompiledProgram {
            ..Default::default()
        };

        expected.functions.push(Function {
            name: "print".to_owned(),
            code: vec![Instruction::Return],
            register_count: 1,
        });

        expected.functions.push(Function {
            name: "test".to_owned(),
            code: vec![
                Instruction::LoadLiteral { dest: 2, src: 2 },
                Instruction::Add {
                    dest: 3,
                    lhs: 2,
                    rhs: 1,
                },
                Instruction::LoadLiteral { dest: 4, src: 0 },
                Instruction::Add {
                    dest: 5,
                    lhs: 3,
                    rhs: 4,
                },
                Instruction::LoadLiteral { dest: 6, src: 3 },
                Instruction::Add {
                    dest: 7,
                    lhs: 3,
                    rhs: 6,
                },
                Instruction::Return,
            ],
            register_count: 8,
        });

        expected.functions.push(Function {
            name: "main".to_owned(),
            code: vec![
                Instruction::LoadLiteral { dest: 1, src: 2 },
                Instruction::LoadLiteral { dest: 2, src: 0 },
                Instruction::Add {
                    dest: 3,
                    lhs: 1,
                    rhs: 2,
                },
                Instruction::LoadLiteral { dest: 4, src: 4 },
                Instruction::Copy { dest: 5, src: 4 },
                // FIXME: don't reload functions that are already in reg
                Instruction::LoadFunction { dest: 6, src: 0 },
                Instruction::CallFunction { src: 6, args: 5..6 },
                Instruction::Copy { dest: 7, src: 3 },
                Instruction::LoadFunction { dest: 8, src: 0 },
                Instruction::CallFunction { src: 8, args: 7..8 },
                Instruction::LoadLiteral { dest: 9, src: 1 },
                Instruction::Copy { dest: 10, src: 9 },
                Instruction::LoadFunction { dest: 11, src: 1 },
                Instruction::CallFunction {
                    src: 11,
                    args: 10..11,
                },
                Instruction::Copy { dest: 12, src: 3 },
                Instruction::LoadFunction { dest: 13, src: 0 },
                Instruction::CallFunction {
                    src: 13,
                    args: 12..13,
                },
                Instruction::Return,
            ],
            register_count: 14,
        });

        expected.global_code = vec![
            Instruction::LoadLiteral { dest: 1, src: 0 },
            Instruction::LoadLiteral { dest: 2, src: 1 },
            Instruction::Add {
                dest: 3,
                lhs: 1,
                rhs: 2,
            },
            Instruction::LoadFunction { dest: 4, src: 2 },
            Instruction::CallFunction { src: 4, args: 4..4 },
        ];
        expected.global_register_count = 5;

        expected.literals = vec![
            Literal::Integer(3),
            Literal::Integer(4),
            Literal::Float(1.3),
            Literal::Integer(2),
            Literal::String("Hello".to_owned()),
        ];

        assert_eq!(expected, output)
    }
}
