use std::fmt::Display;

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
