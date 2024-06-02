#[derive(Debug, PartialEq)]
pub struct Function {
    name: String,
    parameters: Vec<String>,
    body: Box<Statement>,
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

#[derive(Debug, PartialEq)]
pub enum Literal {
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
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
pub enum Expression {
    Literal(Literal),
    Variable(String),
    FunctionCall { name: String, args: Vec<Expression> },
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
