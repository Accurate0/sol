use crate::instructions::Register;
use std::{cell::RefCell, collections::HashMap};

#[derive(Debug)]
pub struct Value {
    pub register: Register,
}

#[derive(Debug, PartialEq)]
pub enum ScopeType {
    Global,
    Local,
}

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

    pub fn is_global(&self) -> bool {
        self.r#type == ScopeType::Global
    }

    pub fn define(&self, name: &str, register: Register) {
        self.symbols
            .borrow_mut()
            .insert(name.to_owned(), Value { register });
    }

    pub fn contains(&self, name: &str) -> Option<Register> {
        self.symbols.borrow().get(name).map(|v| v.register)
    }
}
