use crate::{
    ast::{self, Expression, Statement},
    parser::{self},
};
use itertools::Itertools;
use ordermap::OrderMap;
use thiserror::Error;
use types::{DefinedType, TypecheckerScope};

mod types;

pub struct Typechecker {
    scope_stack: Vec<TypecheckerScope>,
    #[cfg(debug_assertions)]
    validated_types: Vec<String>,
}

#[derive(Debug, Error)]
pub enum TypecheckerError {
    #[error("{0}")]
    ParserError(#[from] parser::ParserError),
    #[error("type error: expected {expected} but got {got}")]
    TypeMismatch { expected: String, got: String },
    #[error("type error: expected {expected} but got {got:?}")]
    TypeMismatchMulti { expected: String, got: Vec<String> },
    #[error("type error: {mismatch1} is not {mismatch2}")]
    TypeMismatchBothWrong {
        mismatch1: String,
        mismatch2: String,
    },
    #[error("type error: unexpected {got}")]
    UnexpectedType { got: String },
    #[error("type error: {what} not found with name '{val}'")]
    NotFound { val: String, what: &'static str },
}

fn recursively_find_all_return<'a>(
    statements: &'a Vec<Statement>,
    collection: &mut Vec<&'a Expression>,
) {
    for statement in statements {
        match statement {
            Statement::Return(e) => collection.push(e),
            Statement::If {
                body,
                else_statement,
                ..
            } => {
                match body.as_ref() {
                    Statement::Block { body } => recursively_find_all_return(body, collection),
                    _ => unreachable!(),
                };

                if let Some(s) = else_statement {
                    match s.as_ref() {
                        Statement::Block { body } => recursively_find_all_return(body, collection),
                        _ => unreachable!(),
                    };
                }
            }
            Statement::Block { body } => {
                recursively_find_all_return(body, collection);
            }
            Statement::Loop { body } => {
                match body.as_ref() {
                    Statement::Block { body } => recursively_find_all_return(body, collection),
                    _ => unreachable!(),
                };
            }
            _ => {}
        }
    }
}

impl Typechecker {
    pub fn new() -> Self {
        let mut initial_scope = TypecheckerScope::new();
        // FIXME: read from map with macro or something to generate this
        initial_scope.define_function_return("print".to_owned(), DefinedType::Nil);

        Self {
            scope_stack: vec![initial_scope],
            #[cfg(debug_assertions)]
            validated_types: vec![],
        }
    }

    #[inline(always)]
    pub fn print_validation_if_debug(&mut self) {
        #[cfg(all(not(test), debug_assertions))]
        if std::env::var("PLRS_TEST").is_err() {
            for validated_type in &self.validated_types {
                tracing::info!("{}", validated_type)
            }
        }

        self.validated_types.clear();
    }

    #[inline(always)]
    fn add_validated_types_for_debug(&mut self, s: String) {
        #[cfg(debug_assertions)]
        self.validated_types.push(s);
    }

    fn add_scope(&mut self) {
        self.scope_stack.push(TypecheckerScope::new());
    }

    fn remove_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn define_function_return_current_scope(&mut self, name: String, type_name: DefinedType) {
        self.scope_stack
            .last_mut()
            .unwrap()
            .define_function_return(name, type_name);
    }

    fn resolve_function_return_type(&mut self, name: &str) -> Option<&DefinedType> {
        let scope_stack = &mut self.scope_stack.iter().rev();
        for v in scope_stack {
            if let Some(t) = v.get_function_return_for(name) {
                return Some(t);
            }
        }

        None
    }

    fn define_type_current_scope(&mut self, name: String, type_name: DefinedType) {
        self.scope_stack.last_mut().unwrap().define(name, type_name);
    }

    fn resolve_type(&mut self, name: &str) -> Option<&DefinedType> {
        let scope_stack = &mut self.scope_stack.iter().rev();
        for v in scope_stack {
            if let Some(reg) = v.get_type_for(name) {
                return Some(reg);
            }
        }

        None
    }

    fn typecheck_statement(&mut self, statement: &Statement) -> Result<(), TypecheckerError> {
        match statement {
            Statement::Const {
                name,
                value,
                type_name,
            } => self.typecheck_let(
                name,
                value,
                type_name,
                #[cfg(debug_assertions)]
                "const",
            ),
            Statement::Let {
                name,
                value,
                type_name,
                ..
            } => self.typecheck_let(
                name,
                value,
                type_name,
                #[cfg(debug_assertions)]
                "let",
            ),
            Statement::Block { body } => self.typecheck_block(body),
            Statement::Reassignment { name, value } => self.typecheck_reassignment(name, value),
            Statement::ObjectMutation { path, value } => {
                self.typecheck_object_mutation(path, value)
            }
            Statement::If {
                condition,
                body,
                else_statement,
            } => self.typecheck_if(condition, body, else_statement),
            Statement::Loop { body } => self.typecheck_statement(body),
            Statement::Function(function) => self.typecheck_function(function),
            Statement::Expression(expression) => self.typecheck_expression(expression).map(|_| ()),
            Statement::Return(expression) => self.typecheck_expression(expression).map(|_| ()),
            Statement::Break => Ok(()),
        }
    }

    fn typecheck_block(&mut self, body: &Vec<Statement>) -> Result<(), TypecheckerError> {
        self.add_scope();

        for s in body {
            self.typecheck_statement(s)?;
        }

        self.remove_scope();

        Ok(())
    }

    fn typecheck_reassignment(
        &mut self,
        name: &str,
        value: &Expression,
    ) -> Result<(), TypecheckerError> {
        let existing_var_type =
            self.resolve_type(name)
                .cloned()
                .ok_or_else(|| TypecheckerError::NotFound {
                    val: name.to_owned(),
                    what: "variable",
                })?;

        let new_var_type = self.typecheck_expression(value)?;

        if existing_var_type == new_var_type {
            Ok(())
        } else {
            Err(TypecheckerError::TypeMismatch {
                expected: existing_var_type.to_string(),
                got: new_var_type.to_string(),
            })
        }
    }

    fn typecheck_object_mutation(
        &mut self,
        path: &Expression,
        value: &Expression,
    ) -> Result<(), TypecheckerError> {
        let path_type = self.typecheck_expression(path)?;
        let value_type = self.typecheck_expression(value)?;

        if path_type == value_type {
            Ok(())
        } else {
            Err(TypecheckerError::TypeMismatch {
                expected: path_type.to_string(),
                got: value_type.to_string(),
            })
        }
    }

    fn typecheck_if(
        &mut self,
        condition: &Expression,
        body: &Statement,
        else_statement: &Option<Box<Statement>>,
    ) -> Result<(), TypecheckerError> {
        let t = self.typecheck_expression(condition)?;
        if t != DefinedType::Bool {
            return Err(TypecheckerError::TypeMismatch {
                expected: "bool".to_string(),
                got: t.to_string(),
            });
        }

        self.typecheck_statement(body)?;
        if let Some(else_statement) = else_statement {
            self.typecheck_statement(else_statement)?;
        }

        Ok(())
    }

    fn typecheck_function(&mut self, function: &ast::Function) -> Result<(), TypecheckerError> {
        let ast::Function {
            name,
            return_type_name,
            body,
            parameters,
        } = function;

        let statements = match body.as_ref() {
            Statement::Block { body } => body,
            _ => unreachable!(),
        };

        for parameter in parameters {
            self.define_type_current_scope(
                parameter.name.to_string(),
                DefinedType::try_from(&parameter.type_name)?,
            );
        }

        for statement in statements {
            self.typecheck_statement(statement)?
        }

        let defined_return_type = return_type_name.as_ref().map(DefinedType::try_from);

        let mut return_statements = Vec::new();
        recursively_find_all_return(statements, &mut return_statements);

        let mut return_types = Vec::with_capacity(return_statements.len());
        for return_statement in return_statements {
            // FIXME: we can't evaluate these like this?
            let defined_ret_type = self.typecheck_expression(return_statement)?;
            return_types.push(defined_ret_type);
        }

        let all_equal = return_types.iter().all_equal_value();
        match all_equal {
            Ok(inferred_type) => {
                if let Some(dt) = defined_return_type {
                    let func_ret_type = dt?;
                    if func_ret_type == *inferred_type {
                        self.add_validated_types_for_debug(format!(
                            "{:8} -> inferred: {inferred_type}, defined: {func_ret_type}",
                            "fn"
                        ));

                        self.define_function_return_current_scope(name.to_owned(), func_ret_type);
                    } else {
                        return Err(TypecheckerError::TypeMismatch {
                            expected: func_ret_type.to_string(),
                            got: inferred_type.to_string(),
                        });
                    }
                } else {
                    self.add_validated_types_for_debug(format!(
                        "{:8} -> inferred: {inferred_type}, using inferred as defined",
                        "fn"
                    ));

                    self.define_function_return_current_scope(
                        name.to_owned(),
                        inferred_type.clone(),
                    );
                }
            }
            Err(types) => {
                if let Some((mismatch1, mismatch2)) = types {
                    return Err(TypecheckerError::TypeMismatchBothWrong {
                        mismatch1: mismatch1.to_string(),
                        mismatch2: mismatch2.to_string(),
                    });
                } else {
                    self.add_validated_types_for_debug(format!("{:8} -> nil", "fn"));

                    self.define_function_return_current_scope(name.to_owned(), DefinedType::Nil);
                }
            }
        };

        Ok(())
    }

    // we return the typename of the expression return value
    fn typecheck_expression(&mut self, expr: &Expression) -> Result<DefinedType, TypecheckerError> {
        let is_numeric = |t: &DefinedType| *t == DefinedType::I64 || *t == DefinedType::F64;

        match expr {
            Expression::Prefix { op, expr } => {
                let expr = self.typecheck_expression(expr)?;

                match op {
                    ast::Operator::Plus | ast::Operator::Minus => {
                        if is_numeric(&expr) {
                            Ok(expr)
                        } else {
                            Err(TypecheckerError::TypeMismatch {
                                expected: "numeric".to_string(),
                                got: expr.to_string(),
                            })
                        }
                    }
                    ast::Operator::Not => {
                        if expr == DefinedType::Bool {
                            Ok(expr)
                        } else {
                            Err(TypecheckerError::TypeMismatch {
                                expected: "bool".to_string(),
                                got: expr.to_string(),
                            })
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expression::Infix { op, lhs, rhs } => {
                let lhs = self.typecheck_expression(lhs)?;
                let rhs = self.typecheck_expression(rhs)?;

                match op {
                    ast::Operator::Plus
                    | ast::Operator::Minus
                    | ast::Operator::Multiply
                    | ast::Operator::Divide => {
                        if is_numeric(&lhs) && is_numeric(&rhs) {
                            Ok(match (lhs, rhs) {
                                (DefinedType::I64, DefinedType::I64) => DefinedType::I64,
                                (DefinedType::I64, DefinedType::F64) => DefinedType::F64,
                                (DefinedType::F64, DefinedType::I64) => DefinedType::F64,
                                (DefinedType::F64, DefinedType::F64) => DefinedType::F64,
                                _ => unreachable!(),
                            })
                        } else {
                            Err(TypecheckerError::TypeMismatchMulti {
                                expected: "numeric".to_owned(),
                                got: vec![lhs.to_string(), rhs.to_string()],
                            })
                        }
                    }

                    ast::Operator::GreaterThan
                    | ast::Operator::GreaterThanOrEqual
                    | ast::Operator::LessThan
                    | ast::Operator::LessThanOrEqual => {
                        if is_numeric(&lhs) && is_numeric(&rhs) {
                            Ok(DefinedType::Bool)
                        } else {
                            Err(TypecheckerError::TypeMismatchMulti {
                                expected: "numeric".to_owned(),
                                got: vec![lhs.to_string(), rhs.to_string()],
                            })
                        }
                    }
                    ast::Operator::Equal | ast::Operator::NotEqual => {
                        if lhs == rhs {
                            Ok(DefinedType::Bool)
                        } else {
                            Err(TypecheckerError::TypeMismatch {
                                expected: lhs.to_string(),
                                got: rhs.to_string(),
                            })
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expression::Literal(literal) => {
                let defined_type = match literal {
                    crate::types::Literal::String(_) => DefinedType::String,
                    crate::types::Literal::Float(_) => DefinedType::F64,
                    crate::types::Literal::Integer(_) => DefinedType::I64,
                    crate::types::Literal::Boolean(_) => DefinedType::Bool,
                };

                self.add_validated_types_for_debug(format!(
                    "{:8} -> defined: {literal:?}, expression: {defined_type}",
                    "literal"
                ));

                Ok(defined_type)
            }
            Expression::Variable(name) => {
                self.resolve_type(name)
                    .cloned()
                    .ok_or_else(|| TypecheckerError::NotFound {
                        val: name.to_owned(),
                        what: "variable",
                    })
            }
            Expression::FunctionCall { name, args: _ } => self
                .resolve_function_return_type(name)
                .cloned()
                .ok_or_else(|| TypecheckerError::NotFound {
                    val: name.to_owned(),
                    what: "function",
                }),
            Expression::Object { fields } => {
                let mut typed_fields = OrderMap::<String, DefinedType>::default();

                for (name, expr) in fields {
                    typed_fields.insert(name.to_owned(), self.typecheck_expression(expr)?);
                }

                Ok(DefinedType::Object {
                    fields: typed_fields,
                })
            }
            Expression::ObjectAccess { path } => {
                let object_name = path.first().unwrap();
                let obj_type = self.resolve_type(object_name);
                if let Some(obj_type) = obj_type {
                    match obj_type {
                        DefinedType::Object { fields } => {
                            let path_to_take = path.iter().skip(1);
                            let mut last_item = None;
                            for item in path_to_take {
                                last_item = fields.get(item)
                            }

                            if let Some(last_item) = last_item {
                                Ok(last_item.clone())
                            } else {
                                unreachable!()
                            }
                        }
                        t => Err(TypecheckerError::UnexpectedType { got: t.to_string() }),
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }

    fn typecheck_let(
        &mut self,
        name: &String,
        value: &Expression,
        type_name: &Option<String>,
        #[cfg(debug_assertions)] in_statement: &'static str,
    ) -> Result<(), TypecheckerError> {
        let expression_type_name = self.typecheck_expression(value)?;
        match type_name {
            None => self.define_type_current_scope(name.to_owned(), expression_type_name),
            Some(s) => {
                let defined_type = DefinedType::try_from(s)?;
                if defined_type == expression_type_name {
                    self.add_validated_types_for_debug(format!(
                        "{:8} -> defined: {defined_type}, expression: {expression_type_name}",
                        in_statement
                    ));

                    self.define_type_current_scope(name.to_owned(), defined_type)
                } else {
                    return Err(TypecheckerError::TypeMismatch {
                        expected: type_name.as_ref().unwrap().to_owned(),
                        got: expression_type_name.to_string(),
                    });
                }
            }
        };

        Ok(())
    }

    // Once day we get "typechecked" AST, not yet...
    pub fn check(mut self, statements: &[Statement]) -> Result<(), TypecheckerError> {
        for statement in statements {
            self.typecheck_statement(statement)?;
        }

        self.print_validation_if_debug();
        Ok(())
    }
}

impl Default for Typechecker {
    fn default() -> Self {
        Self::new()
    }
}
