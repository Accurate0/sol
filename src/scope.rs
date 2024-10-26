use crate::instructions::Register;
use std::{cell::RefCell, collections::HashMap};

#[derive(Debug)]
pub struct Value {
    pub register: Register,
    pub is_mutable: bool,
}

#[derive(Debug, PartialEq)]
pub enum ScopeType {
    Global,
    Local,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Scope {
    r#type: ScopeType,
    symbols: RefCell<HashMap<String, Value>>,
}

impl Scope {
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            r#type: scope_type,
            symbols: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn is_global(&self) -> bool {
        self.r#type == ScopeType::Global
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
