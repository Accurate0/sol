use crate::{
    ast::{self, Statement},
    lexer::{Span, Token, TokenKind},
};
use std::{
    iter::Peekable,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

pub struct Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    tokens: Peekable<I>,
    input: &'a String,
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("expected token: {0} not found", expected)]
    ExpectedTokenNotFound { expected: TokenKind },
    #[error("expected token: got {0}, expected: {1:?}", actual, expected)]
    InvalidToken {
        expected: TokenKind,
        actual: TokenKind,
    },
    #[error(
        "unexpected token: {0:?}, {1} in function {2}",
        token,
        text,
        in_function
    )]
    UnexpectedToken {
        token: Token,
        text: String,
        in_function: &'static str,
    },
    #[error("error parsing float: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("error parsing integer: {0}")]
    ParseIntegerError(#[from] ParseIntError),
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(tokens: I, input: &'a String) -> Self {
        Self {
            tokens: tokens.peekable(),
            input,
        }
    }

    fn text(&self, token: &Token) -> &'a str {
        token.text(self.input)
    }

    fn peek(&mut self) -> TokenKind {
        *self
            .tokens
            .peek()
            .map(|t| t.kind())
            .unwrap_or(&TokenKind::EndOfFile)
    }

    fn peek_token(&mut self) -> Token {
        *self.tokens.peek().unwrap_or(&Token::new(
            TokenKind::EndOfFile,
            Span {
                start: 0,
                end: 0,
                line: 0,
            },
        ))
    }

    fn parse_const(&mut self) -> Result<ast::Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier)?.text(self.input);
        self.consume(TokenKind::Eq)?;
        // could be expr in future
        let literal = self.parse_literal()?;

        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Const {
            name: name.to_owned(),
            value: literal,
        })
    }

    fn parse_function(&mut self) -> Result<ast::Function, ParserError> {
        let name = self.consume(TokenKind::Identifier)?.text(self.input);

        let _open_paren = self.consume(TokenKind::OpenParen)?;
        let args = self.parse_parameters()?;
        let _close_paren = self.consume(TokenKind::CloseParen)?;

        let block = self.parse_block()?;

        Ok(ast::Function::new(name.to_owned(), args, block.into()))
    }

    fn parse_let(&mut self) -> Result<ast::Statement, ParserError> {
        let variable_name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::Eq)?;

        let expression = self.parse_expression(0)?;
        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Let {
            name: self.text(&variable_name).to_owned(),
            value: expression.into(),
        })
    }

    fn parse_literal(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.consume(TokenKind::Literal)?;
        let text = self.text(&token);

        let expr = if text == "true" {
            ast::Expression::Literal(ast::Literal::Boolean(true))
        } else if text == "false" {
            ast::Expression::Literal(ast::Literal::Boolean(false))
        } else if text.starts_with('"') && text.ends_with('"') {
            ast::Expression::Literal(ast::Literal::String(text[1..text.len() - 1].to_owned()))
        } else if text.contains('.') {
            ast::Expression::Literal(ast::Literal::Float(text.parse()?))
        } else {
            ast::Expression::Literal(ast::Literal::Integer(text.parse()?))
        };

        Ok(expr)
    }

    fn parse_expression(&mut self, binding_power: u8) -> Result<ast::Expression, ParserError> {
        let mut lhs = {
            match self.peek() {
                TokenKind::Identifier => self.parse_expression_identifier(),
                TokenKind::OpenParen => {
                    self.consume(TokenKind::OpenParen)?;
                    let expr = self.parse_expression(0)?;
                    self.consume(TokenKind::CloseParen)?;
                    Ok(expr)
                }
                TokenKind::Add | TokenKind::Subtract | TokenKind::Not => {
                    let token = self.peek();
                    self.consume(token)?;
                    let op = match token {
                        TokenKind::Add => ast::Operator::Plus,
                        TokenKind::Subtract => ast::Operator::Minus,
                        TokenKind::Not => ast::Operator::Not,
                        _ => unreachable!(),
                    };

                    let ((), right_binding_power) = op.prefix_binding_power();
                    let rhs = self.parse_expression(right_binding_power)?;

                    Ok(ast::Expression::Prefix {
                        op,
                        expr: Box::new(rhs),
                    })
                }
                TokenKind::Literal => self.parse_literal(),
                _ => {
                    let peeked_token = self.peek_token();
                    Err(ParserError::UnexpectedToken {
                        token: peeked_token,
                        text: self.text(&peeked_token).to_owned(),
                        in_function: stringify!(parse_expression + lhs),
                    })
                }
            }
        };

        loop {
            let token = self.peek();
            let op = match token {
                TokenKind::Add => ast::Operator::Plus,
                TokenKind::Subtract => ast::Operator::Minus,
                TokenKind::Multiply => ast::Operator::Multiply,
                TokenKind::Divide => ast::Operator::Divide,
                // these don't belong to us, leave it for someone else to consume
                TokenKind::Comma => break,
                TokenKind::OpenBrace => break,
                TokenKind::CloseParen => break,
                TokenKind::CloseBrace => break,
                TokenKind::EndOfLine => break,

                _ => {
                    let peeked_token = self.peek_token();
                    return Err(ParserError::UnexpectedToken {
                        token: peeked_token,
                        text: self.text(&peeked_token).to_owned(),
                        in_function: stringify!(parse_expression + rhs),
                    });
                }
            };

            if let Some((left_binding_power, right_binding_power)) = op.infix_binding_power() {
                if left_binding_power < binding_power {
                    // previous operator has higher binding power than
                    // new one --> end of expression
                    break;
                }

                self.consume(token)?;
                let rhs = self.parse_expression(right_binding_power)?;
                lhs = Ok(ast::Expression::Infix {
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs),
                    op,
                });

                continue;
            }
        }

        lhs
    }

    fn parse_expression_identifier(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.consume(TokenKind::Identifier)?;

        let expr = match self.text(&token) {
            // hmmmm
            "true" => Ok(ast::Expression::Literal(ast::Literal::Boolean(true))),
            "false" => Ok(ast::Expression::Literal(ast::Literal::Boolean(false))),
            name if self.peek() == TokenKind::OpenParen => self.parse_function_call(name, false),
            name => self.parse_variable(name),
        }?;

        Ok(expr)
    }

    fn parse_statement_identifier(&mut self) -> Result<ast::Statement, ParserError> {
        let identifier = self.consume(TokenKind::Identifier)?;
        match self.text(&identifier) {
            "let" => self.parse_let(),
            "const" => self.parse_const(),
            "fn" => Ok(ast::Statement::Function(self.parse_function()?)),
            "if" => self.parse_if_statement(),
            name if self.peek() == TokenKind::OpenParen => Ok(ast::Statement::Expression(
                self.parse_function_call(name, true)?,
            )),
            name => Ok(ast::Statement::Expression(self.parse_variable(name)?)),
        }
    }

    fn parse_block(&mut self) -> Result<ast::Statement, ParserError> {
        self.consume(TokenKind::OpenBrace)?;

        let mut statements = Vec::new();
        loop {
            let token = self.peek();
            if token == TokenKind::CloseBrace {
                break;
            }

            let statement = self.parse_statement()?;
            statements.push(statement);
        }

        self.consume(TokenKind::CloseBrace)?;

        Ok(ast::Statement::Block { body: statements })
    }

    fn parse_if_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let condition = self.parse_expression(0)?;

        let block = self.parse_block()?;

        let maybe_else = self.peek_token();
        let else_statement =
            if *maybe_else.kind() == TokenKind::Identifier && self.text(&maybe_else) == "else" {
                Some(self.parse_else_statement()?)
            } else {
                None
            };

        Ok(ast::Statement::If {
            condition: condition.into(),
            body: block.into(),
            else_statement: else_statement.map(|s| s.into()),
        })
    }

    fn parse_else_statement(&mut self) -> Result<ast::Statement, ParserError> {
        self.consume(TokenKind::Identifier)?;
        self.parse_block()
    }

    fn parse_variable(&mut self, name: &str) -> Result<ast::Expression, ParserError> {
        Ok(ast::Expression::Variable(name.to_owned()))
    }

    fn parse_function_call(
        &mut self,
        name: &str,
        is_statement: bool,
    ) -> Result<ast::Expression, ParserError> {
        self.consume(TokenKind::OpenParen)?;

        let mut args = Vec::new();
        loop {
            let token = self.peek();
            if token == TokenKind::CloseParen {
                break;
            }

            let expr = self.parse_expression(0)?;
            args.push(expr);

            // if no comma, should break and expect close paren
            if self.peek() == TokenKind::Comma {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::CloseParen)?;

        // if we parsed as part of a full statement, then it should have end of line
        // but if it was something like an expression, there is probably more
        if is_statement {
            self.consume(TokenKind::EndOfLine)?;
        }

        Ok(ast::Expression::FunctionCall {
            name: name.to_owned(),
            args,
        })
    }

    fn parse_statement(&mut self) -> Result<ast::Statement, ParserError> {
        match self.peek() {
            TokenKind::Literal => {
                let expr = ast::Statement::Expression(self.parse_expression(0)?);
                self.consume(TokenKind::EndOfLine)?;

                Ok(expr)
            }
            TokenKind::Identifier => self.parse_statement_identifier(),
            TokenKind::OpenBrace => self.parse_block(),
            _ => {
                let peeked_token = self.peek_token();
                return Err(ParserError::UnexpectedToken {
                    token: peeked_token,
                    text: self.text(&peeked_token).to_owned(),
                    in_function: stringify!(parse_statement),
                });
            }
        }
    }

    fn parse_parameters(&mut self) -> Result<Vec<String>, ParserError> {
        let mut args = Vec::new();

        loop {
            if self.peek() == TokenKind::CloseParen {
                break;
            }

            let identifier = self.consume(TokenKind::Identifier)?;
            args.push(self.text(&identifier).to_owned());
            if self.peek() == TokenKind::Comma {
                self.consume(TokenKind::Comma)?;
            }
        }

        Ok(args)
    }

    pub fn consume(&mut self, expected: TokenKind) -> Result<Token, ParserError> {
        let token = self
            .next()
            .ok_or(ParserError::ExpectedTokenNotFound { expected })?;

        if *token.kind() != expected {
            return Err(ParserError::InvalidToken {
                expected,
                actual: *token.kind(),
            });
        }

        Ok(token)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.next();

        // tracing::info!("{:?}", token);
        if token
            .as_ref()
            .is_some_and(|t| *t.kind() == TokenKind::EndOfFile)
        {
            None
        } else {
            let token = token.unwrap();
            // tracing::info!("{}", self.text(&token));
            Some(token)
        }
    }
}

impl<'a, I> Iterator for Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.peek();
        if token == TokenKind::EndOfFile {
            return None;
        }

        match self.peek() {
            TokenKind::Identifier => {
                let statement = self.parse_statement_identifier();
                match statement {
                    Ok(s) => Some(s),
                    Err(e) => {
                        tracing::error!("{e}");
                        None
                    }
                }
            }
            TokenKind::EndOfLine => {
                let token = self.consume(TokenKind::EndOfLine);
                if let Err(e) = token {
                    tracing::error!("{e}");
                }

                None
            }
            _ => {
                let peeked_token = self.peek_token();
                let e = ParserError::UnexpectedToken {
                    token: peeked_token,
                    text: self.text(&peeked_token).to_owned(),
                    in_function: stringify!(parse),
                };

                tracing::error!("{e}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::lexer::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn small_input() {
        let input = r#"
            const wow = 3;
            fn test() {}
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![
                Statement::Const {
                    name: "wow".to_owned(),
                    value: Expression::Literal(Literal::Integer(3)),
                },
                Statement::Function(Function::new(
                    "test".to_owned(),
                    vec![],
                    Statement::Block { body: vec![] }.into()
                )),
            ]
        );
    }

    #[test]
    fn larger_test() {
        let input = r#"
const wow = 3;

fn main(argv) {
    let x = 2;
    let y = true;
    print("test");
    print(1.3);


    print(x);
    print(2);

    test();
}

fn test(){
    if true {

    } else {
// comment
        print(2);
    }
}

fn new_function(arg1, arg2, arg3) {
{

    test ();
}
}"#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![
                Statement::Const {
                    name: "wow".to_owned(),
                    value: Expression::Literal(Literal::Integer(3)),
                },
                Statement::Function(Function::new(
                    "main".to_owned(),
                    vec!["argv".to_owned()],
                    Statement::Block {
                        body: vec![
                            Statement::Let {
                                name: "x".to_owned(),
                                value: Expression::Literal(Literal::Integer(2)).into(),
                            },
                            Statement::Let {
                                name: "y".to_owned(),
                                value: Expression::Literal(Literal::Boolean(true)).into(),
                            },
                            Statement::Expression(Expression::FunctionCall {
                                name: "print".to_owned(),
                                args: vec![Expression::Literal(Literal::String("test".to_owned()))],
                            }),
                            Statement::Expression(Expression::FunctionCall {
                                name: "print".to_owned(),
                                args: vec![Expression::Literal(Literal::Float(1.3))],
                            }),
                            Statement::Expression(Expression::FunctionCall {
                                name: "print".to_owned(),
                                args: vec![Expression::Variable("x".to_owned())],
                            }),
                            Statement::Expression(Expression::FunctionCall {
                                name: "print".to_owned(),
                                args: vec![Expression::Literal(Literal::Integer(2))],
                            }),
                            Statement::Expression(Expression::FunctionCall {
                                name: "test".to_owned(),
                                args: vec![],
                            }),
                        ],
                    }
                    .into(),
                ),),
                Statement::Function(Function::new(
                    "test".to_owned(),
                    vec![],
                    Statement::Block {
                        body: vec![Statement::If {
                            condition: Expression::Literal(Literal::Boolean(true)).into(),
                            body: Statement::Block { body: vec![] }.into(),
                            else_statement: Some(
                                Statement::Block {
                                    body: vec![Statement::Expression(Expression::FunctionCall {
                                        name: "print".to_owned(),
                                        args: vec![Expression::Literal(Literal::Integer(2))],
                                    }),]
                                }
                                .into()
                            ),
                        },]
                    }
                    .into()
                ),),
                Statement::Function(Function::new(
                    "new_function".to_owned(),
                    vec!["arg1".to_owned(), "arg2".to_owned(), "arg3".to_owned()],
                    Statement::Block {
                        body: vec![Statement::Block {
                            body: vec![Statement::Expression(Expression::FunctionCall {
                                name: "test".to_owned(),
                                args: vec![],
                            }),],
                        },],
                    }
                    .into()
                ))
            ]
        );
    }

    #[test]
    fn complex_math() {
        let input = r#"
            fn test() {
                let z = (2 * 2) / ((3 - 4) * -2);
            }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![Statement::Function(Function::new(
                "test".to_owned(),
                vec![],
                Statement::Block {
                    body: vec![Statement::Let {
                        name: "z".to_owned(),
                        value: Expression::Infix {
                            lhs: Box::new(Expression::Infix {
                                lhs: Box::new(Expression::Literal(Literal::Integer(2))),
                                op: Operator::Multiply,
                                rhs: Box::new(Expression::Literal(Literal::Integer(2))),
                            }),
                            op: Operator::Divide,
                            rhs: Box::new(Expression::Infix {
                                lhs: Box::new(Expression::Infix {
                                    lhs: Box::new(Expression::Literal(Literal::Integer(3))),
                                    op: Operator::Minus,
                                    rhs: Box::new(Expression::Literal(Literal::Integer(4))),
                                }),
                                op: Operator::Multiply,
                                rhs: Box::new(Expression::Prefix {
                                    op: Operator::Minus,
                                    expr: Box::new(Expression::Literal(Literal::Integer(2))),
                                }),
                            }),
                        }
                        .into(),
                    },]
                }
                .into()
            ))]
        )
    }

    #[test]
    fn math() {
        let input = r#"
            fn test() {
                let x = 1 + 2 / 3;
            }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![Statement::Function(Function::new(
                "test".to_owned(),
                vec![],
                Statement::Block {
                    body: vec![Statement::Let {
                        name: "x".to_owned(),
                        value: Expression::Infix {
                            lhs: Box::new(Expression::Literal(Literal::Integer(1))),
                            rhs: Box::new(Expression::Infix {
                                lhs: Box::new(Expression::Literal(Literal::Integer(2))),
                                rhs: Box::new(Expression::Literal(Literal::Integer(3))),
                                op: Operator::Divide,
                            }),
                            op: Operator::Plus,
                        }
                        .into(),
                    },]
                }
                .into()
            ))]
        )
    }

    #[test]
    fn large_input() {
        let input = r#"
        const wow = 3;
        fn test(argv) {
            // this is a comment
            let a = "hello";
        }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);

        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![
                Statement::Const {
                    name: "wow".to_owned(),
                    value: Expression::Literal(Literal::Integer(3)),
                },
                Statement::Function(Function::new(
                    "test".to_owned(),
                    vec!["argv".to_owned()],
                    Statement::Block {
                        body: vec![Statement::Let {
                            name: "a".to_owned(),
                            value: Expression::Literal(Literal::String("hello".to_owned())).into(),
                        }]
                    }
                    .into()
                )),
            ]
        );
    }

    #[test]
    fn function_call_return() {
        let input = r#"
        fn test() {
            let x = test2();
        }

        fn test2() {

        }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![
                Statement::Function(Function::new(
                    "test".to_owned(),
                    vec![],
                    Statement::Block {
                        body: vec![Statement::Let {
                            name: "x".to_owned(),
                            value: Expression::FunctionCall {
                                name: "test2".to_owned(),
                                args: vec![]
                            }
                            .into()
                        },]
                    }
                    .into()
                )),
                Statement::Function(Function::new(
                    "test2".to_owned(),
                    vec![],
                    Statement::Block { body: vec![] }.into()
                ))
            ]
        )
    }

    // ..? maybe illegal
    #[test]
    fn useless_expression() {
        let input = r#"
        fn test() {
            2 + 2.3;
        }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);

        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![Statement::Function(Function::new(
                "test".to_owned(),
                vec![],
                Statement::Block {
                    body: vec![Statement::Expression(Expression::Infix {
                        op: Operator::Plus,
                        lhs: Expression::Literal(Literal::Integer(2)).into(),
                        rhs: Expression::Literal(Literal::Float(2.3)).into(),
                    })]
                }
                .into()
            )),]
        )
    }

    #[test]
    fn function_call_with_addition() {
        let input = r#"
        fn test() {
            let x = test2() + 1;
        }

        fn test2() {

        }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![
                Statement::Function(Function::new(
                    "test".to_owned(),
                    vec![],
                    Statement::Block {
                        body: vec![Statement::Let {
                            name: "x".to_owned(),
                            value: Expression::Infix {
                                op: Operator::Plus,
                                lhs: Expression::FunctionCall {
                                    name: "test2".to_owned(),
                                    args: vec![]
                                }
                                .into(),
                                rhs: Expression::Literal(Literal::Integer(1)).into()
                            }
                            .into()
                        },]
                    }
                    .into()
                )),
                Statement::Function(Function::new(
                    "test2".to_owned(),
                    vec![],
                    Statement::Block { body: vec![] }.into()
                ))
            ]
        )
    }

    #[test]
    fn variable_and_operation() {
        let input = r#"
        fn test() {
            let x = 1;
            let z = 2 + x;
            let y = x + 3;
            let r = x + z;
        }
        "#
        .to_owned();

        let mut lexer = Lexer::new(&input);
        let parser = Parser::new(&mut lexer, &input);
        let statements = parser.collect::<Vec<_>>();

        assert_eq!(
            statements,
            vec![Statement::Function(Function::new(
                "test".to_owned(),
                vec![],
                Statement::Block {
                    body: vec![
                        Statement::Let {
                            name: "x".to_owned(),
                            value: Expression::Literal(Literal::Integer(1)).into()
                        },
                        Statement::Let {
                            name: "z".to_owned(),
                            value: Expression::Infix {
                                op: Operator::Plus,
                                lhs: Expression::Literal(Literal::Integer(2)).into(),
                                rhs: Expression::Variable("x".to_owned()).into(),
                            }
                            .into()
                        },
                        Statement::Let {
                            name: "y".to_owned(),
                            value: Expression::Infix {
                                op: Operator::Plus,
                                lhs: Expression::Variable("x".to_owned()).into(),
                                rhs: Expression::Literal(Literal::Integer(3)).into()
                            }
                            .into()
                        },
                        Statement::Let {
                            name: "r".to_owned(),
                            value: Expression::Infix {
                                op: Operator::Plus,
                                lhs: Expression::Variable("x".to_owned()).into(),
                                rhs: Expression::Variable("z".to_owned()).into(),
                            }
                            .into()
                        }
                    ]
                }
                .into()
            ))]
        )
    }
}
