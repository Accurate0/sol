use std::{
    fmt::{self, Display},
    iter::Peekable,
    ops::{Index, Range},
    str::Chars,
};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenKind {
    Comment,
    Identifier,
    Literal,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Add,
    Subtract,
    Multiply,
    Comma,
    Assignment,
    Divide,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    Equal,
    NotEqual,
    Whitespace,
    Colon,
    Dot,
    EndOfLine,
    Not,

    EndOfFile,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[Range::<usize>::from(index)]
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} - <{}, {}>, line: {}",
            // "Token::new(TokenKind::{:?}, Span {{ start: {}, end: {}, line: {} }})",
            self.kind,
            self.span.start,
            self.span.end,
            self.span.line
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

#[derive(Clone, Copy)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn text<'a>(&self, input: &'a str) -> &'a str {
        &input[self.span]
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

pub struct Cursor<'a> {
    chars: Peekable<Chars<'a>>,
    current_consumed: usize,
    line: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: chars.peekable(),
            current_consumed: 0,
            line: 1,
        }
    }

    fn peek(&mut self) -> char {
        *self.chars.peek().unwrap_or(&'\0')
    }

    fn current(&self) -> usize {
        self.current_consumed
    }

    fn next(&mut self) -> Option<char> {
        self.current_consumed += 1;
        self.chars.next()
    }

    fn consume_until(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while !predicate(self.peek()) {
            self.next();
        }
    }

    fn is_start_of_identifier(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_in_identifier(&self, c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn consume_identifier(&mut self, initial: char) -> Token {
        let start = self.current() - 1;
        let mut identifier = String::new();
        identifier.push(initial);

        loop {
            let c = self.peek();
            if self.is_in_identifier(c) {
                identifier.push(c);
                self.next();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::Identifier,
            Span {
                start,
                end: self.current(),
                line: self.line,
            },
        )
    }

    fn consume_number(&mut self, initial: char) -> Token {
        let start = self.current() - 1;
        let mut number = String::new();
        number.push(initial);

        let mut is_floating = false;
        loop {
            let c = self.peek();
            if c.is_ascii_digit() {
                number.push(c);
                self.next();
            } else if c == '.' && !is_floating {
                is_floating = true;
                number.push(c);
                self.next();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::Literal,
            Span {
                start,
                end: self.current(),
                line: self.line,
            },
        )
    }

    fn consume_quoted_string(&mut self) -> Token {
        let start = self.current() - 1;
        let mut s = String::new();
        loop {
            let c = self.peek();
            if c == '"' {
                self.next();
                return Token::new(
                    TokenKind::Literal,
                    Span {
                        start,
                        end: self.current(),
                        line: self.line,
                    },
                );
            }

            s.push(c);
            self.next();
        }
    }

    fn consume_comment_or_divide(&mut self) -> Token {
        let start = self.current() - 1;
        let token_kind = if self.peek() == '/' {
            self.next();
            self.consume_until(|c| c == '\n');
            TokenKind::Comment
        } else {
            TokenKind::Divide
        };

        Token::new(
            token_kind,
            Span {
                start,
                end: self.current(),
                line: self.line,
            },
        )
    }

    pub fn next_token(&mut self) -> Token {
        let next = self.next();
        if next.is_none() {
            return Token::new(
                TokenKind::EndOfFile,
                Span {
                    start: self.current(),
                    end: self.current(),
                    line: self.line,
                },
            );
        }

        let single_char_span = Span {
            start: self.current() - 1,
            end: self.current(),
            line: self.line,
        };

        let next = next.unwrap();
        match next {
            '=' if self.peek() == '=' => {
                self.next();
                Token::new(
                    TokenKind::Equal,
                    Span {
                        start: self.current() - 2,
                        end: self.current(),
                        line: self.line,
                    },
                )
            }
            '=' => Token::new(TokenKind::Assignment, single_char_span),

            '>' if self.peek() == '=' => {
                self.next();
                Token::new(
                    TokenKind::GreaterThanOrEquals,
                    Span {
                        start: self.current() - 2,
                        end: self.current(),
                        line: self.line,
                    },
                )
            }

            '>' => Token::new(TokenKind::GreaterThan, single_char_span),
            '<' if self.peek() == '=' => {
                self.next();
                Token::new(
                    TokenKind::LessThanOrEquals,
                    Span {
                        start: self.current() - 2,
                        end: self.current(),
                        line: self.line,
                    },
                )
            }
            '<' => Token::new(TokenKind::LessThan, single_char_span),

            '(' => Token::new(TokenKind::OpenParen, single_char_span),
            ')' => Token::new(TokenKind::CloseParen, single_char_span),
            '{' => Token::new(TokenKind::OpenBrace, single_char_span),
            '}' => Token::new(TokenKind::CloseBrace, single_char_span),
            '+' => Token::new(TokenKind::Add, single_char_span),
            '-' => Token::new(TokenKind::Subtract, single_char_span),
            '*' => Token::new(TokenKind::Multiply, single_char_span),
            ',' => Token::new(TokenKind::Comma, single_char_span),

            '!' if self.peek() == '=' => {
                self.next();
                Token::new(
                    TokenKind::NotEqual,
                    Span {
                        start: self.current() - 2,
                        end: self.current(),
                        line: self.line,
                    },
                )
            }
            '!' => Token::new(TokenKind::Not, single_char_span),

            '"' => self.consume_quoted_string(),
            '/' => self.consume_comment_or_divide(),
            c @ '0'..='9' => self.consume_number(c),
            ';' => Token::new(
                TokenKind::EndOfLine,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                    line: self.line,
                },
            ),
            '\n' => {
                self.line += 1;

                Token::new(
                    TokenKind::Whitespace,
                    Span {
                        start: 0,
                        end: 0,
                        line: self.line,
                    },
                )
            }
            ':' => Token::new(
                TokenKind::Colon,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                    line: self.line,
                },
            ),
            '.' => Token::new(
                TokenKind::Dot,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                    line: self.line,
                },
            ),
            c if c.is_ascii_whitespace() => Token::new(
                TokenKind::Whitespace,
                Span {
                    start: 0,
                    end: 0,
                    line: self.line,
                },
            ),
            c if self.is_start_of_identifier(c) => self.consume_identifier(c),

            c => todo!("{}", c),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let token = self.cursor.next_token();
            match token.kind {
                TokenKind::EndOfFile => return None,
                TokenKind::Whitespace => continue,
                TokenKind::Comment => continue,
                _ => return Some(token),
            }
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            cursor: Cursor::new(contents.chars()),
        }
    }
}
