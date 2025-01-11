use crate::{
    ast::{self, FunctionParameter, Statement},
    lexer::{Span, Token, TokenKind},
    types,
};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::Files,
    term::termcolor::StandardStream,
};
use ordermap::OrderMap;
use std::iter::Peekable;

mod error;
pub use error::ParserError;

pub struct Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    tokens: Peekable<I>,
    input: &'a str,
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(tokens: I, input: &'a str) -> Self {
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
                file_id: 0,
                start: 0,
                end: 0,
                line: 0,
            },
        ))
    }

    fn parse_const(&mut self) -> Result<ast::Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier)?.text(self.input);

        let type_name = if self.peek() == TokenKind::Colon {
            self.consume(TokenKind::Colon)?;
            let type_name_token = self.consume(TokenKind::Identifier)?;

            Some(self.text(&type_name_token).to_owned())
        } else {
            None
        };

        self.consume(TokenKind::Assignment)?;
        // could be expr in future
        let literal = self.parse_literal()?;

        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Const {
            name: name.to_owned(),
            value: literal,
            type_name,
        })
    }

    fn parse_function(&mut self) -> Result<ast::Function, ParserError> {
        let name = self.consume(TokenKind::Identifier)?.text(self.input);

        let _open_paren = self.consume(TokenKind::OpenParen)?;
        let args = self.parse_parameters()?;
        let _close_paren = self.consume(TokenKind::CloseParen)?;

        let return_type_name = if self.peek() == TokenKind::Subtract {
            self.consume(TokenKind::Subtract)?;
            self.consume(TokenKind::GreaterThan)?;

            let return_type_name_token = self.consume(TokenKind::Identifier)?;
            Some(self.text(&return_type_name_token).to_owned())
        } else {
            None
        };

        let block = self.parse_block()?;

        Ok(ast::Function::new(
            name.to_owned(),
            args,
            block.into(),
            return_type_name,
        ))
    }

    fn parse_let(&mut self) -> Result<ast::Statement, ParserError> {
        let maybe_mutable_token = self.peek_token();
        let has_mutable_token = *maybe_mutable_token.kind() == TokenKind::Identifier
            && self.text(&maybe_mutable_token) == "mut";

        if has_mutable_token {
            self.consume(TokenKind::Identifier)?;
        }

        let variable_name = self.consume(TokenKind::Identifier)?;

        let type_name = if self.peek() == TokenKind::Colon {
            self.consume(TokenKind::Colon)?;
            let type_name_token = self.consume(TokenKind::Identifier)?;

            Some(self.text(&type_name_token).to_owned())
        } else {
            None
        };

        self.consume(TokenKind::Assignment)?;

        let expression = self.parse_expression(0)?;
        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Let {
            name: self.text(&variable_name).to_owned(),
            value: expression.into(),
            is_mutable: has_mutable_token,
            type_name,
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
            ast::Expression::Literal(types::Literal::Boolean(true))
        } else if text == "false" {
            ast::Expression::Literal(types::Literal::Boolean(false))
        } else if text.starts_with('"') && text.ends_with('"') {
            ast::Expression::Literal(types::Literal::String(text[1..text.len() - 1].to_owned()))
        } else if text.contains('.') {
            let float = text.parse::<f64>();
            if float.is_err() {
                let diagnostic = Diagnostic::error()
                    .with_message("could not convert to float")
                    .with_labels(vec![Label::primary(token.span().file_id, token.span())
                        .with_message("this is not a valid float")]);

                return Err(ParserError::Diagnostic(diagnostic));
            }

            ast::Expression::Literal(types::Literal::Float(float.unwrap()))
        } else {
            let integer = text.parse::<i64>();
            if integer.is_err() {
                let diagnostic = Diagnostic::error()
                    .with_message("could not convert to integer")
                    .with_labels(vec![Label::primary(token.span().file_id, token.span())
                        .with_message("this is not a valid integer")]);

                return Err(ParserError::Diagnostic(diagnostic));
            }
            ast::Expression::Literal(types::Literal::Integer(integer.unwrap()))
        };

        Ok(expr)
    }

    fn parse_object(&mut self) -> Result<ast::Expression, ParserError> {
        self.consume(TokenKind::OpenBrace)?;

        let mut fields = OrderMap::new();
        // left side is identifier only.
        loop {
            if self.peek() == TokenKind::CloseBrace {
                break;
            }

            let key = self.consume(TokenKind::Identifier)?;
            self.consume(TokenKind::Colon)?;
            let value = self.parse_expression(0)?;

            fields.insert(self.text(&key).to_string(), value);

            if self.peek() == TokenKind::Comma {
                self.consume(TokenKind::Comma)?;
            } else {
                break;
            }
        }

        self.consume(TokenKind::CloseBrace)?;

        Ok(ast::Expression::Object { fields })
    }

    fn parse_expression(&mut self, binding_power: u8) -> Result<ast::Expression, ParserError> {
        let lhs = {
            match self.peek() {
                TokenKind::OpenBrace => self.parse_object(),
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

                    let diagnostic = Diagnostic::error()
                        .with_message("unexpected token")
                        .with_labels(vec![Label::primary(
                            peeked_token.span().file_id,
                            peeked_token.span(),
                        )
                        .with_message(format!(
                            "did not expect token of `{}` type",
                            peeked_token.kind()
                        ))]);

                    Err(ParserError::Diagnostic(diagnostic))
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

                    let diagnostic = Diagnostic::error()
                        .with_message("unexpected token")
                        .with_labels(vec![Label::primary(
                            peeked_token.span().file_id,
                            peeked_token.span(),
                        )
                        .with_message(format!(
                            "did not expect token of `{}` type",
                            peeked_token.kind()
                        ))]);

                    break Err(ParserError::Diagnostic(diagnostic));
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

    fn parse_object_access(&mut self, first: &str) -> Result<ast::Expression, ParserError> {
        let mut path = vec![first.to_string()];

        loop {
            if self.peek() == TokenKind::Dot {
                self.consume(TokenKind::Dot)?;
            } else {
                break;
            }

            let token = self.consume(TokenKind::Identifier)?;
            path.push(self.text(&token).to_string());
        }

        Ok(ast::Expression::ObjectAccess { path })
    }

    fn parse_expression_identifier(&mut self) -> Result<ast::Expression, ParserError> {
        let token = self.consume(TokenKind::Identifier)?;

        let expr = match self.text(&token) {
            // hmmmm
            "true" => Ok(ast::Expression::Literal(types::Literal::Boolean(true))),
            "false" => Ok(ast::Expression::Literal(types::Literal::Boolean(false))),
            name if self.peek() == TokenKind::Dot => self.parse_object_access(name),
            name if self.peek() == TokenKind::OpenParen => self.parse_function_call(name, false),
            name => self.parse_variable(name),
        }?;

        Ok(expr)
    }

    fn parse_return(&mut self) -> Result<ast::Statement, ParserError> {
        let expr = self.parse_expression(0)?;
        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Return(expr))
    }

    fn parse_loop(&mut self) -> Result<ast::Statement, ParserError> {
        let block = self.parse_block()?;
        Ok(ast::Statement::Loop { body: block.into() })
    }

    fn parse_break(&mut self) -> Result<ast::Statement, ParserError> {
        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::Break)
    }

    fn parse_object_mutation(&mut self, first: &str) -> Result<ast::Statement, ParserError> {
        let object_access = self.parse_object_access(first)?;

        self.consume(TokenKind::Assignment)?;

        let expr = self.parse_expression(0)?;

        self.consume(TokenKind::EndOfLine)?;

        Ok(ast::Statement::ObjectMutation {
            path: object_access,
            value: expr.into(),
        })
    }

    fn parse_statement_identifier(&mut self) -> Result<ast::Statement, ParserError> {
        let identifier = self.consume(TokenKind::Identifier)?;
        match self.text(&identifier) {
            "let" => self.parse_let(),
            "const" => self.parse_const(),
            "fn" => Ok(ast::Statement::Function(self.parse_function()?)),
            "if" => self.parse_if_statement(),
            "return" => self.parse_return(),
            "loop" => self.parse_loop(),
            "break" => self.parse_break(),
            name if self.peek() == TokenKind::OpenParen => Ok(ast::Statement::Expression(
                self.parse_function_call(name, true)?,
            )),
            name if self.peek() == TokenKind::Dot => self.parse_object_mutation(name),
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

                let diagnostic = Diagnostic::error()
                    .with_message("unexpected token")
                    .with_labels(vec![Label::primary(
                        peeked_token.span().file_id,
                        peeked_token.span(),
                    )
                    .with_message(format!(
                        "did not expect token of `{}` type",
                        peeked_token.kind()
                    ))]);

                Err(ParserError::Diagnostic(diagnostic))
            }
        }
    }

    fn parse_parameters(&mut self) -> Result<Vec<FunctionParameter>, ParserError> {
        let mut args = Vec::new();

        loop {
            if self.peek() == TokenKind::CloseParen {
                break;
            }

            let identifier = self.consume(TokenKind::Identifier)?;
            let name = self.text(&identifier).to_owned();

            self.consume(TokenKind::Colon)?;

            let type_name_token = self.consume(TokenKind::Identifier)?;
            let type_name = self.text(&type_name_token);

            args.push(FunctionParameter {
                name,
                type_name: type_name.to_string(),
            });

            if self.peek() == TokenKind::Comma {
                self.consume(TokenKind::Comma)?;
            }
        }

        Ok(args)
    }

    pub fn consume(&mut self, expected: TokenKind) -> Result<Token, ParserError> {
        let token = self.next();

        if token.is_none() {
            let prev_token = self.peek_token();

            let diagnostic = Diagnostic::error()
                .with_message("unexpected token")
                .with_labels(vec![Label::primary(
                    prev_token.span().file_id,
                    prev_token.span(),
                )
                .with_message(format!("expected type of `{}`", expected))]);

            return Err(ParserError::Diagnostic(diagnostic));
        }

        let token = token.unwrap();
        if *token.kind() != expected {
            let diagnostic = Diagnostic::error()
                .with_message("unexpected token")
                .with_labels(vec![Label::primary(token.span().file_id, token.span())
                    .with_message(format!(
                        "expected `{}`, got `{}`",
                        expected,
                        token.kind()
                    ))]);

            return Err(ParserError::Diagnostic(diagnostic));
        }

        Ok(token)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.next()?;

        // tracing::info!("{:?}", token);

        if *token.kind() == TokenKind::EndOfFile {
            None
        } else {
            Some(token)
        }
    }

    pub fn collect_and_emit_diagnostics<T>(
        self,
        writer: &StandardStream,
        config: &codespan_reporting::term::Config,
        files: &'a T,
    ) -> Result<Vec<Statement>, Box<dyn std::error::Error>>
    where
        T: Files<'a, FileId = usize> + 'a,
    {
        let mut statements = Vec::new();
        for statement in self.into_iter() {
            if let Err(e) = statement {
                let ParserError::Diagnostic(diagnostic) = e;
                codespan_reporting::term::emit(&mut writer.lock(), config, files, &diagnostic)?;

                // cause statuscode to be set
                return Err("".into());
            }

            statements.push(statement.unwrap());
        }

        Ok(statements)
    }
}

impl<I> Iterator for Parser<'_, I>
where
    I: Iterator<Item = Token>,
{
    // FIXME: DUMB
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
                match token {
                    Ok(_) => None,
                    Err(e) => Some(Err(e)),
                }
            }
            _ => {
                let peeked_token = self.peek_token();

                let diagnostic = Diagnostic::error()
                    .with_message("unexpected token")
                    .with_labels(vec![Label::primary(
                        peeked_token.span().file_id,
                        peeked_token.span(),
                    )
                    .with_message(format!("unexpected `{}`", peeked_token.kind()))]);

                Some(Err(ParserError::Diagnostic(diagnostic)))
            }
        }
    }
}
