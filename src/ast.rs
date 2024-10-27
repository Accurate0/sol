use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Box<Statement>,
}

impl Function {
    pub fn new(name: String, parameters: Vec<String>, body: Box<Statement>) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }
}

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

#[derive(Debug, PartialEq)]
pub enum Statement {
    Const {
        name: String,
        value: Expression,
    },
    Let {
        name: String,
        value: Box<Expression>,
        is_mutable: bool,
    },
    LetMutation {
        name: String,
        value: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        body: Box<Statement>,
        else_statement: Option<Box<Statement>>,
    },
    Block {
        body: Vec<Statement>,
    },
    Function(Function),
    Expression(Expression),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Not,
    Divide,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

impl Operator {
    // https://domenicquirl.github.io/blog/parsing-basics/
    // ... :)
    pub fn prefix_binding_power(&self) -> ((), u8) {
        match self {
            Self::Minus | Self::Plus | Self::Not => ((), 51),
            _ => unreachable!(),
        }
    }

    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self {
            Self::Equal | Self::NotEqual => Some((5, 6)),
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => Some((7, 8)),
            Self::Plus | Self::Minus => Some((9, 10)),
            Self::Multiply | Self::Divide => Some((11, 12)),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Prefix {
        op: Operator,
        expr: Box<Expression>,
    },
    Infix {
        op: Operator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Literal(Literal),
    Variable(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}
