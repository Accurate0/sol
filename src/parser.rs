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
        self.consume(TokenKind::Assignment)?;
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
        let maybe_mutable_token = self.peek_token();
        let has_mutable_token = *maybe_mutable_token.kind() == TokenKind::Identifier
            && self.text(&maybe_mutable_token) == "mut";

        if has_mutable_token {
            self.consume(TokenKind::Identifier)?;
        }

        let variable_name = self.consume(TokenKind::Identifier)?;

        self.consume(TokenKind::Assignment)?;

        let expression = self.parse_expression(0)?;
        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Let {
            name: self.text(&variable_name).to_owned(),
            value: expression.into(),
            is_mutable: has_mutable_token,
        })
    }

    fn parse_let_mutation(&mut self, name: &str) -> Result<ast::Statement, ParserError> {
        self.consume(TokenKind::Assignment)?;

        let expression = self.parse_expression(0)?;
        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Reassignment {
            name: name.to_owned(),
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
        let lhs = {
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
                TokenKind::GreaterThan => ast::Operator::GreaterThan,
                TokenKind::GreaterThanOrEquals => ast::Operator::GreaterThanOrEqual,
                TokenKind::LessThan => ast::Operator::LessThan,
                TokenKind::LessThanOrEquals => ast::Operator::LessThanOrEqual,
                TokenKind::Equal => ast::Operator::Equal,
                TokenKind::NotEqual => ast::Operator::NotEqual,
                // these don't belong to us, leave it for someone else to consume
                TokenKind::Comma => break lhs,
                TokenKind::OpenBrace => break lhs,
                TokenKind::CloseParen => break lhs,
                TokenKind::CloseBrace => break lhs,
                TokenKind::EndOfLine => break lhs,

                // FIXME: invalid operators seem to infinite loop somehow here
                _ => {
                    let peeked_token = self.peek_token();

                    break Err(ParserError::UnexpectedToken {
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
                    break lhs;
                }

                self.consume(token)?;
                let rhs = self.parse_expression(right_binding_power)?;
                break Ok(ast::Expression::Infix {
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs),
                    op,
                });
            }
        }
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
            name if self.peek() == TokenKind::Assignment => self.parse_let_mutation(name),
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
                self.consume(TokenKind::Identifier)?;

                let maybe_if = self.peek_token();
                if *maybe_if.kind() == TokenKind::Identifier && self.text(&maybe_if) == "if" {
                    self.consume(TokenKind::Identifier)?;
                    Some(self.parse_if_statement()?)
                } else {
                    Some(self.parse_block()?)
                }
            } else {
                None
            };

        Ok(ast::Statement::If {
            condition: condition.into(),
            body: block.into(),
            else_statement: else_statement.map(|s| s.into()),
        })
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
            Some(token)
        }
    }
}

impl<'a, I> Iterator for Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    type Item = Result<Statement, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.peek();
        if token == TokenKind::EndOfFile {
            return None;
        }

        match self.peek() {
            TokenKind::Identifier => Some(self.parse_statement_identifier()),
            TokenKind::OpenBrace => Some(self.parse_block()),
            TokenKind::EndOfLine => {
                let token = self.consume(TokenKind::EndOfLine);
                if let Err(e) = token {
                    Some(Err::<Statement, ParserError>(e))
                } else {
                    None
                }
            }
            _ => {
                let peeked_token = self.peek_token();
                let e = ParserError::UnexpectedToken {
                    token: peeked_token,
                    text: self.text(&peeked_token).to_owned(),
                    in_function: stringify!(parse),
                };

                Some(Err(e))
            }
        }
    }
}
