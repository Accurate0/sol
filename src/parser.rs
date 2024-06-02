use crate::ast::Function;
use crate::{
    ast::{self, Expression, Program, Statement},
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
    #[error("expected token: {0}, got {1}, {2:?}", expected, actual, token)]
    InvalidToken {
        token: Token,
        expected: TokenKind,
        actual: TokenKind,
    },

    #[error("expected identifier: {0}, got {1}", expected, actual)]
    InvalidIdentifier {
        expected: &'static str,
        actual: String,
    },

    #[error("unexpected token: {0}", token)]
    UnexpectedToken { token: Token },
    #[error("error parsing float: {0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("error parsing integer: {0}")]
    ParseIntegerError(#[from] ParseIntError),
    #[error("unknown data store error")]
    Unknown,
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
        *self
            .tokens
            .peek()
            .unwrap_or(&Token::new(TokenKind::EndOfFile, Span { start: 0, end: 0 }))
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut program = Program {
            statements: Vec::default(),
        };
        loop {
            let token = self.peek();
            if token == TokenKind::EndOfFile {
                break;
            }

            match self.peek() {
                TokenKind::Identifier => {
                    let statement = self.parse_statement_identifier()?;
                    tracing::info!("{:?}", statement);
                    program.statements.push(statement);
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        token: self.peek_token(),
                    })
                }
            };
        }

        Ok(program)
    }

    fn parse_const(&mut self) -> Result<ast::Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier)?.text(self.input);
        self.consume(TokenKind::Eq)?;
        // could be expr in future
        let literal = self.parse_literal()?;

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

        let expression = self.parse_expression()?;

        Ok(Statement::Let {
            name: self.text(&variable_name).to_owned(),
            value: expression.into(),
        })
    }

    fn parse_literal(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.consume(TokenKind::Literal)?;
        let text = self.text(&token);
        let expr = if text.starts_with('"') && text.ends_with('"') {
            Expression::Literal(ast::Literal::String(text[1..text.len() - 1].to_owned()))
        } else if text.contains('.') {
            Expression::Literal(ast::Literal::Float(text.parse()?))
        } else {
            Expression::Literal(ast::Literal::Integer(text.parse()?))
        };

        Ok(expr)
    }

    fn parse_expression(&mut self) -> Result<ast::Expression, ParserError> {
        match self.peek() {
            TokenKind::Identifier => self.parse_expression_identifier(),
            TokenKind::Literal => self.parse_literal(),
            _ => Err(ParserError::UnexpectedToken {
                token: self.peek_token(),
            }),
        }
    }

    fn parse_expression_identifier(&mut self) -> Result<ast::Expression, ParserError> {
        let name = self.consume(TokenKind::Identifier)?;
        match self.text(&name) {
            name if self.peek() == TokenKind::OpenParen => self.parse_function_call(name),
            name => self.parse_variable(name),
        }
    }

    fn parse_statement_identifier(&mut self) -> Result<ast::Statement, ParserError> {
        let identifier = self.consume(TokenKind::Identifier)?;
        match self.text(&identifier) {
            "let" => self.parse_let(),
            "const" => self.parse_const(),
            "fn" => Ok(ast::Statement::Function {
                function: self.parse_function()?,
            }),
            "if" => self.parse_if_statement(),
            name if self.peek() == TokenKind::OpenParen => {
                Ok(ast::Statement::Expression(self.parse_function_call(name)?))
            }
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
            tracing::info!("{:?}", statement);
            statements.push(statement);
        }

        self.consume(TokenKind::CloseBrace)?;

        Ok(Statement::Block { body: statements })
    }

    fn parse_if_statement(&mut self) -> Result<ast::Statement, ParserError> {
        let condition = self.parse_expression_identifier()?;

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

    fn parse_else_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(TokenKind::Identifier)?;
        self.parse_block()
    }

    fn parse_variable(&mut self, name: &str) -> Result<ast::Expression, ParserError> {
        Ok(ast::Expression::Variable(name.to_owned()))
    }

    fn parse_function_call(&mut self, name: &str) -> Result<ast::Expression, ParserError> {
        self.consume(TokenKind::OpenParen)?;

        let mut args = Vec::new();
        loop {
            let token = self.peek();
            if token == TokenKind::CloseParen {
                break;
            }

            let expr = self.parse_expression()?;
            args.push(expr);
        }

        self.consume(TokenKind::CloseParen)?;

        Ok(ast::Expression::FunctionCall {
            name: name.to_owned(),
            args,
        })
    }

    fn parse_statement(&mut self) -> Result<ast::Statement, ParserError> {
        match self.peek() {
            TokenKind::Identifier => self.parse_statement_identifier(),
            TokenKind::OpenBrace => self.parse_block(),
            _ => Err(ParserError::UnexpectedToken {
                token: self.peek_token(),
            }),
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
                token,
                expected,
                actual: *token.kind(),
            });
        }

        Ok(token)
    }
}

impl<'a, I> Iterator for Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
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
