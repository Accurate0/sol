use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{self, Literal, Statement},
    instructions::{FunctionId, Instruction, LiteralId, Register},
    scope::{Scope, ScopeType},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Unknown error")]
    UnknownError,
    #[error("{0} not found in scope", variable)]
    VariableNotFound { variable: String },
}

#[derive(Default, Debug)]
pub struct CompiledProgram {
    pub functions: Vec<Function>,
    pub global_code: Vec<Instruction>,
    pub literals: Vec<Literal>,
}

#[derive(Debug)]
pub enum Function {
    Defined {
        name: String,
        code: Vec<Instruction>,
    },
    Undefined {
        name: String,
    },
}

#[derive(Debug)]
pub struct Compiler<I>
where
    I: Iterator<Item = Statement>,
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
    I: Iterator<Item = Statement>,
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
            self.compile_statement(&statement)?;
        }

        drop(self.current_code);

        Ok(CompiledProgram {
            functions: self.functions,
            global_code: Rc::into_inner(self.global_code)
                .ok_or(CompilerError::UnknownError)?
                .into_inner(),
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

        let function_code = self.current_code.replace(prev_code);

        let functions = self.functions.iter_mut().rev();
        let mut found_existing_function = None;
        for f in functions {
            match f {
                Function::Undefined { name } if *name == func.name => {
                    found_existing_function = Some(f)
                }
                Function::Defined { name, .. } if *name == func.name => unreachable!(),
                Function::Defined { .. } => continue,
                Function::Undefined { .. } => continue,
            }
        }

        if let Some(f) = found_existing_function {
            *f = Function::Defined {
                name: func.name.to_owned(),
                code: function_code,
            }
        } else {
            self.functions.push(Function::Defined {
                name: func.name.to_owned(),
                code: function_code,
            });
        }

        self.remove_scope();
        self.next_available_register = prev_register_count;

        Ok(())
    }

    fn compile_let(&mut self, name: &str, value: &ast::Expression) -> Result<(), CompilerError> {
        let expression_value_register = self.compile_expression(value)?;
        self.define_current_scope(name, expression_value_register);

        Ok(())
    }

    #[allow(unused)]
    fn compile_expression(&mut self, expr: &ast::Expression) -> Result<Register, CompilerError> {
        // FIXME: potentially wasting registers
        match expr {
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
                let mut literal_list = self.literals.iter().rev().enumerate();
                let mut found_existing_literal = None;
                let mut found_id = None;
                for (index, literal) in literal_list {
                    if literal == lit {
                        found_id = Some(index as LiteralId);
                        found_existing_literal = Some(literal);
                        break;
                    }
                }

                let literal_id = if let Some(existing_literal) = found_existing_literal {
                    found_id.unwrap()
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
                let mut function_list = self.functions.iter().rev().enumerate();
                let mut found_id = None;
                for (index, function) in function_list {
                    match function {
                        Function::Defined { name, code } if function_to_call == name => {
                            found_id = Some(index);
                            break;
                        }
                        Function::Defined { name, code } => continue,
                        Function::Undefined { name } if function_to_call == name => {
                            found_id = Some(index);
                            break;
                        }
                        Function::Undefined { name } => continue,
                    }
                }

                let found_id = match found_id {
                    Some(f) => (self.functions.len() - f - 1) as FunctionId,
                    None => {
                        self.functions.push(Function::Undefined {
                            name: function_to_call.to_owned(),
                        });
                        (self.functions.len() - 1) as FunctionId
                    }
                };

                let reg = self.get_register();
                let instruction = Instruction::LoadFunction {
                    dest: reg,
                    src: found_id,
                };

                self.current_code.borrow_mut().push(instruction);

                let instruction = Instruction::CallFunction { src: reg };

                self.current_code.borrow_mut().push(instruction);

                Ok(reg)
            }
        }
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
            Statement::Block { body } => {
                for statement in body {
                    self.compile_statement(statement)?
                }
            }
            Statement::Function(func) => self.compile_function(func)?,
            Statement::Expression(expr) => {
                self.compile_expression(expr);
            }
        }

        Ok(())
    }
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
