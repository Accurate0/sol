use crate::instructions::Register;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

#[derive(Debug)]
pub struct Value {
    pub register: Register,
    pub is_mutable: bool,
}

#[derive(Debug, PartialEq, Default)]
pub enum ScopeType {
    Global,
    #[default]
    Local,
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Scope {
    r#type: ScopeType,
    functions: RefCell<HashSet<String>>,
    symbols: RefCell<HashMap<String, Value>>,
}

impl Scope {
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            r#type: scope_type,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn is_global(&self) -> bool {
        self.r#type == ScopeType::Global
    }

    pub fn define_function(&self, function_name: &str) {
        self.functions.borrow_mut().insert(function_name.to_owned());
    }

    pub fn contains_function(&self, function_name: &str) -> bool {
        self.functions.borrow().contains(function_name)
    }

    pub fn define_immutable(&self, name: &str, register: Register) {
        self.symbols.borrow_mut().insert(
            name.to_owned(),
            Value {
                register,
                is_mutable: false,
            },
        );
    }

    pub fn define_mutable(&self, name: &str, register: Register) {
        self.symbols.borrow_mut().insert(
            name.to_owned(),
            Value {
                register,
                is_mutable: true,
            },
        );
    }

    pub fn contains(&self, name: &str) -> Option<Register> {
        self.symbols.borrow().get(name).map(|v| v.register)
    }

    pub fn is_mutable(&self, name: &str) -> Option<bool> {
        self.symbols.borrow().get(name).map(|v| v.is_mutable)
    }
}
