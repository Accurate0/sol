use crate::{
    ast::{self, Expression, Statement},
    instructions::{FunctionId, Instruction, JumpOffset, LiteralId, Register},
    parser::ParserError,
    scope::{Scope, ScopeType},
    types::Literal,
};
use std::{cell::RefCell, fmt::Display, num::TryFromIntError};
use thiserror::Error;

impl From<ParserError> for CompilerError {
    fn from(value: ParserError) -> Self {
        CompilerError::ParserError { source: value }
    }
}

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("{source}")]
    ParserError { source: ParserError },
    #[error("{cause}")]
    GeneralError { cause: String },
    #[error("variable '{0}' not found in scope", variable)]
    VariableNotFound { variable: String },
    #[error("variable '{0}' is not mutable", variable)]
    MutationNotAllowed { variable: String },
    #[error("integer conversion error: '{0}'")]
    IntegerConversionError(#[from] TryFromIntError),
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

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<function: {} - code len: {}>",
            self.name,
            self.code.len()
        )
    }
}

#[derive(Debug)]
pub struct Compiler {
    scope_stack: Vec<Scope>,
    next_available_register: Register,
    functions: Vec<Function>,
    literals: Vec<Literal>,
    // FIXME: probably doesn't need to be a RefCell
    bytecode: RefCell<Vec<Instruction>>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let bytecode = Vec::new().into();
        Self {
            scope_stack: vec![Scope::new(ScopeType::Global)],
            literals: vec![],
            next_available_register: 1,
            functions: Default::default(),
            bytecode,
        }
    }

    pub fn compile(
        mut self,
        statements: &Vec<Statement>,
    ) -> Result<CompiledProgram, CompilerError> {
        for statement in statements {
            self.compile_statement(statement)?;
        }

        let global_register_count = self.next_available_register;

        Ok(CompiledProgram {
            functions: self.functions,
            global_code: self.bytecode.into_inner(),
            global_register_count,
            literals: self.literals,
        })
    }

    fn add_scope(&mut self) {
        self.scope_stack.push(Scope::new(ScopeType::Local));
    }

    fn define_immutable_current_scope(&mut self, name: &str, register: Register) {
        let current_scope = self.scope_stack.last_mut().unwrap();
        current_scope.define_immutable(name, register);
    }

    fn define_mutable_current_scope(&mut self, name: &str, register: Register) {
        let current_scope = self.scope_stack.last_mut().unwrap();
        current_scope.define_mutable(name, register);
    }

    fn define_function_current_scope(&mut self, name: &str) {
        let current_scope = self.scope_stack.last_mut().unwrap();
        current_scope.define_function(name);
    }

    fn can_mutate_variable(&mut self, name: &str) -> bool {
        let scope_stack = &mut self.scope_stack.iter().rev();
        for v in scope_stack {
            if v.get_register_for(name).is_some() {
                return v.is_mutable(name).is_some_and(|m| m);
            }
        }

        false
    }

    fn resolve(&mut self, name: &str) -> Option<Register> {
        let scope_stack = &mut self.scope_stack.iter().rev();
        for v in scope_stack {
            if let Some(reg) = v.get_register_for(name) {
                return Some(reg);
            }
        }

        None
    }

    fn resolve_function(&mut self, name: &str) -> bool {
        let scope_stack = &mut self.scope_stack.iter().rev();
        for v in scope_stack {
            if v.contains_function(name) {
                return true;
            }
        }
        false
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

        self.define_function_current_scope(&func.name);
        self.add_scope();
        let prev_code = self.bytecode.replace(Vec::new());

        for param in &func.parameters {
            self.define_immutable_current_scope(&param.name, self.next_available_register);
            self.next_available_register += 1;
        }

        match *func.body {
            Statement::Block { ref body } => {
                for statement in body {
                    self.compile_statement(statement)?;
                }
            }
            _ => {
                return Err(CompilerError::GeneralError {
                    cause: "function body must contain block".to_owned(),
                })
            }
        }

        self.bytecode.borrow_mut().push(Instruction::FunctionReturn);

        let function_code = self.bytecode.replace(prev_code);
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

    fn compile_let(
        &mut self,
        name: &str,
        value: &ast::Expression,
        is_mutable: bool,
    ) -> Result<(), CompilerError> {
        let expression_value_register = self.compile_expression(value)?;
        if is_mutable {
            self.define_mutable_current_scope(name, expression_value_register);
        } else {
            self.define_immutable_current_scope(name, expression_value_register);
        }

        Ok(())
    }

    fn compile_let_mutation(
        &mut self,
        name: &str,
        value: &ast::Expression,
    ) -> Result<(), CompilerError> {
        let can_mutate = self.can_mutate_variable(name);
        if can_mutate {
            let expression_value_register = self.compile_expression(value)?;
            let mutable_value_register = self.resolve(name).unwrap();

            let instruction = Instruction::Copy {
                dest: mutable_value_register,
                src: expression_value_register,
            };

            self.bytecode.borrow_mut().push(instruction);
            // ???
            self.define_mutable_current_scope(name, mutable_value_register);

            Ok(())
        } else {
            Err(CompilerError::MutationNotAllowed {
                variable: name.to_owned(),
            })
        }
    }

    fn compile_expression(&mut self, expr: &ast::Expression) -> Result<Register, CompilerError> {
        // FIXME: potentially wasting registers
        match expr {
            ast::Expression::Prefix { op, expr } => {
                let rhs = self.compile_expression(expr)?;
                let dest = self.get_register();

                let instruction = match op {
                    ast::Operator::Minus => Instruction::PrefixSub { dest, rhs },
                    ast::Operator::Not => Instruction::PrefixNot { dest, rhs },
                    _ => {
                        return Err(CompilerError::GeneralError {
                            cause: "prefix expression only works with '-' and '!'".to_owned(),
                        })
                    }
                };

                self.bytecode.borrow_mut().push(instruction);

                Ok(dest)
            }
            ast::Expression::Infix { op, lhs, rhs } => {
                let lhs = self.compile_expression(lhs)?;
                let rhs = self.compile_expression(rhs)?;

                let dest = self.get_register();

                let instruction = match op {
                    // FIXME: isn't this dumb?
                    ast::Operator::Plus => Instruction::Add { dest, lhs, rhs },
                    ast::Operator::Minus => Instruction::Sub { dest, lhs, rhs },
                    ast::Operator::Divide => Instruction::Div { dest, lhs, rhs },
                    ast::Operator::Multiply => Instruction::Mul { dest, lhs, rhs },
                    ast::Operator::Equal => Instruction::Equals { dest, lhs, rhs },
                    ast::Operator::NotEqual => Instruction::NotEquals { dest, lhs, rhs },
                    ast::Operator::GreaterThan => Instruction::GreaterThan { dest, lhs, rhs },
                    ast::Operator::GreaterThanOrEqual => {
                        Instruction::GreaterThanOrEquals { dest, lhs, rhs }
                    }
                    ast::Operator::LessThan => Instruction::LessThan { dest, lhs, rhs },
                    ast::Operator::LessThanOrEqual => {
                        Instruction::LessThanOrEquals { dest, lhs, rhs }
                    }
                    _ => {
                        return Err(CompilerError::GeneralError {
                            cause: "infix expression only works for '+', '-', '/', '*'".to_owned(),
                        })
                    }
                };

                self.bytecode.borrow_mut().push(instruction);

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

                self.bytecode.borrow_mut().push(instruction);

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
                // if it exists in scope we'll get the id
                // if we don't have the id for the function, then we'll act like its a
                // global/native function
                let found_id = if self.resolve_function(function_to_call) {
                    self.functions
                        .iter()
                        .rev()
                        .enumerate()
                        .find(|(_, f)| f.name == *function_to_call)
                        .map(|(i, _)| i)
                } else {
                    None
                };

                let mut regs = vec![];
                for arg in args {
                    regs.push(self.compile_expression(arg)?);
                }

                let start_reg = self.next_available_register;
                for reg in regs {
                    let dest = self.get_register();
                    let mut current_code = self.bytecode.borrow_mut();
                    current_code.push(Instruction::Copy { dest, src: reg });
                }

                let last_reg = self.next_available_register;

                let found_id = match found_id {
                    Some(f) => (self.functions.len() - f - 1) as FunctionId,
                    _ => {
                        // TODO: instead of assuming it's native, we can set a placeholder
                        //       and figure out at runtime which function to call?
                        //       right now, we can only call functions which we've parsed
                        //       see 'call_before_declare_function.rl' test case
                        //
                        // Note: We can check if stdlib functions exist, but not at runtime with VM
                        //       defined functions.... maybe we need to require definitions earlier
                        //       at the compiler level rather than at VM

                        // if no existing function, assume there is a native function
                        // available in the VM, this is now a runtime error if it doesn't exist
                        let register = self.compile_expression(&Expression::Literal(
                            Literal::String(function_to_call.to_owned()),
                        ))?;

                        let return_value = self.get_register();

                        let instruction = Instruction::CallNativeFunction {
                            src: register,
                            arg_count: last_reg - start_reg,
                            return_val: return_value,
                        };

                        self.bytecode.borrow_mut().push(instruction);

                        return Ok(return_value);
                    }
                };

                let reg = self.get_register();
                let return_value = self.get_register();
                let instruction = Instruction::LoadFunction {
                    dest: reg,
                    src: found_id,
                };

                self.bytecode.borrow_mut().push(instruction);

                let instruction = Instruction::CallFunction {
                    src: reg,
                    arg_count: last_reg - start_reg,
                    return_val: return_value,
                };

                self.bytecode.borrow_mut().push(instruction);

                Ok(return_value)
            }
            Expression::Object { fields } => {
                let reg = self.get_register();

                let instruction = Instruction::AllocateObject { dest: reg };
                self.bytecode.borrow_mut().push(instruction);

                // FIXME: use a deterministic hashmap or something instead of sorting (cave man)
                for (name, value) in fields.iter() {
                    let name = self.compile_expression(&Expression::Literal(Literal::String(
                        name.to_string(),
                    )))?;
                    let value = self.compile_expression(value)?;

                    let instruction = Instruction::SetObjectField {
                        object: reg,
                        field: name,
                        value,
                    };

                    self.bytecode.borrow_mut().push(instruction);
                }

                Ok(reg)
            }
            Expression::ObjectAccess { path } => {
                let register = self.get_register();
                let base_obj = path.first().unwrap();
                let mut obj_reg =
                    self.compile_expression(&Expression::Variable(base_obj.to_string()))?;

                for path_value in path.iter().skip(1) {
                    let path_reg = self.compile_expression(&Expression::Literal(
                        Literal::String(path_value.to_string()),
                    ))?;

                    let instruction = Instruction::GetObjectField {
                        object: obj_reg,
                        field: path_reg,
                        return_val: register,
                    };

                    self.bytecode.borrow_mut().push(instruction);

                    obj_reg = register;
                }

                Ok(register)
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

    pub fn compile_if(
        &mut self,
        condition: &Expression,
        body: &Statement,
        else_statement: &Option<Box<Statement>>,
    ) -> Result<(), CompilerError> {
        let expression_value_register = self.compile_expression(condition)?;

        // FIXME: use guards or something way better
        let if_statement_body = Vec::new();
        let old_current_code = self.bytecode.replace(if_statement_body);

        self.compile_statement(body)?;

        let mut if_statement_body = self.bytecode.replace(old_current_code);

        let offset = if else_statement.is_none() { 0 } else { 1 };
        let instruction = Instruction::JumpIfFalse {
            src: expression_value_register,
            // FIXME: size limit...
            offset: if_statement_body
                .len()
                .try_into()
                // 1 for going after if statement and 1 for going after jump that's might be added below
                .map(|i: u16| i + 1u16 + offset)?,
        };

        self.bytecode.borrow_mut().push(instruction);
        self.bytecode.borrow_mut().append(&mut if_statement_body);

        if else_statement.is_none() {
            return Ok(());
        }

        let else_statements = Vec::new();
        let old_current_code = self.bytecode.replace(else_statements);

        let else_statement = else_statement.as_deref().unwrap();
        match else_statement {
            Statement::If {
                condition,
                body,
                else_statement: next_else,
            } => self.compile_if(condition, body, next_else),
            Statement::Block { body } => self.compile_block(body),
            _ => unreachable!(),
        }?;

        let mut else_statement_body = self.bytecode.replace(old_current_code);

        let instruction = Instruction::Jump {
            // ?
            offset: else_statement_body.len() as u16 + 1,
        };

        self.bytecode.borrow_mut().push(instruction);

        self.bytecode.borrow_mut().append(&mut else_statement_body);

        Ok(())
    }

    pub fn compile_return(&mut self, expression: &Expression) -> Result<(), CompilerError> {
        let expr_register = self.compile_expression(expression)?;
        let instruction = Instruction::Return { val: expr_register };
        self.bytecode.borrow_mut().push(instruction);

        Ok(())
    }

    pub fn compile_loop(&mut self, body: &Statement) -> Result<(), CompilerError> {
        let bytecode_size = self.bytecode.borrow().len();

        match body {
            Statement::Block { body } => self.compile_block(body)?,
            _ => unreachable!(),
        };

        let body_size = self.bytecode.borrow().len() - bytecode_size;

        let mut bytecode = self.bytecode.borrow_mut();
        let mut i = 0;
        loop {
            let offset = body_size - i + 2;
            if i >= body_size {
                break;
            }

            let instruction = &mut bytecode[i];

            if let Instruction::Jump {
                offset: maybe_placeholder_offset,
            } = instruction
            {
                if *maybe_placeholder_offset == 0xDEAD {
                    bytecode[i] = Instruction::Jump {
                        offset: offset.try_into().map(|o: JumpOffset| o + 1)?,
                    }
                }
            }

            i += 1;
        }

        let instruction = Instruction::JumpReverse {
            offset: body_size.try_into()?,
        };

        bytecode.push(instruction);

        Ok(())
    }

    pub fn compile_break(&mut self) -> Result<(), CompilerError> {
        // breaks should only exist in loops, so we need to update this offset
        let instruction = Instruction::Jump { offset: 0xDEAD };
        self.bytecode.borrow_mut().push(instruction);

        Ok(())
    }

    pub fn compile_object_mutation(
        &mut self,
        path: &ast::Expression,
        value: &ast::Expression,
    ) -> Result<(), CompilerError> {
        let path = match path {
            Expression::ObjectAccess { path } => path,
            _ => unreachable!(),
        };

        let register = self.get_register();
        let base_obj = path.first().unwrap();
        let mut obj_reg = self.compile_expression(&Expression::Variable(base_obj.to_string()))?;

        for path_value in path.iter().skip(1).take(path.len() - 2) {
            let path_reg = self.compile_expression(&Expression::Literal(Literal::String(
                path_value.to_string(),
            )))?;

            let instruction = Instruction::GetObjectField {
                object: obj_reg,
                field: path_reg,
                return_val: register,
            };

            self.bytecode.borrow_mut().push(instruction);

            obj_reg = register;
        }

        let last = path.last().unwrap();

        let last_reg =
            self.compile_expression(&Expression::Literal(Literal::String(last.to_string())))?;

        let value = self.compile_expression(value)?;

        let instruction = Instruction::SetObjectField {
            object: obj_reg,
            field: last_reg,
            value,
        };

        self.bytecode.borrow_mut().push(instruction);

        Ok(())
    }

    pub fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompilerError> {
        match statement {
            // kinda sus?
            Statement::Const { name, value, .. } => self.compile_let(name, value, false),
            Statement::Let {
                name,
                value,
                is_mutable,
                ..
            } => self.compile_let(name, value, *is_mutable),
            Statement::Reassignment { name, value } => self.compile_let_mutation(name, value),
            Statement::If {
                condition,
                body,
                else_statement,
            } => self.compile_if(condition, body, else_statement),
            Statement::Block { body } => self.compile_block(body),
            Statement::Function(func) => self.compile_function(func),
            Statement::Expression(expr) => self.compile_expression(expr).map(|_| ()),
            Statement::Return(expression) => self.compile_return(expression),
            Statement::Loop { body } => self.compile_loop(body),
            Statement::Break => self.compile_break(),
            Statement::ObjectMutation { path, value } => self.compile_object_mutation(path, value),
        }
    }
}
