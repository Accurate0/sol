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

#[derive(Debug, Clone, Copy, PartialEq)]
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
            // "Token::new(TokenKind::{:?}, Span {{ start: {}, end: {} }})",
            self.kind,
            self.span.start,
            self.span.end
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Clone, Copy, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn small_input() {
        let input = r#"
            const wow = 3
            fn test() {}
        "#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 13, end: 18 }),
                Token::new(TokenKind::Identifier, Span { start: 19, end: 22 }),
                Token::new(TokenKind::Eq, Span { start: 23, end: 24 }),
                Token::new(TokenKind::Literal, Span { start: 25, end: 26 }),
                Token::new(TokenKind::Identifier, Span { start: 39, end: 41 }),
                Token::new(TokenKind::Identifier, Span { start: 42, end: 46 }),
                Token::new(TokenKind::OpenParen, Span { start: 46, end: 47 }),
                Token::new(TokenKind::CloseParen, Span { start: 47, end: 48 }),
                Token::new(TokenKind::OpenBrace, Span { start: 49, end: 50 }),
                Token::new(TokenKind::CloseBrace, Span { start: 50, end: 51 }),
            ]
        );
    }

    #[test]
    fn larger_test() {
        let input = r#"
const wow = 3

fn main(argv) {
    let x = 2
    let y = true
    print("test")
    print(1.3)


    print(x)
    print(2)

    test()
}

fn test(){
    if true {

    } else {
// comment
        print(2)
    }
}

fn new_function(arg1, arg2, arg3) {
{

    test ()
}
}"#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 1, end: 6 }),
                Token::new(TokenKind::Identifier, Span { start: 7, end: 10 }),
                Token::new(TokenKind::Eq, Span { start: 11, end: 12 }),
                Token::new(TokenKind::Literal, Span { start: 13, end: 14 }),
                Token::new(TokenKind::Identifier, Span { start: 16, end: 18 }),
                Token::new(TokenKind::Identifier, Span { start: 19, end: 23 }),
                Token::new(TokenKind::OpenParen, Span { start: 23, end: 24 }),
                Token::new(TokenKind::Identifier, Span { start: 24, end: 28 }),
                Token::new(TokenKind::CloseParen, Span { start: 28, end: 29 }),
                Token::new(TokenKind::OpenBrace, Span { start: 30, end: 31 }),
                Token::new(TokenKind::Identifier, Span { start: 36, end: 39 }),
                Token::new(TokenKind::Identifier, Span { start: 40, end: 41 }),
                Token::new(TokenKind::Eq, Span { start: 42, end: 43 }),
                Token::new(TokenKind::Literal, Span { start: 44, end: 45 }),
                Token::new(TokenKind::Identifier, Span { start: 50, end: 53 }),
                Token::new(TokenKind::Identifier, Span { start: 54, end: 55 }),
                Token::new(TokenKind::Eq, Span { start: 56, end: 57 }),
                Token::new(TokenKind::Identifier, Span { start: 58, end: 62 }),
                Token::new(TokenKind::Identifier, Span { start: 67, end: 72 }),
                Token::new(TokenKind::OpenParen, Span { start: 72, end: 73 }),
                Token::new(TokenKind::Literal, Span { start: 73, end: 79 }),
                Token::new(TokenKind::CloseParen, Span { start: 79, end: 80 }),
                Token::new(TokenKind::Identifier, Span { start: 85, end: 90 }),
                Token::new(TokenKind::OpenParen, Span { start: 90, end: 91 }),
                Token::new(TokenKind::Literal, Span { start: 91, end: 94 }),
                Token::new(TokenKind::CloseParen, Span { start: 94, end: 95 }),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 102,
                        end: 107
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 107,
                        end: 108
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 108,
                        end: 109
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 109,
                        end: 110
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 115,
                        end: 120
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 120,
                        end: 121
                    }
                ),
                Token::new(
                    TokenKind::Literal,
                    Span {
                        start: 121,
                        end: 122
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 122,
                        end: 123
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 129,
                        end: 133
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 133,
                        end: 134
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 134,
                        end: 135
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 136,
                        end: 137
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 139,
                        end: 141
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 142,
                        end: 146
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 146,
                        end: 147
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 147,
                        end: 148
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 148,
                        end: 149
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 154,
                        end: 156
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 157,
                        end: 161
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 162,
                        end: 163
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 169,
                        end: 170
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 171,
                        end: 175
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 176,
                        end: 177
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 197,
                        end: 202
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 202,
                        end: 203
                    }
                ),
                Token::new(
                    TokenKind::Literal,
                    Span {
                        start: 203,
                        end: 204
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 204,
                        end: 205
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 210,
                        end: 211
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 212,
                        end: 213
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 215,
                        end: 217
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 218,
                        end: 230
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 230,
                        end: 231
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 231,
                        end: 235
                    }
                ),
                Token::new(
                    TokenKind::Comma,
                    Span {
                        start: 235,
                        end: 236
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 237,
                        end: 241
                    }
                ),
                Token::new(
                    TokenKind::Comma,
                    Span {
                        start: 241,
                        end: 242
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 243,
                        end: 247
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 247,
                        end: 248
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 249,
                        end: 250
                    }
                ),
                Token::new(
                    TokenKind::OpenBrace,
                    Span {
                        start: 251,
                        end: 252
                    }
                ),
                Token::new(
                    TokenKind::Identifier,
                    Span {
                        start: 258,
                        end: 262
                    }
                ),
                Token::new(
                    TokenKind::OpenParen,
                    Span {
                        start: 263,
                        end: 264
                    }
                ),
                Token::new(
                    TokenKind::CloseParen,
                    Span {
                        start: 264,
                        end: 265
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 266,
                        end: 267
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 268,
                        end: 269
                    }
                ),
            ]
        )
    }

    #[test]
    fn large_input() {
        let input = r#"
        const wow = 3
        fn test(argv) {
            // this is a comment
            let a = "hello"
        }
        "#;

        let lexer = Lexer::new(input);
        let tokens = lexer.into_iter().collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, Span { start: 9, end: 14 }),
                Token::new(TokenKind::Identifier, Span { start: 15, end: 18 }),
                Token::new(TokenKind::Eq, Span { start: 19, end: 20 }),
                Token::new(TokenKind::Literal, Span { start: 21, end: 22 }),
                Token::new(TokenKind::Identifier, Span { start: 31, end: 33 }),
                Token::new(TokenKind::Identifier, Span { start: 34, end: 38 }),
                Token::new(TokenKind::OpenParen, Span { start: 38, end: 39 }),
                Token::new(TokenKind::Identifier, Span { start: 39, end: 43 }),
                Token::new(TokenKind::CloseParen, Span { start: 43, end: 44 }),
                Token::new(TokenKind::OpenBrace, Span { start: 45, end: 46 }),
                Token::new(TokenKind::Identifier, Span { start: 92, end: 95 }),
                Token::new(TokenKind::Identifier, Span { start: 96, end: 97 }),
                Token::new(TokenKind::Eq, Span { start: 98, end: 99 }),
                Token::new(
                    TokenKind::Literal,
                    Span {
                        start: 100,
                        end: 107
                    }
                ),
                Token::new(
                    TokenKind::CloseBrace,
                    Span {
                        start: 116,
                        end: 117
                    }
                ),
            ]
        );
    }
}
