use std::{cell::RefCell, fmt::Display, rc::Rc};

use ordermap::OrderMap;

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
pub struct Object {
    fields: OrderMap<String, Rc<RefCell<ObjectValue>>>,
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
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(
            Self {
                fields: Default::default(),
            }
            .into(),
        )
    }

    pub fn insert(&mut self, k: String, v: Rc<RefCell<ObjectValue>>) {
        self.fields.insert(k, v);
    }

    pub fn index(&self, idx: &Literal) -> Option<Rc<RefCell<ObjectValue>>> {
        match idx {
            Literal::String(s) => self.fields.get(s).cloned(),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectValue {
    Object(Rc<RefCell<Object>>),
    Literal(Literal),
    // object values use function indexes?
    Function(usize),
}

impl Display for ObjectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectValue::Object(rc) => write!(f, "{}", rc.borrow()),
            ObjectValue::Literal(literal) => write!(f, "{}", literal),
            ObjectValue::Function(fn_id) => write!(f, "fn:{}", fn_id),
        }
    }
}
