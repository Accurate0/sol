use crate::{
    compiler,
    types::{self, Literal, Object, ObjectValue},
};
use std::{borrow::Cow, cell::RefCell, cmp::Ordering, rc::Rc};

// we reference count all objects :)
pub type VMObject = Rc<RefCell<Object>>;
pub type VMObjectValue = Rc<RefCell<ObjectValue>>;
pub type VMFunction = Rc<compiler::Function>;

// FIXME: is this too big?
#[derive(Default, Debug, Clone)]
pub enum VMValue<'a> {
    #[default]
    Empty,
    Literal(Cow<'a, types::Literal>),
    Object(VMObject),
    Function(VMFunction),
}

impl PartialEq for VMValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for VMValue<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (VMValue::Empty, VMValue::Empty) => Some(Ordering::Equal),
            (VMValue::Literal(l1), VMValue::Literal(l2)) => match (l1.as_ref(), l2.as_ref()) {
                (Literal::String(l1), Literal::String(l2)) => l1.partial_cmp(l2),
                (Literal::Float(l1), Literal::Float(l2)) => l1.partial_cmp(l2),
                (Literal::Integer(l1), Literal::Integer(l2)) => l1.partial_cmp(l2),
                (Literal::Boolean(l1), Literal::Boolean(l2)) => l1.partial_cmp(l2),

                _ => None,
            },

            _ => None,
        }
    }
}
