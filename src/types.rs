use crate::vm::{VMArray, VMFunction, VMObject, VMObjectValue};
use ordermap::OrderMap;
use std::{fmt::Display, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{}", s),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::Integer(n) => write!(f, "{}", n),
            Literal::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    this: Vec<VMObjectValue>,
}

impl Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.this.iter().map(|v| v.borrow().to_string()))
            .finish()
    }
}

impl Array {
    pub fn create_for_vm() -> VMArray {
        Rc::new(
            Self {
                this: Default::default(),
            }
            .into(),
        )
    }

    pub fn set(&mut self, idx: usize, v: VMObjectValue) {
        if idx >= self.this.len() || self.this.is_empty() {
            self.this
                .resize(2 * (idx + 1), Rc::new(ObjectValue::Nil.into()));
        }

        self.this[idx] = v;
    }

    pub fn index(&self, idx: usize) -> Option<VMObjectValue> {
        self.this.get(idx).cloned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    fields: OrderMap<String, VMObjectValue>,
}

// FIXME: nesting leads to extra quotes
impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.fields.iter().map(|(k, v)| (k, v.borrow().to_string())))
            .finish()
    }
}

impl Object {
    pub fn create_for_vm() -> VMObject {
        Rc::new(
            Self {
                fields: Default::default(),
            }
            .into(),
        )
    }

    pub fn insert(&mut self, k: String, v: VMObjectValue) {
        self.fields.insert(k, v);
    }

    pub fn index(&self, idx: &Literal) -> Option<VMObjectValue> {
        match idx {
            Literal::String(s) => self.fields.get(s).cloned(),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectValue {
    Nil,
    Object(VMObject),
    Array(VMArray),
    Literal(Literal),
    // object values use function indexes?
    Function(VMFunction),
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectValue::Object(rc) => write!(f, "{}", rc.borrow()),
            ObjectValue::Literal(literal) => write!(f, "{}", literal),
            ObjectValue::Function(func) => write!(f, "{}", func),
            ObjectValue::Array(rc) => write!(f, "{}", rc.borrow()),
            ObjectValue::Nil => write!(f, "nil"),
        }
    }
}
