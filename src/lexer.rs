use std::{
    fmt::{self, Display},
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
    Comma,
    Eq,
    Divide,
    Gt,
    Lt,
    Whitespace,

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
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
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
            "{:?} - <{}, {}>",
            self.kind, self.span.start, self.span.end
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
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
    chars: Chars<'a>,
    current_consumed: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            current_consumed: 0,
        }
    }

    fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or('\0')
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
                },
            );
        }

        let next = next.unwrap();
        match next {
            '=' => Token::new(
                TokenKind::Eq,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '>' => Token::new(
                TokenKind::Gt,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '<' => Token::new(
                TokenKind::Lt,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '(' => Token::new(
                TokenKind::OpenParen,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            ')' => Token::new(
                TokenKind::CloseParen,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '{' => Token::new(
                TokenKind::OpenBrace,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '}' => Token::new(
                TokenKind::CloseBrace,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '+' => Token::new(
                TokenKind::Add,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),

            ',' => Token::new(
                TokenKind::Comma,
                Span {
                    start: self.current() - 1,
                    end: self.current(),
                },
            ),
            '"' => self.consume_quoted_string(),
            '/' => self.consume_comment_or_divide(),
            c @ '0'..='9' => self.consume_number(c),
            c if c.is_ascii_whitespace() => {
                Token::new(TokenKind::Whitespace, Span { start: 0, end: 0 })
            }
            c if self.is_start_of_identifier(c) => self.consume_identifier(c),

            c => todo!("{}", c),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
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
